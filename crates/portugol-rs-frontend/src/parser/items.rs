use std::collections::HashMap;

use portugol_hir as hir;
use portugol_syntax::{Diagnostic, Span};

use super::constants::*;
use super::{FunctionParameter, FunctionSignature, Parser};
use crate::lexer::{Kind, Token};

impl Parser {
    pub(super) fn collect_signatures(&mut self) -> Vec<Diagnostic> {
        self.cursor = 0;
        let mut next_id = 0;
        let mut diagnostics = Vec::new();
        while !matches!(self.current().kind, Kind::Eof) {
            let item_start = self.cursor;
            let result = (|| {
                if self.eat_keyword("registro") {
                    self.collect_record_definition()?;
                    return Ok(());
                }
                if self.eat_keyword("enum") {
                    self.collect_enum_definition()?;
                    return Ok(());
                }
                let (name, span, signature) = self.function_header()?;
                if self.signatures.contains_key(&name) {
                    return self.fail(
                        "pors.function-redeclared",
                        format!("A função `{name}` já foi declarada"),
                        span,
                    );
                }
                let id = if name == "inicio" {
                    if !signature.parameters.is_empty() || signature.return_type.is_some() {
                        return self.fail(
                            "pors.entry-signature",
                            "A função inicio deve ser declarada sem parâmetros e sem retorno",
                            span,
                        );
                    }
                    None
                } else {
                    let id = hir::FunctionId(next_id);
                    next_id += 1;
                    Some(id)
                };
                self.signatures
                    .insert(name, FunctionSignature { id, ..signature });
                self.skip_function_body()
            })();
            if let Err(errors) = result {
                append_unique(&mut diagnostics, errors);
                let end = self.synchronize_top_level(item_start);
                self.invalid_items.push((item_start, end));
            }
        }
        if !self.signatures.contains_key("inicio") && diagnostics.is_empty() {
            if self.has_mvu_entry_triple() {
                if let Err(errors) = self.validate_mvu_entry_signatures() {
                    append_unique(&mut diagnostics, errors);
                }
            } else {
                append_unique(
                    &mut diagnostics,
                    vec![Diagnostic::error(
                        "pors.entry-function",
                        "O programa precisa declarar uma função inicio()",
                        self.current().span,
                    )],
                );
            }
        }
        self.cursor = 0;
        diagnostics
    }

    pub(super) fn synchronize_top_level(&mut self, item_start: usize) -> usize {
        let mut cursor = self.cursor.max(item_start.saturating_add(1));
        while cursor < self.tokens.len() {
            if cursor > item_start && is_top_level_start(&self.tokens[cursor].kind) {
                self.cursor = cursor;
                return cursor;
            }
            cursor += 1;
        }
        self.cursor = self.tokens.len().saturating_sub(1);
        self.cursor
    }

    pub(super) fn reset_function_state(&mut self) {
        self.bindings.clear();
        self.scopes = vec![HashMap::new()];
        self.return_type = None;
        self.in_function = false;
        self.loop_depth = 0;
        self.function_has_syntax_error = false;
    }

    pub(super) fn has_mvu_entry_triple(&self) -> bool {
        self.signatures.contains_key("iniciar")
            && self.signatures.contains_key("atualizar")
            && self.signatures.contains_key("visualizar")
    }

