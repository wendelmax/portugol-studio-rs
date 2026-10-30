//! Compilador AST -> bytecode.
//!
//! Supõe um programa que já passou pela análise semântica: aqui só se
//! verifica o que é necessário para gerar código, e qualquer inconsistência
//! vira [`CompileError`] em vez de diagnóstico detalhado.
//!
//! Tipos estáticos só são usados onde o destino é conhecido (declaração,
//! atribuição, parâmetro, retorno, `leia`), para emitir [`Op::Convert`]. As
//! operações em si são resolvidas pela VM a partir das tags dos valores, o
//! que reproduz a promoção `inteiro -> real` do Java gerado pelo original.

use std::collections::HashMap;

use portugol_syntax::Span;
use portugol_syntax::ast::{
    AssignOp, Assignment, BinOp, Case, Declarations, Declarator, Expr, ForInit, ForStep,
    Function as AstFunction, Item, Program, Stmt, Type, UnaryOp,
};

use crate::op::{Function, GlobalInfo, LocalInfo, Module, NativeRef, Op, ValueType};

#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub message: String,
    pub span: Span,
}

type Result<T> = std::result::Result<T, CompileError>;

fn error<T>(message: impl Into<String>, span: Span) -> Result<T> {
    Err(CompileError {
        message: message.into(),
        span,
    })
}

/// Nome da função sintética de entrada.
pub const ENTRY_NAME: &str = "<programa>";

pub fn compile(program: &Program) -> Result<Module> {
    Compiler::new(program)?.run(program)
}

#[derive(Debug, Clone, Copy)]
enum Slot {
    Local(u32),
    Global(u32),
    /// Parâmetro por referência: o slot guarda uma referência.
    Deref(u32),
}

#[derive(Debug, Clone, Copy)]
struct Var {
    slot: Slot,
    ty: Type,
    dims: u8,
}

#[derive(Debug, Clone, Copy)]
struct ParamSig {
    ty: Type,
    dims: u8,
    by_ref: bool,
}

#[derive(Debug, Clone)]
struct Signature {
    index: u32,
    params: Vec<ParamSig>,
}

/// Destino de uma escrita, com os operandos já avaliados em temporários.
#[derive(Debug, Clone, Copy)]
enum Place {
    Var(Var),
    Elem {
        array: u32,
        index: u32,
        ty: Type,
    },
    MatElem {
        matrix: u32,
        row: u32,
        col: u32,
        ty: Type,
    },
}

impl Place {
    fn ty(self) -> Type {
        match self {
            Place::Var(var) => var.ty,
            Place::Elem { ty, .. } | Place::MatElem { ty, .. } => ty,
        }
    }

    fn is_aggregate(self) -> bool {
        matches!(self, Place::Var(var) if var.dims > 0)
    }
}

#[derive(Default)]
struct FnState {
    code: Vec<Op>,
    spans: Vec<Span>,
    locals: Vec<LocalInfo>,
    scopes: Vec<HashMap<String, Var>>,
    /// Saltos pendentes de `pare`, um nível por laço/escolha.
    breaks: Vec<Vec<usize>>,
    return_type: Option<Type>,
}

struct Compiler {
    functions: HashMap<String, Signature>,
    globals: HashMap<String, Var>,
    /// Nome ou alias -> nome da biblioteca.
    libraries: HashMap<String, String>,
    module: Module,
    strings: HashMap<String, u32>,
    natives: HashMap<NativeRef, u32>,
    native_consts: HashMap<NativeRef, u32>,
    f: FnState,
}

