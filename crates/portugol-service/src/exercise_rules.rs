//! Regras estáticas opcionais dos exercícios .pex. Os nomes Java identificam
//! regras declarativas conhecidas. A execução usa a AST e os escopos do port.

use std::collections::{HashMap, HashSet};

use portugol_syntax::ast::*;

enum Rule {
    Required(Vec<String>),
    Forbidden(Vec<String>),
    ReadWriteOrder,
    ReadBeforeAssignment,
    Auxiliary,
    Unused,
    Unsupported(String),
}

pub(super) struct Model {
    signature: Vec<&'static str>,
    rules: Vec<Rule>,
}

pub(super) fn models(root: roxmltree::Node<'_, '_>) -> Result<Vec<Model>, String> {
    root.children()
        .filter(|node| node.has_tag_name("solucoes"))
        .flat_map(|node| node.children())
        .filter(|node| node.has_tag_name("solucao"))
        .map(|model| {
            let code = model
                .children()
                .find(|node| node.has_tag_name("modelo"))
                .and_then(|node| node.text())
                .ok_or("Uma solução .pex está sem o código modelo.")?;
            let parsed = portugol_syntax::parse(code);
            if parsed.has_errors() {
                return Err("O código modelo do exercício contém erro de sintaxe.".into());
            }
            let rules = model
                .descendants()
                .filter(|node| node.has_tag_name("visitor"))
                .map(parse_rule)
                .collect();
            Ok(Model {
                signature: Facts::for_program(&parsed.program).nodes,
                rules,
            })
        })
        .collect()
}

fn simple_class(name: &str) -> &str {
    name.trim().rsplit('.').next().unwrap_or_default()
}

fn parse_rule(node: roxmltree::Node<'_, '_>) -> Rule {
    let name = node.attribute("class-name").unwrap_or_default();
    let arguments = || {
        node.descendants()
            .filter(|child| child.is_element() && *child != node)
            .filter_map(|child| {
                child.attribute("valor").or_else(|| {
                    if child.has_tag_name("valor") {
                        child.text()
                    } else {
                        None
                    }
                })
            })
            .map(simple_class)
            .filter(|arg| !arg.is_empty())
            .map(str::to_owned)
            .collect()
    };
    match simple_class(name) {
        "MandatoryInstructions" => Rule::Required(arguments()),
        "ProhibitedInstructions" => Rule::Forbidden(arguments()),
        "ReadWriteOrder" => Rule::ReadWriteOrder,
        "ReadAfterOperation" => Rule::ReadBeforeAssignment,
        "UsingAux" => Rule::Auxiliary,
        "VarIsNotUsed" => Rule::Unused,
        _ => Rule::Unsupported(name.to_owned()),
    }
}

enum Event {
    Read(u32),
    Write(u32),
    Call(String),
    Assign(Option<(u32, u32)>),
}

#[derive(Default)]
struct Facts {
    nodes: Vec<&'static str>,
    events: Vec<Event>,
    functions: HashMap<String, Vec<Event>>,
}

impl Facts {
    fn for_program(program: &Program) -> Self {
        let mut facts = Self::default();
        facts.nodes.push("ASAPrograma");
        for _ in &program.includes {
            facts.nodes.push("NoInclusaoBiblioteca");
        }
        let mut top_level = Vec::new();
        for item in &program.items {
            match item {
                Item::Declarations(decl) => facts.declaration(decl),
                Item::Stmt(statement) => facts.statement(statement),
                Item::Function(function) => {
                    top_level.append(&mut facts.events);
                    facts.nodes.push("NoDeclaracaoFuncao");
                    for _ in &function.params {
                        facts.nodes.push("NoDeclaracaoParametro");
                    }
                    facts.body(&function.body);
                    facts.functions.insert(
                        function.name.name.clone(),
                        std::mem::take(&mut facts.events),
                    );
                }
            }
        }
        top_level.append(&mut facts.events);
        facts.events = top_level;
        facts
    }

