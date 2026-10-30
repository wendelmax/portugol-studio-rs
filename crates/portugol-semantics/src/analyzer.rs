//! Análise semântica do Portugol 2.0.
//!
//! O desenho segue o `AnalisadorSemantico` do Portugol Studio, mas foi adaptado
//! para a AST deste port e para um conjunto de diagnósticos estáveis
//! (`sem.*`). As diferenças deliberadas estão documentadas em
//! `docs/ARQUITETURA.md`.
//!
//! Decisões importantes:
//!
//! * funções são pré-registradas antes de qualquer corpo, e todas as variáveis
//!   globais são analisadas antes dos corpos das funções. Isso reproduz o
//!   comportamento do original, em que uma global declarada depois de `inicio`
//!   já é visível dentro da função;
//! * tipos que não podem ser determinados (`Ty::Unknown`) propagam
//!   permissividade: não geram erro e não invalidam as verificações seguintes;
//! * bibliotecas não são modeladas função a função. Apenas a inclusão, o alias
//!   e a existência do escopo são validados; retornos e constantes de
//!   bibliotecas viram `Ty::Unknown`.

use std::collections::HashMap;

use portugol_syntax::ast::{
    AssignOp, Assignment, BinOp, Case, Declarations, Declarator, Expr, ForInit, ForStep, Function,
    Ident, Item, Param, Program, Stmt, Type, UnaryOp,
};
use portugol_syntax::{Diagnostic, Span};

use crate::symbols::{FunctionInfo, ParamInfo, ScopeTable, Symbol, SymbolKind};
use crate::types::{Compat, Ty, binary_result, compatible, compound_op, unary_result};

/// Bibliotecas reconhecidas pelo Portugol Studio.
const KNOWN_LIBRARIES: &[&str] = &[
    "Arquivos",
    "Calendario",
    "Graficos",
    "Internet",
    "Matematica",
    "Mouse",
    "Objetos",
    "ServicosWeb",
    "Sons",
    "Teclado",
    "Texto",
    "Tipos",
    "Util",
];

/// Funções fornecidas pela linguagem, que não podem ser redeclaradas.
const RESERVED_FUNCTIONS: &[&str] = &["leia", "escreva", "sorteia", "limpa"];

const MAX_AGGREGATE_SIZE: i64 = 16_777_216;

/// Analisa semanticamente um programa já parseado.
pub fn analyze(program: &Program) -> Vec<Diagnostic> {
    let mut analyzer = Analyzer::new(program);
    analyzer.run();
    analyzer.diagnostics
}

#[derive(Debug)]
struct Target {
    ty: Ty,
    name: Option<String>,
    previous_initialized: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Size {
    Int(i64),
    /// Uma constante de biblioteca: o valor não é conhecido, mas a expressão é
    /// tratada como constante para não gerar falsos positivos.
    Unknown,
    Invalid,
}

struct Analyzer<'a> {
    program: &'a Program,
    diagnostics: Vec<Diagnostic>,
    scopes: ScopeTable,
    /// Nome real ou alias -> nome real da biblioteca.
    libraries: HashMap<String, String>,
    current_function: Option<FunctionInfo>,
    has_entry_point: bool,
}