impl Compiler {
    fn new(program: &Program) -> Result<Self> {
        let mut compiler = Compiler {
            functions: HashMap::new(),
            globals: HashMap::new(),
            libraries: HashMap::new(),
            module: Module {
                functions: Vec::new(),
                entry: 0,
                globals: Vec::new(),
                strings: Vec::new(),
                natives: Vec::new(),
                native_consts: Vec::new(),
            },
            strings: HashMap::new(),
            natives: HashMap::new(),
            native_consts: HashMap::new(),
            f: FnState::default(),
        };

        for include in &program.includes {
            let library = include.library.name.clone();
            if let Some(alias) = &include.alias {
                compiler
                    .libraries
                    .insert(alias.name.clone(), library.clone());
            }
            compiler.libraries.insert(library.clone(), library);
        }

        // Índice 0 é a entrada; funções do usuário vêm depois.
        let mut index = 1;
        for item in &program.items {
            if let Item::Function(function) = item {
                let params = function
                    .params
                    .iter()
                    .map(|p| ParamSig {
                        ty: p.ty,
                        dims: p.dims,
                        by_ref: p.by_ref,
                    })
                    .collect();
                compiler
                    .functions
                    .insert(function.name.name.clone(), Signature { index, params });
                index += 1;
            }
        }

        // Globais são pré-registradas: são visíveis em qualquer função,
        // mesmo declaradas depois dela.
        for item in &program.items {
            if let Item::Declarations(decl) = item {
                for declarator in &decl.declarators {
                    let name = declarator.name().name.clone();
                    let dims = declarator_dims(declarator);
                    let slot = compiler.module.globals.len() as u32;
                    compiler.module.globals.push(GlobalInfo {
                        name: name.clone(),
                        ty: decl.ty,
                        dims,
                    });
                    compiler.globals.insert(
                        name,
                        Var {
                            slot: Slot::Global(slot),
                            ty: decl.ty,
                            dims,
                        },
                    );
                }
            }
        }

        Ok(compiler)
    }

    fn run(mut self, program: &Program) -> Result<Module> {
        self.begin_function(None);
        for item in &program.items {
            match item {
                Item::Declarations(decl) => self.declarations(decl, true)?,
                Item::Stmt(stmt) => self.stmt(stmt)?,
                Item::Function(_) => {}
            }
        }
        if let Some(signature) = self.functions.get("inicio") {
            let function = signature.index;
            let argc = signature.params.len() as u32;
            if argc == 1 {
                self.emit(Op::ProgramArgs, program.span);
            }
            self.emit(Op::Call { function, argc }, program.span);
            self.emit(Op::Pop, program.span);
        }
        self.emit(Op::PushVoid, program.span);
        self.emit(Op::Return, program.span);
        let entry = self.end_function(ENTRY_NAME.to_string(), 0, ValueType::Vazio, program.span);
        self.module.functions.push(entry);

        for item in &program.items {
            if let Item::Function(function) = item {
                let compiled = self.function(function)?;
                self.module.functions.push(compiled);
            }
        }

        Ok(self.module)
    }

    // -----------------------------------------------------------------
    // Funções
    // -----------------------------------------------------------------

    fn begin_function(&mut self, return_type: Option<Type>) {
        self.f = FnState {
            scopes: vec![HashMap::new()],
            return_type,
            ..FnState::default()
        };
    }

    fn end_function(
        &mut self,
        name: String,
        arity: u32,
        return_type: ValueType,
        span: Span,
    ) -> Function {
        let state = std::mem::take(&mut self.f);
        Function {
            name,
            arity,
            return_type,
            locals: state.locals,
            code: state.code,
            spans: state.spans,
            span,
        }
    }

    fn function(&mut self, function: &AstFunction) -> Result<Function> {
        self.begin_function(Some(function.return_type.unwrap_or(Type::Vazio)));

        for param in &function.params {
            let slot = self.new_local(&param.name.name, param.ty, param.dims, param.by_ref);
            let slot = if param.by_ref {
                Slot::Deref(slot)
            } else {
                Slot::Local(slot)
            };
            self.declare(
                &param.name.name,
                Var {
                    slot,
                    ty: param.ty,
                    dims: param.dims,
                },
            );
        }

        self.block(&function.body)?;
        let end = Span::new(function.span.end.saturating_sub(1), function.span.end);
        self.emit(Op::PushVoid, end);
        self.emit(Op::Return, end);

        let return_type = function
            .return_type
            .map(ValueType::from)
            .unwrap_or(ValueType::Vazio);
        Ok(self.end_function(
            function.name.name.clone(),
            function.params.len() as u32,
            return_type,
            function.span,
        ))
    }

    // -----------------------------------------------------------------
    // Emissão
    // -----------------------------------------------------------------