    fn expression(&mut self, expr: &Expr) {
        self.nodes.push(match expr {
            Expr::Int { .. } => "NoInteiro",
            Expr::Real { .. } => "NoReal",
            Expr::Bool { .. } => "NoLogico",
            Expr::Char { .. } => "NoCaracter",
            Expr::Str { .. } => "NoCadeia",
            Expr::Ident(_) => "NoReferenciaVariavel",
            Expr::Call { .. } => "NoChamadaFuncao",
            Expr::Index { indices, .. } if indices.len() == 1 => "NoReferenciaVetor",
            Expr::Index { .. } => "NoReferenciaMatriz",
            Expr::Binary { op, .. } => binary_name(*op),
            Expr::Unary { op, .. } => match op {
                UnaryOp::Neg => "NoMenosUnario",
                UnaryOp::Plus => "NoMaisUnario",
                UnaryOp::Not => "NoNao",
                UnaryOp::BitNot => "NoBitwiseNao",
            },
            Expr::IncDec { .. } | Expr::Assign { .. } => "NoOperacaoAtribuicao",
            Expr::Paren { .. } => "NoParenteses",
        });
        match expr {
            Expr::Call {
                scope, name, args, ..
            } => {
                for arg in args {
                    self.expression(arg);
                }
                if scope.is_none() {
                    match name.as_str() {
                        "leia" | "escreva" => {
                            for at in args.iter().filter_map(reference) {
                                self.events.push(if name == "leia" {
                                    Event::Read(at)
                                } else {
                                    Event::Write(at)
                                });
                            }
                        }
                        _ => self.events.push(Event::Call(name.clone())),
                    }
                }
            }
            Expr::Index { indices, .. } => {
                for index in indices {
                    self.expression(index);
                }
            }
            Expr::Unary { expr, .. } | Expr::Paren { expr, .. } => self.expression(expr),
            Expr::IncDec { target, .. } => {
                self.expression(target);
                self.events.push(Event::Assign(None));
            }
            Expr::Binary { left, right, .. } => {
                self.expression(left);
                self.expression(right);
            }
            Expr::Assign {
                target, value, op, ..
            } => self.assignment(target, value, *op),
            Expr::Ident(_)
            | Expr::Int { .. }
            | Expr::Real { .. }
            | Expr::Bool { .. }
            | Expr::Char { .. }
            | Expr::Str { .. } => {}
        }
    }

    fn assignment(&mut self, target: &Expr, value: &Expr, op: AssignOp) {
        self.expression(target);
        self.expression(value);
        let pair = (op == AssignOp::Assign)
            .then(|| reference(target).zip(reference(value)))
            .flatten();
        self.events.push(Event::Assign(pair));
    }

    fn declaration(&mut self, declarations: &Declarations) {
        for declarator in &declarations.declarators {
            match declarator {
                Declarator::Variable { init, .. } => {
                    self.nodes.push("NoDeclaracaoVariavel");
                    if let Some(expr) = init {
                        self.expression(expr);
                    }
                }
                Declarator::Array { size, init, .. } => {
                    self.nodes.push("NoDeclaracaoVetor");
                    if let Some(expr) = size {
                        self.expression(expr);
                    }
                    if let Some(values) = init {
                        self.nodes.push("NoVetor");
                        for value in values {
                            self.expression(value);
                        }
                    }
                }
                Declarator::Matrix {
                    rows, cols, init, ..
                } => {
                    self.nodes.push("NoDeclaracaoMatriz");
                    for dimension in rows.iter().chain(cols) {
                        self.expression(dimension);
                    }
                    if let Some(values) = init {
                        self.nodes.push("NoMatriz");
                        for value in values.iter().flatten() {
                            self.expression(value);
                        }
                    }
                }
            }
        }
    }

    fn body(&mut self, body: &[Stmt]) {
        for statement in body {
            self.statement(statement);
        }
    }

