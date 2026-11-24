//! Conversão verificada de um subconjunto de Portugol em Rust compilável.

#[cfg(test)]
use std::collections::HashMap;

mod codegen;
mod hir;

/// Baixa o subconjunto clássico suportado para a HIR tipada compartilhada.
pub fn lower_to_hir(source: &str) -> Result<portugol_hir::Program, Diagnostic> {
    hir::lower(source)
}

use portugol_syntax::Diagnostic;
#[cfg(test)]
use portugol_syntax::ast::{
    AssignOp, Assignment, BinOp, Declarations, Declarator, Expr as AstExpr, Function, Item,
    Program, Stmt as AstStmt, Type, UnaryOp,
};
#[cfg(test)]
use portugol_syntax::{Severity, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceMapQuality {
    Exact,
    Approximate,
    GeneratedOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceMapSegment {
    pub generated: SourceRange,
    pub source: Option<SourceRange>,
    pub quality: SourceMapQuality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedRust {
    pub code: String,
    pub mappings: Vec<SourceMapSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    Inteiro,
    Real,
    Logico,
    Cadeia,
    Caracter,
}

#[cfg(test)]
impl Ty {
    fn rust(self) -> &'static str {
        match self {
            Self::Inteiro => "i32",
            Self::Real => "f64",
            Self::Logico => "bool",
            Self::Cadeia => "String",
            Self::Caracter => "char",
        }
    }
}

#[cfg(test)]
#[derive(Debug)]
struct HProgram {
    body: Vec<HStmt>,
}

#[cfg(test)]
#[derive(Debug)]
enum HStmt {
    Declare {
        name: String,
        ty: Ty,
        init: Option<HExpr>,
        is_const: bool,
    },
    Assign {
        name: String,
        ty: Ty,
        op: AssignOp,
        value: HExpr,
    },
    Write(Vec<HExpr>),
    If {
        condition: HExpr,
        then_branch: Vec<HStmt>,
        else_branch: Option<Vec<HStmt>>,
    },
}

#[cfg(test)]
#[derive(Debug)]
struct HExpr {
    code: String,
    ty: Ty,
    span: Span,
}

/// Gera Rust completo a partir da HIR para o subconjunto documentado.
/// Erros de sintaxe/semântica e spans da fonte são preservados pelo lowering.
pub fn export(source: &str) -> Result<String, Diagnostic> {
    export_with_map(source).map(|generated| generated.code)
}

/// Gera Rust para Portugol clássico e preserva os spans HIR de origem.
pub fn export_with_map(source: &str) -> Result<GeneratedRust, Diagnostic> {
    let hir = hir::lower(source)?;
    Ok(codegen::render_with_map(&hir))
}

/// Gera Rust para o subset inicial do dialeto educacional Portugol-RS.
pub fn export_portugol_rs(source: &str) -> Result<String, Vec<Diagnostic>> {
    export_portugol_rs_with_map(source).map(|generated| generated.code)
}

/// Projeto Cargo pronto gerado a partir de um programa `.pors` (console ou visual MVU).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedCargoProject {
    pub package_name: String,
    pub cargo_toml: String,
    pub main_rs: String,
    pub readme_md: String,
    pub mappings: Vec<SourceMapSegment>,
}

/// Gera Rust para Portugol-RS e preserva os spans HIR de origem.
pub fn export_portugol_rs_with_map(source: &str) -> Result<GeneratedRust, Vec<Diagnostic>> {
    let hir = portugol_rs_frontend::lower(source)?;
    if let Some((module, name, span)) = find_first_unsupported_host_call(&hir) {
        return Err(vec![Diagnostic::error(
            "portugol-rust.host-call-unsupported",
            format!(
                "A exportação Rust para chamadas nativas `{module}.{name}` ainda não está disponível."
            ),
            portugol_syntax::Span::new(span.start, span.end),
        )]);
    }
    Ok(codegen::render_pors_with_map(&hir))
}

/// Gera um projeto Cargo completo (Edition 2024, sem dependências externas) a partir
/// de um programa `.pors`, incluindo apps visuais declarativos (`UI.*` / MVU).
pub fn export_portugol_rs_cargo_project(
    source: &str,
    requested_name: &str,
) -> Result<ExportedCargoProject, Vec<Diagnostic>> {
    let generated = export_portugol_rs_with_map(source)?;
    let package_name = sanitize_cargo_package_name(requested_name);
    let is_visual = generated.code.contains("mod __pors_ui");
    let cargo_toml =
        format!("[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    let readme_md = if is_visual {
        format!(
            "# {package_name}\n\nProjeto Rust gerado a partir de Portugol-RS (`.pors`) com UI declarativa MVU.\n\n## Como executar\n\n```bash\ncargo run\n```\n\nO programa imprime o snapshot JSON inicial (`step: 0`) e lê eventos pela entrada padrão, um por linha:\n\n- `clique:<id>` ou `{{\"kind\":\"clique\",\"id\":\"<id>\"}}`\n- `entrada_alterada:<id>:<valor>` ou `{{\"kind\":\"entrada_alterada\",\"id\":\"<id>\",\"value\":\"<valor>\"}}`\n"
        )
    } else {
        format!(
            "# {package_name}\n\nProjeto Rust gerado a partir de Portugol-RS (`.pors`).\n\n## Como executar\n\n```bash\ncargo run\n```\n"
        )
    };
    Ok(ExportedCargoProject {
        package_name,
        cargo_toml,
        main_rs: generated.code,
        readme_md,
        mappings: generated.mappings,
    })
}

/// Grava um [`ExportedCargoProject`] em disco (`Cargo.toml`, `src/main.rs` e `README.md`).
pub fn write_cargo_project(
    project: &ExportedCargoProject,
    dir: &std::path::Path,
) -> std::io::Result<()> {
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(dir.join("Cargo.toml"), &project.cargo_toml)?;
    std::fs::write(src_dir.join("main.rs"), &project.main_rs)?;
    std::fs::write(dir.join("README.md"), &project.readme_md)?;
    Ok(())
}

fn sanitize_cargo_package_name(raw: &str) -> String {
    let mut sanitized = String::new();
    let mut last_dash = false;
    for ch in raw.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '_' {
            sanitized.push(lower);
            last_dash = false;
        } else if !sanitized.is_empty() && !last_dash {
            sanitized.push('-');
            last_dash = true;
        }
    }
    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        return "programa-pors".to_owned();
    }
    if trimmed.as_bytes()[0].is_ascii_digit() {
        format!("pors-{trimmed}")
    } else {
        trimmed.to_owned()
    }
}

fn find_first_unsupported_host_call(
    program: &portugol_hir::Program,
) -> Option<(&str, &str, portugol_hir::Span)> {
    find_host_call_in_stmts(&program.body).or_else(|| {
        program
            .functions
            .iter()
            .find_map(|function| find_host_call_in_stmts(&function.body))
    })
}