    fn emit(&mut self, op: Op, span: Span) -> usize {
        if let Op::Convert(ty) = op {
            // `convert` só altera inteiro/real/caracter, e é inócuo logo após um
            // literal do próprio tipo.
            let redundant = matches!(
                (ty, self.f.code.last()),
                (ValueType::Logico | ValueType::Cadeia | ValueType::Vazio, _)
                    | (ValueType::Inteiro, Some(Op::PushInt(_)))
                    | (ValueType::Real, Some(Op::PushReal(_)))
                    | (ValueType::Caracter, Some(Op::PushChar(_)))
            );
            if redundant {
                return self.f.code.len().saturating_sub(1);
            }
        }
        self.f.code.push(op);
        self.f.spans.push(span);
        self.f.code.len() - 1
    }

    fn here(&self) -> u32 {
        self.f.code.len() as u32
    }

    fn patch(&mut self, at: usize) {
        let target = self.here();
        match &mut self.f.code[at] {
            Op::Jump(t)
            | Op::JumpIfFalse(t)
            | Op::JumpIfTrue(t)
            | Op::JumpIfFalseKeep(t)
            | Op::JumpIfTrueKeep(t) => *t = target,
            other => unreachable!("patch em instrução que não é salto: {other:?}"),
        }
    }

    fn string(&mut self, value: &str) -> u32 {
        if let Some(&index) = self.strings.get(value) {
            return index;
        }
        let index = self.module.strings.len() as u32;
        self.module.strings.push(value.to_string());
        self.strings.insert(value.to_string(), index);
        index
    }

    fn library(&self, scope: &str, span: Span) -> Result<String> {
        match self.libraries.get(scope) {
            Some(library) => Ok(library.clone()),
            None => error(format!("biblioteca \"{scope}\" não incluída"), span),
        }
    }

    fn native(&mut self, library: String, name: &str) -> u32 {
        let key = NativeRef {
            library,
            name: name.to_string(),
        };
        if let Some(&index) = self.natives.get(&key) {
            return index;
        }
        let index = self.module.natives.len() as u32;
        self.module.natives.push(key.clone());
        self.natives.insert(key, index);
        index
    }

    fn native_const(&mut self, library: String, name: &str) -> u32 {
        let key = NativeRef {
            library,
            name: name.to_string(),
        };
        if let Some(&index) = self.native_consts.get(&key) {
            return index;
        }
        let index = self.module.native_consts.len() as u32;
        self.module.native_consts.push(key.clone());
        self.native_consts.insert(key, index);
        index
    }

    // -----------------------------------------------------------------
    // Escopos
    // -----------------------------------------------------------------

    fn new_local(&mut self, name: &str, ty: Type, dims: u8, by_ref: bool) -> u32 {
        self.f.locals.push(LocalInfo {
            name: name.to_string(),
            ty,
            dims,
            by_ref,
        });
        (self.f.locals.len() - 1) as u32
    }

    fn temp(&mut self) -> u32 {
        self.new_local("", Type::Inteiro, 0, false)
    }

    fn declare(&mut self, name: &str, var: Var) {
        self.f
            .scopes
            .last_mut()
            .expect("sempre há um escopo")
            .insert(name.to_string(), var);
    }