impl<'a> Analyzer<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            program,
            diagnostics: Vec::new(),
            scopes: ScopeTable::new(),
            libraries: HashMap::new(),
            current_function: None,
            has_entry_point: false,
        }
    }

    fn run(&mut self) {
        self.register_includes();
        self.register_functions();

        if self.program.has_program_block && !self.has_entry_point {
            self.error(
                "sem.missing-entry-point",
                "o programa não possui a função \"inicio\"",
                self.program.span,
            )
            .with_help("declare `funcao inicio() { ... }` ou remova o bloco `programa`");
        }

        // Primeiro todas as globais, depois os corpos das funções.
        for item in &self.program.items {
            if let Item::Declarations(declarations) = item {
                self.declarations(declarations);
            }
        }

        for item in &self.program.items {
            if let Item::Function(function) = item {
                self.function(function);
            }
        }

        // Extensão do port: comandos soltos no nível do arquivo.
        self.scopes.push_scope();
        for item in &self.program.items {
            if let Item::Stmt(statement) = item {
                self.stmt(statement);
            }
        }
        self.scopes.pop_scope();
    }

    // -----------------------------------------------------------------
    // Diagnósticos
    // -----------------------------------------------------------------

    fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) -> Diagnostic {
        let diagnostic = Diagnostic::error(code, message, span);
        self.diagnostics.push(diagnostic.clone());
        diagnostic
    }

    fn warning(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.diagnostics
            .push(Diagnostic::warning(code, message, span));
    }

    fn conversion_warning(&mut self, from: Ty, to: Ty, span: Span) {
        self.warning(
            "sem.conversion",
            format!(
                "o valor do tipo \"{}\" será convertido para \"{}\"",
                from.name(),
                to.name()
            ),
            span,
        );
    }

    // -----------------------------------------------------------------
    // Bibliotecas
    // -----------------------------------------------------------------

    fn register_includes(&mut self) {
        for include in &self.program.includes {
            let name = &include.library.name;

            if !KNOWN_LIBRARIES.contains(&name.as_str()) {
                self.error(
                    "sem.unknown-library",
                    format!("a biblioteca \"{}\" não existe", name),
                    include.library.span,
                );
                continue;
            }

            if self.libraries.contains_key(name) {
                self.error(
                    "sem.duplicate-library",
                    format!("a biblioteca \"{}\" já foi incluída", name),
                    include.library.span,
                );
            } else {
                self.libraries.insert(name.clone(), name.clone());
            }

            if let Some(alias) = &include.alias {
                if let Some(existing) = self.libraries.get(&alias.name).cloned() {
                    self.error(
                        "sem.duplicate-alias",
                        format!(
                            "o alias \"{}\" já está sendo utilizado pela biblioteca \"{}\"",
                            alias.name, existing
                        ),
                        alias.span,
                    );
                } else {
                    self.libraries.insert(alias.name.clone(), name.clone());
                }
            }
        }
    }

    fn check_library_scope(&mut self, scope: &str, span: Span) {
        if !self.libraries.contains_key(scope) {
            self.error(
                "sem.unknown-library-reference",
                format!("a biblioteca ou alias \"{}\" não foi incluída", scope),
                span,
            );
        }
    }

    // -----------------------------------------------------------------
    // Funções
    // -----------------------------------------------------------------

    fn register_functions(&mut self) {
        for item in &self.program.items {
            let Item::Function(function) = item else {
                continue;
            };

            let info = function_info(function);
            if info.name == "inicio" {
                self.has_entry_point = true;
            }

            if RESERVED_FUNCTIONS.contains(&info.name.as_str()) {
                self.error(
                    "sem.reserved-function",
                    format!("a função \"{}\" é reservada para a linguagem", info.name),
                    function.name.span,
                );
                self.scopes.insert_function(Symbol::function(info));
                continue;
            }

            match self.scopes.insert(Symbol::function(info)) {
                Ok(_) => {}
                Err(_) => {
                    self.error(
                        "sem.redeclared",
                        format!("a função \"{}\" já foi declarada", function.name.name),
                        function.name.span,
                    );
                }
            }
        }
    }

    fn function(&mut self, function: &Function) {
        let info = function_info(function);

        self.current_function = Some(info.clone());
        self.scopes.push_scope();

        for param in &function.params {
            self.parameter(param);
        }

        self.block(&function.body);

        if info.return_ty != Ty::Vazio && !body_guarantees_return(&function.body) {
            self.error(
                "sem.missing-return",
                format!(
                    "a função \"{}\" deve retornar um valor do tipo \"{}\" em todos os caminhos",
                    info.name,
                    info.return_ty.name()
                ),
                function.span,
            );
        }

        self.current_function = None;
        self.scopes.pop_scope();
    }

    fn parameter(&mut self, param: &Param) {
        let symbol = Symbol::parameter(
            param.name.name.clone(),
            Ty::from_ast(param.ty),
            param.dims,
            param.name.span,
        );

        match self.scopes.insert(symbol) {
            Ok(false) => {}
            Ok(true) => {
                self.warning(
                    "sem.shadowed",
                    format!(
                        "o parâmetro \"{}\" oculta um símbolo global de mesmo nome",
                        param.name.name
                    ),
                    param.name.span,
                );
            }
            Err(_) => {
                self.error(
                    "sem.redeclared",
                    format!("o parâmetro \"{}\" já foi declarado", param.name.name),
                    param.name.span,
                );
            }
        }
    }

    // -----------------------------------------------------------------
    // Declarações
    // -----------------------------------------------------------------

    fn declarations(&mut self, declarations: &Declarations) {
        for declarator in &declarations.declarators {
            self.declarator(declarations.is_const, declarations.ty, declarator);
        }
    }

    fn declarator(&mut self, is_const: bool, ty: Type, declarator: &Declarator) {
        match declarator {
            Declarator::Variable { name, init, .. } => {
                self.variable(name, is_const, ty, init.as_ref())
            }
            Declarator::Array {
                name,
                size,
                init,
                span,
            } => self.array(name, is_const, ty, size.as_deref(), init.as_deref(), *span),
            Declarator::Matrix {
                name,
                rows,
                cols,
                init,
                span,
            } => self.matrix(
                name,
                is_const,
                ty,
                rows.as_deref(),
                cols.as_deref(),
                init.as_deref(),
                *span,
            ),
        }
    }

    fn variable(&mut self, name: &Ident, is_const: bool, ty: Type, init: Option<&Expr>) {
        let value_ty = Ty::from_ast(ty);
        let mut symbol = Symbol::variable(name.name.clone(), value_ty, name.span);
        symbol.is_const = is_const;

        if !self.insert_symbol(symbol, name.span) {
            return;
        }

        if is_const && init.is_none() {
            self.error(
                "sem.const-without-init",
                format!("a constante \"{}\" precisa ser inicializada", name.name),
                name.span,
            );
        }

        let mut initialized = false;

        if let Some(init) = init {
            if is_const && !is_literal(init) {
                self.error(
                    "sem.const-initializer",
                    format!(
                        "a constante \"{}\" só pode ser inicializada com um valor literal",
                        name.name
                    ),
                    init.span(),
                );
            }

            let init_ty = self.expr(init);
            initialized = self.check_compat(
                value_ty,
                init_ty,
                init.span(),
                &format!("na inicialização de \"{}\"", name.name),
            );

            if is_const
                && initialized
                && let Some(value) = literal_int(init)
                && let Some(symbol) = self.scopes.get_mut(&name.name)
            {
                symbol.const_int = Some(value);
            }
        }

        if initialized {
            self.scopes.set_initialized(&name.name, true);
        }
    }

    fn array(
        &mut self,
        name: &Ident,
        is_const: bool,
        ty: Type,
        size: Option<&Expr>,
        init: Option<&[Expr]>,
        span: Span,
    ) {
        let value_ty = Ty::from_ast(ty);
        // O tamanho é resolvido antes de inserir o símbolo: `inteiro a[a]` não
        // pode enxergar `a` como uma constante.
        let size = size.and_then(|size| self.array_size(size, name.span));

        let mut symbol =
            Symbol::aggregate(name.name.clone(), value_ty, SymbolKind::Vector, name.span);
        symbol.is_const = is_const;
        symbol.initialized = true;

        if !self.insert_symbol(symbol, name.span) {
            return;
        }

        if is_const && init.is_none() {
            self.error(
                "sem.const-without-init",
                format!("a constante \"{}\" precisa ser inicializada", name.name),
                name.span,
            );
        }

        if let Some(values) = init {
            if let Some(expected) = size
                && expected >= 0
                && expected as usize != values.len()
            {
                self.error(
                        "sem.array-size-mismatch",
                        format!(
                            "o vetor \"{}\" foi declarado com {} elemento(s), mas a inicialização possui {}",
                            name.name,
                            expected,
                            values.len()
                        ),
                        span,
                    );
            }

            self.literal_group(value_ty, values, is_const, span, &name.name);
        }

        if size == Some(1) {
            self.warning(
                "sem.vector-could-be-variable",
                format!(
                    "o vetor \"{}\" possui apenas um elemento e poderia ser uma variável",
                    name.name
                ),
                name.span,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn matrix(
        &mut self,
        name: &Ident,
        is_const: bool,
        ty: Type,
        rows: Option<&Expr>,
        cols: Option<&Expr>,
        init: Option<&[Vec<Expr>]>,
        span: Span,
    ) {
        let value_ty = Ty::from_ast(ty);
        let rows = rows.and_then(|rows| self.array_size(rows, name.span));
        let cols = cols.and_then(|cols| self.array_size(cols, name.span));

        if let (Some(rows), Some(cols)) = (rows, cols) {
            let product = (rows as i128) * (cols as i128);
            if product > MAX_AGGREGATE_SIZE as i128 {
                self.error(
                    "sem.matrix-too-large",
                    format!(
                        "a matriz \"{}\" possui {} elementos; o máximo é {}",
                        name.name, product, MAX_AGGREGATE_SIZE
                    ),
                    name.span,
                );
            }

            if rows == 1 && cols == 1 {
                self.warning(
                    "sem.matrix-could-be-variable",
                    format!(
                        "a matriz \"{}\" possui apenas um elemento e poderia ser uma variável",
                        name.name
                    ),
                    name.span,
                );
            } else if rows == 1 || cols == 1 {
                self.warning(
                    "sem.matrix-could-be-vector",
                    format!(
                        "a matriz \"{}\" possui uma única linha ou coluna e poderia ser um vetor",
                        name.name
                    ),
                    name.span,
                );
            }
        }

        let mut symbol =
            Symbol::aggregate(name.name.clone(), value_ty, SymbolKind::Matrix, name.span);
        symbol.is_const = is_const;
        symbol.initialized = true;

        if !self.insert_symbol(symbol, name.span) {
            return;
        }

        if is_const && init.is_none() {
            self.error(
                "sem.const-without-init",
                format!("a constante \"{}\" precisa ser inicializada", name.name),
                name.span,
            );
        }

        if let Some(values) = init {
            if let Some(expected) = rows
                && expected >= 0
                && expected as usize != values.len()
            {
                self.error(
                        "sem.matrix-rows-mismatch",
                        format!(
                            "a matriz \"{}\" foi declarada com {} linha(s), mas a inicialização possui {}",
                            name.name,
                            expected,
                            values.len()
                        ),
                        span,
                    );
            }

            if let Some(expected) = cols {
                for (line, values) in values.iter().enumerate() {
                    if expected >= 0 && expected as usize != values.len() {
                        self.error(
                            "sem.matrix-cols-mismatch",
                            format!(
                                "a linha {} da matriz \"{}\" deveria ter {} elemento(s), mas possui {}",
                                line,
                                name.name,
                                expected,
                                values.len()
                            ),
                            span,
                        );
                    }
                }
            }

            let flattened: Vec<Expr> = values.iter().flatten().cloned().collect();
            self.literal_group(value_ty, &flattened, is_const, span, &name.name);
        }
    }

    /// Insere um símbolo no escopo atual e produz o diagnóstico adequado.
    /// Devolve `false` quando houve redeclaração (o símbolo não foi inserido).
    fn insert_symbol(&mut self, symbol: Symbol, span: Span) -> bool {
        let name = symbol.name.clone();

        match self.scopes.insert(symbol) {
            Ok(false) => true,
            Ok(true) => {
                self.warning(
                    "sem.shadowed",
                    format!(
                        "a declaração de \"{}\" oculta um símbolo global de mesmo nome",
                        name
                    ),
                    span,
                );
                true
            }
            Err(_) => {
                self.error(
                    "sem.redeclared",
                    format!("o símbolo \"{}\" já foi declarado neste escopo", name),
                    span,
                );
                false
            }
        }
    }

    fn literal_group(
        &mut self,
        declared: Ty,
        values: &[Expr],
        is_const: bool,
        span: Span,
        name: &str,
    ) {
        if values.is_empty() {
            return;
        }

        let mut types = Vec::with_capacity(values.len());
        for value in values {
            if is_const && !is_literal(value) {
                self.error(
                    "sem.const-initializer",
                    format!(
                        "a constante \"{}\" só pode ser inicializada com valores literais",
                        name
                    ),
                    value.span(),
                );
            }
            types.push(self.expr(value));
        }

        let first = types[0];
        if first != Ty::Unknown && types.iter().any(|ty| *ty != Ty::Unknown && *ty != first) {
            self.error(
                "sem.literal-type-mismatch",
                format!(
                    "os elementos da inicialização de \"{}\" precisam ter o mesmo tipo",
                    name
                ),
                span,
            );
            return;
        }

        if first != Ty::Unknown {
            self.check_compat(
                declared,
                first,
                span,
                &format!("na inicialização de \"{}\"", name),
            );
        }
    }

    // -----------------------------------------------------------------
    // Tamanho de vetores e matrizes
    // -----------------------------------------------------------------

    fn array_size(&mut self, expression: &Expr, span: Span) -> Option<i64> {
        let ty = self.expr(expression);

        if ty == Ty::Unknown {
            return None;
        }

        if ty != Ty::Inteiro {
            self.error(
                "sem.array-size-type",
                format!(
                    "o tamanho do vetor/matriz deve ser \"inteiro\", mas foi passado \"{}\"",
                    ty.name()
                ),
                span,
            );
            return None;
        }

        match self.const_size(expression) {
            Size::Int(value) if value > 0 => Some(value),
            Size::Int(_) => {
                self.error(
                    "sem.array-size-invalid",
                    "o tamanho do vetor/matriz deve ser maior que zero",
                    span,
                );
                None
            }
            Size::Invalid => {
                self.error(
                    "sem.array-size-not-constant",
                    "o tamanho do vetor/matriz deve ser uma constante inteira",
                    span,
                );
                None
            }
            Size::Unknown => None,
        }
    }

    fn const_size(&self, expression: &Expr) -> Size {
        match expression {
            Expr::Int { value, .. } => Size::Int(*value),
            Expr::Unary { op, expr, .. } => match (op, self.const_size(expr)) {
                (UnaryOp::Plus, Size::Int(value)) => Size::Int(value),
                (UnaryOp::Neg, Size::Int(value)) => {
                    value.checked_neg().map(Size::Int).unwrap_or(Size::Invalid)
                }
                (UnaryOp::BitNot, Size::Int(value)) => Size::Int(!value),
                (_, Size::Unknown) => Size::Unknown,
                _ => Size::Invalid,
            },
            Expr::Binary {
                op, left, right, ..
            } => {
                let left = self.const_size(left);
                let right = self.const_size(right);

                match (left, right) {
                    (Size::Int(left), Size::Int(right)) => const_size_binary(*op, left, right),
                    (Size::Unknown, _) | (_, Size::Unknown) => Size::Unknown,
                    _ => Size::Invalid,
                }
            }
            Expr::Paren { expr, .. } => self.const_size(expr),
            Expr::Ident(ident) => {
                if ident.scope.is_some() {
                    return Size::Unknown;
                }

                match self.scopes.get(&ident.name) {
                    Some(symbol) if symbol.is_const => {
                        symbol.const_int.map(Size::Int).unwrap_or(Size::Invalid)
                    }
                    _ => Size::Invalid,
                }
            }
            _ => Size::Invalid,
        }
    }

    // -----------------------------------------------------------------
    // Comandos
    // -----------------------------------------------------------------

    fn block(&mut self, statements: &[Stmt]) {
        self.scopes.push_scope();
        for statement in statements {
            self.stmt(statement);
        }
        self.scopes.pop_scope();
    }

    fn stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Declarations(declarations) => self.declarations(declarations),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.condition(condition);
                self.block(then_branch);
                if let Some(else_branch) = else_branch {
                    self.block(else_branch);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.condition(condition);
                self.block(body);
            }
            Stmt::DoWhile {
                body, condition, ..
            } => {
                self.block(body);
                self.condition(condition);
            }
            Stmt::For {
                init,
                condition,
                step,
                body,
                span,
            } => self.for_stmt(
                init.as_ref(),
                condition.as_ref(),
                step.as_ref(),
                body,
                *span,
            ),
            Stmt::Switch { value, cases, .. } => self.switch(value, cases),
            Stmt::Return { value, span } => self.return_stmt(value.as_ref(), *span),
            Stmt::Break { .. } => {}
            Stmt::Assign(assignment) => {
                self.assignment(assignment);
            }
            Stmt::Expr { expr, .. } => {
                self.expr(expr);
            }
        }
    }

    fn for_stmt(
        &mut self,
        init: Option<&ForInit>,
        condition: Option<&Expr>,
        step: Option<&ForStep>,
        body: &[Stmt],
        span: Span,
    ) {
        self.scopes.push_scope();

        if let Some(init) = init {
            match init {
                ForInit::Declarations(declarations) => self.declarations(declarations),
                ForInit::Assignment(assignment) => {
                    self.assignment(assignment);
                }
                ForInit::Expression(expr) => {
                    self.expr(expr);
                }
            }
        }

        match condition {
            Some(condition) => self.condition(condition),
            None => {
                self.error(
                    "sem.for-missing-condition",
                    "o comando \"para\" precisa de uma condição",
                    span,
                );
            }
        }

        if let Some(step) = step {
            match step {
                ForStep::Assignment(assignment) => {
                    self.assignment(assignment);
                }
                ForStep::Expression(expr) => {
                    self.expr(expr);
                }
            }
        }

        self.block(body);
        self.scopes.pop_scope();
    }

    fn switch(&mut self, value: &Expr, cases: &[Case]) {
        let switch_ty = self.expr(value);

        if switch_ty != Ty::Unknown && switch_ty != Ty::Inteiro && switch_ty != Ty::Caracter {
            self.error(
                "sem.switch-type",
                format!(
                    "o comando \"escolha\" espera \"inteiro\" ou \"caracter\", mas recebeu \"{}\"",
                    switch_ty.name()
                ),
                value.span(),
            );
        }

        for case in cases {
            if let Some(case_value) = &case.value {
                let case_ty = self.expr(case_value);

                if case_ty != Ty::Unknown {
                    if switch_ty == Ty::Inteiro || switch_ty == Ty::Caracter {
                        if case_ty != switch_ty {
                            self.error(
                                "sem.case-type",
                                format!(
                                    "este caso deveria ser do tipo \"{}\", mas recebeu \"{}\"",
                                    switch_ty.name(),
                                    case_ty.name()
                                ),
                                case_value.span(),
                            );
                        }
                    } else if case_ty != Ty::Inteiro && case_ty != Ty::Caracter {
                        self.error(
                            "sem.case-type",
                            format!(
                                "o caso deveria ser \"inteiro\" ou \"caracter\", mas recebeu \"{}\"",
                                case_ty.name()
                            ),
                            case_value.span(),
                        );
                    }
                }
            }

            self.block(&case.body);
        }
    }

    fn condition(&mut self, expression: &Expr) {
        let ty = self.expr(expression);
        if ty != Ty::Unknown && ty != Ty::Logico {
            self.error(
                "sem.condition-type",
                format!(
                    "a condição deve ser do tipo \"logico\", mas é do tipo \"{}\"",
                    ty.name()
                ),
                expression.span(),
            );
        }
    }

    fn return_stmt(&mut self, value: Option<&Expr>, span: Span) {
        let Some(function) = self.current_function.clone() else {
            self.error(
                "sem.return-outside-function",
                "o comando \"retorne\" só pode ser usado dentro de uma função",
                span,
            );
            if let Some(value) = value {
                self.expr(value);
            }
            return;
        };

        match value {
            Some(value) => {
                let ty = self.expr(value);

                if function.return_ty == Ty::Vazio {
                    self.error(
                        "sem.return-type",
                        format!(
                            "a função \"{}\" não retorna valor, mas \"retorne\" recebeu uma expressão",
                            function.name
                        ),
                        span,
                    );
                } else {
                    self.check_compat(
                        function.return_ty,
                        ty,
                        value.span(),
                        &format!("no retorno da função \"{}\"", function.name),
                    );
                }
            }
            None => {
                if function.return_ty != Ty::Vazio {
                    self.error(
                        "sem.return-type",
                        format!(
                            "a função \"{}\" deve retornar um valor do tipo \"{}\"",
                            function.name,
                            function.return_ty.name()
                        ),
                        span,
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // Atribuições
    // -----------------------------------------------------------------

    fn assignment(&mut self, assignment: &Assignment) -> Ty {
        self.assignment_impl(
            &assignment.target,
            assignment.op,
            &assignment.value,
            assignment.span,
        )
    }

    fn assignment_impl(&mut self, target: &Expr, op: AssignOp, value: &Expr, span: Span) -> Ty {
        let target = self.target(target);

        if let (Some(name), Some(previous)) = (&target.name, target.previous_initialized) {
            self.scopes.set_initialized(name, previous);
        }

        let value_ty = self.expr(value);

        let result_ty = match compound_op(op) {
            None => {
                self.check_compat(target.ty, value_ty, value.span(), "na atribuição");
                target.ty
            }
            Some(binary_op) => match binary_result(binary_op, target.ty, value_ty) {
                Some(result) => {
                    self.check_compat(target.ty, result, span, "na atribuição composta");
                    result
                }
                None => {
                    self.error(
                        "sem.incompatible-types",
                        format!(
                            "a operação \"{}\" não aceita os tipos \"{}\" e \"{}\"",
                            assign_op_name(op),
                            target.ty.name(),
                            value_ty.name()
                        ),
                        span,
                    );
                    Ty::Unknown
                }
            },
        };

        if let Some(name) = &target.name {
            self.scopes.set_initialized(name, true);
        }

        result_ty
    }

    fn target(&mut self, target: &Expr) -> Target {
        match target {
            Expr::Ident(ident) => {
                if let Some(scope) = &ident.scope {
                    self.check_library_scope(scope, ident.span);
                    self.error(
                        "sem.assign-to-library",
                        format!(
                            "não é possível atribuir um valor a \"{}\"",
                            ident.qualified_name()
                        ),
                        ident.span,
                    );
                    return Target {
                        ty: Ty::Unknown,
                        name: None,
                        previous_initialized: None,
                    };
                }

                let Some(symbol) = self.scopes.get(&ident.name) else {
                    self.error(
                        "sem.symbol-not-declared",
                        format!("o símbolo \"{}\" não foi declarado", ident.name),
                        ident.span,
                    );
                    return Target {
                        ty: Ty::Unknown,
                        name: None,
                        previous_initialized: None,
                    };
                };

                let kind = symbol.kind;
                let ty = symbol.ty;
                let is_const = symbol.is_const;
                let initialized = symbol.initialized;

                if kind == SymbolKind::Function {
                    self.error(
                        "sem.invalid-reference",
                        format!(
                            "\"{}\" é uma função e não pode receber atribuição",
                            ident.name
                        ),
                        ident.span,
                    );
                    return Target {
                        ty: Ty::Unknown,
                        name: None,
                        previous_initialized: None,
                    };
                }

                if is_const {
                    self.error(
                        "sem.assign-to-const",
                        format!(
                            "não é possível atribuir um valor à constante \"{}\"",
                            ident.name
                        ),
                        ident.span,
                    );
                }

                if kind != SymbolKind::Variable {
                    self.error(
                        "sem.invalid-reference",
                        format!(
                            "\"{}\" é um vetor/matriz; use um índice para atribuir",
                            ident.name
                        ),
                        ident.span,
                    );
                    return Target {
                        ty,
                        name: None,
                        previous_initialized: None,
                    };
                }

                self.scopes.set_initialized(&ident.name, true);

                Target {
                    ty,
                    name: Some(ident.name.clone()),
                    previous_initialized: Some(initialized),
                }
            }
            Expr::Index {
                scope,
                name,
                indices,
                span,
            } => Target {
                ty: self.index_target(scope.as_deref(), name, indices, *span),
                name: None,
                previous_initialized: None,
            },
            _ => {
                self.error(
                    "sem.assign-to-expression",
                    "o lado esquerdo de uma atribuição precisa ser uma variável ou um índice",
                    target.span(),
                );
                self.expr(target);
                Target {
                    ty: Ty::Unknown,
                    name: None,
                    previous_initialized: None,
                }
            }
        }
    }

    fn index_target(
        &mut self,
        scope: Option<&str>,
        name: &str,
        indices: &[Expr],
        span: Span,
    ) -> Ty {
        if let Some(scope) = scope {
            self.check_library_scope(scope, span);
            for index in indices {
                self.expr(index);
            }
            self.error(
                "sem.assign-to-library",
                format!("não é possível atribuir um valor a \"{}.{}\"", scope, name),
                span,
            );
            return Ty::Unknown;
        }

        for index in indices {
            let ty = self.expr(index);
            if ty != Ty::Unknown && ty != Ty::Inteiro {
                self.error(
                    "sem.invalid-index",
                    format!(
                        "o índice deve ser \"inteiro\", mas é do tipo \"{}\"",
                        ty.name()
                    ),
                    index.span(),
                );
            }
        }

        let Some(symbol) = self.scopes.get(name) else {
            self.error(
                "sem.symbol-not-declared",
                format!("o símbolo \"{}\" não foi declarado", name),
                span,
            );
            return Ty::Unknown;
        };

        let kind = symbol.kind;
        let ty = symbol.ty;
        let is_const = symbol.is_const;

        if is_const {
            self.error(
                "sem.assign-to-const",
                format!("não é possível atribuir um valor à constante \"{}\"", name),
                span,
            );
        }

        if kind.dims() as usize != indices.len() {
            self.error(
                "sem.invalid-reference",
                format!(
                    "\"{}\" não é um {} com {} índice(s)",
                    name,
                    if indices.len() == 1 {
                        "vetor"
                    } else {
                        "matriz"
                    },
                    indices.len()
                ),
                span,
            );
        }

        ty
    }

    // -----------------------------------------------------------------
    // Expressões
    // -----------------------------------------------------------------

    fn expr(&mut self, expression: &Expr) -> Ty {
        self.expr_ctx(expression, false)
    }

    fn expr_ctx(&mut self, expression: &Expr, allow_aggregate: bool) -> Ty {
        match expression {
            Expr::Ident(ident) => self.ident_ty(ident, allow_aggregate),
            Expr::Int { .. } => Ty::Inteiro,
            Expr::Real { .. } => Ty::Real,
            Expr::Bool { .. } => Ty::Logico,
            Expr::Char { .. } => Ty::Caracter,
            Expr::Str { .. } => Ty::Cadeia,
            Expr::Call {
                scope,
                name,
                args,
                span,
            } => self.call(scope.as_deref(), name, args, *span),
            Expr::Index {
                scope,
                name,
                indices,
                span,
            } => self.index_ty(scope.as_deref(), name, indices, *span),
            Expr::Unary { op, expr, .. } => {
                let ty = self.expr(expr);
                match unary_result(*op, ty) {
                    Some(result) => result,
                    None => {
                        self.error(
                            "sem.incompatible-types",
                            format!("a operação unária não aceita o tipo \"{}\"", ty.name()),
                            expression.span(),
                        );
                        Ty::Unknown
                    }
                }
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let left_ty = self.expr(left);
                let right_ty = self.expr(right);

                match binary_result(*op, left_ty, right_ty) {
                    Some(result) => result,
                    None => {
                        self.error(
                            "sem.incompatible-types",
                            format!(
                                "a operação \"{}\" não aceita os tipos \"{}\" e \"{}\"",
                                bin_op_name(*op),
                                left_ty.name(),
                                right_ty.name()
                            ),
                            expression.span(),
                        );
                        Ty::Unknown
                    }
                }
            }
            Expr::IncDec { target, span, .. } => self.inc_dec(target, *span),
            Expr::Paren { expr, .. } => self.expr_ctx(expr, allow_aggregate),
            Expr::Assign {
                target,
                op,
                value,
                span,
            } => self.assignment_impl(target, *op, value, *span),
        }
    }

    fn ident_ty(&mut self, ident: &Ident, allow_aggregate: bool) -> Ty {
        if let Some(scope) = &ident.scope {
            self.check_library_scope(scope, ident.span);
            return Ty::Unknown;
        }

        let Some(symbol) = self.scopes.get(&ident.name) else {
            self.error(
                "sem.symbol-not-declared",
                format!("o símbolo \"{}\" não foi declarado", ident.name),
                ident.span,
            );
            return Ty::Unknown;
        };

        let kind = symbol.kind;
        let ty = symbol.ty;
        let initialized = symbol.initialized;

        if kind == SymbolKind::Function {
            self.error(
                "sem.invalid-reference",
                format!("\"{}\" é uma função, não um valor", ident.name),
                ident.span,
            );
            return Ty::Unknown;
        }

        if !initialized && kind == SymbolKind::Variable {
            self.error(
                "sem.symbol-not-initialized",
                format!(
                    "o símbolo \"{}\" foi declarado, mas ainda não foi inicializado",
                    ident.name
                ),
                ident.span,
            );
        }

        if kind != SymbolKind::Variable && !allow_aggregate {
            self.error(
                "sem.invalid-reference",
                format!(
                    "\"{}\" é um vetor/matriz e precisa ser usado com índices",
                    ident.name
                ),
                ident.span,
            );
        }

        ty
    }

    fn index_ty(&mut self, scope: Option<&str>, name: &str, indices: &[Expr], span: Span) -> Ty {
        if let Some(scope) = scope {
            self.check_library_scope(scope, span);
            for index in indices {
                self.expr(index);
            }
            return Ty::Unknown;
        }

        for index in indices {
            let ty = self.expr(index);
            if ty != Ty::Unknown && ty != Ty::Inteiro {
                self.error(
                    "sem.invalid-index",
                    format!(
                        "o índice deve ser \"inteiro\", mas é do tipo \"{}\"",
                        ty.name()
                    ),
                    index.span(),
                );
            }
        }

        let Some(symbol) = self.scopes.get(name) else {
            self.error(
                "sem.symbol-not-declared",
                format!("o símbolo \"{}\" não foi declarado", name),
                span,
            );
            return Ty::Unknown;
        };

        let ty = symbol.ty;
        if symbol.kind.dims() as usize != indices.len() {
            self.error(
                "sem.invalid-reference",
                format!(
                    "\"{}\" não é um {} com {} índice(s)",
                    name,
                    if indices.len() == 1 {
                        "vetor"
                    } else {
                        "matriz"
                    },
                    indices.len()
                ),
                span,
            );
        }

        ty
    }

    fn inc_dec(&mut self, target: &Expr, span: Span) -> Ty {
        let ty = match target {
            Expr::Ident(ident) => {
                if let Some(scope) = &ident.scope {
                    self.check_library_scope(scope, ident.span);
                    self.error(
                        "sem.assign-to-library",
                        format!("não é possível alterar \"{}\"", ident.qualified_name()),
                        ident.span,
                    );
                    Ty::Unknown
                } else {
                    match self.scopes.get(&ident.name) {
                        None => {
                            self.error(
                                "sem.symbol-not-declared",
                                format!("o símbolo \"{}\" não foi declarado", ident.name),
                                ident.span,
                            );
                            Ty::Unknown
                        }
                        Some(symbol) => {
                            let kind = symbol.kind;
                            let ty = symbol.ty;
                            let is_const = symbol.is_const;
                            let initialized = symbol.initialized;

                            if kind == SymbolKind::Function || kind != SymbolKind::Variable {
                                self.error(
                                    "sem.invalid-reference",
                                    format!("\"{}\" não é uma variável escalar", ident.name),
                                    ident.span,
                                );
                                if !initialized && kind == SymbolKind::Variable {
                                    self.error(
                                        "sem.symbol-not-initialized",
                                        format!(
                                            "o símbolo \"{}\" ainda não foi inicializado",
                                            ident.name
                                        ),
                                        ident.span,
                                    );
                                }
                                return Ty::Unknown;
                            }

                            if is_const {
                                self.error(
                                    "sem.assign-to-const",
                                    format!(
                                        "não é possível alterar a constante \"{}\"",
                                        ident.name
                                    ),
                                    ident.span,
                                );
                            }

                            if !initialized {
                                self.error(
                                    "sem.symbol-not-initialized",
                                    format!(
                                        "o símbolo \"{}\" foi declarado, mas ainda não foi inicializado",
                                        ident.name
                                    ),
                                    ident.span,
                                );
                            }

                            self.scopes.set_initialized(&ident.name, true);
                            ty
                        }
                    }
                }
            }
            Expr::Index {
                scope,
                name,
                indices,
                span: index_span,
            } => self.index_target(scope.as_deref(), name, indices, *index_span),
            _ => {
                self.error(
                    "sem.assign-to-expression",
                    "o operador de incremento precisa de uma variável ou índice",
                    target.span(),
                );
                self.expr(target);
                Ty::Unknown
            }
        };

        if ty != Ty::Unknown && !ty.is_numeric() {
            self.error(
                "sem.incompatible-types",
                format!(
                    "o operador de incremento espera \"inteiro\" ou \"real\", mas recebeu \"{}\"",
                    ty.name()
                ),
                span,
            );
            return Ty::Unknown;
        }

        ty
    }

    // -----------------------------------------------------------------
    // Chamadas de função
    // -----------------------------------------------------------------

    fn call(&mut self, scope: Option<&str>, name: &str, args: &[Expr], span: Span) -> Ty {
        if let Some(scope) = scope {
            self.check_library_scope(scope, span);
            // Funções de biblioteca podem receber vetores/matrizes inteiros
            // (por exemplo `Util.numero_elementos(vetor)`).
            for arg in args {
                self.expr_ctx(arg, true);
            }
            return Ty::Unknown;
        }

        match name {
            "leia" => {
                self.leia(args, span);
                Ty::Vazio
            }
            "escreva" => {
                if args.is_empty() {
                    self.error(
                        "sem.wrong-argument-count",
                        "\"escreva\" espera pelo menos um argumento",
                        span,
                    );
                }

                for arg in args {
                    let ty = self.expr(arg);
                    if ty == Ty::Vazio {
                        self.error(
                            "sem.argument-type",
                            "\"escreva\" não aceita uma expressão sem retorno (\"vazio\")",
                            arg.span(),
                        );
                    }
                }

                Ty::Vazio
            }
            "sorteia" => {
                if args.len() != 2 {
                    self.error(
                        "sem.wrong-argument-count",
                        format!(
                            "\"sorteia\" espera 2 argumentos, mas recebeu {}",
                            args.len()
                        ),
                        span,
                    );
                }

                for (index, arg) in args.iter().enumerate() {
                    let ty = self.expr(arg);
                    if index < 2 && ty != Ty::Unknown && ty != Ty::Inteiro {
                        self.error(
                            "sem.argument-type",
                            format!(
                                "\"sorteia\" espera \"inteiro\" no argumento {}, mas recebeu \"{}\"",
                                index + 1,
                                ty.name()
                            ),
                            arg.span(),
                        );
                    }
                }

                Ty::Inteiro
            }
            "limpa" => {
                if !args.is_empty() {
                    self.error(
                        "sem.wrong-argument-count",
                        format!(
                            "\"limpa\" não espera argumentos, mas recebeu {}",
                            args.len()
                        ),
                        span,
                    );
                }

                for arg in args {
                    self.expr(arg);
                }

                Ty::Vazio
            }
            _ => self.user_call(name, args, span),
        }
    }

    fn user_call(&mut self, name: &str, args: &[Expr], span: Span) -> Ty {
        let Some(symbol) = self.scopes.get(name) else {
            self.error(
                "sem.symbol-not-declared",
                format!("a função \"{}\" não foi declarada", name),
                span,
            );
            for arg in args {
                self.expr(arg);
            }
            return Ty::Unknown;
        };

        if symbol.kind != SymbolKind::Function {
            self.error(
                "sem.invalid-reference",
                format!("\"{}\" não é uma função", name),
                span,
            );
            for arg in args {
                self.expr(arg);
            }
            return Ty::Unknown;
        }

        let Some(info) = symbol.function.clone() else {
            return Ty::Unknown;
        };

        if args.len() != info.params.len() {
            self.error(
                "sem.wrong-argument-count",
                format!(
                    "a função \"{}\" espera {} argumento(s), mas recebeu {}",
                    name,
                    info.params.len(),
                    args.len()
                ),
                span,
            );
        }

        for (index, arg) in args.iter().enumerate() {
            let Some(param) = info.params.get(index) else {
                self.expr(arg);
                self.error(
                    "sem.extra-argument",
                    format!(
                        "argumento {} excedente na chamada de \"{}\"",
                        index + 1,
                        name
                    ),
                    arg.span(),
                );
                continue;
            };

            self.argument(param, arg);
        }

        info.return_ty
    }

    fn argument(&mut self, param: &ParamInfo, arg: &Expr) {
        let actual_dims = self.expr_dims(arg);

        // Argumento de agregado (vetor/matriz).
        if param.dims > 0 {
            if actual_dims != Some(param.dims) {
                self.error(
                    "sem.invalid-argument",
                    format!(
                        "o parâmetro \"{}\" espera um {} de {} dimensão(ões)",
                        param.name,
                        if param.dims == 1 { "vetor" } else { "matriz" },
                        param.dims
                    ),
                    arg.span(),
                );

                let ty = if matches!(actual_dims, Some(dims) if dims > 0) {
                    self.expr_ctx(arg, true)
                } else {
                    self.expr(arg)
                };
                self.parameter_compat(param, ty, arg.span());
                return;
            }

            if param.by_ref {
                if let Expr::Ident(ident) = arg {
                    if let Some(symbol) = self.scopes.get(&ident.name)
                        && symbol.is_const
                    {
                        self.error(
                                "sem.invalid-argument",
                                format!(
                                    "o parâmetro \"{}\" é passado por referência e não aceita uma constante",
                                    param.name
                                ),
                                arg.span(),
                            );
                    }
                } else {
                    self.error(
                        "sem.invalid-argument",
                        format!(
                            "o parâmetro \"{}\" é passado por referência e exige um vetor/matriz",
                            param.name
                        ),
                        arg.span(),
                    );
                }
            }

            let ty = self.expr_ctx(arg, true);
            self.parameter_compat(param, ty, arg.span());
            return;
        }

        // Parâmetro escalar.
        if param.by_ref {
            let valid = match arg {
                Expr::Ident(ident) if ident.scope.is_none() => match self.scopes.get(&ident.name) {
                    Some(symbol) => symbol.kind == SymbolKind::Variable && !symbol.is_const,
                    None => {
                        self.error(
                            "sem.symbol-not-declared",
                            format!("o símbolo \"{}\" não foi declarado", ident.name),
                            ident.span,
                        );
                        return;
                    }
                },
                _ => false,
            };

            if !valid {
                self.error(
                    "sem.invalid-argument",
                    format!(
                        "o parâmetro \"{}\" é passado por referência e exige uma variável mutável",
                        param.name
                    ),
                    arg.span(),
                );
                self.expr(arg);
                return;
            }
        }

        if matches!(actual_dims, Some(dims) if dims > 0) {
            self.error(
                "sem.invalid-argument",
                format!(
                    "o parâmetro \"{}\" espera um valor escalar, mas recebeu um vetor/matriz",
                    param.name
                ),
                arg.span(),
            );
            let ty = self.expr_ctx(arg, true);
            self.parameter_compat(param, ty, arg.span());
            return;
        }

        let ty = self.expr(arg);
        self.parameter_compat(param, ty, arg.span());
    }

    /// Dimensões de um argumento quando ele é um agregado. Índices são
    /// elementos escalares, assim como no analisador original.
    fn expr_dims(&self, expression: &Expr) -> Option<u8> {
        match expression {
            Expr::Ident(ident) if ident.scope.is_none() => self
                .scopes
                .get(&ident.name)
                .map(|symbol| symbol.kind.dims()),
            _ => Some(0),
        }
    }

    fn parameter_compat(&mut self, param: &ParamInfo, value: Ty, span: Span) {
        match compatible(param.ty, value) {
            Compat::Ok(_) => {}
            Compat::Convert { from, to } => {
                if param.by_ref {
                    self.error(
                        "sem.invalid-argument",
                        format!(
                            "o parâmetro \"{}\" é passado por referência e não aceita conversão de \"{}\" para \"{}\"",
                            param.name,
                            from.name(),
                            to.name()
                        ),
                        span,
                    );
                } else {
                    self.conversion_warning(from, to, span);
                }
            }
            Compat::Incompatible => {
                self.error(
                    "sem.argument-type",
                    format!(
                        "o parâmetro \"{}\" espera \"{}\", mas recebeu \"{}\"",
                        param.name,
                        param.ty.name(),
                        value.name()
                    ),
                    span,
                );
            }
        }
    }

    fn leia(&mut self, args: &[Expr], span: Span) {
        if args.is_empty() {
            self.error(
                "sem.wrong-argument-count",
                "\"leia\" espera pelo menos um argumento",
                span,
            );
            return;
        }

        for arg in args {
            match arg {
                Expr::Ident(ident) if ident.scope.is_none() => match self.scopes.get(&ident.name) {
                    None => {
                        self.error(
                            "sem.symbol-not-declared",
                            format!("o símbolo \"{}\" não foi declarado", ident.name),
                            ident.span,
                        );
                    }
                    Some(symbol) => {
                        let invalid = symbol.kind != SymbolKind::Variable || symbol.is_const;
                        if invalid {
                            self.error(
                                "sem.invalid-argument",
                                "\"leia\" exige uma variável ou um elemento de vetor/matriz mutável",
                                ident.span,
                            );
                        } else {
                            self.scopes.set_initialized(&ident.name, true);
                        }
                    }
                },
                Expr::Ident(ident) => {
                    self.check_library_scope(
                        ident.scope.as_deref().unwrap_or_default(),
                        ident.span,
                    );
                    self.error(
                        "sem.invalid-argument",
                        "\"leia\" não aceita constantes de biblioteca",
                        ident.span,
                    );
                }
                Expr::Index {
                    scope,
                    name,
                    indices,
                    span,
                } => {
                    if let Some(scope) = scope {
                        self.check_library_scope(scope, *span);
                        self.error(
                            "sem.invalid-argument",
                            "\"leia\" não aceita elementos de biblioteca",
                            *span,
                        );
                        continue;
                    }

                    if let Some(symbol) = self.scopes.get(name)
                        && symbol.is_const
                    {
                        self.error(
                            "sem.invalid-argument",
                            format!(
                                "\"leia\" exige um elemento mutável, mas \"{}\" é constante",
                                name
                            ),
                            *span,
                        );
                    }

                    self.index_ty(None, name, indices, *span);
                }
                _ => {
                    self.error(
                        "sem.invalid-argument",
                        "\"leia\" exige uma variável ou um elemento de vetor/matriz mutável",
                        arg.span(),
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------
    // Compatibilidade
    // -----------------------------------------------------------------

    fn check_compat(&mut self, target: Ty, value: Ty, span: Span, context: &str) -> bool {
        match compatible(target, value) {
            Compat::Ok(_) => true,
            Compat::Convert { from, to } => {
                self.conversion_warning(from, to, span);
                true
            }
            Compat::Incompatible => {
                self.error(
                    "sem.incompatible-types",
                    format!(
                        "não é possível usar um valor do tipo \"{}\" como \"{}\" {}",
                        value.name(),
                        target.name(),
                        context
                    ),
                    span,
                );
                false
            }
        }
    }
}

// ---------------------------------------------------------------------
// Funções auxiliares
// ---------------------------------------------------------------------

fn function_info(function: &Function) -> FunctionInfo {
    FunctionInfo {
        name: function.name.name.clone(),
        return_ty: function.return_type.map(Ty::from_ast).unwrap_or(Ty::Vazio),
        params: function
            .params
            .iter()
            .map(|param| ParamInfo {
                name: param.name.name.clone(),
                ty: Ty::from_ast(param.ty),
                by_ref: param.by_ref,
                dims: param.dims,
                span: param.name.span,
            })
            .collect(),
        span: function.name.span,
    }
}

fn is_literal(expression: &Expr) -> bool {
    match expression {
        Expr::Int { .. }
        | Expr::Real { .. }
        | Expr::Bool { .. }
        | Expr::Char { .. }
        | Expr::Str { .. } => true,
        Expr::Unary { op, expr, .. } => {
            matches!(op, UnaryOp::Neg | UnaryOp::Plus) && is_literal(expr)
        }
        Expr::Paren { expr, .. } => is_literal(expr),
        _ => false,
    }
}

fn literal_int(expression: &Expr) -> Option<i64> {
    match expression {
        Expr::Int { value, .. } => Some(*value),
        Expr::Unary { op, expr, .. } => {
            let value = literal_int(expr)?;
            match op {
                UnaryOp::Plus => Some(value),
                UnaryOp::Neg => value.checked_neg(),
                _ => None,
            }
        }
        Expr::Paren { expr, .. } => literal_int(expr),
        _ => None,
    }
}

fn const_size_binary(op: BinOp, left: i64, right: i64) -> Size {
    let value = match op {
        BinOp::Add => left.checked_add(right),
        BinOp::Sub => left.checked_sub(right),
        BinOp::Mul => left.checked_mul(right),
        BinOp::Div => {
            if right == 0 {
                None
            } else {
                left.checked_div(right)
            }
        }
        BinOp::Mod => {
            if right == 0 {
                None
            } else {
                left.checked_rem(right)
            }
        }
        BinOp::Shl => {
            if !(0..64).contains(&right) {
                None
            } else {
                left.checked_shl(right as u32)
            }
        }
        BinOp::Shr => {
            if !(0..64).contains(&right) {
                None
            } else {
                left.checked_shr(right as u32)
            }
        }
        BinOp::BitAnd => Some(left & right),
        BinOp::BitOr => Some(left | right),
        BinOp::Xor => Some(left ^ right),
        _ => None,
    };

    value.map(Size::Int).unwrap_or(Size::Invalid)
}

fn stmt_guarantees_return(statement: &Stmt) -> bool {
    match statement {
        Stmt::Return { .. } => true,
        Stmt::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => list_guarantees_return(then_branch) && list_guarantees_return(else_branch),
        Stmt::Switch { cases, .. } => switch_guarantees_return(cases),
        _ => false,
    }
}

fn list_guarantees_return(statements: &[Stmt]) -> bool {
    statements.iter().any(stmt_guarantees_return)
}

fn switch_guarantees_return(cases: &[Case]) -> bool {
    let Some(last) = cases.last() else {
        return false;
    };

    // Precisa de um `caso contrario` no final.
    if last.value.is_some() {
        return false;
    }

    cases.iter().all(|case| {
        let before_break: Vec<&Stmt> = case
            .body
            .iter()
            .take_while(|statement| !matches!(statement, Stmt::Break { .. }))
            .collect();

        before_break
            .iter()
            .any(|statement| stmt_guarantees_return(statement))
    })
}

fn body_guarantees_return(statements: &[Stmt]) -> bool {
    list_guarantees_return(statements)
}

fn bin_op_name(op: BinOp) -> &'static str {
    match op {
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Gt => ">",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Ge => ">=",
        BinOp::And => "e",
        BinOp::Or => "ou",
        BinOp::Xor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::BitAnd => "&",
        BinOp::BitOr => "|",
    }
}

fn assign_op_name(op: AssignOp) -> &'static str {
    match op {
        AssignOp::Assign => "=",
        AssignOp::Add => "+=",
        AssignOp::Sub => "-=",
        AssignOp::Mul => "*=",
        AssignOp::Div => "/=",
        AssignOp::Mod => "%=",
        AssignOp::Xor => "^=",
        AssignOp::BitAnd => "&=",
        AssignOp::BitOr => "|=",
    }
}