fn find_host_call_in_stmts(
    stmts: &[portugol_hir::Stmt],
) -> Option<(&str, &str, portugol_hir::Span)> {
    use portugol_hir::StmtKind as S;
    for stmt in stmts {
        let found = match &stmt.kind {
            S::Declare { initializer, .. } => initializer.as_ref().and_then(find_host_call_in_expr),
            S::Assign { value, .. } | S::FieldAssign { value, .. } => find_host_call_in_expr(value),
            S::Choose { value, arms } => find_host_call_in_expr(value).or_else(|| {
                arms.iter()
                    .find_map(|arm| find_host_call_in_stmts(&arm.body))
            }),
            S::Read { .. } | S::LoopControl { .. } => None,
            S::Call { arguments, .. } | S::Write(arguments) => {
                arguments.iter().find_map(find_host_call_in_expr)
            }
            S::HostCall {
                module,
                name,
                arguments,
            } => {
                if !matches!(
                    module.as_str(),
                    "UI" | "Matematica"
                        | "Texto"
                        | "Tipos"
                        | "Util"
                        | "Calendario"
                        | "Arquivos"
                        | "Json"
                ) {
                    Some((module.as_str(), name.as_str(), stmt.span))
                } else {
                    arguments.iter().find_map(find_host_call_in_expr)
                }
            }
            S::If {
                condition,
                then_branch,
                else_branch,
            } => find_host_call_in_expr(condition)
                .or_else(|| find_host_call_in_stmts(then_branch))
                .or_else(|| else_branch.as_deref().and_then(find_host_call_in_stmts)),
            S::While { condition, body } => {
                find_host_call_in_expr(condition).or_else(|| find_host_call_in_stmts(body))
            }
            S::For {
                initializer,
                condition,
                step,
                body,
            } => find_host_call_in_stmts(std::slice::from_ref(initializer.as_ref()))
                .or_else(|| find_host_call_in_expr(condition))
                .or_else(|| find_host_call_in_stmts(std::slice::from_ref(step.as_ref())))
                .or_else(|| find_host_call_in_stmts(body)),
            S::Return { value } => value.as_ref().and_then(find_host_call_in_expr),
            S::IndexAssign { index, value, .. } => {
                find_host_call_in_expr(index).or_else(|| find_host_call_in_expr(value))
            }
            S::VectorPush { value, .. } => find_host_call_in_expr(value),
            S::VectorPop { .. } => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

fn find_host_call_in_expr(expr: &portugol_hir::Expr) -> Option<(&str, &str, portugol_hir::Span)> {
    use portugol_hir::ExprKind as E;
    match &expr.kind {
        E::HostCall {
            module,
            name,
            arguments,
        } => {
            if !matches!(
                module.as_str(),
                "UI" | "Matematica"
                    | "Texto"
                    | "Tipos"
                    | "Util"
                    | "Calendario"
                    | "Arquivos"
                    | "Json"
            ) {
                Some((module.as_str(), name.as_str(), expr.span))
            } else {
                arguments.iter().find_map(find_host_call_in_expr)
            }
        }
        E::Call { arguments, .. }
        | E::VectorLiteral(arguments)
        | E::RecordLiteral {
            fields: arguments, ..
        }
        | E::EnumConstructor {
            payload: arguments, ..
        } => arguments.iter().find_map(find_host_call_in_expr),
        E::Field { value, .. }
        | E::Index { index: value, .. }
        | E::VectorGet { index: value, .. }
        | E::Parenthesized(value)
        | E::Try { operand: value }
        | E::Unary { operand: value, .. } => find_host_call_in_expr(value),
        E::Binary { left, right, .. } => {
            find_host_call_in_expr(left).or_else(|| find_host_call_in_expr(right))
        }
        E::Binding(_)
        | E::Length { .. }
        | E::VectorPop { .. }
        | E::Integer(_)
        | E::Real(_)
        | E::Boolean(_)
        | E::String(_)
        | E::Character(_) => None,
    }
}

#[cfg(test)]
fn export_legacy(source: &str) -> Result<String, Diagnostic> {
    let parsed = portugol_syntax::parse(source);
    if let Some(error) = parsed
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(error.clone());
    }
    if let Some(error) = portugol_semantics::analyze(&parsed.program)
        .into_iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(error);
    }

    let hir = Lowerer::new().program(&parsed.program)?;
    Ok(render(&hir))
}

#[cfg(test)]
struct Lowerer {
    scopes: Vec<HashMap<String, Ty>>,
}

#[cfg(test)]
impl Lowerer {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    fn program(mut self, program: &Program) -> Result<HProgram, Diagnostic> {
        if !program.includes.is_empty() {
            return Err(unsupported("bibliotecas", program.includes[0].span));
        }
        let functions: Vec<&Function> = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => Some(function),
                _ => None,
            })
            .collect();
        if program
            .items
            .iter()
            .any(|item| !matches!(item, Item::Function(_)))
        {
            let span = program
                .items
                .iter()
                .find_map(|item| match item {
                    Item::Function(_) => None,
                    Item::Declarations(declarations) => Some(declarations.span),
                    Item::Stmt(statement) => Some(statement.span()),
                })
                .unwrap_or(program.span);
            return Err(unsupported("comandos fora de funcao inicio", span));
        }
        if functions.len() != 1 || functions[0].name.name != "inicio" {
            let span = functions
                .iter()
                .find(|function| function.name.name != "inicio")
                .map(|function| function.name.span)
                .unwrap_or(program.span);
            return Err(unsupported("programa sem uma única função inicio", span));
        }
        let function = functions[0];
        if !function.params.is_empty() || function.return_type.is_some() {
            return Err(unsupported(
                "parâmetros ou retorno em funcao inicio",
                function.span,
            ));
        }
        let body = self.statements(&function.body)?;
        Ok(HProgram { body })
    }

    fn statements(&mut self, statements: &[AstStmt]) -> Result<Vec<HStmt>, Diagnostic> {
        let mut result = Vec::with_capacity(statements.len());
        for statement in statements {
            result.push(self.statement(statement)?);
        }
        Ok(result)
    }

    fn statement(&mut self, statement: &AstStmt) -> Result<HStmt, Diagnostic> {
        match statement {
            AstStmt::Declarations(declarations) => self.declaration(declarations),
            AstStmt::Assign(assignment) => self.assignment(assignment),
            AstStmt::Expr {
                expr:
                    AstExpr::Call {
                        name,
                        args,
                        span,
                        scope: None,
                    },
                ..
            } if name == "escreva" => Ok(HStmt::Write(
                args.iter()
                    .map(|arg| self.expression(arg))
                    .collect::<Result<_, _>>()?,
            )),
            AstStmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition = self.expression(condition)?;
                if condition.ty != Ty::Logico {
                    return Err(unsupported("condição não lógica", condition.span));
                }
                self.scopes.push(HashMap::new());
                let then_branch = self.statements(then_branch)?;
                self.scopes.pop();
                let else_branch = if let Some(branch) = else_branch {
                    self.scopes.push(HashMap::new());
                    let lowered = self.statements(branch)?;
                    self.scopes.pop();
                    Some(lowered)
                } else {
                    None
                };
                Ok(HStmt::If {
                    condition,
                    then_branch,
                    else_branch,
                })
            }
            AstStmt::Expr {
                expr: AstExpr::Call { name, span, .. },
                ..
            } => Err(unsupported(name, *span)),
            AstStmt::Expr { expr, .. } => Err(unsupported("expressão", expr.span())),
            AstStmt::While { span, .. } => Err(unsupported("enquanto", *span)),
            AstStmt::DoWhile { span, .. } => Err(unsupported("faca-enquanto", *span)),
            AstStmt::For { span, .. } => Err(unsupported("para", *span)),
            AstStmt::Switch { span, .. } => Err(unsupported("escolha", *span)),
            AstStmt::Return { span, .. } => Err(unsupported("retorno", *span)),
            AstStmt::Break { span } => Err(unsupported("pare", *span)),
        }
    }

    fn declaration(&mut self, declarations: &Declarations) -> Result<HStmt, Diagnostic> {
        let ty =
            map_type(declarations.ty).ok_or_else(|| unsupported("tipo vago", declarations.span))?;
        if declarations.declarators.len() != 1 {
            return Err(unsupported("declaração múltipla", declarations.span));
        }
        let Declarator::Variable { name, init, .. } = &declarations.declarators[0] else {
            return Err(unsupported("vetor ou matriz", declarations.span));
        };
        let init = init
            .as_ref()
            .map(|expression| self.expression(expression))
            .transpose()?;
        if let Some(value) = &init
            && !compatible(ty, value.ty)
        {
            return Err(unsupported("conversao implicita de tipo", value.span));
        }
        if declarations.is_const && (init.is_none() || ty == Ty::Cadeia) {
            return Err(unsupported(
                "constante sem expressão constante em Rust",
                declarations.span,
            ));
        }
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.name.clone(), ty);
        Ok(HStmt::Declare {
            name: rust_ident(&name.name),
            ty,
            init,
            is_const: declarations.is_const,
        })
    }

    fn assignment(&mut self, assignment: &Assignment) -> Result<HStmt, Diagnostic> {
        let AstExpr::Ident(ident) = &assignment.target else {
            return Err(unsupported("atribuição indireta", assignment.target.span()));
        };
        let ty = self
            .lookup(&ident.name)
            .ok_or_else(|| unsupported("variável fora do escopo", ident.span))?;
        let value = self.expression(&assignment.value)?;
        if !compatible(ty, value.ty) {
            return Err(unsupported("conversao implicita de tipo", value.span));
        }
        let valid_compound = match assignment.op {
            AssignOp::Assign => true,
            AssignOp::Add => matches!(ty, Ty::Inteiro | Ty::Real | Ty::Cadeia),
            AssignOp::Sub | AssignOp::Mul | AssignOp::Div => matches!(ty, Ty::Inteiro | Ty::Real),
            AssignOp::Mod | AssignOp::Xor | AssignOp::BitAnd | AssignOp::BitOr => ty == Ty::Inteiro,
        };
        if !valid_compound {
            return Err(unsupported(
                "operador de atribuição composta",
                assignment.span,
            ));
        }
        Ok(HStmt::Assign {
            name: rust_ident(&ident.name),
            ty,
            op: assignment.op,
            value,
        })
    }

    fn lookup(&self, name: &str) -> Option<Ty> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn expression(&self, expression: &AstExpr) -> Result<HExpr, Diagnostic> {
        let span = expression.span();
        let (code, ty) = match expression {
            AstExpr::Ident(ident) => {
                let ty = self
                    .lookup(&ident.name)
                    .ok_or_else(|| unsupported("variável fora do escopo", ident.span))?;
                (rust_ident(&ident.name), ty)
            }
            AstExpr::Int { value, .. } => {
                let value = i32::try_from(*value)
                    .map_err(|_| unsupported("literal inteiro fora de 32 bits", span))?;
                (format!("{value}_i32"), Ty::Inteiro)
            }
            AstExpr::Real { value, .. } => (format_real(*value), Ty::Real),
            AstExpr::Bool { value, .. } => (value.to_string(), Ty::Logico),
            AstExpr::Char { value, .. } => (format!("{value:?}"), Ty::Caracter),
            AstExpr::Str { value, .. } => (format!("String::from({value:?})"), Ty::Cadeia),
            AstExpr::Paren { expr, .. } => {
                let expr = self.expression(expr)?;
                (format!("({})", expr.code), expr.ty)
            }
            AstExpr::Unary { op, expr, .. } => {
                if *op == UnaryOp::Neg
                    && matches!(expr.as_ref(), AstExpr::Int { value, .. } if *value == i32::MAX as i64 + 1)
                {
                    ("i32::MIN".to_owned(), Ty::Inteiro)
                } else {
                    let expr = self.expression(expr)?;
                    if *op == UnaryOp::Neg && expr.ty == Ty::Inteiro {
                        (format!("({}).wrapping_neg()", expr.code), Ty::Inteiro)
                    } else {
                        let (operator, ty) = match op {
                            UnaryOp::Neg if matches!(expr.ty, Ty::Inteiro | Ty::Real) => {
                                ("-", expr.ty)
                            }
                            UnaryOp::Plus if matches!(expr.ty, Ty::Inteiro | Ty::Real) => {
                                return Ok(HExpr {
                                    code: expr.code,
                                    ty: expr.ty,
                                    span,
                                });
                            }
                            UnaryOp::Not if expr.ty == Ty::Logico => ("!", Ty::Logico),
                            UnaryOp::BitNot if expr.ty == Ty::Inteiro => ("!", Ty::Inteiro),
                            _ => return Err(unsupported("operacao unaria", span)),
                        };
                        (format!("{operator}({})", expr.code), ty)
                    }
                }
            }
            AstExpr::Binary {
                op, left, right, ..
            } => {
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                if *op == BinOp::Add
                    && ((left.ty == Ty::Cadeia && right.ty == Ty::Cadeia)
                        || (left.ty == Ty::Caracter && right.ty == Ty::Caracter))
                {
                    (
                        format!(
                            "format!(\"{}{}\", {}, {})",
                            "{}", "{}", left.code, right.code
                        ),
                        Ty::Cadeia,
                    )
                } else {
                    let ty = binary_type(*op, left.ty, right.ty)
                        .ok_or_else(|| unsupported("operacao entre tipos", span))?;
                    if ty == Ty::Inteiro {
                        let method = match op {
                            BinOp::Add => Some("wrapping_add"),
                            BinOp::Sub => Some("wrapping_sub"),
                            BinOp::Mul => Some("wrapping_mul"),
                            BinOp::Div => Some("wrapping_div"),
                            BinOp::Mod => Some("wrapping_rem"),
                            BinOp::Shl => Some("wrapping_shl"),
                            BinOp::Shr => Some("wrapping_shr"),
                            _ => None,
                        };
                        if let Some(method) = method {
                            let right = if matches!(op, BinOp::Shl | BinOp::Shr) {
                                format!("({} as u32)", right.code)
                            } else {
                                right.code
                            };
                            (format!("({}).{}({right})", left.code, method), ty)
                        } else {
                            let operator = match op {
                                BinOp::Xor => "^",
                                BinOp::BitAnd => "&",
                                BinOp::BitOr => "|",
                                _ => unreachable!(),
                            };
                            (format!("({}) {operator} ({})", left.code, right.code), ty)
                        }
                    } else {
                        let operator = match op {
                            BinOp::And => "&&",
                            BinOp::Or => "||",
                            BinOp::Xor => "^",
                            BinOp::Eq => "==",
                            BinOp::Ne => "!=",
                            BinOp::Gt => ">",
                            BinOp::Lt => "<",
                            BinOp::Le => "<=",
                            BinOp::Ge => ">=",
                            BinOp::Shl => "<<",
                            BinOp::Shr => ">>",
                            BinOp::BitAnd => "&",
                            BinOp::BitOr => "|",
                            BinOp::Mul => "*",
                            BinOp::Div => "/",
                            BinOp::Mod => "%",
                            BinOp::Add => "+",
                            BinOp::Sub => "-",
                        };
                        (format!("({}) {operator} ({})", left.code, right.code), ty)
                    }
                }
            }
            _ => return Err(unsupported("expressão", span)),
        };
        Ok(HExpr { code, ty, span })
    }
}