    pub(super) fn validate_mvu_entry_signatures(&self) -> Result<(), Vec<Diagnostic>> {
        let init = &self.signatures["iniciar"];
        let Some(state_ty) = init.return_type else {
            return self.fail(
                "pors.ui-entry-signature",
                "A função `iniciar` precisa retornar o tipo de estado da interface",
                init.span,
            );
        };
        if !init.parameters.is_empty() {
            return self.fail(
                "pors.ui-entry-signature",
                "A função `iniciar` não deve receber parâmetros",
                init.span,
            );
        }

        let update = &self.signatures["atualizar"];
        if !(2..=4).contains(&update.parameters.len()) {
            return self.fail(
                "pors.ui-entry-signature",
                "A função `atualizar` deve receber entre 2 e 4 parâmetros (estado e dados do evento)",
                update.span,
            );
        }
        if update.parameters[0].ty != state_ty {
            return self.fail(
                "pors.ui-entry-signature",
                "O primeiro parâmetro de `atualizar` precisa ter o mesmo tipo retornado por `iniciar`",
                update.parameters[0].span,
            );
        }
        for param in &update.parameters[1..] {
            if param.ty != hir::Type::String {
                return self.fail(
                    "pors.ui-entry-signature",
                    format!(
                        "O parâmetro `{}` de `atualizar` deve ter tipo `cadeia`",
                        param.name
                    ),
                    param.span,
                );
            }
        }
        if update.return_type != Some(state_ty) {
            return self.fail(
                "pors.ui-entry-signature",
                "A função `atualizar` precisa retornar o mesmo tipo de estado de `iniciar`",
                update.span,
            );
        }

        let view = &self.signatures["visualizar"];
        if view.parameters.len() != 1 {
            return self.fail(
                "pors.ui-entry-signature",
                "A função `visualizar` deve receber exatamente 1 parâmetro (o estado atual)",
                view.span,
            );
        }
        if view.parameters[0].ty != state_ty {
            return self.fail(
                "pors.ui-entry-signature",
                "O parâmetro de `visualizar` precisa ter o mesmo tipo retornado por `iniciar`",
                view.parameters[0].span,
            );
        }
        if view.return_type != Some(hir::Type::Integer) {
            return self.fail(
                "pors.ui-entry-signature",
                "A função `visualizar` deve retornar `inteiro` (identificador do nó raiz de UI)",
                view.span,
            );
        }
        Ok(())
    }

    pub(super) fn function_header(
        &mut self,
    ) -> Result<(String, Span, FunctionSignature), Vec<Diagnostic>> {
        let start = self.current().span;
        self.keyword("funcao")?;
        let (name, name_span) = self.identifier()?;
        if name != "inicio" && is_reserved(&name) {
            return self.fail(
                "pors.reserved-name",
                format!("`{name}` é reservado e não pode ser usado como nome de função"),
                name_span,
            );
        }
        self.expect(Kind::LParen, "Esperado `(` após nome da função")?;
        let mut parameters = Vec::new();
        if !matches!(self.current().kind, Kind::RParen) {
            loop {
                let mutable = self.eat_keyword("mutavel");
                let (parameter_name, parameter_span) = self.identifier()?;
                if is_reserved(&parameter_name) {
                    return self.fail(
                        "pors.reserved-name",
                        format!("`{parameter_name}` é reservado e não pode ser usado como nome"),
                        parameter_span,
                    );
                }
                self.expect(Kind::Colon, "Esperado `:` entre parâmetro e tipo")?;
                let ty = self.type_name()?;
                parameters.push(FunctionParameter {
                    name: parameter_name,
                    ty,
                    mutable,
                    span: parameter_span,
                });
                if !self.eat(&Kind::Comma) {
                    break;
                }
            }
        }
        let close = self.expect(Kind::RParen, "Esperado `)` após parâmetros")?;
        let return_type = if self.eat(&Kind::Arrow) {
            Some(self.type_name()?)
        } else {
            None
        };
        let header_span = start.join(return_type.map_or(close.span, |_| self.previous_span()));
        if name == "inicio" && (!parameters.is_empty() || return_type.is_some()) {
            return self.fail(
                "pors.entry-signature",
                "A função inicio deve ser declarada sem parâmetros e sem retorno",
                name_span.join(header_span),
            );
        }
        Ok((
            name,
            header_span,
            FunctionSignature {
                id: None,
                parameters,
                return_type,
                span: header_span,
            },
        ))
    }

    pub(super) fn intern_builtin_enum(
        &mut self,
        builtin: hir::BuiltinEnum,
        span: Span,
    ) -> hir::Type {
        if let Some(&id) = self.builtin_enums.get(&builtin) {
            return hir::Type::Enum(id);
        }
        let id = hir::TypeId(self.next_type_id);
        self.next_type_id += 1;
        let hir_span = to_hir_span(span);
        let (name, variants) = match builtin {
            hir::BuiltinEnum::Option(inner) => (
                "Opcao".to_owned(),
                vec![
                    hir::EnumVariant {
                        name: "Algum".to_owned(),
                        payload_types: vec![inner],
                        span: hir_span,
                    },
                    hir::EnumVariant {
                        name: "Nenhum".to_owned(),
                        payload_types: Vec::new(),
                        span: hir_span,
                    },
                ],
            ),
            hir::BuiltinEnum::Result { ok, err } => (
                "Resultado".to_owned(),
                vec![
                    hir::EnumVariant {
                        name: "Ok".to_owned(),
                        payload_types: vec![ok],
                        span: hir_span,
                    },
                    hir::EnumVariant {
                        name: "Erro".to_owned(),
                        payload_types: vec![err],
                        span: hir_span,
                    },
                ],
            ),
        };
        self.enum_types.push(hir::EnumDef {
            id,
            name,
            variants,
            builtin: Some(builtin),
            span: hir_span,
        });
        self.builtin_enums.insert(builtin, id);
        hir::Type::Enum(id)
    }

