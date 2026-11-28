//! Vínculos `{dados.campo}` de uma tela (ADR 0018, nível 3 e Fase 4).
//!
//! A sintaxe é conferida pelo `ferro-tela`; aqui fica o que depende do
//! code-behind: o campo existe no registro de `Dados` e tem o tipo do
//! atributo, os `<DadosExemplo>` combinam com o registro e o código não
//! atribui a uma propriedade que a marcação vincula (a tela tem uma única
//! fonte de verdade: `dados`).

use ferro_hir::{self as hir, Diagnostic, Span, Type};
use ferro_tela::codebehind::{FieldSource, FieldType, ScreenType};
use ferro_tela::schema::{self, Binding, ValueType};
use ferro_tela::value::{TextPart, single_binding, text_parts};
use ferro_tela::{Document, Element};

/// Uma propriedade cujo valor usa vínculos.
#[derive(Debug, Clone)]
pub(crate) struct BoundProperty {
    /// `Nome` do controle; `None` para a própria janela.
    pub control: Option<String>,
    pub attribute: &'static str,
    pub binding: Binding,
    pub value: ValueType,
    pub parts: Vec<TextPart>,
}

impl BoundProperty {
    /// O único caminho, quando o valor é exatamente um vínculo.
    pub fn single_path(&self) -> Option<&[String]> {
        single_binding(&self.parts).map(|(path, _)| path)
    }

    pub fn is_text(&self) -> bool {
        self.value == ValueType::Text
    }

    /// `{dados.nome}` em `CaixaTexto`, `Marcado` e `Selecionado`: o que o
    /// usuário faz no controle vai para `dados`.
    pub fn is_two_way(&self) -> bool {
        self.binding == Binding::TwoWay && self.single_path().is_some()
    }
}

fn collect(element: &Element, is_root: bool, out: &mut Vec<BoundProperty>) {
    if let Some(element_schema) = schema::element(&element.name.text) {
        for attribute in &element.attributes {
            let Some(attribute_schema) = element_schema.attribute(&attribute.name.text) else {
                continue;
            };
            if attribute_schema.binding == Binding::None {
                continue;
            }
            let Ok(parts) = text_parts(&attribute.raw, attribute.value_span.start) else {
                continue;
            };
            if !parts
                .iter()
                .any(|part| matches!(part, TextPart::Binding { .. }))
            {
                continue;
            }
            let control = element.control_name().map(|n| n.value.clone());
            if !is_root && control.is_none() {
                continue;
            }
            out.push(BoundProperty {
                control: if is_root { None } else { control },
                attribute: attribute_schema.name,
                binding: attribute_schema.binding,
                value: attribute_schema.value,
                parts,
            });
        }
    }
    for child in element.elements() {
        collect(child, false, out);
    }
}

/// Propriedades vinculadas da janela e dos controles nomeados.
pub(crate) fn bound_properties(document: &Document) -> Vec<BoundProperty> {
    let mut out = Vec::new();
    collect(&document.root, true, &mut out);
    out
}

fn type_name(program: &hir::Program, ty: Type) -> String {
    match ty {
        Type::Integer => "inteiro".into(),
        Type::Real => "real".into(),
        Type::Boolean => "logico".into(),
        Type::String => "cadeia".into(),
        Type::Character => "caracter".into(),
        Type::Record(id) => program
            .record_types
            .iter()
            .find(|r| r.id == id)
            .map_or("registro".into(), |r| r.name.clone()),
        Type::Enum(id) => program
            .enum_types
            .iter()
            .find(|e| e.id == id)
            .map_or("enum".into(), |e| e.name.clone()),
        Type::Unit => "vazio".into(),
        _ => "vetor".into(),
    }
}

fn is_scalar(ty: Type) -> bool {
    matches!(
        ty,
        Type::Integer | Type::Real | Type::Boolean | Type::String | Type::Character
    )
}

