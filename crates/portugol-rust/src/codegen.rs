use portugol_hir::{
    AssignOp, BinaryOp, BindingId, Expr, ExprKind, Program, Stmt, StmtKind, Type, UnaryOp,
};

use crate::{JAVA_DOUBLE_HELPER, default_value, format_real, rust_ident};

pub(crate) fn render(program: &Program) -> String {
    let mut output = String::from("use std::io::{self, Write};\n\n");
    output.push_str(JAVA_DOUBLE_HELPER);
    output.push_str("\nfn main() {\n");
    render_statements(&program.body, program, 1, &mut output);
    output.push_str("}\n");
    output
}

fn render_statements(statements: &[Stmt], program: &Program, depth: usize, output: &mut String) {
    let indent = "    ".repeat(depth);
    for statement in statements {
        match &statement.kind {
            StmtKind::Declare {
                binding,
                initializer,
            } => {
                let declaration = binding_info(program, *binding);
                output.push_str(&format!(
                    "{indent}{} {}: {}",
                    if declaration.is_const {
                        "let"
                    } else {
                        "let mut"
                    },
                    rust_ident(&declaration.source_name),
                    rust_type(declaration.ty)
                ));
                let initializer = initializer
                    .as_ref()
                    .map(|value| owned_value(value, program))
                    .unwrap_or_else(|| default_value(map_type(declaration.ty)).to_owned());
                output.push_str(&format!(" = {initializer};\n"));
            }
            StmtKind::Assign { binding, op, value } => {
                let declaration = binding_info(program, *binding);
                let name = rust_ident(&declaration.source_name);
                let ty = declaration.ty;
                if ty == Type::Integer {
                    let method = match op {
                        AssignOp::Add => Some("wrapping_add"),
                        AssignOp::Subtract => Some("wrapping_sub"),
                        AssignOp::Multiply => Some("wrapping_mul"),
                        AssignOp::Divide => Some("wrapping_div"),
                        AssignOp::Remainder => Some("wrapping_rem"),
                        _ => None,
                    };
                    if let Some(method) = method {
                        output.push_str(&format!(
                            "{indent}{name} = ({name}).{method}({});\n",
                            render_expr(value, program)
                        ));
                        continue;
                    }
                }
                if ty == Type::String && *op == AssignOp::Add {
                    output.push_str(&format!(
                        "{indent}{name} = format!(\"{{}}{{}}\", {name}, {});\n",
                        owned_value(value, program)
                    ));
                    continue;
                }
                let assignment_op = match op {
                    AssignOp::Set => "=",
                    AssignOp::Add => "+=",
                    AssignOp::Subtract => "-=",
                    AssignOp::Multiply => "*=",
                    AssignOp::Divide => "/=",
                    AssignOp::Remainder => "%=",
                    AssignOp::Xor => "^=",
                    AssignOp::BitAnd => "&=",
                    AssignOp::BitOr => "|=",
                };
                let value = if *op == AssignOp::Set {
                    owned_value(value, program)
                } else {
                    render_expr(value, program)
                };
                output.push_str(&format!("{indent}{name} {assignment_op} {value};\n"));
            }
            StmtKind::Write(values) => {
                let temps: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("__portugol_write_{index}"))
                    .collect();
                for (temp, value) in temps.iter().zip(values) {
                    output.push_str(&format!(
                        "{indent}let {temp} = {};\n",
                        owned_value(value, program)
                    ));
                }
                for (temp, value) in temps.iter().zip(values) {
                    let expression = match value.ty {
                        Type::Boolean => {
                            format!("if {temp} {{ \"verdadeiro\" }} else {{ \"falso\" }}")
                        }
                        Type::Real => format!("__portugol_java_double({temp})"),
                        _ => temp.clone(),
                    };
                    output.push_str(&format!("{indent}print!(\"{{}}\", {expression});\n"));
                }
                output.push_str(&format!("{indent}let _ = io::stdout().flush();\n"));
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                output.push_str(&format!(
                    "{indent}if {} {{\n",
                    render_expr(condition, program)
                ));
                render_statements(then_branch, program, depth + 1, output);
                output.push_str(&format!("{indent}}}"));
                if let Some(else_branch) = else_branch {
                    output.push_str(" else {\n");
                    render_statements(else_branch, program, depth + 1, output);
                    output.push_str(&format!("{indent}}}"));
                }
                output.push('\n');
            }
        }
    }
}

