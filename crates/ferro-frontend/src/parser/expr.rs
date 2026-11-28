use ferro_hir as hir;
use ferro_hir::{Diagnostic, Span};

use super::Parser;
use super::constants::*;
use crate::lexer::{Kind, lex};

impl Parser {
    pub(super) fn expression(&mut self, min_precedence: u8) -> Result<hir::Expr, Vec<Diagnostic>> {
        self.expression_with_expected(min_precedence, None)
    }

    pub(super) fn expression_with_expected(
        &mut self,
        min_precedence: u8,
        expected: Option<hir::Type>,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        let mut left = self.prefix(expected)?;
        loop {
            if matches!(self.current().kind, Kind::Dot) {
                self.advance();
                let (field_name, field_span) = self.identifier()?;
                let hir::Type::Record(type_id) = left.ty else {
                    return self.fail(
                        "ferro.field-non-record",
                        "Acesso de campo exige um registro",
                        field_span,
                    );
                };
                let definition = self
                    .record_types
                    .iter()
                    .find(|definition| definition.id == type_id)
                    .expect("registro tipado possui definição");
                let Some((field_index, field)) = definition
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_, field)| field.name == field_name)
                else {
                    return self.fail(
                        "ferro.unknown-field",
                        format!(
                            "Campo `{field_name}` não existe no registro `{}`",
                            definition.name
                        ),
                        field_span,
                    );
                };
                left = hir::Expr {
                    kind: hir::ExprKind::Field {
                        value: Box::new(left.clone()),
                        field_index,
                    },
                    ty: field.ty,
                    span: to_hir_span(syntax_span(left.span).join(field_span)),
                };
                continue;
            }
            if matches!(self.current().kind, Kind::Question) {
                let question_span = self.advance().span;
                let unwrapped_ty = match self.builtin_enum_of(left.ty) {
                    Some((_, hir::BuiltinEnum::Option(inner))) => {
                        match self.return_type.and_then(|ty| self.builtin_enum_of(ty)) {
                            Some((_, hir::BuiltinEnum::Option(_))) => inner,
                            _ => {
                                return self.fail(
                                    "ferro.try-return-mismatch",
                                    "O operador `?` em `Opcao` exige que a função retorne `Opcao<...>`",
                                    question_span,
                                );
                            }
                        }
                    }
                    Some((_, hir::BuiltinEnum::Result { ok, err })) => {
                        match self.return_type.and_then(|ty| self.builtin_enum_of(ty)) {
                            Some((_, hir::BuiltinEnum::Result { err: ret_err, .. }))
                                if ret_err == err =>
                            {
                                ok
                            }
                            _ => {
                                return self.fail(
                                    "ferro.try-return-mismatch",
                                    "O operador `?` em `Resultado` exige que a função retorne `Resultado<..., E>` com o mesmo tipo de erro",
                                    question_span,
                                );
                            }
                        }
                    }
                    None => {
                        return self.fail(
                            "ferro.try-invalid-type",
                            "O operador `?` só pode ser aplicado a valores `Opcao<T>` ou `Resultado<T, E>`",
                            question_span,
                        );
                    }
                };
                left = hir::Expr {
                    kind: hir::ExprKind::Try {
                        operand: Box::new(left.clone()),
                    },
                    ty: unwrapped_ty,
                    span: to_hir_span(syntax_span(left.span).join(question_span)),
                };
                continue;
            }
            let Some((op, precedence)) = binary(&self.current().kind) else {
                break;
            };
            if precedence < min_precedence {
                break;
            }
            self.advance();
            let right = self.expression(precedence + 1)?;
            let ty = binary_type(op, left.ty, right.ty).ok_or_else(|| {
                vec![Diagnostic::error(
                    "ferro.type-mismatch",
                    "Operandos incompatíveis para este operador",
                    syntax_span(left.span),
                )]
            })?;
            let whole_span = join_hir(left.span, right.span);
            left = hir::Expr {
                kind: hir::ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                ty,
                span: whole_span,
            };
        }
        Ok(left)
    }

    pub(super) fn prefix(
        &mut self,
        expected: Option<hir::Type>,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        let token = self.advance();
        let span = token.span;
        let (kind, ty) = match token.kind {
            Kind::Integer(n) => (
                hir::ExprKind::Integer(i32::try_from(n).map_err(|_| {
                    vec![Diagnostic::error(
                        "ferro.integer-range",
                        "Literal inteiro fora do intervalo de 32 bits",
                        span,
                    )]
                })?),
                hir::Type::Integer,
            ),
            Kind::Real(n) => (hir::ExprKind::Real(n), hir::Type::Real),
            Kind::String { decoded, raw } => {
                let parts = self.string_parts(span, &raw)?;
                if parts
                    .iter()
                    .all(|part| matches!(part, hir::StringPart::Literal(_, _)))
                {
                    (hir::ExprKind::String(decoded), hir::Type::String)
                } else {
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::InterpolatedString { parts },
                        ty: hir::Type::String,
                        span: to_hir_span(span),
                    });
                }
            }
            Kind::Character(value) => (hir::ExprKind::Character(value), hir::Type::Character),
            Kind::Ident(word) if word == "verdadeiro" => {
                (hir::ExprKind::Boolean(true), hir::Type::Boolean)
            }
            Kind::Ident(word) if word == "falso" => {
                (hir::ExprKind::Boolean(false), hir::Type::Boolean)
            }
            Kind::Ident(name) if name == "leia" && matches!(self.current().kind, Kind::LParen) => {
                self.advance();
                if !matches!(self.current().kind, Kind::RParen) {
                    let start = self.current().span;
                    let mut cursor = self.cursor;
                    let mut depth = 0i32;
                    while cursor < self.tokens.len() {
                        match self.tokens[cursor].kind {
                            Kind::LParen => depth += 1,
                            Kind::RParen if depth == 0 => break,
                            Kind::RParen => depth -= 1,
                            Kind::Eof => break,
                            _ => {}
                        }
                        cursor += 1;
                    }
                    let end = self.tokens[cursor.min(self.tokens.len() - 1)].span;
                    return self.fail(
                        "ferro.read-arguments-unsupported",
                        "`leia()` não aceita argumentos; use `leia(variavel)` como comando",
                        start.join(end),
                    );
                }
                let close = self.expect(Kind::RParen, "Esperado `)` após leia")?;
                let target = implicit_read_target(expected);
                return Ok(hir::Expr {
                    kind: hir::ExprKind::Read { target },
                    ty: target.unwrap_or(hir::Type::String),
                    span: to_hir_span(span.join(close.span)),
                });
            }
            Kind::Ident(name) => {
                if name == "UI"
                    && matches!(self.current().kind, Kind::Dot)
                    && self.lookup("UI").is_none()
                {
                    return self.ui_host_call(span);
                }
                if matches!(
                    name.as_str(),
                    "Matematica"
                        | "Texto"
                        | "Tipos"
                        | "Util"
                        | "Calendario"
                        | "Arquivos"
                        | "Dialogo"
                        | "AreaTransferencia"
                        | "Notificacao"
                        | "Json"
                        | "Internet"
                        | "Graficos"
                        | "Teclado"
                        | "Mouse"
                        | "Sons"
                ) && matches!(self.current().kind, Kind::Dot)
                    && self.lookup(&name).is_none()
                {
                    return self.stdlib_member_or_call(&name, span);
                }
                if matches!(self.current().kind, Kind::ColonColon) {
                    self.advance();
                    let (variant_name, variant_span) = self.identifier()?;
                    if name == "Opcao" {
                        return self.parse_builtin_option_constructor(
                            span,
                            &variant_name,
                            variant_span,
                            expected,
                        );
                    }
                    if name == "Resultado" {
                        return self.parse_builtin_result_constructor(
                            span,
                            &variant_name,
                            variant_span,
                            expected,
                        );
                    }
                    let Some(definition) = self
                        .enum_types
                        .iter()
                        .find(|definition| definition.builtin.is_none() && definition.name == name)
                        .cloned()
                    else {
                        return self.fail(
                            "ferro.unknown-type",
                            format!("Enum não declarado: {name}"),
                            span,
                        );
                    };
                    let Some((variant_index, variant)) = definition
                        .variants
                        .iter()
                        .enumerate()
                        .find(|(_, variant)| variant.name == variant_name)
                        .map(|(index, variant)| (index, variant.clone()))
                    else {
                        return self.fail(
                            "ferro.unknown-variant",
                            format!("Variante `{variant_name}` não existe em `{name}`"),
                            variant_span,
                        );
                    };
                    let mut payload = Vec::new();
                    let end = if self.eat(&Kind::LParen) {
                        if !matches!(self.current().kind, Kind::RParen) {
                            loop {
                                let expected_payload =
                                    variant.payload_types.get(payload.len()).copied();
                                payload.push(self.expression_with_expected(0, expected_payload)?);
                                if !self.eat(&Kind::Comma) {
                                    break;
                                }
                            }
                        }
                        self.expect(Kind::RParen, "Esperado `)` após payload do enum")?
                            .span
                    } else {
                        variant_span
                    };
                    if payload.len() != variant.payload_types.len() {
                        return self.fail(
                            "ferro.enum-payload-arity",
                            format!(
                                "A variante `{variant_name}` exige {} valor(es)",
                                variant.payload_types.len()
                            ),
                            variant_span,
                        );
                    }
                    if let Some((argument, _expected)) = payload
                        .iter()
                        .zip(&variant.payload_types)
                        .find(|(argument, expected)| argument.ty != **expected)
                    {
                        return self.fail(
                            "ferro.enum-payload-type",
                            format!("Valor incompatível com o payload de `{variant_name}`"),
                            syntax_span(argument.span),
                        );
                    }
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::EnumConstructor {
                            type_id: definition.id,
                            variant_index,
                            payload,
                        },
                        ty: hir::Type::Enum(definition.id),
                        span: to_hir_span(span.join(end)),
                    });
                }
                if matches!(self.current().kind, Kind::LBrace)
                    && self
                        .record_types
                        .iter()
                        .any(|definition| definition.name == name)
                {
                    let definition = self
                        .record_types
                        .iter()
                        .find(|definition| definition.name == name)
                        .cloned()
                        .expect("verificação anterior encontrou o registro");
                    self.advance();
                    let mut values: Vec<Option<hir::Expr>> = vec![None; definition.fields.len()];
                    while !matches!(self.current().kind, Kind::RBrace | Kind::Eof) {
                        let (field_name, field_span) = self.identifier()?;
                        let Some((index, field)) = definition
                            .fields
                            .iter()
                            .enumerate()
                            .find(|(_, field)| field.name == field_name)
                        else {
                            return self.fail(
                                "ferro.unknown-field",
                                format!("Campo `{field_name}` não existe em `{name}`"),
                                field_span,
                            );
                        };
                        if values[index].is_some() {
                            return self.fail(
                                "ferro.field-redeclared",
                                format!("Campo `{field_name}` foi informado mais de uma vez"),
                                field_span,
                            );
                        }
                        self.expect(Kind::Colon, "Esperado `:` após o nome do campo")?;
                        let value = self.expression_with_expected(0, Some(field.ty))?;
                        if value.ty != field.ty {
                            return self.fail(
                                "ferro.field-type-mismatch",
                                format!("Valor incompatível com o campo `{field_name}`"),
                                syntax_span(value.span),
                            );
                        }
                        values[index] = Some(value);
                        if !self.eat(&Kind::Comma) {
                            break;
                        }
                    }
                    let close =
                        self.expect(Kind::RBrace, "Esperado `}` após campos do registro")?;
                    if let Some(missing) = values.iter().position(Option::is_none) {
                        return self.fail(
                            "ferro.missing-field",
                            format!(
                                "Campo `{}` não foi informado",
                                definition.fields[missing].name
                            ),
                            close.span,
                        );
                    }
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::RecordLiteral {
                            type_id: definition.id,
                            fields: values.into_iter().map(Option::unwrap).collect(),
                        },
                        ty: hir::Type::Record(definition.id),
                        span: to_hir_span(span.join(close.span)),
                    });
                }
                if matches!(self.current().kind, Kind::LParen) {
                    if name == "tamanho" {
                        self.advance();
                        let argument = self.expression(0)?;
                        let close =
                            self.expect(Kind::RParen, "Esperado `)` após argumento de tamanho")?;
                        let Some(element_type) = argument.ty.vector_element() else {
                            return self.fail(
                                "ferro.length-type",
                                "`tamanho` espera um vetor",
                                syntax_span(argument.span),
                            );
                        };
                        let hir::ExprKind::Binding(binding) = argument.kind else {
                            return self.fail(
                                "ferro.length-binding",
                                "`tamanho` precisa receber o nome de um vetor",
                                syntax_span(argument.span),
                            );
                        };
                        let _ = element_type;
                        return Ok(hir::Expr {
                            kind: hir::ExprKind::Length { binding },
                            ty: hir::Type::Integer,
                            span: to_hir_span(span.join(close.span)),
                        });
                    }
                    if name == "remover" {
                        self.advance();
                        let (target_name, target_span) = self.identifier()?;
                        let close =
                            self.expect(Kind::RParen, "Esperado `)` após argumento de remover")?;
                        let symbol = self.lookup(&target_name).ok_or_else(|| {
                            vec![Diagnostic::error(
                                "ferro.unknown-name",
                                format!("Nome não declarado: {target_name}"),
                                target_span,
                            )]
                        })?;
                        let Some(element_type) = symbol.ty.vector_element() else {
                            return self.fail(
                                "ferro.vector-operation-type",
                                format!(
                                    "`remover` espera um vetor, mas `{target_name}` não é um vetor"
                                ),
                                target_span,
                            );
                        };
                        if symbol.mutability != hir::Mutability::Mutable {
                            return self.fail(
                                "ferro.immutable-assignment",
                                format!(
                                    "`{target_name}` é imutável; declare com `seja mutavel` para remover elementos"
                                ),
                                target_span,
                            );
                        }
                        let whole = span.join(close.span);
                        let ty =
                            self.intern_builtin_enum(hir::BuiltinEnum::Option(element_type), whole);
                        return Ok(hir::Expr {
                            kind: hir::ExprKind::VectorPop { binding: symbol.id },
                            ty,
                            span: to_hir_span(whole),
                        });
                    }
                    if name == "obter" {
                        self.advance();
                        let (target_name, target_span) = self.identifier()?;
                        let symbol = self.lookup(&target_name).ok_or_else(|| {
                            vec![Diagnostic::error(
                                "ferro.unknown-name",
                                format!("Nome não declarado: {target_name}"),
                                target_span,
                            )]
                        })?;
                        let Some(element_type) = symbol.ty.vector_element() else {
                            return self.fail(
                                "ferro.vector-operation-type",
                                format!(
                                    "`obter` espera um vetor, mas `{target_name}` não é um vetor"
                                ),
                                target_span,
                            );
                        };
                        self.expect(
                            Kind::Comma,
                            "Esperado `,` entre o vetor e o índice em `obter`",
                        )?;
                        let index = self.expression(0)?;
                        if index.ty != hir::Type::Integer {
                            return self.fail(
                                "ferro.index-type",
                                "O índice do vetor deve ter tipo inteiro",
                                syntax_span(index.span),
                            );
                        }
                        let close =
                            self.expect(Kind::RParen, "Esperado `)` após argumentos de obter")?;
                        let whole = span.join(close.span);
                        let ty =
                            self.intern_builtin_enum(hir::BuiltinEnum::Option(element_type), whole);
                        return Ok(hir::Expr {
                            kind: hir::ExprKind::VectorGet {
                                binding: symbol.id,
                                index: Box::new(index),
                            },
                            ty,
                            span: to_hir_span(whole),
                        });
                    }
                    let signature = self.signatures.get(&name).cloned().ok_or_else(|| {
                        vec![Diagnostic::error(
                            "ferro.unknown-function",
                            format!("Função não declarada: {name}"),
                            span,
                        )]
                    })?;
                    self.advance();
                    let mut arguments = Vec::new();
                    if !matches!(self.current().kind, Kind::RParen) {
                        loop {
                            let expected_arg =
                                signature.parameters.get(arguments.len()).map(|p| p.ty);
                            arguments.push(self.expression_with_expected(0, expected_arg)?);
                            if !self.eat(&Kind::Comma) {
                                break;
                            }
                        }
                    }
                    let close = self.expect(Kind::RParen, "Esperado `)` após argumentos")?;
                    if arguments.len() != signature.parameters.len() {
                        return self.fail(
                            "ferro.call-arity",
                            format!(
                                "A função `{name}` espera {} argumento(s), recebeu {}",
                                signature.parameters.len(),
                                arguments.len()
                            ),
                            span.join(close.span),
                        );
                    }
                    for (argument, parameter) in arguments.iter().zip(&signature.parameters) {
                        if argument.ty != parameter.ty {
                            return self.fail(
                                "ferro.call-type-mismatch",
                                format!(
                                    "Argumento incompatível com o parâmetro `{}`",
                                    parameter.name
                                ),
                                syntax_span(argument.span),
                            );
                        }
                    }
                    let function = signature.id.ok_or_else(|| {
                        vec![Diagnostic::error(
                            "ferro.entry-call",
                            "`inicio` não pode ser chamada diretamente",
                            span,
                        )]
                    })?;
                    let ty = signature.return_type.unwrap_or(hir::Type::Unit);
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::Call {
                            function,
                            arguments,
                        },
                        ty,
                        span: to_hir_span(span.join(close.span)),
                    });
                }
                if matches!(self.current().kind, Kind::LBracket) {
                    let symbol = self.lookup(&name).ok_or_else(|| {
                        vec![Diagnostic::error(
                            "ferro.unknown-name",
                            format!("Nome não declarado: {name}"),
                            span,
                        )]
                    })?;
                    let Some(element_type) = symbol.ty.vector_element() else {
                        return self.fail(
                            "ferro.index-non-vector",
                            format!("`{name}` não é um vetor"),
                            span,
                        );
                    };
                    self.advance();
                    let index = self.expression(0)?;
                    if index.ty != hir::Type::Integer {
                        return self.fail(
                            "ferro.index-type",
                            "O índice do vetor deve ter tipo inteiro",
                            syntax_span(index.span),
                        );
                    }
                    let close = self.expect(Kind::RBracket, "Esperado `]` após índice")?;
                    return Ok(hir::Expr {
                        kind: hir::ExprKind::Index {
                            binding: symbol.id,
                            index: Box::new(index),
                        },
                        ty: element_type,
                        span: to_hir_span(span.join(close.span)),
                    });
                }
                let symbol = self.lookup(&name).ok_or_else(|| {
                    vec![Diagnostic::error(
                        "ferro.unknown-name",
                        format!("Nome não declarado: {name}"),
                        span,
                    )]
                })?;
                (hir::ExprKind::Binding(symbol.id), symbol.ty)
            }
            Kind::Minus if matches!(self.current().kind, Kind::Integer(n) if n == i32::MAX as i64 + 1) =>
            {
                let end = self.advance().span.end;
                return Ok(hir::Expr {
                    kind: hir::ExprKind::Integer(i32::MIN),
                    ty: hir::Type::Integer,
                    span: hir::Span::new(span.start, end),
                });
            }
            Kind::Minus | Kind::Plus | Kind::Bang | Kind::Tilde => {
                let op = match token.kind {
                    Kind::Minus => hir::UnaryOp::Negate,
                    Kind::Plus => hir::UnaryOp::Positive,
                    Kind::Tilde => hir::UnaryOp::BitNot,
                    _ => hir::UnaryOp::Not,
                };
                let operand = self.expression(11)?;
                let ty = match (op, operand.ty) {
                    (
                        hir::UnaryOp::Negate | hir::UnaryOp::Positive,
                        hir::Type::Integer | hir::Type::Real,
                    ) => operand.ty,
                    (hir::UnaryOp::Not, hir::Type::Boolean) => hir::Type::Boolean,
                    (hir::UnaryOp::BitNot, hir::Type::Integer) => hir::Type::Integer,
                    _ => {
                        return self.fail(
                            "ferro.invalid-unary-operator",
                            "Operador unário incompatível com o tipo",
                            span,
                        );
                    }
                };
                let end = operand.span.end;
                return Ok(hir::Expr {
                    kind: hir::ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                    ty,
                    span: hir::Span::new(span.start, end),
                });
            }
            Kind::LParen => {
                let inner = self.expression_with_expected(0, expected)?;
                let close = self.expect(Kind::RParen, "Esperado `)` para fechar expressão")?;
                let ty = inner.ty;
                return Ok(hir::Expr {
                    kind: hir::ExprKind::Parenthesized(Box::new(inner)),
                    ty,
                    span: to_hir_span(span.join(close.span)),
                });
            }
            Kind::LBracket => return self.vector_literal(span, expected),
            other => {
                return self.fail(
                    "ferro.expected-expression",
                    format!("Esperada expressão, encontrado {other:?}"),
                    span,
                );
            }
        };
        Ok(hir::Expr {
            kind,
            ty,
            span: to_hir_span(span),
        })
    }

    fn string_parts(
        &mut self,
        token_span: Span,
        raw: &str,
    ) -> Result<Vec<hir::StringPart>, Vec<Diagnostic>> {
        let content_start = token_span.start as usize + 1;
        debug_assert_eq!(&self.source[content_start..content_start + raw.len()], raw);
        let mut parts = Vec::new();
        let mut literal = String::new();
        let mut literal_start = None;
        let mut cursor = 0usize;

        while cursor < raw.len() {
            let ch = raw[cursor..].chars().next().expect("offset válido em raw");
            if ch == '\\' && cursor + 1 < raw.len() {
                let escaped = raw[cursor + 1..]
                    .chars()
                    .next()
                    .expect("offset válido em raw");
                let decoded = match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '0' => '\0',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                };
                literal_start.get_or_insert(cursor);
                literal.push(decoded);
                cursor += 1 + escaped.len_utf8();
                continue;
            }
            if ch == '{'
                && let Some(close) = unescaped_closing_brace(raw, cursor + 1)
            {
                let content = &raw[cursor + 1..close];
                if !content.is_empty() {
                    let start = (content_start + cursor + 1) as u32;
                    let end = (content_start + close) as u32;
                    let span = Span::new(start, end);
                    let (tokens, lex_errors) = lex(content);
                    if lex_errors.is_empty() {
                        let shifted: Vec<_> = tokens
                            .into_iter()
                            .map(|mut token| {
                                token.span =
                                    Span::new(token.span.start + start, token.span.end + start);
                                token
                            })
                            .collect();
                        let outer_tokens = std::mem::replace(&mut self.tokens, shifted);
                        let outer_cursor = self.cursor;
                        self.cursor = 0;
                        let parsed = self.expression(0);
                        let complete = matches!(self.current().kind, Kind::Eof);
                        let unresolved_name = parsed.as_ref().is_err_and(|errors| {
                            errors.iter().all(|d| {
                                matches!(d.code, "ferro.unknown-name" | "ferro.unknown-function")
                            })
                        });
                        self.tokens = outer_tokens;
                        self.cursor = outer_cursor;
                        if parsed.is_err() && !unresolved_name {
                            return self.fail(
                                "ferro.interpolation-invalid",
                                "Conteúdo inválido dentro da interpolação",
                                span,
                            );
                        }
                        if complete && !unresolved_name {
                            let expr = parsed.expect("expressão de interpolação válida");
                            if !matches!(expr.ty, hir::Type::String | hir::Type::Character) {
                                return self.fail(
                                    "ferro.interpolation-non-string",
                                    "A interpolação exige um valor `cadeia` ou `caracter`",
                                    span,
                                );
                            }
                            flush_string_literal(
                                &mut parts,
                                &mut literal,
                                literal_start,
                                cursor,
                                content_start,
                            );
                            literal_start = None;
                            parts.push(hir::StringPart::Expression(
                                expr,
                                Span::new(start - 1, end + 1),
                            ));
                            cursor = close + 1;
                            continue;
                        }
                    }
                }
            }
            literal_start.get_or_insert(cursor);
            literal.push(ch);
            cursor += ch.len_utf8();
        }
        flush_string_literal(
            &mut parts,
            &mut literal,
            literal_start,
            cursor,
            content_start,
        );
        Ok(parts)
    }

    pub(super) fn parse_builtin_option_constructor(
        &mut self,
        start: Span,
        variant_name: &str,
        variant_span: Span,
        expected: Option<hir::Type>,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        match variant_name {
            "Algum" => {
                let expected_inner = expected.and_then(|ty| match self.builtin_enum_of(ty) {
                    Some((_, hir::BuiltinEnum::Option(inner))) => Some(inner),
                    _ => None,
                });
                let mut payload = Vec::new();
                let end = if self.eat(&Kind::LParen) {
                    if !matches!(self.current().kind, Kind::RParen) {
                        loop {
                            let arg_expected = if payload.is_empty() {
                                expected_inner
                            } else {
                                None
                            };
                            payload.push(self.expression_with_expected(0, arg_expected)?);
                            if !self.eat(&Kind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(Kind::RParen, "Esperado `)` após valor de `Opcao::Algum`")?
                        .span
                } else {
                    variant_span
                };
                if payload.len() != 1 {
                    return self.fail(
                        "ferro.enum-payload-arity",
                        "A variante `Algum` exige 1 valor(es)",
                        variant_span,
                    );
                }
                if payload[0].ty == hir::Type::Unit {
                    return self.fail(
                        "ferro.void-value",
                        "Uma função sem retorno não pode ser usada como valor",
                        syntax_span(payload[0].span),
                    );
                }
                let enum_ty = self
                    .intern_builtin_enum(hir::BuiltinEnum::Option(payload[0].ty), start.join(end));
                let hir::Type::Enum(type_id) = enum_ty else {
                    unreachable!("intern_builtin_enum sempre retorna Type::Enum");
                };
                Ok(hir::Expr {
                    kind: hir::ExprKind::EnumConstructor {
                        type_id,
                        variant_index: 0,
                        payload,
                    },
                    ty: enum_ty,
                    span: to_hir_span(start.join(end)),
                })
            }
            "Nenhum" => {
                let mut payload = Vec::new();
                let end = if self.eat(&Kind::LParen) {
                    if !matches!(self.current().kind, Kind::RParen) {
                        loop {
                            payload.push(self.expression(0)?);
                            if !self.eat(&Kind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(Kind::RParen, "Esperado `)` após `Opcao::Nenhum`")?
                        .span
                } else {
                    variant_span
                };
                if !payload.is_empty() {
                    return self.fail(
                        "ferro.enum-payload-arity",
                        "A variante `Nenhum` exige 0 valor(es)",
                        variant_span,
                    );
                }
                let Some((type_id, hir::BuiltinEnum::Option(_))) =
                    expected.and_then(|ty| self.builtin_enum_of(ty))
                else {
                    return self.fail(
                        "ferro.option-type-inference",
                        "Não foi possível inferir o tipo de `Opcao::Nenhum`; informe o tipo `Opcao<T>`",
                        start.join(end),
                    );
                };
                Ok(hir::Expr {
                    kind: hir::ExprKind::EnumConstructor {
                        type_id,
                        variant_index: 1,
                        payload: Vec::new(),
                    },
                    ty: hir::Type::Enum(type_id),
                    span: to_hir_span(start.join(end)),
                })
            }
            _ => self.fail(
                "ferro.unknown-variant",
                format!("Variante `{variant_name}` não existe em `Opcao`"),
                variant_span,
            ),
        }
    }

    pub(super) fn parse_builtin_result_constructor(
        &mut self,
        start: Span,
        variant_name: &str,
        variant_span: Span,
        expected: Option<hir::Type>,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        let variant_index = match variant_name {
            "Ok" => 0,
            "Erro" => 1,
            _ => {
                return self.fail(
                    "ferro.unknown-variant",
                    format!("Variante `{variant_name}` não existe em `Resultado`"),
                    variant_span,
                );
            }
        };
        let expected_result = expected.and_then(|ty| match self.builtin_enum_of(ty) {
            Some((type_id, hir::BuiltinEnum::Result { ok, err })) => Some((type_id, ok, err)),
            _ => None,
        });
        let expected_payload =
            expected_result.map(|(_, ok, err)| if variant_index == 0 { ok } else { err });
        let mut payload = Vec::new();
        let end = if self.eat(&Kind::LParen) {
            if !matches!(self.current().kind, Kind::RParen) {
                loop {
                    let arg_expected = if payload.is_empty() {
                        expected_payload
                    } else {
                        None
                    };
                    payload.push(self.expression_with_expected(0, arg_expected)?);
                    if !self.eat(&Kind::Comma) {
                        break;
                    }
                }
            }
            self.expect(
                Kind::RParen,
                "Esperado `)` após valor de variante de `Resultado`",
            )?
            .span
        } else {
            variant_span
        };
        if payload.len() != 1 {
            return self.fail(
                "ferro.enum-payload-arity",
                format!("A variante `{variant_name}` exige 1 valor(es)"),
                variant_span,
            );
        }
        if payload[0].ty == hir::Type::Unit {
            return self.fail(
                "ferro.void-value",
                "Uma função sem retorno não pode ser usada como valor",
                syntax_span(payload[0].span),
            );
        }
        let Some((type_id, expected_ok, expected_err)) = expected_result else {
            return self.fail(
                "ferro.result-type-inference",
                format!(
                    "Não foi possível inferir os tipos de `Resultado::{variant_name}`; informe o tipo `Resultado<T, E>`"
                ),
                start.join(end),
            );
        };
        let expected_ty = if variant_index == 0 {
            expected_ok
        } else {
            expected_err
        };
        if payload[0].ty != expected_ty {
            return self.fail(
                "ferro.enum-payload-type",
                format!("Valor incompatível com o payload de `{variant_name}`"),
                syntax_span(payload[0].span),
            );
        }
        Ok(hir::Expr {
            kind: hir::ExprKind::EnumConstructor {
                type_id,
                variant_index,
                payload,
            },
            ty: hir::Type::Enum(type_id),
            span: to_hir_span(start.join(end)),
        })
    }

    pub(super) fn vector_literal(
        &mut self,
        start: Span,
        expected: Option<hir::Type>,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        if matches!(self.current().kind, Kind::RBracket) {
            let close = self.advance();
            let whole = start.join(close.span);
            let Some(ty) = expected.filter(|ty| ty.is_vector()) else {
                return self.fail(
                    "ferro.empty-vector-type-inference",
                    "Não foi possível inferir o tipo do vetor vazio `[]`; informe o tipo `vetor<T>`",
                    whole,
                );
            };
            return Ok(hir::Expr {
                kind: hir::ExprKind::VectorLiteral(Vec::new()),
                ty,
                span: to_hir_span(whole),
            });
        }
        let expected_element = expected.and_then(hir::Type::vector_element);
        let mut elements = Vec::new();
        loop {
            elements.push(self.expression_with_expected(0, expected_element)?);
            if !self.eat(&Kind::Comma) {
                break;
            }
        }
        let close = self.expect(Kind::RBracket, "Esperado `]` após elementos do vetor")?;
        let element_type = elements[0].ty;
        if element_type.is_vector() || element_type == hir::Type::Unit {
            return self.fail(
                "ferro.vector-element-type",
                "Vetores aninhados e valores sem retorno não podem ser elementos",
                syntax_span(elements[0].span),
            );
        }
        if let Some(incompatible) = elements.iter().find(|element| element.ty != element_type) {
            return self.fail(
                "ferro.vector-element-type",
                "Todos os elementos do vetor devem ter o mesmo tipo",
                syntax_span(incompatible.span),
            );
        }
        let ty = hir::Type::vector_of(element_type).expect("element type is scalar");
        Ok(hir::Expr {
            kind: hir::ExprKind::VectorLiteral(elements),
            ty,
            span: to_hir_span(start.join(close.span)),
        })
    }
}

fn implicit_read_target(expected: Option<hir::Type>) -> Option<hir::Type> {
    expected.filter(|ty| ty.is_implicit_read_target())
}

fn flush_string_literal(
    parts: &mut Vec<hir::StringPart>,
    literal: &mut String,
    start: Option<usize>,
    end: usize,
    content_start: usize,
) {
    if literal.is_empty() {
        return;
    }
    let start = start.expect("texto literal possui início");
    parts.push(hir::StringPart::Literal(
        std::mem::take(literal),
        Span::new((content_start + start) as u32, (content_start + end) as u32),
    ));
}

fn unescaped_closing_brace(raw: &str, from: usize) -> Option<usize> {
    let bytes = raw.as_bytes();
    let mut cursor = from;
    while cursor < bytes.len() {
        if bytes[cursor] == b'\\' {
            cursor += 1;
            if cursor < bytes.len() {
                let ch = raw[cursor..].chars().next().expect("offset válido em raw");
                cursor += ch.len_utf8();
            }
            continue;
        }
        if bytes[cursor] == b'}' {
            return Some(cursor);
        }
        let ch = raw[cursor..].chars().next().expect("offset válido em raw");
        cursor += ch.len_utf8();
    }
    None
}