#[cfg(test)]
fn compatible(target: Ty, value: Ty) -> bool {
    target == value
}

#[cfg(test)]
fn map_type(ty: Type) -> Option<Ty> {
    match ty {
        Type::Inteiro => Some(Ty::Inteiro),
        Type::Real => Some(Ty::Real),
        Type::Logico => Some(Ty::Logico),
        Type::Cadeia => Some(Ty::Cadeia),
        Type::Caracter => Some(Ty::Caracter),
        Type::Vazio => None,
    }
}

#[cfg(test)]
fn binary_type(op: BinOp, left: Ty, right: Ty) -> Option<Ty> {
    match op {
        BinOp::Add if left == Ty::Cadeia && right == Ty::Cadeia => Some(Ty::Cadeia),
        BinOp::Add if left == Ty::Caracter && right == Ty::Caracter => Some(Ty::Cadeia),
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div
            if left == right && matches!(left, Ty::Inteiro | Ty::Real) =>
        {
            Some(left)
        }
        BinOp::Mod | BinOp::Xor | BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr
            if left == Ty::Inteiro && right == Ty::Inteiro =>
        {
            Some(Ty::Inteiro)
        }
        BinOp::And | BinOp::Or if left == Ty::Logico && right == Ty::Logico => Some(Ty::Logico),
        BinOp::Eq | BinOp::Ne if left == right => Some(Ty::Logico),
        BinOp::Gt | BinOp::Lt | BinOp::Le | BinOp::Ge
            if left == right && matches!(left, Ty::Inteiro | Ty::Real | Ty::Caracter) =>
        {
            Some(Ty::Logico)
        }
        _ => None,
    }
}

#[cfg(test)]
fn render(program: &HProgram) -> String {
    let mut output = String::from("use std::io::{self, Write};\n\n");
    output.push_str(JAVA_DOUBLE_HELPER);
    output.push_str("\nfn main() {\n");
    render_statements(&program.body, 1, &mut output);
    output.push_str("}\n");
    output
}

