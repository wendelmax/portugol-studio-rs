//! Conversão verificada de um subconjunto de Portugol em Rust compilável.

use std::collections::HashMap;

use portugol_syntax::ast::{
    AssignOp, Assignment, BinOp, Declarations, Declarator, Expr as AstExpr, Function, Item,
    Program, Stmt as AstStmt, Type, UnaryOp,
};
use portugol_syntax::{Diagnostic, Severity, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    Inteiro,
    Real,
    Logico,
    Cadeia,
    Caracter,
}

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

#[derive(Debug)]
struct HProgram {
    body: Vec<HStmt>,
}

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

#[derive(Debug)]
struct HExpr {
    code: String,
    ty: Ty,
    span: Span,
}

/// Gera Rust completo para um subconjunto documentado de Portugol.
/// Erros de sintaxe/semântica existentes são preservados; o primeiro recurso
/// fora do subconjunto retorna `rust.unsupported` com o span original.
pub fn export(source: &str) -> Result<String, Diagnostic> {
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

struct Lowerer {
    scopes: Vec<HashMap<String, Ty>>,
}

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
            return Err(unsupported(
                "programa sem uma única função inicio",
                program.span,
            ));
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
        let Declarator::Variable { name, init, span } = &declarations.declarators[0] else {
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
        if is_reserved_identifier(&name.name) {
            return Err(unsupported(
                "identificador reservado pela sintaxe Rust",
                name.span,
            ));
        }
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.name.clone(), ty);
        let _ = span;
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
        if is_reserved_identifier(&ident.name) {
            return Err(unsupported(
                "identificador reservado pela sintaxe Rust",
                ident.span,
            ));
        }
        let ty = self
            .lookup(&ident.name)
            .ok_or_else(|| unsupported("variável fora do escopo", ident.span))?;
        let value = self.expression(&assignment.value)?;
        if !compatible(ty, value.ty) {
            return Err(unsupported("conversao implicita de tipo", value.span));
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
                        (format!("{}.wrapping_neg()", expr.code), Ty::Inteiro)
                    } else {
                        let (operator, ty) = match op {
                            UnaryOp::Neg if matches!(expr.ty, Ty::Inteiro | Ty::Real) => {
                                ("-", expr.ty)
                            }
                            UnaryOp::Plus if matches!(expr.ty, Ty::Inteiro | Ty::Real) => {
                                ("+", expr.ty)
                            }
                            UnaryOp::Not if expr.ty == Ty::Logico => ("!", Ty::Logico),
                            UnaryOp::BitNot if expr.ty == Ty::Inteiro => ("!", Ty::Inteiro),
                            _ => return Err(unsupported("operacao unaria", span)),
                        };
                        (format!("{operator}{}", expr.code), ty)
                    }
                }
            }
            AstExpr::Binary {
                op, left, right, ..
            } => {
                let left = self.expression(left)?;
                let right = self.expression(right)?;
                if *op == BinOp::Add
                    && (left.ty == Ty::Cadeia
                        || right.ty == Ty::Cadeia
                        || (left.ty == Ty::Caracter && right.ty == Ty::Caracter))
                {
                    (
                        format!(
                            "format!(\"{}{}\", {}, {})",
                            format_placeholder(left.ty),
                            format_placeholder(right.ty),
                            left.code,
                            right.code
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
                            (format!("{}.{}({right})", left.code, method), ty)
                        } else {
                            let operator = match op {
                                BinOp::Xor => "^",
                                BinOp::BitAnd => "&",
                                BinOp::BitOr => "|",
                                _ => unreachable!(),
                            };
                            (format!("{} {operator} {}", left.code, right.code), ty)
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
                        (format!("{} {operator} {}", left.code, right.code), ty)
                    }
                }
            }
            _ => return Err(unsupported("expressão", span)),
        };
        Ok(HExpr { code, ty, span })
    }
}

fn compatible(target: Ty, value: Ty) -> bool {
    target == value
}

fn format_placeholder(ty: Ty) -> &'static str {
    if ty == Ty::Real { "{:?}" } else { "{}" }
}

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

fn binary_type(op: BinOp, left: Ty, right: Ty) -> Option<Ty> {
    match op {
        BinOp::Add if left == Ty::Cadeia || right == Ty::Cadeia => Some(Ty::Cadeia),
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

fn render(program: &HProgram) -> String {
    let mut output = String::from("use std::io::{self, Write};\n\nfn main() {\n");
    render_statements(&program.body, 1, &mut output);
    output.push_str("}\n");
    output
}

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
                    if *is_const { "const" } else { "let mut" },
                    ty.rust()
                ));
                if let Some(init) = init {
                    output.push_str(&format!(" = {}", init.code));
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
                            "{indent}{name} = {name}.{method}({});\n",
                            value.code
                        ));
                        continue;
                    }
                }
                if *ty == Ty::Cadeia && *op == AssignOp::Add {
                    output.push_str(&format!(
                        "{indent}{name} = format!(\"{{}}{{}}\", {name}, {});\n",
                        value.code
                    ));
                    continue;
                }
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
                output.push_str(&format!("{indent}{name} {op} {};\n", value.code));
            }
            HStmt::Write(values) => {
                for value in values {
                    output.push_str(&format!(
                        "{indent}print!(\"{}\", {});\n",
                        format_placeholder(value.ty),
                        value.code
                    ));
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
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn",
    ];
    if KEYWORDS.contains(&name) && !matches!(name, "self" | "Self" | "super" | "crate") {
        format!("r#{name}")
    } else {
        name.to_owned()
    }
}

fn is_reserved_identifier(name: &str) -> bool {
    matches!(name, "self" | "Self" | "super" | "crate")
}

fn format_real(value: f64) -> String {
    let value = value.to_string();
    if value.contains(['.', 'e', 'E']) {
        value
    } else {
        format!("{value}.0")
    }
}

fn unsupported(feature: &str, span: Span) -> Diagnostic {
    Diagnostic::error(
        "rust.unsupported",
        format!("A exportação para Rust ainda não oferece suporte a {feature} neste programa."),
        span,
    )
}

#[cfg(test)]
mod tests {
    use super::export;
    use std::process::Command;

    const SOURCE: &str = "programa { funcao inicio() { inteiro total = 40 + 2 escreva(total) } }";

    #[test]
    fn lowers_typed_declarations_and_preserves_write_without_newline() {
        let generated = export(SOURCE).unwrap();
        assert!(generated.contains("let mut total: i32 = 40_i32.wrapping_add(2_i32);"));
        assert!(generated.contains("print!(\"{}\", total);"));
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
        let generated = export(source).unwrap();
        assert!(generated.contains(r#"format!("{}{}", String::from("Oi, "), nome)"#));
        assert!(generated.contains("if 1_i32 < 2_i32 {"));
        let dir =
            std::env::temp_dir().join(format!("portugol-rust-condition-{}", std::process::id()));
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
        assert!(generated.contains("let mut n: i32 = i32::MIN;"));
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
