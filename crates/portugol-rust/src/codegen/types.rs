use portugol_hir::{
    BinaryOp, BindingId, BuiltinEnum, EnumDef, Expr, ExprKind, Program, Stmt, StmtKind, Type,
};

use crate::rust_ident;

use super::expr::{ExpressionPart, render_expr};

pub(super) fn integer_method(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("wrapping_add"),
        BinaryOp::Subtract => Some("wrapping_sub"),
        BinaryOp::Multiply => Some("wrapping_mul"),
        BinaryOp::Divide => Some("wrapping_div"),
        BinaryOp::Remainder => Some("wrapping_rem"),
        BinaryOp::ShiftLeft => Some("wrapping_shl"),
        BinaryOp::ShiftRight => Some("wrapping_shr"),
        _ => None,
    }
}

pub(super) fn binary_operator(op: BinaryOp) -> &'static str {
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

pub(super) fn binding_info(program: &Program, id: BindingId) -> &portugol_hir::Binding {
    &program.bindings[id.0 as usize]
}

pub(super) fn statements_have_read(statements: &[Stmt]) -> bool {
    statements
        .iter()
        .any(|statement| statement_has_read(&statement.kind))
}

fn statement_has_read(kind: &StmtKind) -> bool {
    use StmtKind as S;
    match kind {
        S::Read { .. } => true,
        S::Declare { initializer, .. } => initializer.as_ref().is_some_and(expression_has_read),
        S::Assign { value, .. } | S::FieldAssign { value, .. } => expression_has_read(value),
        S::Choose { value, arms } => {
            expression_has_read(value) || arms.iter().any(|arm| statements_have_read(&arm.body))
        }
        S::Call { arguments, .. } | S::HostCall { arguments, .. } => {
            arguments.iter().any(expression_has_read)
        }
        S::Write(arguments) => arguments.iter().any(expression_has_read),
        S::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expression_has_read(condition)
                || statements_have_read(then_branch)
                || else_branch.as_deref().is_some_and(statements_have_read)
        }
        S::While { condition, body } | S::DoWhile { condition, body } => {
            expression_has_read(condition) || statements_have_read(body)
        }
        S::For {
            initializer,
            condition,
            step,
            body,
        } => {
            expression_has_read(condition)
                || statement_has_read(&initializer.kind)
                || statement_has_read(&step.kind)
                || statements_have_read(body)
        }
        S::Return { value } => value.as_ref().is_some_and(expression_has_read),
        S::IndexAssign { index, value, .. } => {
            expression_has_read(index) || expression_has_read(value)
        }
        S::VectorPush { value, .. } => expression_has_read(value),
        S::LoopControl { .. } | S::VectorPop { .. } => false,
    }
}

pub(super) fn read_target_label(target: Option<Type>) -> &'static str {
    match target {
        Some(Type::Integer) => "inteiro",
        Some(Type::Real) => "real",
        Some(Type::Character) => "caracter",
        Some(Type::Boolean) => "logico",
        Some(Type::String) | None => "cadeia",
        _ => unreachable!("tipo de leitura não suportado: {target:?}"),
    }
}

pub(super) fn expression_has_read(expr: &Expr) -> bool {
    use ExprKind as E;
    match &expr.kind {
        E::Read { .. } => true,
        E::Call { arguments, .. }
        | E::HostCall { arguments, .. }
        | E::VectorLiteral(arguments)
        | E::RecordLiteral {
            fields: arguments, ..
        }
        | E::EnumConstructor {
            payload: arguments, ..
        } => arguments.iter().any(expression_has_read),
        E::Field { value, .. }
        | E::Index { index: value, .. }
        | E::VectorGet { index: value, .. }
        | E::Parenthesized(value)
        | E::Try { operand: value }
        | E::Unary { operand: value, .. } => expression_has_read(value),
        E::InterpolatedString { parts } => parts.iter().any(|part| match part {
            portugol_hir::StringPart::Literal(_, _) => false,
            portugol_hir::StringPart::Expression(value, _) => expression_has_read(value),
        }),
        E::Binary { left, right, .. } => expression_has_read(left) || expression_has_read(right),
        E::Binding(_)
        | E::Length { .. }
        | E::VectorPop { .. }
        | E::Integer(_)
        | E::Real(_)
        | E::Boolean(_)
        | E::String(_)
        | E::Character(_) => false,
    }
}