#[cfg(test)]
fn render_statements(statements: &[HStmt], depth: usize, output: &mut String) {
    let indent = "    ".repeat(depth);
    for statement in statements {
        match statement {
            HStmt::Declare {
                name,
                ty,
                init,
                is_const,
            } => {
                output.push_str(&format!(
                    "{indent}{} {name}: {}",
                    if *is_const { "let" } else { "let mut" },
                    ty.rust()
                ));
                if let Some(init) = init {
                    output.push_str(&format!(" = {}", owned_value(init)));
                } else {
                    output.push_str(&format!(" = {}", default_value(*ty)));
                }
                output.push_str(";\n");
            }
            HStmt::Assign {
                name,
                ty,
                op,
                value,
            } => {
                if *ty == Ty::Inteiro {
                    let method = match op {
                        AssignOp::Add => Some("wrapping_add"),
                        AssignOp::Sub => Some("wrapping_sub"),
                        AssignOp::Mul => Some("wrapping_mul"),
                        AssignOp::Div => Some("wrapping_div"),
                        AssignOp::Mod => Some("wrapping_rem"),
                        _ => None,
                    };
                    if let Some(method) = method {
                        output.push_str(&format!(
                            "{indent}{name} = ({name}).{method}({});\n",
                            value.code
                        ));
                        continue;
                    }
                }
                if *ty == Ty::Cadeia && *op == AssignOp::Add {
                    output.push_str(&format!(
                        "{indent}{name} = format!(\"{{}}{{}}\", {name}, {});\n",
                        owned_value(value)
                    ));
                    continue;
                }
                let assignment_op = *op;
                let op = match op {
                    AssignOp::Assign => "=",
                    AssignOp::Add => "+=",
                    AssignOp::Sub => "-=",
                    AssignOp::Mul => "*=",
                    AssignOp::Div => "/=",
                    AssignOp::Mod => "%=",
                    AssignOp::Xor => "^=",
                    AssignOp::BitAnd => "&=",
                    AssignOp::BitOr => "|=",
                };
                let value = if assignment_op == AssignOp::Assign {
                    owned_value(value)
                } else {
                    value.code.clone()
                };
                output.push_str(&format!("{indent}{name} {op} {value};\n"));
            }
            HStmt::Write(values) => {
                let temps: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("__portugol_write_{index}"))
                    .collect();
                for (temp, value) in temps.iter().zip(values) {
                    output.push_str(&format!("{indent}let {temp} = {};\n", owned_value(value)));
                }
                for (temp, value) in temps.iter().zip(values) {
                    let expression = match value.ty {
                        Ty::Logico => {
                            format!("if {temp} {{ \"verdadeiro\" }} else {{ \"falso\" }}")
                        }
                        Ty::Real => format!("__portugol_java_double({temp})"),
                        _ => temp.clone(),
                    };
                    output.push_str(&format!("{indent}print!(\"{{}}\", {expression});\n"));
                }
                output.push_str(&format!("{indent}let _ = io::stdout().flush();\n"));
            }
            HStmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                output.push_str(&format!("{indent}if {} {{\n", condition.code));
                render_statements(then_branch, depth + 1, output);
                output.push_str(&format!("{indent}}}"));
                if let Some(else_branch) = else_branch {
                    output.push_str(" else {\n");
                    render_statements(else_branch, depth + 1, output);
                    output.push_str(&format!("{indent}}}"));
                }
                output.push('\n');
            }
        }
    }
}

fn rust_ident(name: &str) -> String {
    format!("pt_{name}")
}

#[cfg(test)]
fn owned_value(expression: &HExpr) -> String {
    if expression.ty == Ty::Cadeia {
        format!("({}).clone()", expression.code)
    } else {
        expression.code.clone()
    }
}

fn default_value(ty: Ty) -> &'static str {
    match ty {
        Ty::Inteiro => "0_i32",
        Ty::Real => "0.0_f64",
        Ty::Logico => "false",
        Ty::Cadeia => "String::new()",
        Ty::Caracter => "'\\0'",
    }
}

fn format_real(value: f64) -> String {
    if value.is_nan() {
        return "f64::NAN".to_owned();
    }
    if value == f64::INFINITY {
        return "f64::INFINITY".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "f64::NEG_INFINITY".to_owned();
    }
    let value = value.to_string();
    if value.contains(['.', 'e', 'E']) {
        value
    } else {
        format!("{value}.0")
    }
}

const JAVA_DOUBLE_HELPER: &str = r#"fn __portugol_java_double(value: f64) -> String {
    if value.is_nan() { return "NaN".to_owned(); }
    if value.is_infinite() { return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned(); }
    if value == 0.0 { return if value.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned(); }
    let sci = format!("{:e}", value.abs());
    let (mantissa, exponent) = sci.split_once('e').expect("{:e} format");
    let exponent: i32 = exponent.parse().expect("exponent");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let mut out = String::new();
    if value < 0.0 { out.push('-'); }
    let magnitude = value.abs();
    if (1e-3..1e7).contains(&magnitude) {
        if exponent >= 0 {
            let int_len = exponent as usize + 1;
            if digits.len() <= int_len {
                out.push_str(&digits);
                out.extend(std::iter::repeat_n('0', int_len - digits.len()));
                out.push_str(".0");
            } else {
                out.push_str(&digits[..int_len]);
                out.push('.');
                out.push_str(&digits[int_len..]);
            }
        } else {
            out.push_str("0.");
            out.extend(std::iter::repeat_n('0', (-exponent - 1) as usize));
            out.push_str(&digits);
        }
    } else {
        out.push_str(&digits[..1]); out.push('.');
        if digits.len() > 1 { out.push_str(&digits[1..]); } else { out.push('0'); }
        out.push_str(&format!("E{exponent}"));
    }
    out
}"#;