fn render_expr(expr: &Expr, program: &Program) -> String {
    match &expr.kind {
        ExprKind::Binding(id) => rust_ident(&binding_info(program, *id).source_name),
        ExprKind::Integer(value) if *value == i32::MIN => "i32::MIN".to_owned(),
        ExprKind::Integer(value) => format!("{value}_i32"),
        ExprKind::Real(value) => format_real(*value),
        ExprKind::Boolean(value) => value.to_string(),
        ExprKind::String(value) => format!("String::from({value:?})"),
        ExprKind::Character(value) => format!("{value:?}"),
        ExprKind::Parenthesized(inner) => format!("({})", render_expr(inner, program)),
        ExprKind::Unary { op, operand } => {
            let inner = render_expr(operand, program);
            match (op, expr.ty) {
                (UnaryOp::Negate, Type::Integer) => format!("({inner}).wrapping_neg()"),
                (UnaryOp::Negate, Type::Real) => format!("-({inner})"),
                (UnaryOp::Positive, _) => inner,
                (UnaryOp::Not | UnaryOp::BitNot, _) => format!("!({inner})"),
                _ => unreachable!("unary operand types are validated by the lowering"),
            }
        }
        ExprKind::Binary { op, left, right } => {
            let left_ty = left.ty;
            let right_ty = right.ty;
            let left = render_expr(left, program);
            let right = render_expr(right, program);
            if *op == BinaryOp::Add
                && ((left_ty == Type::String && right_ty == Type::String)
                    || (left_ty == Type::Character && right_ty == Type::Character))
            {
                return format!("format!(\"{{}}{{}}\", {left}, {right})");
            }
            if expr.ty == Type::Integer {
                let method = match op {
                    BinaryOp::Add => Some("wrapping_add"),
                    BinaryOp::Subtract => Some("wrapping_sub"),
                    BinaryOp::Multiply => Some("wrapping_mul"),
                    BinaryOp::Divide => Some("wrapping_div"),
                    BinaryOp::Remainder => Some("wrapping_rem"),
                    BinaryOp::ShiftLeft => Some("wrapping_shl"),
                    BinaryOp::ShiftRight => Some("wrapping_shr"),
                    _ => None,
                };
                if let Some(method) = method {
                    let right = if matches!(op, BinaryOp::ShiftLeft | BinaryOp::ShiftRight) {
                        format!("({right} as u32)")
                    } else {
                        right
                    };
                    return format!("({left}).{method}({right})");
                }
            }
            format!("({left}) {} ({right})", binary_operator(*op))
        }
    }
}

fn binary_operator(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Remainder => "%",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::Xor => "^",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Greater => ">",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
    }
}

fn owned_value(expr: &Expr, program: &Program) -> String {
    let value = render_expr(expr, program);
    if expr.ty == Type::String {
        format!("({value}).clone()")
    } else {
        value
    }
}

fn binding_info(program: &Program, id: BindingId) -> &portugol_hir::Binding {
    &program.bindings[id.0 as usize]
}

fn rust_type(ty: Type) -> &'static str {
    match ty {
        Type::Integer => "i32",
        Type::Real => "f64",
        Type::Boolean => "bool",
        Type::String => "String",
        Type::Character => "char",
    }
}

fn map_type(ty: Type) -> crate::Ty {
    match ty {
        Type::Integer => crate::Ty::Inteiro,
        Type::Real => crate::Ty::Real,
        Type::Boolean => crate::Ty::Logico,
        Type::String => crate::Ty::Cadeia,
        Type::Character => crate::Ty::Caracter,
    }
}