pub(super) fn pors_clone_value(ty: Type, value: &str, program: &Program) -> String {
    match ty {
        Type::String => format!("__pors_string({value})"),
        Type::VectorString => format!("__pors_vec_string_clone({value})"),
        Type::Record(id) => program
            .record_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| format!("__pors_clone_{}({value})", item.id.0))
            .unwrap_or_else(|| format!("(*{value}).clone()")),
        Type::VectorRecord(id) => program
            .record_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| {
                format!(
                    "{value}.iter().map(|item| __pors_clone_{}(item)).collect()",
                    item.id.0
                )
            })
            .unwrap_or_else(|| format!("{value}.clone()")),
        Type::Enum(id) => program
            .enum_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| format!("__pors_clone_enum_{}({value})", item.id.0))
            .unwrap_or_else(|| format!("(*{value}).clone()")),
        Type::VectorEnum(id) => program
            .enum_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| {
                format!(
                    "{value}.iter().map(|item| __pors_clone_enum_{}(item)).collect()",
                    item.id.0
                )
            })
            .unwrap_or_else(|| format!("{value}.clone()")),
        vector if vector.is_vector() => format!("{value}.clone()"),
        Type::Unit => "()".to_owned(),
        _ => format!("*{value}"),
    }
}

pub(super) fn rust_type(ty: Type, program: &Program) -> String {
    match ty {
        Type::Integer => "i32".into(),
        Type::Real => "f64".into(),
        Type::Boolean => "bool".into(),
        Type::String => "String".into(),
        Type::Character => "char".into(),
        Type::Unit => "()".into(),
        Type::VectorInteger => "Vec<i32>".into(),
        Type::VectorReal => "Vec<f64>".into(),
        Type::VectorBoolean => "Vec<bool>".into(),
        Type::VectorString => "Vec<String>".into(),
        Type::VectorCharacter => "Vec<char>".into(),
        Type::Record(id) => program
            .record_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| rust_ident(&definition.name))
            .unwrap_or_else(|| "__invalid_record".into()),
        Type::VectorRecord(id) => program
            .record_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| format!("Vec<{}>", rust_ident(&definition.name)))
            .unwrap_or_else(|| "Vec<()>".into()),
        Type::Enum(id) => program
            .enum_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| rust_enum_type_name(definition, program))
            .unwrap_or_else(|| "__invalid_enum".into()),
        Type::VectorEnum(id) => program
            .enum_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| format!("Vec<{}>", rust_enum_type_name(definition, program)))
            .unwrap_or_else(|| "Vec<()>".into()),
    }
}

pub(super) fn rust_enum_type_name(definition: &EnumDef, program: &Program) -> String {
    match definition.builtin {
        Some(BuiltinEnum::Option(inner)) => format!("Option<{}>", rust_type(inner, program)),
        Some(BuiltinEnum::Result { ok, err }) => {
            format!(
                "Result<{}, {}>",
                rust_type(ok, program),
                rust_type(err, program)
            )
        }
        None => rust_ident(&definition.name),
    }
}

pub(super) fn rust_enum_variant_path(definition: &EnumDef, variant_index: usize) -> String {
    match definition.builtin {
        Some(BuiltinEnum::Option(_)) => {
            if variant_index == 0 {
                "Option::Some".to_owned()
            } else {
                "Option::None".to_owned()
            }
        }
        Some(BuiltinEnum::Result { .. }) => {
            if variant_index == 0 {
                "Result::Ok".to_owned()
            } else {
                "Result::Err".to_owned()
            }
        }
        None => {
            let variant_name = definition
                .variants
                .get(variant_index)
                .map(|variant| rust_ident(&variant.name))
                .unwrap_or_else(|| "__invalid_variant".to_owned());
            format!("{}::{variant_name}", rust_ident(&definition.name))
        }
    }
}