/// Tipo do caminho `dados.a.b`, ou o diagnóstico de onde ele para.
fn resolve(
    program: &hir::Program,
    data: Type,
    path: &[String],
    span: Span,
) -> Result<Type, Diagnostic> {
    let mut current = data;
    for (depth, field) in path.iter().enumerate().skip(1) {
        let shown = path[..=depth].join(".");
        let Type::Record(id) = current else {
            return Err(Diagnostic::error(
                "design.binding-not-record",
                format!(
                    "`{}` não é um registro; não tem o campo `{field}`",
                    path[..depth].join(".")
                ),
                span,
            ));
        };
        let record = program
            .record_types
            .iter()
            .find(|r| r.id == id)
            .expect("registro existe");
        let Some(found) = record.fields.iter().find(|f| f.name == *field) else {
            let options: Vec<_> = record.fields.iter().map(|f| f.name.as_str()).collect();
            let help = if options.is_empty() {
                format!("O registro `{}` ainda não tem campos.", record.name)
            } else {
                format!("Campos de `{}`: {}.", record.name, options.join(", "))
            };
            return Err(Diagnostic::error(
                "design.binding-unknown-field",
                format!(
                    "`{shown}` não existe: o registro `{}` não tem o campo `{field}`",
                    record.name
                ),
                span,
            )
            .with_help(help));
        };
        current = found.ty;
    }
    Ok(current)
}

/// Tipo que um atributo não textual exige do vínculo.
fn expected(value: ValueType) -> Option<Type> {
    match value {
        ValueType::Bool => Some(Type::Boolean),
        ValueType::Integer { .. } => Some(Type::Integer),
        _ => None,
    }
}

/// Confere cada vínculo contra o registro de `Dados`.
pub(crate) fn check_bindings(
    document: &Document,
    program: &hir::Program,
    data: Option<Type>,
) -> Vec<Diagnostic> {
    let Some(data) = data else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();
    for property in bound_properties(document) {
        let attribute = property.attribute;
        for part in &property.parts {
            let TextPart::Binding { path, span } = part else {
                continue;
            };
            let ty = match resolve(program, data, path, *span) {
                Ok(ty) => ty,
                Err(diagnostic) => {
                    diagnostics.push(diagnostic);
                    continue;
                }
            };
            let shown = path.join(".");
            let wanted = if property.is_text() {
                (property.binding == Binding::TwoWay).then_some(Type::String)
            } else {
                expected(property.value)
            };
            match wanted {
                Some(wanted) if ty != wanted => diagnostics.push(
                    Diagnostic::error(
                        "design.binding-type",
                        format!(
                            "`{attribute}` espera {}, mas `{shown}` é {}",
                            type_name(program, wanted),
                            type_name(program, ty)
                        ),
                        *span,
                    )
                    .with_help(if property.binding == Binding::TwoWay {
                        format!("Num vínculo de mão dupla, o campo precisa ser {}.", type_name(program, wanted))
                    } else {
                        format!("Use um campo {} de `dados`.", type_name(program, wanted))
                    }),
                ),
                None if !is_scalar(ty) => diagnostics.push(
                    Diagnostic::error(
                        "design.binding-type",
                        format!("`{shown}` é {} e não pode ser mostrado como texto", type_name(program, ty)),
                        *span,
                    )
                    .with_help("Mostre um campo de tipo simples: inteiro, real, logico, cadeia ou caracter."),
                ),
                _ => {}
            }
        }
    }
    diagnostics.extend(check_sample_data(document, program, data));
    diagnostics
}

/// `<DadosExemplo campo="valor" />`: campos de `dados` com valores do tipo deles.
fn check_sample_data(document: &Document, program: &hir::Program, data: Type) -> Vec<Diagnostic> {
    let Some(sample) = document
        .root
        .elements()
        .find(|e| e.name.text == schema::SAMPLE_DATA)
    else {
        return Vec::new();
    };
    let Type::Record(id) = data else {
        return Vec::new();
    };
    let record = program
        .record_types
        .iter()
        .find(|r| r.id == id)
        .expect("registro existe");
    let mut diagnostics = Vec::new();
    for attribute in &sample.attributes {
        let field = attribute.name.text.as_str();
        let Some(found) = record.fields.iter().find(|f| f.name == field) else {
            diagnostics.push(Diagnostic::error(
                "design.sample-unknown-field",
                format!("O registro `{}` não tem o campo `{field}`", record.name),
                attribute.name.span,
            ));
            continue;
        };
        let value = attribute.value.trim();
        let ok = match found.ty {
            Type::Integer => value.parse::<i32>().is_ok(),
            Type::Real => value.replace(',', ".").parse::<f64>().is_ok(),
            Type::Boolean => matches!(value, "verdadeiro" | "falso"),
            Type::Character => attribute.value.chars().count() == 1,
            Type::String => true,
            _ => false,
        };
        if !ok {
            let ty = type_name(program, found.ty);
            diagnostics.push(
                Diagnostic::error(
                    "design.sample-type",
                    format!("O exemplo de `{field}` precisa ser {ty}"),
                    attribute.value_span,
                )
                .with_help(match found.ty {
                    Type::Boolean => "Use `verdadeiro` ou `falso`.".to_owned(),
                    Type::Integer => "Use um número inteiro, como `42`.".to_owned(),
                    Type::Real => "Use um número, como `3.5`.".to_owned(),
                    Type::Character => "Use um único caractere.".to_owned(),
                    _ => format!("Campos do tipo {ty} não têm exemplo."),
                }),
            );
        }
    }
    diagnostics
}