    pub(super) fn builtin_enum_of(&self, ty: hir::Type) -> Option<(hir::TypeId, hir::BuiltinEnum)> {
        let hir::Type::Enum(id) = ty else {
            return None;
        };
        let definition = self.enum_types.iter().find(|item| item.id == id)?;
        definition.builtin.map(|builtin| (id, builtin))
    }

    pub(super) fn expect_type_close(&mut self, message: &str) -> Result<Span, Vec<Diagnostic>> {
        if matches!(self.current().kind, Kind::Shr) {
            let span = self.current().span;
            let first = Span::new(span.start, span.start + 1);
            let second = Span::new(span.start + 1, span.end);
            self.tokens[self.cursor] = Token {
                kind: Kind::Greater,
                span: second,
            };
            return Ok(first);
        }
        Ok(self.expect(Kind::Greater, message)?.span)
    }

    pub(super) fn type_name(&mut self) -> Result<hir::Type, Vec<Diagnostic>> {
        let (name, span) = self.identifier()?;
        if name == "vetor" {
            self.expect(Kind::Less, "Esperado `<` após `vetor`")?;
            let element = self.type_name()?;
            let close_span = self.expect_type_close("Esperado `>` após o tipo do vetor")?;
            return hir::Type::vector_of(element).ok_or_else(|| {
                vec![Diagnostic::error(
                    "pors.vector-element-type",
                    "O tipo de elemento não pode ser um vetor",
                    span.join(close_span),
                )]
            });
        }
        if name == "Opcao" {
            self.expect(Kind::Less, "Esperado `<` após `Opcao`")?;
            let inner = self.type_name()?;
            let close_span = self.expect_type_close("Esperado `>` após o tipo de `Opcao`")?;
            return Ok(
                self.intern_builtin_enum(hir::BuiltinEnum::Option(inner), span.join(close_span))
            );
        }
        if name == "Resultado" {
            self.expect(Kind::Less, "Esperado `<` após `Resultado`")?;
            let ok = self.type_name()?;
            self.expect(
                Kind::Comma,
                "Esperado `,` entre os tipos de sucesso e erro de `Resultado`",
            )?;
            let err = self.type_name()?;
            let close_span = self.expect_type_close("Esperado `>` após os tipos de `Resultado`")?;
            return Ok(self
                .intern_builtin_enum(hir::BuiltinEnum::Result { ok, err }, span.join(close_span)));
        }
        scalar_type(&name)
            .or_else(|| self.record_ids.get(&name).copied().map(hir::Type::Record))
            .or_else(|| self.enum_ids.get(&name).copied().map(hir::Type::Enum))
            .ok_or_else(|| {
                vec![Diagnostic::error(
                    "pors.unknown-type",
                    format!("Tipo desconhecido: {name}"),
                    span,
                )]
            })
    }