pub(super) fn map_type(ty: Type) -> crate::Ty {
    match ty {
        Type::Integer => crate::Ty::Inteiro,
        Type::Real => crate::Ty::Real,
        Type::Boolean => crate::Ty::Logico,
        Type::String => crate::Ty::Cadeia,
        Type::Character => crate::Ty::Caracter,
        Type::Unit => unreachable!("tipo unit não possui valor Portugol"),
        Type::VectorInteger
        | Type::VectorReal
        | Type::VectorBoolean
        | Type::VectorString
        | Type::VectorCharacter
        | Type::VectorRecord(_)
        | Type::VectorEnum(_) => unreachable!("vetor não possui tipo escalar"),
        Type::Record(_) | Type::Enum(_) => unreachable!("tipo nominal não possui tipo escalar"),
    }
}

pub(super) fn render_ui_host_call(
    name: &str,
    arguments: &[Expr],
    program: &Program,
    pors: bool,
) -> Vec<ExpressionPart> {
    match name {
        "texto" => {
            let mut parts = vec![ExpressionPart::Text("__pors_ui::texto(&[".to_owned())];
            for (index, argument) in arguments.iter().enumerate() {
                if index > 0 {
                    parts.push(ExpressionPart::Text(", ".to_owned()));
                }
                let rendered = render_expr(argument, program, pors);
                match argument.ty {
                    Type::String => {
                        parts.push(ExpressionPart::Text("(&(".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(")).to_string()".to_owned()));
                    }
                    Type::Real => {
                        parts.push(ExpressionPart::Text("__portugol_java_double(".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(")".to_owned()));
                    }
                    Type::Boolean => {
                        parts.push(ExpressionPart::Text("(if ".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(
                            " { \"verdadeiro\" } else { \"falso\" }).to_owned()".to_owned(),
                        ));
                    }
                    _ => {
                        parts.push(ExpressionPart::Text("(".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(").to_string()".to_owned()));
                    }
                }
            }
            parts.push(ExpressionPart::Text("])".to_owned()));
            parts
        }
        "botao" => vec![
            ExpressionPart::Text("__pors_ui::botao(&(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[0], program, pors)),
            ExpressionPart::Text("), &(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[1], program, pors)),
            ExpressionPart::Text("))".to_owned()),
        ],
        "entrada" => vec![
            ExpressionPart::Text("__pors_ui::entrada(&(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[0], program, pors)),
            ExpressionPart::Text("), &(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[1], program, pors)),
            ExpressionPart::Text("), &(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[2], program, pors)),
            ExpressionPart::Text("))".to_owned()),
        ],
        "coluna" | "linha" | "lista" => vec![
            ExpressionPart::Text(format!("__pors_ui::{name}(&(")),
            ExpressionPart::Nested(render_expr(&arguments[0], program, pors)),
            ExpressionPart::Text("))".to_owned()),
        ],
        _ => vec![ExpressionPart::Text("0_i32".to_owned())],
    }
}

pub(super) fn render_stdlib_host_call(
    rust_mod: &str,
    name: &str,
    arguments: &[Expr],
    program: &Program,
    pors: bool,
) -> Vec<ExpressionPart> {
    let mut parts = vec![ExpressionPart::Text(format!("{rust_mod}::{name}("))];
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            parts.push(ExpressionPart::Text(", ".to_owned()));
        }
        let rendered = render_expr(argument, program, pors);
        if argument.ty == Type::String {
            parts.push(ExpressionPart::Text("&(".to_owned()));
            parts.push(ExpressionPart::Nested(rendered));
            parts.push(ExpressionPart::Text(")".to_owned()));
        } else {
            parts.push(ExpressionPart::Nested(rendered));
        }
    }
    parts.push(ExpressionPart::Text(")".to_owned()));
    parts
}

pub(super) fn mvu_entry_update_arity(program: &Program) -> Option<usize> {
    let has_init = program
        .functions
        .iter()
        .any(|f| f.source_name == "iniciar" && f.parameters.is_empty());
    let has_view = program
        .functions
        .iter()
        .any(|f| f.source_name == "visualizar" && f.parameters.len() == 1);
    let update = program
        .functions
        .iter()
        .find(|f| f.source_name == "atualizar" && (2..=4).contains(&f.parameters.len()))?;
    (has_init && has_view).then_some(update.parameters.len())
}

pub(super) fn program_uses_host_module(program: &Program, target: &str) -> bool {
    stmts_use_host_module(&program.body, target)
        || program.functions.iter().any(|function| {
            stmts_use_host_module(&function.body, target)
                || function
                    .tail_expression
                    .as_ref()
                    .is_some_and(|expr| expr_uses_host_module(expr, target))
        })
}

fn stmts_use_host_module(stmts: &[Stmt], target: &str) -> bool {
    stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Declare { initializer, .. } => initializer
            .as_ref()
            .is_some_and(|expr| expr_uses_host_module(expr, target)),
        StmtKind::Assign { value, .. } | StmtKind::FieldAssign { value, .. } => {
            expr_uses_host_module(value, target)
        }
        StmtKind::Choose { value, arms } => {
            expr_uses_host_module(value, target)
                || arms
                    .iter()
                    .any(|arm| stmts_use_host_module(&arm.body, target))
        }
        StmtKind::Read { .. } | StmtKind::LoopControl { .. } => false,
        StmtKind::Call { arguments, .. } | StmtKind::Write(arguments) => arguments
            .iter()
            .any(|expr| expr_uses_host_module(expr, target)),
        StmtKind::HostCall {
            module, arguments, ..
        } => {
            module == target
                || arguments
                    .iter()
                    .any(|expr| expr_uses_host_module(expr, target))
        }
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_uses_host_module(condition, target)
                || stmts_use_host_module(then_branch, target)
                || else_branch
                    .as_deref()
                    .is_some_and(|branch| stmts_use_host_module(branch, target))
        }
        StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
            expr_uses_host_module(condition, target) || stmts_use_host_module(body, target)
        }
        StmtKind::For {
            initializer,
            condition,
            step,
            body,
        } => {
            stmts_use_host_module(std::slice::from_ref(initializer.as_ref()), target)
                || expr_uses_host_module(condition, target)
                || stmts_use_host_module(std::slice::from_ref(step.as_ref()), target)
                || stmts_use_host_module(body, target)
        }
        StmtKind::Return { value } => value
            .as_ref()
            .is_some_and(|expr| expr_uses_host_module(expr, target)),
        StmtKind::IndexAssign { index, value, .. } => {
            expr_uses_host_module(index, target) || expr_uses_host_module(value, target)
        }
        StmtKind::VectorPush { value, .. } => expr_uses_host_module(value, target),
        StmtKind::VectorPop { .. } => false,
    })
}

fn expr_uses_host_module(expr: &Expr, target: &str) -> bool {
    match &expr.kind {
        ExprKind::HostCall {
            module, arguments, ..
        } => {
            module == target
                || arguments
                    .iter()
                    .any(|arg| expr_uses_host_module(arg, target))
        }
        ExprKind::Call { arguments, .. }
        | ExprKind::VectorLiteral(arguments)
        | ExprKind::RecordLiteral {
            fields: arguments, ..
        }
        | ExprKind::EnumConstructor {
            payload: arguments, ..
        } => arguments
            .iter()
            .any(|arg| expr_uses_host_module(arg, target)),
        ExprKind::Field { value, .. }
        | ExprKind::Index { index: value, .. }
        | ExprKind::VectorGet { index: value, .. }
        | ExprKind::Parenthesized(value)
        | ExprKind::Try { operand: value }
        | ExprKind::Unary { operand: value, .. } => expr_uses_host_module(value, target),
        ExprKind::InterpolatedString { parts } => parts.iter().any(|part| match part {
            portugol_hir::StringPart::Literal(_, _) => false,
            portugol_hir::StringPart::Expression(value, _) => expr_uses_host_module(value, target),
        }),
        ExprKind::Binary { left, right, .. } => {
            expr_uses_host_module(left, target) || expr_uses_host_module(right, target)
        }
        ExprKind::Binding(_)
        | ExprKind::Length { .. }
        | ExprKind::VectorPop { .. }
        | ExprKind::Integer(_)
        | ExprKind::Real(_)
        | ExprKind::Boolean(_)
        | ExprKind::String(_)
        | ExprKind::Read { .. }
        | ExprKind::Character(_) => false,
    }
}