fn visit<'a>(statements: &'a [hir::Stmt], out: &mut Vec<&'a hir::Stmt>) {
    for statement in statements {
        out.push(statement);
        match &statement.kind {
            hir::StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                visit(then_branch, out);
                if let Some(otherwise) = else_branch {
                    visit(otherwise, out);
                }
            }
            hir::StmtKind::While { body, .. } | hir::StmtKind::DoWhile { body, .. } => {
                visit(body, out)
            }
            hir::StmtKind::For {
                initializer,
                step,
                body,
                ..
            } => {
                visit(std::slice::from_ref(initializer), out);
                visit(std::slice::from_ref(step), out);
                visit(body, out);
            }
            hir::StmtKind::Choose { arms, .. } => {
                for arm in arms {
                    visit(&arm.body, out);
                }
            }
            _ => {}
        }
    }
}

fn shown_binding(property: &BoundProperty) -> String {
    match property.single_path() {
        Some(path) => format!("`{{{}}}`", path.join(".")),
        None => "`dados`".to_owned(),
    }
}

/// Atribuições do code-behind a propriedades vinculadas na marcação
/// (`t.lblResultado.texto = ...` com `Texto="{dados.resultado}"`).
pub(crate) fn check_read_only(
    document: &Document,
    screen: &ScreenType,
    program: &hir::Program,
) -> Vec<Diagnostic> {
    let bound = bound_properties(document);
    if bound.is_empty() {
        return Vec::new();
    }
    let Some(window) = program
        .record_types
        .iter()
        .find(|r| r.name == screen.window.name)
    else {
        return Vec::new();
    };
    let window_type = Type::Record(window.id);
    let mut diagnostics = Vec::new();
    for function in &program.functions {
        let mut statements = Vec::new();
        visit(&function.body, &mut statements);
        for statement in statements {
            let hir::StmtKind::FieldAssign {
                binding, fields, ..
            } = &statement.kind
            else {
                continue;
            };
            let Some(variable) = function.bindings.iter().find(|b| b.id == *binding) else {
                continue;
            };
            if variable.ty != window_type {
                continue;
            }
            let Some(window_field) = fields.first().and_then(|i| screen.window.fields.get(*i))
            else {
                continue;
            };
            let (control, property_field) = match (&window_field.source, &window_field.ty) {
                (FieldSource::Control(_), FieldType::Named(tag)) => {
                    let Some(definition) = screen.controls.iter().find(|c| c.name == *tag) else {
                        continue;
                    };
                    let Some(field) = fields.get(1).and_then(|i| definition.fields.get(*i)) else {
                        continue;
                    };
                    (Some(window_field.name.clone()), field)
                }
                (FieldSource::Attribute(_), _) => (None, window_field),
                _ => continue,
            };
            let FieldSource::Attribute(attribute) = property_field.source else {
                continue;
            };
            let Some(property) = bound
                .iter()
                .find(|p| p.control == control && p.attribute == attribute)
            else {
                continue;
            };
            let place = match &control {
                Some(name) => format!("{}.{name}.{}", variable.source_name, property_field.name),
                None => format!("{}.{}", variable.source_name, property_field.name),
            };
            let fix = property
                .single_path()
                .map(|path| {
                    format!(
                        "Mude `{}.{}`; a tela acompanha.",
                        variable.source_name,
                        path.join(".")
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "Mude os campos de `{}.dados` que a marcação mostra.",
                        variable.source_name
                    )
                });
            diagnostics.push(
                Diagnostic::error(
                    "design.binding-read-only",
                    format!(
                        "`{place}` está vinculado a {} na tela e só pode ser lido no código",
                        shown_binding(property)
                    ),
                    statement.span,
                )
                .with_help(fix),
            );
        }
    }
    diagnostics
}