#[cfg(test)]
fn unsupported(feature: &str, span: Span) -> Diagnostic {
    Diagnostic::error(
        "rust.unsupported",
        format!("A exportação para Rust ainda não oferece suporte a {feature} neste programa."),
        span,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        SourceMapQuality, SourceRange, export, export_legacy, export_portugol_rs,
        export_portugol_rs_with_map, export_with_map,
    };
    use std::process::Command;

    const SOURCE: &str = "programa { funcao inicio() { inteiro total = 40 + 2 escreva(total) } }";

    #[test]
    fn lowers_typed_declarations_and_preserves_write_without_newline() {
        let generated = export(SOURCE).unwrap();
        assert!(generated.contains("let mut pt_total: i32 = (40_i32).wrapping_add(2_i32);"));
        assert!(generated.contains("print!(\"{}\", __portugol_write_0);"));
        assert!(!generated.contains("println!"));
    }

    #[test]
    fn rejects_constructs_outside_the_supported_subset_with_source_span() {
        let source = "programa { funcao inicio() { inteiro n leia(n) } }";
        let diagnostic = export(source).unwrap_err();
        assert_eq!(diagnostic.code, "rust.unsupported");
        assert_eq!(diagnostic.span.start, source.find("leia").unwrap() as u32);
        assert!(diagnostic.message.contains("leia"));
    }

    #[test]
    fn exports_portugol_rs_to_rust_and_executes_with_expected_output() {
        let source =
            "funcao inicio() { seja mutavel idade: inteiro = 16; idade += 1; escreva(idade); }";
        assert_rust_output(export_portugol_rs(source).unwrap(), "17");
    }

    #[test]
    fn exports_and_executes_portugol_rs_while_with_source_maps() {
        let source = "funcao inicio() { seja mutavel i: inteiro = 0; enquanto (i < 3) { escreva(i); i += 1; } }";
        let generated = export_portugol_rs_with_map(source).unwrap();
        assert!(generated.code.contains("while (pt_i) < (3_i32) {"));
        for snippet in ["enquanto (i < 3)", "i < 3", "escreva(i)"] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing mapping for {snippet}"
            );
        }
        let source_start = source.find("enquanto (").unwrap() as u32;
        let mapping = generated
            .mappings
            .iter()
            .find(|mapping| {
                mapping
                    .source
                    .is_some_and(|range| range.start == source_start)
                    && mapping.quality == SourceMapQuality::Exact
            })
            .expect("enquanto deve ter um source map exato");
        assert!(
            generated.code[mapping.generated.start as usize..mapping.generated.end as usize]
                .trim_start()
                .starts_with("while ")
        );
        assert_rust_output(generated.code, "012");
    }

    #[test]
    fn exports_and_executes_portugol_rs_for_with_source_maps() {
        let source =
            "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 3; i += 1) { escreva(i); } }";
        let generated = export_portugol_rs_with_map(source).unwrap();
        assert!(
            generated
                .code
                .contains("pt_i = (pt_i).wrapping_add(1_i32);")
        );
        for snippet in ["seja mutavel i", "i < 3", "i += 1", "escreva(i)"] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing exact source mapping for {snippet}"
            );
        }
        let source_start = source.find("para (").unwrap() as u32;
        let mapping = generated
            .mappings
            .iter()
            .find(|mapping| {
                mapping
                    .source
                    .is_some_and(|range| range.start == source_start)
                    && mapping.quality == SourceMapQuality::Exact
            })
            .expect("para deve ter um source map exato");
        let generated_loop = generated.code
            [mapping.generated.start as usize..mapping.generated.end as usize]
            .trim_start();
        assert!(generated_loop.starts_with("{\n"));
        assert!(generated_loop.contains("'pors_for_"));
        assert!(generated_loop.contains(": while "));
        assert_rust_output(generated.code, "012");
    }

    #[test]
    fn exports_and_executes_portugol_rs_loop_control_with_source_maps() {
        let source = "funcao inicio() { seja mutavel visitas: inteiro = 0; para (seja mutavel i: inteiro = 0; i < 5; i += 1) { visitas += 1; se (visitas > 8) { pare; } se (i == 1) { continue; } se (i == 4) { pare; } escreva(i); } }";
        let generated = export_portugol_rs_with_map(source).unwrap();
        let loop_label = source.find("para").unwrap();
        for (keyword, generated_control) in [
            ("continue;", format!("break 'pors_body_{loop_label};")),
            ("pare;", format!("break 'pors_for_{loop_label};")),
        ] {
            let start = source.find(keyword).unwrap() as u32;
            let end = start + keyword.len() as u32;
            let control_mapping = generated
                .mappings
                .iter()
                .find(|mapping| mapping.source == Some(SourceRange { start, end }))
                .unwrap_or_else(|| panic!("missing full source mapping for {keyword}"));
            assert_eq!(control_mapping.quality, SourceMapQuality::Exact);
            let mapped_rust = &generated.code
                [control_mapping.generated.start as usize..control_mapping.generated.end as usize];
            assert_eq!(mapped_rust.trim(), generated_control);
        }
        let step_start = source.find("i += 1)").unwrap() as u32;
        let step_end = step_start + "i += 1".len() as u32;
        let step_mappings: Vec<_> = generated
            .mappings
            .iter()
            .filter(|mapping| {
                mapping.source
                    == Some(SourceRange {
                        start: step_start,
                        end: step_end,
                    })
            })
            .collect();
        assert_eq!(
            step_mappings.len(),
            1,
            "the for step has one source mapping"
        );
        let step_mapping = step_mappings[0];
        assert_eq!(step_mapping.quality, SourceMapQuality::Exact);
        let mapped_step = &generated.code
            [step_mapping.generated.start as usize..step_mapping.generated.end as usize];
        assert_eq!(mapped_step.trim(), "pt_i = (pt_i).wrapping_add(1_i32);");
        assert_rust_output(generated.code, "023");

        let while_source = "funcao inicio() { seja mutavel i: inteiro = 0; enquanto (i < 5) { i += 1; se (i == 2) { continue; } se (i == 4) { pare; } escreva(i); } }";
        assert_rust_output(export_portugol_rs(while_source).unwrap(), "13");

        let nested_source = "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 3; i += 1) { seja mutavel j: inteiro = 0; enquanto (j < 3) { j += 1; se (j == 1) { continue; } se (j == 2) { pare; } escreva(j); } escreva(i); } }";
        assert_rust_output(export_portugol_rs(nested_source).unwrap(), "012");
    }

    #[test]
    fn counted_loop_continue_uses_header_bindings_when_body_shadows_names() {
        let source = "funcao inicio() { seja mutavel passo: inteiro = 1; seja mutavel visitas: inteiro = 0; para (seja mutavel i: inteiro = 0; i < 4; i += passo) { visitas += 1; se (visitas > 4) { pare; } escreva(i); seja passo: inteiro = 2; continue; } }";
        assert_rust_output(export_portugol_rs(source).unwrap(), "0123");

        let source = "funcao inicio() { seja mutavel visitas: inteiro = 0; para (seja mutavel i: inteiro = 0; i < 4; i += 1) { visitas += 1; se (visitas > 4) { pare; } escreva(i); seja i: inteiro = 9; continue; } }";
        assert_rust_output(export_portugol_rs(source).unwrap(), "0123");
    }

    #[test]
    fn counted_loop_keeps_utf8_source_maps_in_bounds_and_executes_nested_shadowing() {
        let source = "// comentário antes do loop: café 😃\nfuncao inicio() { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { escreva(\"é\", i); } escreva(i); } }";
        let generated = export_portugol_rs_with_map(source).unwrap();
        assert!(generated.mappings.iter().all(|mapping| {
            mapping.generated.start < mapping.generated.end
                && mapping.generated.end as usize <= generated.code.len()
                && generated
                    .code
                    .is_char_boundary(mapping.generated.start as usize)
                && generated
                    .code
                    .is_char_boundary(mapping.generated.end as usize)
                && mapping.source.is_none_or(|range| {
                    range.start < range.end
                        && range.end as usize <= source.len()
                        && source.is_char_boundary(range.start as usize)
                        && source.is_char_boundary(range.end as usize)
                })
        }));
        for snippet in [
            "para (",
            "seja mutavel i",
            "i < 2",
            "i += 1",
            "escreva(\"é\", i)",
        ] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing exact source mapping for {snippet}"
            );
        }
        assert!(
            generated
                .mappings
                .iter()
                .filter(|mapping| mapping.quality == SourceMapQuality::GeneratedOnly)
                .all(|mapping| mapping.source.is_none())
        );
        assert_rust_output(generated.code, "é0é10é0é11");
    }

    #[test]
    fn counted_loop_supports_real_control_values() {
        let source = "funcao inicio() { para (seja mutavel i: real = 0.0; i < 1.5; i += 0.5) { escreva(i); } }";
        let generated = export_portugol_rs(source).unwrap();
        assert_rust_output(generated, "0.00.51.0");
    }

    #[test]
    fn adds_string_budget_only_to_portugol_rs_codegen() {
        let pors = export_portugol_rs("funcao inicio() { escreva(\"olá\"); }").unwrap();
        assert!(pors.contains("__pors_account"));
        assert!(pors.contains("__pors_concat"));
        let classic = export(SOURCE).unwrap();
        assert!(!classic.contains("__pors_account"));
    }

    #[test]
    fn maps_generated_rust_declarations_expressions_and_nested_statements_to_source() {
        let source = "funcao inicio() { seja mutavel idade: inteiro = 16; se (verdadeiro) { escreva(idade); } }";
        let generated = export_portugol_rs_with_map(source).unwrap();

        assert!(generated.code.contains("let mut pt_idade: i32 = 16_i32;"));
        for snippet in ["seja mutavel", "verdadeiro", "escreva(idade)"] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing exact source mapping for {snippet}"
            );
        }
        assert!(generated.mappings.iter().all(|mapping| {
            mapping.generated.start < mapping.generated.end
                && mapping.generated.end as usize <= generated.code.len()
                && mapping.source.is_none_or(|range| {
                    range.start < range.end && range.end as usize <= source.len()
                })
        }));
        assert!(generated.mappings.windows(2).all(|pair| {
            (pair[0].generated.start, pair[0].generated.end)
                <= (pair[1].generated.start, pair[1].generated.end)
        }));
    }

    #[test]
    fn marks_rust_prelude_as_generated_only() {
        let source = "funcao inicio() { escreva(1); }";
        let generated = export_portugol_rs_with_map(source).unwrap();
        let prelude = generated.code.find("use std::io").unwrap() as u32;

        assert!(generated.mappings.iter().any(|mapping| {
            mapping.generated.start <= prelude
                && prelude < mapping.generated.end
                && mapping.source.is_none()
                && mapping.quality == SourceMapQuality::GeneratedOnly
        }));
    }

    #[test]
    fn rejects_portugol_rs_assignment_to_immutable_binding() {
        let source = "funcao inicio() { seja idade: inteiro = 16; idade = 17; }";
        let errors = export_portugol_rs(source).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.code == "pors.immutable-assignment")
        );
    }

    #[test]
    fn generated_program_compiles_and_has_the_same_output() {
        let generated = export(SOURCE).unwrap();
        let dir = std::env::temp_dir().join(format!("portugol-rust-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .expect("rustc deve estar disponível nos testes do compilador");
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary).output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "42");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn emits_scoped_conditionals_and_string_concatenation_as_valid_rust() {
        let source = r#"programa { funcao inicio() { cadeia nome = "Ana" escreva("Oi, " + nome) se (1 < 2) { escreva("!") } } }"#;
        let generated = export_with_map(source).unwrap();
        assert!(
            generated
                .code
                .contains(r#"format!("{}{}", String::from("Oi, "), pt_nome)"#)
        );
        assert!(generated.code.contains("if (1_i32) < (2_i32) {"));
        let source_start = source.find("se (1 < 2)").unwrap() as u32;
        let mapping = generated
            .mappings
            .iter()
            .find(|mapping| {
                mapping
                    .source
                    .is_some_and(|range| range.start == source_start)
                    && mapping.quality == SourceMapQuality::Exact
            })
            .expect("se deve ter um source map exato");
        assert!(
            generated.code[mapping.generated.start as usize..mapping.generated.end as usize]
                .trim_start()
                .starts_with("if ")
        );
        let dir =
            std::env::temp_dir().join(format!("portugol-rust-condition-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source_path = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source_path, generated.code).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary).output().unwrap();
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "Oi, Ana!");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn preserves_portugol_integer_width_and_wraparound() {
        let source = "programa { funcao inicio() { inteiro n = 2147483647 n += 1 escreva(n) } }";
        let generated = export(source).unwrap();
        assert_rust_output(generated, "-2147483648");
    }

    #[test]
    fn reports_integer_literals_outside_the_portugol_integer_range() {
        let source = "programa { funcao inicio() { escreva(2147483648) } }";
        let diagnostic = export(source).unwrap_err();
        assert_eq!(diagnostic.code, "rust.unsupported");
        assert!(diagnostic.message.contains("32 bits"));
    }

    #[test]
    fn exports_the_minimum_signed_integer_literal() {
        let source = "programa { funcao inicio() { inteiro n = -2147483648 escreva(n) } }";
        let generated = export(source).unwrap();
        assert!(generated.contains("let mut pt_n: i32 = i32::MIN;"));
        assert_rust_output(generated, "-2147483648");
    }

    #[test]
    fn keeps_the_decimal_part_when_printing_real_values() {
        let generated = export("programa { funcao inicio() { escreva(1.0) } }").unwrap();
        assert_rust_output(generated, "1.0");
    }

    #[test]
    fn preserves_integer_negation_wraparound() {
        let source = "programa { funcao inicio() { inteiro n = -2147483648 n = -n escreva(n) } }";
        assert_rust_output(export(source).unwrap(), "-2147483648");
    }

    #[test]
    fn clones_strings_on_declaration_and_assignment() {
        let source = "programa { funcao inicio() { cadeia origem = \"oi\" cadeia destino = \"\" destino = origem escreva(origem, destino) } }";
        assert_rust_output(export(source).unwrap(), "oioi");
    }

    #[test]
    fn mangles_rust_reserved_and_special_identifiers() {
        let source = "programa { funcao inicio() { inteiro gen = 1 inteiro yield = 2 inteiro Some = 3 inteiro _ = 4 escreva(gen, yield, Some, _) } }";
        assert_rust_output(export(source).unwrap(), "1234");
    }

    #[test]
    fn prints_logicos_in_portugol_spelling() {
        let source = "programa { funcao inicio() { escreva(verdadeiro, falso) } }";
        assert_rust_output(export(source).unwrap(), "verdadeirofalso");
    }

    #[test]
    fn formats_real_exponents_like_portugol() {
        let source = "programa { funcao inicio() { escreva(10000000.0, 0.0001) } }";
        assert_rust_output(export(source).unwrap(), "1.0E71.0E-4");
    }

    #[test]
    fn initializes_uninitialized_scalars_to_their_portugol_defaults() {
        let source = "programa { funcao inicio() { inteiro n se (falso) { n = 1 } escreva(n) } }";
        assert_rust_output(export(source).unwrap(), "0");
    }

    #[test]
    fn lowers_unary_plus_to_a_valid_rust_expression() {
        let source = "programa { funcao inicio() { escreva(+1) } }";
        assert_rust_output(export(source).unwrap(), "1");
    }

    #[test]
    fn preserves_operator_grouping_from_the_ast() {
        let source = "programa { funcao inicio() { escreva(~1 + 2, 1 < 2 == verdadeiro) } }";
        assert_rust_output(export(source).unwrap(), "0verdadeiro");
    }

    #[test]
    fn scopes_constants_in_declaration_order() {
        let source = "programa { funcao inicio() { inteiro n = 1 se (verdadeiro) { escreva(n) const inteiro n = 2 escreva(n) } } }";
        assert_rust_output(export(source).unwrap(), "12");
    }

    #[test]
    fn shared_hir_codegen_matches_legacy_output_for_supported_programs() {
        let examples = [
            "programa { funcao inicio() { inteiro total = 40 + 2 escreva(total) } }",
            "programa { funcao inicio() { cadeia nome = \"Oi\" escreva(nome + \"!\") } }",
            "programa { funcao inicio() { caracter a = 'a' caracter b = 'b' escreva(a + b) } }",
            "programa { funcao inicio() { inteiro n = 1 se (n < 2) { escreva(n) } senao { escreva(0) } } }",
            "programa { funcao inicio() { escreva(-2147483648, +1, 0.25, verdadeiro) } }",
        ];
        for source in examples {
            assert_eq!(
                export(source).unwrap(),
                export_legacy(source).unwrap(),
                "{source}"
            );
        }
    }

    #[test]
    fn rejects_compound_operators_without_rust_equivalents() {
        let source = "programa { funcao inicio() { logico ativo = verdadeiro ativo += falso } }";
        assert_eq!(export(source).unwrap_err().code, "rust.unsupported");
    }

    #[test]
    fn rejects_string_concatenation_that_needs_implicit_conversion() {
        let source = "programa { funcao inicio() { escreva(\"n=\" + 1) } }";
        assert_eq!(export(source).unwrap_err().code, "rust.unsupported");
    }

    #[test]
    fn evaluates_all_escreva_arguments_before_writing_any_of_them() {
        let source = "programa { funcao inicio() { escreva(\"prefixo\", 1 / 0) } }";
        let generated = export(source).unwrap();
        let dir = std::env::temp_dir().join(format!("portugol-rust-write-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source_path = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source_path, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exports_and_runs_visual_mvu_pors_apps_as_standalone_rust_and_cargo_projects() {
        use std::io::Write;
        use std::process::Stdio;

        let counter_source = r#"
enum Humor {
    Neutro,
    Animado
}

registro Estado {
    contagem: inteiro,
    nome: cadeia,
    humor: Humor
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "Visitante", humor: Humor::Neutro };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome, humor: Humor::Animado };
    }
    se (tipo == "clique" && alvo == "zerar") {
        retorne Estado { contagem: 0, nome: estado.nome, humor: Humor::Neutro };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor, humor: estado.humor };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    seja mutavel selo: cadeia = "Neutro";
    escolha estado.humor {
        caso Humor::Neutro => { selo = "Neutro"; }
        caso Humor::Animado => { selo = "Animado"; }
    }
    retorne UI.coluna([
        UI.texto("Olá, ", estado.nome, "!"),
        UI.texto("Contagem: ", estado.contagem, " (", selo, ")"),
        UI.linha([
            UI.botao("inc", "+1"),
            UI.botao("zerar", "Zerar")
        ]),
        UI.entrada("nome", estado.nome, "Digite seu nome"),
        UI.lista([UI.texto("Passo ", estado.contagem)])
    ]);
}
"#;
        let project =
            crate::export_portugol_rs_cargo_project(counter_source, "123 Contador Visual!")
                .unwrap();
        assert_eq!(project.package_name, "pors-123-contador-visual");
        assert!(project.cargo_toml.contains("edition = \"2024\""));
        assert!(project.readme_md.contains("UI declarativa MVU"));

        let dir = std::env::temp_dir().join(format!("portugol-rust-ui-{}", std::process::id()));
        crate::write_cargo_project(&project, &dir).unwrap();
        let binary = dir.join(format!("counter{}", std::env::consts::EXE_SUFFIX));
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(dir.join("src/main.rs"))
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );

        let mut child = Command::new(&binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                b"clique:inc\nentrada_alterada:nome:Ada\n{\"kind\":\"clique\",\"id\":\"zerar\"}\n",
            )
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        let lines: Vec<&str> = std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .collect();
        assert_eq!(lines.len(), 4, "{lines:?}");
        assert!(lines[0].contains("\"step\":0"));
        assert!(lines[0].contains("\"content\":\"Contagem: 0 (Neutro)\""));
        assert!(lines[1].contains("\"step\":1"));
        assert!(lines[1].contains("\"content\":\"Contagem: 1 (Animado)\""));
        assert!(lines[1].contains(
            "\"kind\":\"lista\",\"items\":[{\"kind\":\"texto\",\"content\":\"Passo 1\"}]"
        ));
        assert!(lines[2].contains("\"step\":2"));
        assert!(lines[2].contains("\"content\":\"Olá, Ada!\""));
        assert!(lines[2].contains("\"value\":\"Ada\""));
        assert!(lines[3].contains("\"step\":3"));
        assert!(lines[3].contains("\"content\":\"Contagem: 0 (Neutro)\""));

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exports_and_executes_builtin_option_result_and_try_as_native_rust() {
        let source = r#"
registro Item { valor: inteiro }

funcao buscar(n: inteiro) -> Opcao<Item> {
    se (n > 0) {
        retorne Opcao::Algum(Item { valor: n * 10 });
    }
    retorne Opcao::Nenhum;
}

funcao duplicar_opcional(opt: Opcao<inteiro>) -> Opcao<inteiro> {
    seja a: inteiro = opt?;
    seja b: inteiro = opt?;
    retorne Opcao::Algum(a + b);
}

funcao somar_com_dez(n: inteiro) -> Opcao<inteiro> {
    retorne Opcao::Algum(10 + buscar(n)?.valor);
}

funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao rotular_divisao(a: inteiro, b: inteiro) -> Resultado<cadeia, cadeia> {
    seja total: inteiro = 1 + dividir(a, b)?;
    se (total >= 5) {
        retorne Resultado::Ok("alto");
    }
    retorne Resultado::Ok("baixo");
}

funcao inicio() {
    seja lista: vetor<Opcao<inteiro>> = [Opcao::Algum(7), Opcao::Nenhum];
    escolha duplicar_opcional(lista[0]) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha somar_com_dez(3) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha somar_com_dez(0) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha rotular_divisao(20, 4) {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("E:", e, "|"); }
    }
    escolha rotular_divisao(20, 0) {
        caso Resultado::Ok(s) => { escreva(s); }
        caso Resultado::Erro(e) => { escreva("E:", e); }
    }
}
"#;
        let generated = export_portugol_rs(source).unwrap();
        assert!(generated.contains("Option<pt_Item>"));
        assert!(generated.contains("Option<i32>"));
        assert!(generated.contains("Result<i32, String>"));
        assert!(generated.contains("Result<String, String>"));
        assert!(generated.contains("Option::Some("));
        assert!(generated.contains("Option::None"));
        assert!(generated.contains("Result::Ok("));
        assert!(generated.contains("Result::Err("));
        assert!(generated.contains(")?"));
        assert!(!generated.contains("enum pt_Opcao"));
        assert!(!generated.contains("enum pt_Resultado"));
        assert_rust_output(generated, "14|40|N|alto|E:zero");
    }

    #[test]
    fn exports_and_executes_pors_matematica_texto_and_tipos_with_identical_semantics() {
        let source = r#"
funcao somar_entradas(a: cadeia, b: cadeia) -> Resultado<inteiro, cadeia> {
    seja x: inteiro = Tipos.para_inteiro(Texto.aparar(a))?;
    seja y: inteiro = Tipos.para_inteiro_base(b, 16)?;
    retorne Resultado::Ok(x + y);
}

funcao inicio() {
    seja pi2: real = Matematica.arredondar(Matematica.PI, 2);
    seja pot: real = Matematica.potencia(2.0, 3.0);
    seja raiz: real = Matematica.raiz_quadrada(81.0);
    seja abs_i: inteiro = Matematica.valor_absoluto_inteiro(-7);
    seja max_i: inteiro = Matematica.maior_inteiro(3, 9);
    escreva(pi2, "|", pot, "|", raiz, "|", abs_i, "|", max_i, "|");

    seja frase: cadeia = "  olá 🦀  ";
    seja limpa: cadeia = Texto.aparar(frase);
    seja alta: cadeia = Texto.caixa_alta(limpa);
    seja trocada: cadeia = Texto.substituir(limpa, "olá", "oi");
    seja pad: cadeia = Texto.preencher_a_esquerda('0', 5, "42");
    escreva(Texto.numero_caracteres(limpa), "|", alta, "|", trocada, "|", pad, "|");

    escolha Texto.obter_caracter(limpa, 4) {
        caso Opcao::Algum(c) => { escreva(c, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha Texto.posicao_texto("🦀", limpa, 0) {
        caso Opcao::Algum(pos) => { escreva(pos, "|"); }
        caso Opcao::Nenhum => { escreva("-1|"); }
    }
    escolha Texto.extrair_subtexto(limpa, 0, 3) {
        caso Resultado::Ok(sub) => { escreva(sub, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Texto.extrair_subtexto(limpa, 2, 99) {
        caso Resultado::Ok(sub) => { escreva(sub, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }

    escolha somar_entradas(" 26 ", "0x10") {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha somar_entradas("abc", "0x10") {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Tipos.para_real("2.5") {
        caso Resultado::Ok(r) => { escreva(Tipos.real_para_cadeia(r), "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Tipos.inteiro_para_cadeia_base(255, 16) {
        caso Resultado::Ok(hex) => { escreva(hex, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escreva(Tipos.cadeia_e_inteiro("123"), "|", Tipos.cadeia_e_logico("verdadeiro"));
}
"#;
        let generated = export_portugol_rs(source).unwrap();
        assert!(generated.contains("mod __pors_matematica"));
        assert!(generated.contains("mod __pors_texto"));
        assert!(generated.contains("mod __pors_tipos"));
        assert!(!generated.contains("mod __pors_ui"));
        assert_rust_output(
            generated,
            "3.14|8.0|9.0|7|9|5|OLÁ 🦀|oi 🦀|00042|🦀|4|olá|ERR|42|ERR|2.5|FF|verdadeiro|verdadeiro",
        );

        let minimal = export_portugol_rs("funcao inicio() { escreva(1); }").unwrap();
        assert!(!minimal.contains("mod __pors_matematica"));
        assert!(!minimal.contains("mod __pors_texto"));
        assert!(!minimal.contains("mod __pors_tipos"));
    }

    #[test]
    fn exports_and_executes_dynamic_vectors_and_texto_dividir_as_native_rust() {
        let source = r#"
registro Ponto { x: inteiro }

funcao somar_topo_e_primeiro(mutavel itens: vetor<inteiro>) -> Opcao<inteiro> {
    seja topo: inteiro = remover(itens)?;
    seja primeiro: inteiro = obter(itens, 0)?;
    retorne Opcao::Algum(topo + primeiro);
}

funcao inicio() {
    seja mutavel nums: vetor<inteiro> = [];
    escreva(tamanho(nums), "|");
    adicionar(nums, 5);
    adicionar(nums, 15);
    adicionar(nums, 25);
    escreva(tamanho(nums), "|");
    escolha somar_topo_e_primeiro(nums) {
        caso Opcao::Algum(s) => { escreva(s, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    remover(nums);
    escreva(tamanho(nums), "|");
    escolha obter(nums, -1) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha obter(nums, 99) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }

    seja mutavel pontos: vetor<Ponto> = [];
    adicionar(pontos, Ponto { x: 7 });
    escolha obter(pontos, 0) {
        caso Opcao::Algum(p) => { escreva(p.x, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }

    seja partes: vetor<cadeia> = Texto.dividir("um,dois,três", ",");
    escreva(tamanho(partes), "|", partes[1], "|");
    seja letras: vetor<cadeia> = Texto.dividir("a🦀", "");
    escreva(tamanho(letras), "|", letras[1]);
}
"#;
        let generated = export_portugol_rs(source).unwrap();
        assert!(generated.contains("Vec::<i32>::new()"));
        assert!(generated.contains(".push("));
        assert!(generated.contains(".pop()"));
        assert!(generated.contains(".get("));
        assert!(generated.contains("__pors_texto::dividir("));
        assert_rust_output(generated, "0|3|30|2|N|N|7|3|dois|2|🦀");
    }

    #[test]
    fn exports_and_executes_pors_util_and_calendario_deterministically() {
        let source = r#"
funcao inicio() {
    Util.definir_semente(42);
    escolha Util.sorteia(1, 100) {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Util.sorteia(10, 5) {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escreva(Util.tempo_decorrido(), "|");
    Util.aguarde(1500);
    escreva(Util.tempo_decorrido(), "|");

    escreva(
        Calendario.data_iso_atual(), "|",
        Calendario.ano_atual(), "-",
        Calendario.mes_atual(), "-",
        Calendario.dia_mes_atual(), "|",
        Calendario.dia_semana_atual(), "|",
        Calendario.hora_atual(falso), ":",
        Calendario.hora_atual(verdadeiro), ":",
        Calendario.minuto_atual(), ":",
        Calendario.segundo_atual(), ".",
        Calendario.milisegundo_atual(), "|",
        Calendario.eh_bissexto(2024), "|",
        Calendario.DIA_TERCA_FEIRA, "|",
        Calendario.MES_NOVEMBRO, "|"
    );
    escolha Calendario.dia_semana_completo(Calendario.dia_semana_atual(), falso, falso) {
        caso Resultado::Ok(d) => { escreva(d, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.nome_mes(Calendario.mes_atual()) {
        caso Resultado::Ok(m) => { escreva(m, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.dias_no_mes(2024, Calendario.MES_FEVEREIRO) {
        caso Resultado::Ok(d) => { escreva(d, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.dias_no_mes(2024, 13) {
        caso Resultado::Ok(d) => { escreva(d); }
        caso Resultado::Erro(_) => { escreva("ERR"); }
    }
}
"#;
        let generated = export_portugol_rs(source).unwrap();
        assert!(generated.contains("mod __pors_clock"));
        assert!(generated.contains("mod __pors_util"));
        assert!(generated.contains("mod __pors_calendario"));

        let dir =
            std::env::temp_dir().join(format!("portugol-rust-util-cal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source_path = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source_path, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary)
            .env("PORTUGOL_PORS_SIMULATED_EPOCH_MS", "1700000000500")
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.ends_with(
                "|ERR|0|1500|2023-11-14|2023-11-14|3|22:10:13:22.0|verdadeiro|3|11|Terça-Feira|Novembro|29|ERR"
            ),
            "saída inesperada: {stdout}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exports_and_executes_workspace_confined_pors_arquivos_in_native_rust() {
        let source = r#"
funcao ciclo() -> Resultado<cadeia, cadeia> {
    seja _w: logico = Arquivos.escrever_texto("sub/notas.txt", "alpha\n")?;
    seja _a: logico = Arquivos.acrescentar_texto("sub/notas.txt", "beta\n")?;
    seja tem: logico = Arquivos.existe("sub/notas.txt")?;
    seja linhas: vetor<cadeia> = Arquivos.ler_linhas("sub/notas.txt")?;
    seja itens: vetor<cadeia> = Arquivos.listar("sub")?;
    seja _r: logico = Arquivos.remover("sub/notas.txt")?;
    seja depois: logico = Arquivos.existe("sub/notas.txt")?;
    retorne Resultado::Ok(
        Tipos.logico_para_cadeia(tem) + "|" +
        Tipos.inteiro_para_cadeia(tamanho(linhas)) + "|" +
        linhas[1] + "|" +
        itens[0] + "|" +
        Tipos.logico_para_cadeia(depois)
    );
}

funcao inicio() {
    escolha ciclo() {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("ERR:", e, "|"); }
    }
    escolha Arquivos.ler_texto("../fora.txt") {
        caso Resultado::Ok(_) => { escreva("LEAK|"); }
        caso Resultado::Erro(_) => { escreva("BLOCKED_PARENT|"); }
    }
    escolha Arquivos.ler_texto("/etc/passwd") {
        caso Resultado::Ok(_) => { escreva("LEAK"); }
        caso Resultado::Erro(_) => { escreva("BLOCKED_ABS"); }
    }
}
"#;
        let generated = export_portugol_rs(source).unwrap();
        assert!(generated.contains("mod __pors_arquivos"));

        let dir =
            std::env::temp_dir().join(format!("portugol-rust-arquivos-{}", std::process::id()));
        let ws = dir.join("workspace");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&ws).unwrap();
        let source_path = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source_path, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary)
            .env("PORTUGOL_PORS_WORKSPACE_DIR", &ws)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "verdadeiro|2|beta|notas.txt|falso|BLOCKED_PARENT|BLOCKED_ABS"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exports_and_executes_zero_dep_pors_json_in_native_rust() {
        let source = r#"
funcao executar() -> Resultado<cadeia, cadeia> {
    seja mutavel doc: cadeia = "{}";
    doc = Json.definir_cadeia(doc, "nome", "Ada \"Lovelace\"\n")?;
    doc = Json.definir_inteiro(doc, "ano", 1843)?;
    doc = Json.definir_real(doc, "nota", 9.5)?;
    doc = Json.definir_logico(doc, "ativo", verdadeiro)?;
    doc = Json.definir_json(doc, "tags", "[\"pors\",\"rust\"]")?;
    doc = Json.definir_json(doc, "nums", "[10, 20, 30]")?;
    doc = Json.definir_json(doc, "meta", "{\"emoji\":\"\\uD83D\\uDE00\"}")?;

    seja nome: cadeia = Json.obter_cadeia(doc, "nome")?;
    seja ano: inteiro = Json.obter_inteiro(doc, "ano")?;
    seja nota: real = Json.obter_real(doc, "nota")?;
    seja ativo: logico = Json.obter_logico(doc, "ativo")?;
    seja tags: vetor<cadeia> = Json.obter_lista_cadeia(doc, "tags")?;
    seja nums: vetor<inteiro> = Json.obter_lista_inteiro(doc, "nums")?;
    seja meta: cadeia = Json.obter_json(doc, "meta")?;
    seja emoji: cadeia = Json.obter_cadeia(meta, "emoji")?;
    seja chaves: vetor<cadeia> = Json.obter_chaves(doc)?;

    seja sem_ativo: cadeia = Json.remover_chave(doc, "ativo")?;
    seja tem_ativo: logico = Json.contem_chave(sem_ativo, "ativo")?;
    seja bonito: cadeia = Json.formatar("{\"a\":1,\"b\":[true,null]}")?;
    seja compacto: cadeia = Json.minificar(bonito)?;

    retorne Resultado::Ok(
        Texto.aparar(nome) + "|" +
        Tipos.inteiro_para_cadeia(ano) + "|" +
        Tipos.real_para_cadeia(nota) + "|" +
        Tipos.logico_para_cadeia(ativo) + "|" +
        tags[1] + "|" +
        Tipos.inteiro_para_cadeia(nums[0] + nums[2]) + "|" +
        emoji + "|" +
        Tipos.inteiro_para_cadeia(tamanho(chaves)) + "|" +
        Tipos.logico_para_cadeia(tem_ativo) + "|" +
        compacto + "|" +
        Json.escapar_cadeia("a\"b")
    );
}

funcao inicio() {
    escolha executar() {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("ERR:", e, "|"); }
    }
    escreva(Json.valido("{\"ok\":1}"), "|", Json.valido("{invalido}"), "|");
    escolha Json.obter_inteiro("{\"x\":3.5}", "x") {
        caso Resultado::Ok(_) => { escreva("BAD_INT|"); }
        caso Resultado::Erro(_) => { escreva("INT_ERR|"); }
    }
    seja mutavel fundo: cadeia = "0";
    para (seja mutavel i: inteiro = 0; i < 70; i += 1) {
        fundo = "[" + fundo + "]";
    }
    escreva(Json.valido(fundo));
}
"#;
        let generated = export_portugol_rs(source).unwrap();
        assert!(generated.contains("mod __pors_json"));
        assert_rust_output(
            generated,
            "Ada \"Lovelace\"|1843|9.5|verdadeiro|rust|40|\u{1F600}|7|falso|{\"a\":1,\"b\":[true,null]}|\"a\\\"b\"|verdadeiro|falso|INT_ERR|falso",
        );
    }

    #[test]
    fn exports_and_executes_bitwise_shifts_and_choose_wildcard_in_native_rust() {
        let source = r#"
enum Cor {
    Vermelho,
    Verde,
    Azul
}

funcao rotulo(c: Cor) -> cadeia {
    escolha c {
        caso Cor::Vermelho => { retorne "R"; }
        caso _ => { retorne "OUTRA"; }
    }
}

funcao inicio() {
    seja mutavel bits: inteiro = (1 << 4) | 5;
    bits &= 21;
    bits |= 2;
    bits ^= 1;
    seja shifted: inteiro = bits >> 1;
    seja not_zero: inteiro = ~0;
    escreva(
        bits, "|",
        shifted, "|",
        not_zero, "|",
        rotulo(Cor::Vermelho), "|",
        rotulo(Cor::Azul)
    );
}
"#;
        let generated = export_portugol_rs(source).unwrap();
        assert!(generated.contains("wrapping_shl"));
        assert!(generated.contains("wrapping_shr"));
        assert!(generated.contains("_ =>"));
        assert_rust_output(generated, "22|11|-1|R|OUTRA");
    }

    fn assert_rust_output(generated: String, expected: &str) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "portugol-rust-{}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary).output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