    fn statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Declarations(declarations) => self.declaration(declarations),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.nodes.push("NoSe");
                self.expression(condition);
                self.body(then_branch);
                if let Some(branch) = else_branch {
                    self.body(branch);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.nodes.push("NoEnquanto");
                self.expression(condition);
                self.body(body);
            }
            Stmt::DoWhile {
                body, condition, ..
            } => {
                self.nodes.push("NoFacaEnquanto");
                self.body(body);
                self.expression(condition);
            }
            Stmt::For {
                init,
                condition,
                step,
                body,
                ..
            } => {
                self.nodes.push("NoPara");
                match init {
                    Some(ForInit::Declarations(decl)) => self.declaration(decl),
                    Some(ForInit::Assignment(value)) => self.assigned(value),
                    Some(ForInit::Expression(expr)) => self.expression(expr),
                    None => {}
                }
                if let Some(condition) = condition {
                    self.expression(condition);
                }
                self.body(body);
                match step {
                    Some(ForStep::Assignment(value)) => self.assigned(value),
                    Some(ForStep::Expression(expr)) => self.expression(expr),
                    None => {}
                }
            }
            Stmt::Switch { value, cases, .. } => {
                self.nodes.push("NoEscolha");
                self.expression(value);
                for case in cases {
                    self.nodes.push("NoCaso");
                    if let Some(value) = &case.value {
                        self.expression(value);
                    }
                    self.body(&case.body);
                }
            }
            Stmt::Return { value, .. } => {
                self.nodes.push("NoRetorne");
                if let Some(value) = value {
                    self.expression(value);
                }
            }
            Stmt::Break { .. } => self.nodes.push("NoPare"),
            Stmt::Assign(value) => self.assigned(value),
            Stmt::Expr { expr, .. } => self.expression(expr),
        }
    }

    fn assigned(&mut self, value: &Assignment) {
        self.nodes.push("NoOperacaoAtribuicao");
        self.assignment(&value.target, &value.value, value.op);
    }

    fn execution(&self) -> Vec<&Event> {
        let Some(start) = self.functions.get("inicio") else {
            return self.events.iter().collect();
        };
        let mut result = Vec::new();
        let mut stack = vec![("inicio", start.iter())];
        let mut path = HashSet::from(["inicio"]);
        // Inlining is structural. Avoid recursing indefinitely or expanding
        // the same shared call graph until memory is exhausted.
        for _ in 0..100_000 {
            let Some((name, events)) = stack.last_mut() else {
                break;
            };
            match events.next() {
                Some(Event::Call(callee)) => {
                    if let Some(body) = self.functions.get(callee)
                        && path.insert(callee)
                    {
                        stack.push((callee, body.iter()));
                    }
                }
                Some(event) => result.push(event),
                None => {
                    path.remove(*name);
                    stack.pop();
                }
            }
        }
        result
    }
}

fn reference(expr: &Expr) -> Option<u32> {
    match expr {
        Expr::Ident(ident) if ident.scope.is_none() => Some(ident.span.start),
        Expr::Index {
            scope: None, span, ..
        } => Some(span.start),
        Expr::Paren { expr, .. } => reference(expr),
        _ => None,
    }
}

fn binary_name(op: BinOp) -> &'static str {
    match op {
        BinOp::Mul => "NoOperacaoMultiplicacao",
        BinOp::Div => "NoOperacaoDivisao",
        BinOp::Mod => "NoOperacaoModulo",
        BinOp::Add => "NoOperacaoSoma",
        BinOp::Sub => "NoOperacaoSubtracao",
        BinOp::Eq => "NoOperacaoLogicaIgualdade",
        BinOp::Ne => "NoOperacaoLogicaDiferenca",
        BinOp::Gt => "NoOperacaoLogicaMaior",
        BinOp::Lt => "NoOperacaoLogicaMenor",
        BinOp::Le => "NoOperacaoLogicaMenorIgual",
        BinOp::Ge => "NoOperacaoLogicaMaiorIgual",
        BinOp::And => "NoOperacaoLogicaE",
        BinOp::Or => "NoOperacaoLogicaOU",
        BinOp::Xor => "NoOperacaoBitwiseXOR",
        BinOp::Shl => "NoOperacaoBitwiseLeftShift",
        BinOp::Shr => "NoOperacaoBitwiseRightShift",
        BinOp::BitAnd => "NoOperacaoBitwiseE",
        BinOp::BitOr => "NoOperacaoBitwiseOu",
    }
}