    fn lookup(&self, name: &str, span: Span) -> Result<Var> {
        for scope in self.f.scopes.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Ok(*var);
            }
        }
        match self.globals.get(name) {
            Some(var) => Ok(*var),
            None => error(format!("variável \"{name}\" não declarada"), span),
        }
    }

    fn scoped<T>(&mut self, body: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.f.scopes.push(HashMap::new());
        let result = body(self);
        self.f.scopes.pop();
        result
    }

    fn block(&mut self, stmts: &[Stmt]) -> Result<()> {
        self.scoped(|c| {
            for stmt in stmts {
                c.stmt(stmt)?;
            }
            Ok(())
        })
    }

    // -----------------------------------------------------------------
    // Declarações
    // -----------------------------------------------------------------

    fn declarations(&mut self, decl: &Declarations, global: bool) -> Result<()> {
        for declarator in &decl.declarators {
            let span = declarator.span();
            self.declarator_value(decl.ty, declarator)?;

            let name = &declarator.name().name;
            if global {
                let var = self.lookup(name, span)?;
                self.store_var(var, span);
            } else {
                let dims = declarator_dims(declarator);
                let slot = self.new_local(name, decl.ty, dims, false);
                self.emit(Op::StoreLocal(slot), span);
                // Declarado só depois do inicializador, como no Java.
                self.declare(
                    name,
                    Var {
                        slot: Slot::Local(slot),
                        ty: decl.ty,
                        dims,
                    },
                );
            }
        }
        Ok(())
    }

    /// Empilha o valor inicial de um declarador.
    fn declarator_value(&mut self, ty: Type, declarator: &Declarator) -> Result<()> {
        let value_type = ValueType::from(ty);
        match declarator {
            Declarator::Variable { init, span, .. } => match init {
                Some(init) => {
                    self.expr(init)?;
                    self.emit(Op::Convert(value_type), *span);
                }
                None => {
                    self.emit(Op::PushDefault(value_type), *span);
                }
            },
            Declarator::Array {
                size, init, span, ..
            } => match (init, size) {
                (Some(values), _) => {
                    for value in values {
                        self.expr(value)?;
                        self.emit(Op::Convert(value_type), value.span());
                    }
                    self.emit(Op::ArrayLit(values.len() as u32), *span);
                }
                (None, Some(size)) => {
                    self.expr(size)?;
                    self.emit(Op::NewArray(value_type), *span);
                }
                (None, None) => {
                    return error("vetor sem tamanho e sem valores iniciais", *span);
                }
            },
            Declarator::Matrix {
                rows,
                cols,
                init,
                span,
                ..
            } => match (init, rows, cols) {
                (Some(values), _, _) => {
                    let row_count = values.len() as u32;
                    let col_count = values.first().map_or(0, |row| row.len()) as u32;
                    for row in values {
                        if row.len() as u32 != col_count {
                            return error("linhas da matriz com tamanhos diferentes", *span);
                        }
                        for value in row {
                            self.expr(value)?;
                            self.emit(Op::Convert(value_type), value.span());
                        }
                    }
                    self.emit(
                        Op::MatrixLit {
                            rows: row_count,
                            cols: col_count,
                        },
                        *span,
                    );
                }
                (None, Some(rows), Some(cols)) => {
                    self.expr(rows)?;
                    self.expr(cols)?;
                    self.emit(Op::NewMatrix(value_type), *span);
                }
                _ => return error("matriz sem tamanho e sem valores iniciais", *span),
            },
        }
        Ok(())
    }

    // -----------------------------------------------------------------
    // Comandos
    // -----------------------------------------------------------------

    fn stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Declarations(decl) => self.declarations(decl, false)?,
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                span,
            } => {
                self.expr(condition)?;
                let to_else = self.emit(Op::JumpIfFalse(0), *span);
                self.block(then_branch)?;
                match else_branch {
                    Some(else_branch) => {
                        let to_end = self.emit(Op::Jump(0), *span);
                        self.patch(to_else);
                        self.block(else_branch)?;
                        self.patch(to_end);
                    }
                    None => self.patch(to_else),
                }
            }
            Stmt::While {
                condition,
                body,
                span,
            } => {
                let start = self.here();
                self.expr(condition)?;
                let to_end = self.emit(Op::JumpIfFalse(0), *span);
                self.loop_body(body, |c| {
                    c.emit(Op::Jump(start), *span);
                    Ok(())
                })?;
                self.patch(to_end);
                self.finish_breaks();
            }
            Stmt::DoWhile {
                body,
                condition,
                span,
            } => {
                let start = self.here();
                self.loop_body(body, |c| {
                    c.expr(condition)?;
                    c.emit(Op::JumpIfTrue(start), *span);
                    Ok(())
                })?;
                self.finish_breaks();
            }
            Stmt::For {
                init,
                condition,
                step,
                body,
                span,
            } => {
                self.scoped(|c| {
                    match init {
                        Some(ForInit::Declarations(decl)) => c.declarations(decl, false)?,
                        Some(ForInit::Assignment(assignment)) => c.assignment(assignment, false)?,
                        Some(ForInit::Expression(expr)) => {
                            c.discard(expr)?;
                        }
                        None => {}
                    }
                    let start = c.here();
                    let to_end = match condition {
                        Some(condition) => {
                            c.expr(condition)?;
                            Some(c.emit(Op::JumpIfFalse(0), *span))
                        }
                        None => None,
                    };
                    c.loop_body(body, |c| {
                        match step {
                            Some(ForStep::Assignment(assignment)) => {
                                c.assignment(assignment, false)?
                            }
                            Some(ForStep::Expression(expr)) => {
                                c.discard(expr)?;
                            }
                            None => {}
                        }
                        c.emit(Op::Jump(start), *span);
                        Ok(())
                    })?;
                    if let Some(to_end) = to_end {
                        c.patch(to_end);
                    }
                    c.finish_breaks();
                    Ok(())
                })?;
            }
            Stmt::Switch { value, cases, span } => self.switch(value, cases, *span)?,
            Stmt::Return { value, span } => {
                let return_type = self.f.return_type.unwrap_or(Type::Vazio);
                match value {
                    Some(value) => {
                        self.expr(value)?;
                        if return_type != Type::Vazio {
                            self.emit(Op::Convert(return_type.into()), *span);
                        }
                    }
                    None => {
                        self.emit(Op::PushVoid, *span);
                    }
                }
                self.emit(Op::Return, *span);
            }
            Stmt::Break { span } => {
                let jump = self.emit(Op::Jump(0), *span);
                match self.f.breaks.last_mut() {
                    Some(breaks) => breaks.push(jump),
                    None => return error("\"pare\" fora de laço ou escolha", *span),
                }
            }
            Stmt::Assign(assignment) => self.assignment(assignment, false)?,
            Stmt::Expr { expr, .. } => self.discard(expr)?,
        }
        Ok(())
    }

    /// Corpo de laço com seu nível de `pare`; `tail` gera o salto de volta.
    /// O chamador resolve os `pare` com [`Self::finish_breaks`] depois de
    /// emitir o que vem após o laço.
    fn loop_body(
        &mut self,
        body: &[Stmt],
        tail: impl FnOnce(&mut Self) -> Result<()>,
    ) -> Result<()> {
        self.f.breaks.push(Vec::new());
        self.block(body)?;
        tail(self)
    }

    fn finish_breaks(&mut self) {
        let breaks = self.f.breaks.pop().unwrap_or_default();
        for jump in breaks {
            self.patch(jump);
        }
    }

    fn switch(&mut self, value: &Expr, cases: &[Case], span: Span) -> Result<()> {
        let tmp = self.temp();
        self.expr(value)?;
        self.emit(Op::StoreLocal(tmp), span);

        let mut to_body = Vec::with_capacity(cases.len());
        for case in cases {
            match &case.value {
                Some(case_value) => {
                    self.emit(Op::LoadLocal(tmp), case.span);
                    self.expr(case_value)?;
                    self.emit(Op::Eq, case.span);
                    to_body.push(Some(self.emit(Op::JumpIfTrue(0), case.span)));
                }
                None => to_body.push(None),
            }
        }
        // Sem caso correspondente: vai para `caso contrario` ou para o fim.
        let to_default = self.emit(Op::Jump(0), span);
        let mut default_patched = false;

        self.f.breaks.push(Vec::new());
        self.scoped(|c| {
            for (case, jump) in cases.iter().zip(&to_body) {
                match jump {
                    Some(jump) => c.patch(*jump),
                    None => {
                        c.patch(to_default);
                        default_patched = true;
                    }
                }
                // Os casos compartilham o bloco e "caem" no próximo sem `pare`,
                // como o `switch` do Java gerado pelo original.
                for stmt in &case.body {
                    c.stmt(stmt)?;
                }
            }
            Ok(())
        })?;
        if !default_patched {
            self.patch(to_default);
        }
        self.finish_breaks();
        Ok(())
    }

    // -----------------------------------------------------------------
    // Atribuição e lugares
    // -----------------------------------------------------------------

    fn assignment(&mut self, assignment: &Assignment, keep: bool) -> Result<()> {
        self.assign(
            &assignment.target,
            assignment.op,
            &assignment.value,
            keep,
            assignment.span,
        )
    }

    fn assign(
        &mut self,
        target: &Expr,
        op: AssignOp,
        value: &Expr,
        keep: bool,
        span: Span,
    ) -> Result<()> {
        let place = self.place(target)?;
        match compound_op(op) {
            None => self.expr(value)?,
            Some(op) => {
                self.load_place(place, span);
                self.expr(value)?;
                self.emit(op, span);
            }
        }
        if !place.is_aggregate() {
            self.emit(Op::Convert(place.ty().into()), span);
        }
        if keep {
            self.emit(Op::Dup, span);
        }
        self.store_place(place, span);
        Ok(())
    }

    /// Avalia o alvo de uma escrita. Vetor e índices vão para temporários, para
    /// que leitura e escrita (em `+=`, `++`) não os avaliem duas vezes e a ordem
    /// de avaliação seja a do Java: alvo, índices e só então o valor.
    fn place(&mut self, target: &Expr) -> Result<Place> {
        match target {
            Expr::Ident(ident) if ident.scope.is_none() => {
                Ok(Place::Var(self.lookup(&ident.name, ident.span)?))
            }
            Expr::Index {
                scope: None,
                name,
                indices,
                span,
            } => {
                let var = self.lookup(name, *span)?;
                let base = self.temp();
                self.load_var(var, *span);
                self.emit(Op::StoreLocal(base), *span);
                let mut temps = Vec::with_capacity(indices.len());
                for index in indices {
                    let tmp = self.temp();
                    self.expr(index)?;
                    self.emit(Op::StoreLocal(tmp), index.span());
                    temps.push(tmp);
                }
                match temps.as_slice() {
                    [index] => Ok(Place::Elem {
                        array: base,
                        index: *index,
                        ty: var.ty,
                    }),
                    [row, col] => Ok(Place::MatElem {
                        matrix: base,
                        row: *row,
                        col: *col,
                        ty: var.ty,
                    }),
                    _ => error("número de índices inválido", *span),
                }
            }
            Expr::Paren { expr, .. } => self.place(expr),
            other => error("expressão não pode receber atribuição", other.span()),
        }
    }

    fn load_place(&mut self, place: Place, span: Span) {
        match place {
            Place::Var(var) => self.load_var(var, span),
            Place::Elem { array, index, .. } => {
                self.emit(Op::LoadLocal(array), span);
                self.emit(Op::LoadLocal(index), span);
                self.emit(Op::LoadElem, span);
            }
            Place::MatElem {
                matrix, row, col, ..
            } => {
                self.emit(Op::LoadLocal(matrix), span);
                self.emit(Op::LoadLocal(row), span);
                self.emit(Op::LoadLocal(col), span);
                self.emit(Op::LoadMatElem, span);
            }
        }
    }

    /// `[x]` -> `[]`.
    fn store_place(&mut self, place: Place, span: Span) {
        match place {
            Place::Var(var) => self.store_var(var, span),
            Place::Elem { array, index, .. } => {
                self.emit(Op::LoadLocal(array), span);
                self.emit(Op::LoadLocal(index), span);
                self.emit(Op::StoreElem, span);
            }
            Place::MatElem {
                matrix, row, col, ..
            } => {
                self.emit(Op::LoadLocal(matrix), span);
                self.emit(Op::LoadLocal(row), span);
                self.emit(Op::LoadLocal(col), span);
                self.emit(Op::StoreMatElem, span);
            }
        }
    }

    fn load_var(&mut self, var: Var, span: Span) {
        let op = match var.slot {
            Slot::Local(slot) => Op::LoadLocal(slot),
            Slot::Global(slot) => Op::LoadGlobal(slot),
            Slot::Deref(slot) => Op::LoadDeref(slot),
        };
        self.emit(op, span);
    }

    fn store_var(&mut self, var: Var, span: Span) {
        let op = match var.slot {
            Slot::Local(slot) => Op::StoreLocal(slot),
            Slot::Global(slot) => Op::StoreGlobal(slot),
            Slot::Deref(slot) => Op::StoreDeref(slot),
        };
        self.emit(op, span);
    }

    /// Empilha uma referência para o argumento de um parâmetro `&`.
    fn reference(&mut self, arg: &Expr) -> Result<()> {
        match arg {
            Expr::Ident(ident) if ident.scope.is_none() => {
                let var = self.lookup(&ident.name, ident.span)?;
                let op = match var.slot {
                    Slot::Local(slot) => Op::RefLocal(slot),
                    Slot::Global(slot) => Op::RefGlobal(slot),
                    // Repassa a referência recebida.
                    Slot::Deref(slot) => Op::LoadLocal(slot),
                };
                self.emit(op, ident.span);
            }
            Expr::Index {
                scope: None,
                name,
                indices,
                span,
            } => {
                let var = self.lookup(name, *span)?;
                self.load_var(var, *span);
                for index in indices {
                    self.expr(index)?;
                }
                let op = if indices.len() == 2 {
                    Op::RefMatElem
                } else {
                    Op::RefElem
                };
                self.emit(op, *span);
            }
            Expr::Paren { expr, .. } => self.reference(expr)?,
            other => {
                // Não é lvalue: a função recebe uma cópia num temporário.
                let tmp = self.temp();
                self.expr(other)?;
                self.emit(Op::StoreLocal(tmp), other.span());
                self.emit(Op::RefLocal(tmp), other.span());
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------
    // Expressões
    // -----------------------------------------------------------------

    fn expr(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::Ident(ident) => match &ident.scope {
                None => {
                    let var = self.lookup(&ident.name, ident.span)?;
                    self.load_var(var, ident.span);
                }
                Some(scope) => {
                    let library = self.library(scope, ident.span)?;
                    let index = self.native_const(library, &ident.name);
                    self.emit(Op::LoadNativeConst(index), ident.span);
                }
            },
            Expr::Int { value, span } => {
                let Ok(value) = i32::try_from(*value) else {
                    return error(
                        format!("o valor {value} não cabe em um inteiro de 32 bits"),
                        *span,
                    );
                };
                self.emit(Op::PushInt(value), *span);
            }
            Expr::Real { value, span } => {
                self.emit(Op::PushReal(*value), *span);
            }
            Expr::Bool { value, span } => {
                self.emit(Op::PushBool(*value), *span);
            }
            Expr::Char { value, span } => {
                self.emit(Op::PushChar(*value), *span);
            }
            Expr::Str { value, span } => {
                let index = self.string(value);
                self.emit(Op::PushStr(index), *span);
            }
            Expr::Call {
                scope,
                name,
                args,
                span,
            } => self.call(scope.as_deref(), name, args, *span)?,
            Expr::Index {
                scope,
                name,
                indices,
                span,
            } => {
                if scope.is_some() {
                    return error("vetores de biblioteca não são suportados", *span);
                }
                let var = self.lookup(name, *span)?;
                self.load_var(var, *span);
                for index in indices {
                    self.expr(index)?;
                }
                let op = match indices.len() {
                    1 => Op::LoadElem,
                    2 => Op::LoadMatElem,
                    _ => return error("número de índices inválido", *span),
                };
                self.emit(op, *span);
            }
            Expr::Unary { op, expr, span } => {
                self.expr(expr)?;
                match op {
                    UnaryOp::Neg => {
                        self.emit(Op::Neg, *span);
                    }
                    UnaryOp::Plus => {}
                    UnaryOp::Not => {
                        self.emit(Op::Not, *span);
                    }
                    UnaryOp::BitNot => {
                        self.emit(Op::BitNot, *span);
                    }
                }
            }
            Expr::Binary {
                op,
                left,
                right,
                span,
            } => match op {
                BinOp::And | BinOp::Or => {
                    self.expr(left)?;
                    let jump = if *op == BinOp::And {
                        Op::JumpIfFalseKeep(0)
                    } else {
                        Op::JumpIfTrueKeep(0)
                    };
                    let jump = self.emit(jump, *span);
                    self.expr(right)?;
                    self.patch(jump);
                }
                _ => {
                    self.expr(left)?;
                    self.expr(right)?;
                    self.emit(binary_op(*op), *span);
                }
            },
            Expr::IncDec {
                target,
                delta,
                prefix,
                span,
            } => self.inc_dec(target, *delta, Some(*prefix), *span)?,
            Expr::Paren { expr, .. } => self.expr(expr)?,
            Expr::Assign {
                target,
                op,
                value,
                span,
            } => self.assign(target, *op, value, true, *span)?,
        }
        Ok(())
    }

    /// `++`/`--`. Com `result = None` (uso como comando) nada fica na pilha;
    /// senão fica o valor novo (prefixo) ou o antigo (sufixo).
    fn inc_dec(
        &mut self,
        target: &Expr,
        delta: i32,
        result: Option<bool>,
        span: Span,
    ) -> Result<()> {
        let place = self.place(target)?;
        self.load_place(place, span);
        if result == Some(false) {
            // Valor antigo fica embaixo, como resultado da expressão.
            self.emit(Op::Dup, span);
        }
        self.emit(Op::PushInt(1), span);
        self.emit(if delta > 0 { Op::Add } else { Op::Sub }, span);
        self.emit(Op::Convert(place.ty().into()), span);
        if result == Some(true) {
            self.emit(Op::Dup, span);
        }
        self.store_place(place, span);
        Ok(())
    }

    /// Expressão usada como comando: o resultado é descartado.
    fn discard(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::IncDec {
                target,
                delta,
                span,
                ..
            } => self.inc_dec(target, *delta, None, *span),
            Expr::Assign {
                target,
                op,
                value,
                span,
            } => self.assign(target, *op, value, false, *span),
            _ => {
                self.expr(expr)?;
                self.emit(Op::Pop, expr.span());
                Ok(())
            }
        }
    }

    fn call(&mut self, scope: Option<&str>, name: &str, args: &[Expr], span: Span) -> Result<()> {
        if let Some(scope) = scope {
            let library = self.library(scope, span)?;
            for arg in args {
                self.expr(arg)?;
            }
            let native = self.native(library, name);
            self.emit(
                Op::CallNative {
                    native,
                    argc: args.len() as u32,
                },
                span,
            );
            return Ok(());
        }

        match name {
            "escreva" => {
                for arg in args {
                    self.expr(arg)?;
                }
                self.emit(Op::Write(args.len() as u32), span);
                self.emit(Op::PushVoid, span);
                return Ok(());
            }
            "leia" => {
                for arg in args {
                    let place = self.place(arg)?;
                    self.emit(Op::Read(place.ty().into()), arg.span());
                    self.store_place(place, arg.span());
                }
                self.emit(Op::PushVoid, span);
                return Ok(());
            }
            "limpa" => {
                self.emit(Op::Clear, span);
                self.emit(Op::PushVoid, span);
                return Ok(());
            }
            "sorteia" => {
                for arg in args {
                    self.expr(arg)?;
                    self.emit(Op::Convert(ValueType::Inteiro), arg.span());
                }
                self.emit(Op::Random, span);
                return Ok(());
            }
            _ => {}
        }

        let Some(signature) = self.functions.get(name).cloned() else {
            return error(format!("função \"{name}\" não declarada"), span);
        };
        if signature.params.len() != args.len() {
            return error(
                format!(
                    "a função \"{name}\" espera {} argumento(s), mas recebeu {}",
                    signature.params.len(),
                    args.len()
                ),
                span,
            );
        }
        for (arg, param) in args.iter().zip(&signature.params) {
            if param.by_ref {
                self.reference(arg)?;
            } else {
                self.expr(arg)?;
                if param.dims == 0 {
                    self.emit(Op::Convert(param.ty.into()), arg.span());
                }
            }
        }
        self.emit(
            Op::Call {
                function: signature.index,
                argc: args.len() as u32,
            },
            span,
        );
        Ok(())
    }
}

