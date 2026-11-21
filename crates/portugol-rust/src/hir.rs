use std::collections::HashMap;

use portugol_hir as hir;
use portugol_syntax::ast::{self, AssignOp, BinOp, Declarator, Expr, Item, Stmt, Type, UnaryOp};
use portugol_syntax::{Diagnostic, Severity, Span};

/// Analisa Portugol clássico e baixa o subconjunto suportado para a HIR comum.
/// Não gera código do destino; nomes são resolvidos para `BindingId`.
pub fn lower(source: &str) -> Result<hir::Program, Diagnostic> {
    let parsed = portugol_syntax::parse(source);
    if let Some(error) = parsed
        .diagnostics
        .iter()
        .find(|d| d.severity == Severity::Error)
    {
        return Err(error.clone());
    }
    if let Some(error) = portugol_semantics::analyze(&parsed.program)
        .into_iter()
        .find(|d| d.severity == Severity::Error)
    {
        return Err(error);
    }
    Lowerer::new().program(&parsed.program)
}

struct Lowerer {
    scopes: Vec<HashMap<String, (hir::BindingId, hir::Type)>>,
    bindings: Vec<hir::Binding>,
}

impl Lowerer {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            bindings: vec![],
        }
    }
    fn program(mut self, program: &ast::Program) -> Result<hir::Program, Diagnostic> {
        if !program.includes.is_empty() {
            return Err(unsupported("bibliotecas", program.includes[0].span));
        }
        let functions: Vec<_> = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(f) => Some(f),
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
                    Item::Declarations(d) => Some(d.span),
                    Item::Stmt(s) => Some(s.span()),
                    Item::Function(_) => None,
                })
                .unwrap_or(program.span);
            return Err(unsupported("comandos fora de funcao inicio", span));
        }
        if functions.len() != 1 || functions[0].name.name != "inicio" {
            return Err(unsupported(
                "programa sem uma única função inicio",
                functions.first().map_or(program.span, |f| f.name.span),
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
        Ok(hir::Program {
            bindings: self.bindings,
            body,
            span: span(program.span),
        })
    }
    fn statements(&mut self, statements: &[Stmt]) -> Result<Vec<hir::Stmt>, Diagnostic> {
        statements.iter().map(|s| self.statement(s)).collect()
    }
    fn statement(&mut self, stmt: &Stmt) -> Result<hir::Stmt, Diagnostic> {
        let node_span = span(stmt.span());
        let kind = match stmt {
            Stmt::Declarations(d) => {
                let ty = map_type(d.ty).ok_or_else(|| unsupported("tipo vago", d.span))?;
                if d.declarators.len() != 1 {
                    return Err(unsupported("declaração múltipla", d.span));
                }
                let Declarator::Variable { name, init, .. } = &d.declarators[0] else {
                    return Err(unsupported("vetor ou matriz", d.span));
                };
                let init = init.as_ref().map(|e| self.expression(e)).transpose()?;
                if init.as_ref().is_some_and(|e| e.ty != ty) {
                    return Err(unsupported("conversao implicita de tipo", d.span));
                }
                if d.is_const && (init.is_none() || ty == hir::Type::String) {
                    return Err(unsupported(
                        "constante sem expressão constante em Rust",
                        d.span,
                    ));
                }
                let id = hir::BindingId(self.bindings.len() as u32);
                self.bindings.push(hir::Binding {
                    id,
                    source_name: name.name.clone(),
                    ty,
                    mutability: if d.is_const {
                        hir::Mutability::Immutable
                    } else {
                        hir::Mutability::Mutable
                    },
                    is_const: d.is_const,
                    span: span(name.span),
                });
                self.scopes
                    .last_mut()
                    .unwrap()
                    .insert(name.name.clone(), (id, ty));
                hir::StmtKind::Declare {
                    binding: id,
                    initializer: init,
                }
            }
            Stmt::Assign(a) => {
                let Expr::Ident(name) = &a.target else {
                    return Err(unsupported("atribuição indireta", a.target.span()));
                };
                let (binding, ty) = self
                    .lookup(&name.name)
                    .ok_or_else(|| unsupported("variável fora do escopo", name.span))?;
                let value = self.expression(&a.value)?;
                if value.ty != ty {
                    return Err(unsupported("conversao implicita de tipo", a.value.span()));
                }
                hir::StmtKind::Assign {
                    binding,
                    op: map_assign(a.op),
                    value,
                }
            }
            Stmt::Expr {
                expr:
                    Expr::Call {
                        name,
                        args,
                        span: call_span,
                        scope: None,
                    },
                ..
            } if name == "escreva" => hir::StmtKind::Write(
                args.iter()
                    .map(|a| self.expression(a))
                    .collect::<Result<_, _>>()?,
            ),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition = self.expression(condition)?;
                if condition.ty != hir::Type::Boolean {
                    return Err(unsupported("condição não lógica", stmt.span()));
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
                hir::StmtKind::If {
                    condition,
                    then_branch,
                    else_branch,
                }
            }
            Stmt::Expr {
                expr: Expr::Call { name, span, .. },
                ..
            } => return Err(unsupported(name, *span)),
            Stmt::Expr { expr, .. } => return Err(unsupported("expressão", expr.span())),
            Stmt::While { span, .. } => return Err(unsupported("enquanto", *span)),
            Stmt::DoWhile { span, .. } => return Err(unsupported("faca-enquanto", *span)),
            Stmt::For { span, .. } => return Err(unsupported("para", *span)),
            Stmt::Switch { span, .. } => return Err(unsupported("escolha", *span)),
            Stmt::Return { span, .. } => return Err(unsupported("retorno", *span)),
            Stmt::Break { span } => return Err(unsupported("pare", *span)),
        };
        let _ = node_span;
        Ok(hir::Stmt {
            kind,
            span: span(stmt.span()),
        })
    }
    fn lookup(&self, name: &str) -> Option<(hir::BindingId, hir::Type)> {
        self.scopes.iter().rev().find_map(|s| s.get(name).copied())
    }
    fn expression(&self, expr: &Expr) -> Result<hir::Expr, Diagnostic> {
        let sp = span(expr.span());
        let (kind, ty) = match expr {
            Expr::Ident(i) => {
                let (id, ty) = self
                    .lookup(&i.name)
                    .ok_or_else(|| unsupported("variável fora do escopo", i.span))?;
                (hir::ExprKind::Binding(id), ty)
            }
            Expr::Int { value, .. } => (
                hir::ExprKind::Integer(
                    i32::try_from(*value)
                        .map_err(|_| unsupported("literal inteiro fora de 32 bits", expr.span()))?,
                ),
                hir::Type::Integer,
            ),
            Expr::Real { value, .. } => (hir::ExprKind::Real(*value), hir::Type::Real),
            Expr::Bool { value, .. } => (hir::ExprKind::Boolean(*value), hir::Type::Boolean),
            Expr::Char { value, .. } => (hir::ExprKind::Character(*value), hir::Type::Character),
            Expr::Str { value, .. } => (hir::ExprKind::String(value.clone()), hir::Type::String),
            Expr::Paren { expr, .. } => {
                let e = self.expression(expr)?;
                let ty = e.ty;
                return Ok(hir::Expr {
                    kind: e.kind,
                    ty,
                    span: sp,
                });
            }
            Expr::Unary { op, expr, .. } => {
                let e = self.expression(expr)?;
                let ty = match op {
                    UnaryOp::Neg | UnaryOp::Plus
                        if matches!(e.ty, hir::Type::Integer | hir::Type::Real) =>
                    {
                        e.ty
                    }
                    UnaryOp::Not if e.ty == hir::Type::Boolean => hir::Type::Boolean,
                    UnaryOp::BitNot if e.ty == hir::Type::Integer => hir::Type::Integer,
                    _ => return Err(unsupported("operacao unaria", expr.span())),
                };
                (
                    hir::ExprKind::Unary {
                        op: map_unary(*op),
                        operand: Box::new(e),
                    },
                    ty,
                )
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let l = self.expression(left)?;
                let r = self.expression(right)?;
                let ty = binary_type(*op, l.ty, r.ty)
                    .ok_or_else(|| unsupported("operacao entre tipos", expr.span()))?;
                (
                    hir::ExprKind::Binary {
                        op: map_binary(*op),
                        left: Box::new(l),
                        right: Box::new(r),
                    },
                    ty,
                )
            }
            _ => return Err(unsupported("expressão", expr.span())),
        };
        Ok(hir::Expr { kind, ty, span: sp })
    }
}
fn span(s: Span) -> hir::Span {
    hir::Span::new(s.start, s.end)
}
fn map_type(t: Type) -> Option<hir::Type> {
    Some(match t {
        Type::Inteiro => hir::Type::Integer,
        Type::Real => hir::Type::Real,
        Type::Logico => hir::Type::Boolean,
        Type::Cadeia => hir::Type::String,
        Type::Caracter => hir::Type::Character,
        Type::Vazio => return None,
    })
}
fn map_assign(op: AssignOp) -> hir::AssignOp {
    match op {
        AssignOp::Assign => hir::AssignOp::Set,
        AssignOp::Add => hir::AssignOp::Add,
        AssignOp::Sub => hir::AssignOp::Subtract,
        AssignOp::Mul => hir::AssignOp::Multiply,
        AssignOp::Div => hir::AssignOp::Divide,
        AssignOp::Mod => hir::AssignOp::Remainder,
        AssignOp::Xor => hir::AssignOp::Xor,
        AssignOp::BitAnd => hir::AssignOp::BitAnd,
        AssignOp::BitOr => hir::AssignOp::BitOr,
    }
}
fn map_unary(op: UnaryOp) -> hir::UnaryOp {
    match op {
        UnaryOp::Neg => hir::UnaryOp::Negate,
        UnaryOp::Plus => hir::UnaryOp::Positive,
        UnaryOp::Not => hir::UnaryOp::Not,
        UnaryOp::BitNot => hir::UnaryOp::BitNot,
    }
}
fn map_binary(op: BinOp) -> hir::BinaryOp {
    match op {
        BinOp::Add => hir::BinaryOp::Add,
        BinOp::Sub => hir::BinaryOp::Subtract,
        BinOp::Mul => hir::BinaryOp::Multiply,
        BinOp::Div => hir::BinaryOp::Divide,
        BinOp::Mod => hir::BinaryOp::Remainder,
        BinOp::Eq => hir::BinaryOp::Equal,
        BinOp::Ne => hir::BinaryOp::NotEqual,
        BinOp::Gt => hir::BinaryOp::Greater,
        BinOp::Lt => hir::BinaryOp::Less,
        BinOp::Le => hir::BinaryOp::LessEqual,
        BinOp::Ge => hir::BinaryOp::GreaterEqual,
        BinOp::And => hir::BinaryOp::And,
        BinOp::Or => hir::BinaryOp::Or,
        BinOp::Xor => hir::BinaryOp::Xor,
        BinOp::Shl => hir::BinaryOp::ShiftLeft,
        BinOp::Shr => hir::BinaryOp::ShiftRight,
        BinOp::BitAnd => hir::BinaryOp::BitAnd,
        BinOp::BitOr => hir::BinaryOp::BitOr,
    }
}
fn binary_type(op: BinOp, l: hir::Type, r: hir::Type) -> Option<hir::Type> {
    use hir::Type::*;
    Some(match op {
        BinOp::Add if l == String && r == String || l == Character && r == Character => String,
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div
            if l == r && matches!(l, Integer | Real) =>
        {
            l
        }
        BinOp::Mod | BinOp::Xor | BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr
            if l == Integer && r == Integer =>
        {
            Integer
        }
        BinOp::And | BinOp::Or if l == Boolean && r == Boolean => Boolean,
        BinOp::Eq | BinOp::Ne if l == r => Boolean,
        BinOp::Gt | BinOp::Lt | BinOp::Le | BinOp::Ge
            if l == r && matches!(l, Integer | Real | Character) =>
        {
            Boolean
        }
        _ => return None,
    })
}
fn unsupported(feature: &str, span: Span) -> Diagnostic {
    Diagnostic::error(
        "rust.unsupported",
        format!(
            "A representação intermediária ainda não oferece suporte a {feature} neste programa."
        ),
        span,
    )
}

#[cfg(test)]
mod tests {
    use super::lower;
    use portugol_hir::{ExprKind, StmtKind};

    #[test]
    fn lowers_names_to_distinct_bindings_when_shadowed() {
        let source = "programa { funcao inicio() { inteiro valor = 1 se (verdadeiro) { inteiro valor = 2 escreva(valor) } escreva(valor) } }";
        let program = lower(source).expect("subconjunto deve baixar para HIR");
        assert_eq!(program.bindings.len(), 2);
        assert_ne!(program.bindings[0].id, program.bindings[1].id);
        let StmtKind::If { then_branch, .. } = &program.body[1].kind else {
            panic!("esperava if")
        };
        let StmtKind::Write(args) = &then_branch[1].kind else {
            panic!("esperava escreva")
        };
        assert!(matches!(args[0].kind, ExprKind::Binding(id) if id == program.bindings[1].id));
        let StmtKind::Write(args) = &program.body[2].kind else {
            panic!("esperava escreva")
        };
        assert!(matches!(args[0].kind, ExprKind::Binding(id) if id == program.bindings[0].id));
    }

    #[test]
    fn retains_original_source_span_for_unsupported_constructs() {
        let source = "programa { funcao inicio() { inteiro n leia(n) } }";
        let error = lower(source).unwrap_err();
        assert_eq!(error.code, "rust.unsupported");
        assert_eq!(error.span.start, source.find("leia").unwrap() as u32);
    }
}