fn distance(left: &[&str], right: &[&str]) -> usize {
    const MAX_NODES: usize = 2_048;
    let left = &left[..left.len().min(MAX_NODES)];
    let right = &right[..right.len().min(MAX_NODES)];
    let mut row: Vec<_> = (0..=right.len()).collect();
    for (i, token) in left.iter().enumerate() {
        let mut old = row[0];
        row[0] = i + 1;
        for (j, other) in right.iter().enumerate() {
            let next = row[j + 1];
            row[j + 1] = (old + usize::from(token != other)).min(row[j].min(next) + 1);
            old = next;
        }
    }
    row[right.len()]
}

pub(super) fn hints(models: &[Model], program: &Program, source: &str) -> Vec<String> {
    if models.is_empty() {
        return Vec::new();
    }
    let facts = Facts::for_program(program);
    let model = models
        .iter()
        .min_by_key(|model| distance(&facts.nodes, &model.signature))
        .unwrap();
    let nodes: HashSet<_> = facts.nodes.iter().copied().collect();
    let (_, bindings) = portugol_semantics::analyzer::symbol_bindings(program);
    let events = facts.execution();
    let read: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Read(at) => bindings.get(at).copied(),
            _ => None,
        })
        .collect();
    let writes: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Write(at) => bindings.get(at).copied(),
            _ => None,
        })
        .collect();
    let mut result = Vec::new();
    for rule in &model.rules {
        match rule {
            Rule::Required(required) => result.extend(
                required
                    .iter()
                    .filter(|name| !nodes.contains(name.as_str()))
                    .map(|name| format!("Inclua a construção {name}.")),
            ),
            Rule::Forbidden(forbidden) => result.extend(
                forbidden
                    .iter()
                    .filter(|name| nodes.contains(name.as_str()))
                    .map(|name| format!("Evite a construção {name}.")),
            ),
            Rule::ReadWriteOrder if read != writes => {
                result.push("Mostre as variáveis na mesma ordem em que foram lidas.".into());
            }
            Rule::ReadBeforeAssignment
                if events
                    .iter()
                    .skip_while(|e| !matches!(e, Event::Assign(_)))
                    .any(|e| matches!(e, Event::Read(_))) =>
            {
                result.push("Leia as entradas antes de executar atribuições.".into());
            }
            Rule::Auxiliary if !uses_auxiliary(&events, &bindings, &read) => {
                result.push("Use uma variável auxiliar para trocar os valores lidos.".into());
            }
            Rule::Unused => {
                let used: HashSet<_> = bindings
                    .iter()
                    .filter_map(|(&at, &decl)| (at != decl).then_some(decl))
                    .collect();
                let entry = program.items.iter().find_map(|item| match item {
                    Item::Function(function) if function.name.name == "inicio" => {
                        Some(function.name.span.start)
                    }
                    _ => None,
                });
                for token in portugol_syntax::lexer::lex(source).tokens {
                    let start = token.span.start;
                    if bindings.get(&start) == Some(&start)
                        && !used.contains(&start)
                        && entry != Some(start)
                    {
                        let name = &source[start as usize..token.span.end as usize];
                        result.push(format!("O símbolo {name} foi declarado e ficou sem uso."));
                    }
                }
            }
            Rule::Unsupported(name) => {
                result.push(format!("A regra estática {name:?} não está disponível."))
            }
            Rule::ReadWriteOrder | Rule::ReadBeforeAssignment | Rule::Auxiliary => {}
        }
    }
    result
}

fn uses_auxiliary(events: &[&Event], bindings: &HashMap<u32, u32>, read: &[u32]) -> bool {
    let assignments: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Assign(Some((left, right))) => {
                Some((*bindings.get(left)?, *bindings.get(right)?))
            }
            _ => None,
        })
        .collect();
    assignments.windows(3).any(|window| {
        let [(aux, first), (a, second), (b, saved)] = window else {
            unreachable!()
        };
        first == a
            && second == b
            && aux == saved
            && first != second
            && !read.contains(aux)
            && read.contains(first)
            && read.contains(second)
    })
}