fn declarator_dims(declarator: &Declarator) -> u8 {
    match declarator {
        Declarator::Variable { .. } => 0,
        Declarator::Array { .. } => 1,
        Declarator::Matrix { .. } => 2,
    }
}

fn binary_op(op: BinOp) -> Op {
    match op {
        BinOp::Mul => Op::Mul,
        BinOp::Div => Op::Div,
        BinOp::Mod => Op::Mod,
        BinOp::Add => Op::Add,
        BinOp::Sub => Op::Sub,
        BinOp::Eq => Op::Eq,
        BinOp::Ne => Op::Ne,
        BinOp::Gt => Op::Gt,
        BinOp::Lt => Op::Lt,
        BinOp::Le => Op::Le,
        BinOp::Ge => Op::Ge,
        BinOp::Xor => Op::BitXor,
        BinOp::Shl => Op::Shl,
        BinOp::Shr => Op::Shr,
        BinOp::BitAnd => Op::BitAnd,
        BinOp::BitOr => Op::BitOr,
        BinOp::And | BinOp::Or => unreachable!("tratados com curto-circuito"),
    }
}

fn compound_op(op: AssignOp) -> Option<Op> {
    match op {
        AssignOp::Assign => None,
        AssignOp::Add => Some(Op::Add),
        AssignOp::Sub => Some(Op::Sub),
        AssignOp::Mul => Some(Op::Mul),
        AssignOp::Div => Some(Op::Div),
        AssignOp::Mod => Some(Op::Mod),
        AssignOp::Xor => Some(Op::BitXor),
        AssignOp::BitAnd => Some(Op::BitAnd),
        AssignOp::BitOr => Some(Op::BitOr),
    }
}