    pub(super) fn collect_record_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
        let (name, name_span) = self.identifier()?;
        if is_reserved(&name) {
            return self.fail(
                "pors.reserved-name",
                format!("`{name}` é reservado e não pode ser usado como nome de registro"),
                name_span,
            );
        }
        if self.declared_types.contains_key(&name) {
            return self.fail(
                "pors.type-redeclared",
                format!("O tipo `{name}` já foi declarado"),
                name_span,
            );
        }
        let id = hir::TypeId(self.next_type_id);
        self.expect(Kind::LBrace, "Esperado `{` após o nome do registro")?;
        let mut fields = Vec::new();
        let mut field_names = HashMap::new();
        while !matches!(self.current().kind, Kind::RBrace | Kind::Eof) {
            let (field_name, span) = self.identifier()?;
            if field_names.insert(field_name.clone(), ()).is_some() {
                return self.fail(
                    "pors.field-redeclared",
                    format!("O campo `{field_name}` já foi declarado"),
                    span,
                );
            }
            self.expect(Kind::Colon, "Esperado `:` entre campo e tipo")?;
            let ty = self.type_name()?;
            fields.push(hir::RecordField {
                name: field_name,
                ty,
                span: to_hir_span(span),
            });
            if !self.eat(&Kind::Comma) {
                break;
            }
        }
        let close = self.expect(Kind::RBrace, "Esperado `}` após campos do registro")?;
        let span = to_hir_span(name_span.join(close.span));
        self.next_type_id += 1;
        self.declared_types.insert(name.clone(), id);
        self.record_ids.insert(name.clone(), id);
        self.record_types.push(hir::RecordDef {
            id,
            name,
            fields,
            span,
        });
        Ok(())
    }

    pub(super) fn skip_record_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
        self.identifier()?;
        self.expect(Kind::LBrace, "Esperado `{` após o nome do registro")?;
        let mut depth = 1usize;
        while depth > 0 {
            match self.advance() {
                Token {
                    kind: Kind::LBrace, ..
                } => depth += 1,
                Token {
                    kind: Kind::RBrace, ..
                } => depth -= 1,
                Token {
                    kind: Kind::Eof,
                    span,
                } => return self.fail("pors.unclosed-record", "Registro não foi fechado", span),
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn collect_enum_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
        let (name, name_span) = self.identifier()?;
        if is_reserved(&name) {
            return self.fail(
                "pors.reserved-name",
                format!("`{name}` é reservado e não pode ser usado como nome de enum"),
                name_span,
            );
        }
        if self.declared_types.contains_key(&name) {
            return self.fail(
                "pors.type-redeclared",
                format!("O tipo `{name}` já foi declarado"),
                name_span,
            );
        }
        let id = hir::TypeId(self.next_type_id);
        self.expect(Kind::LBrace, "Esperado `{` após o nome do enum")?;
        let mut variants = Vec::new();
        let mut variant_names = HashMap::new();
        while !matches!(self.current().kind, Kind::RBrace | Kind::Eof) {
            let (variant_name, span) = self.identifier()?;
            if variant_names.insert(variant_name.clone(), ()).is_some() {
                return self.fail(
                    "pors.variant-redeclared",
                    format!("A variante `{variant_name}` já foi declarada"),
                    span,
                );
            }
            let mut payload_types = Vec::new();
            let variant_span = if self.eat(&Kind::LParen) {
                if !matches!(self.current().kind, Kind::RParen) {
                    loop {
                        payload_types.push(self.type_name()?);
                        if !self.eat(&Kind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(Kind::RParen, "Esperado `)` após payload da variante")?
                    .span
            } else {
                span
            };
            variants.push(hir::EnumVariant {
                name: variant_name,
                payload_types,
                span: to_hir_span(span.join(variant_span)),
            });
            if !self.eat(&Kind::Comma) {
                break;
            }
        }
        let close = self.expect(Kind::RBrace, "Esperado `}` após variantes do enum")?;
        self.next_type_id += 1;
        self.declared_types.insert(name.clone(), id);
        self.enum_ids.insert(name.clone(), id);
        self.enum_types.push(hir::EnumDef {
            id,
            name,
            variants,
            builtin: None,
            span: to_hir_span(name_span.join(close.span)),
        });
        Ok(())
    }

    pub(super) fn skip_enum_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
        self.identifier()?;
        self.expect(Kind::LBrace, "Esperado `{` após o nome do enum")?;
        let mut depth = 1usize;
        while depth > 0 {
            match self.advance() {
                Token {
                    kind: Kind::LBrace, ..
                } => depth += 1,
                Token {
                    kind: Kind::RBrace, ..
                } => depth -= 1,
                Token {
                    kind: Kind::Eof,
                    span,
                } => return self.fail("pors.unclosed-enum", "Enum não foi fechado", span),
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn skip_function_body(&mut self) -> Result<(), Vec<Diagnostic>> {
        self.expect(Kind::LBrace, "Esperado `{` para iniciar o corpo da função")?;
        let mut depth = 1usize;
        while depth > 0 {
            if is_top_level_start(&self.current().kind) {
                return self.fail(
                    "pors.unexpected-token",
                    "Bloco não foi fechado com `}` antes da próxima declaração",
                    self.current().span,
                );
            }
            match self.advance() {
                Token {
                    kind: Kind::LBrace, ..
                } => depth += 1,
                Token {
                    kind: Kind::RBrace, ..
                } => depth -= 1,
                Token {
                    kind: Kind::Eof,
                    span,
                } => {
                    return self.fail(
                        "pors.unexpected-token",
                        "Bloco não foi fechado com `}`",
                        span,
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }
}
