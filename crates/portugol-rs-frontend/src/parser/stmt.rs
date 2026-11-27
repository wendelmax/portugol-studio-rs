use std::collections::HashMap;

use portugol_hir as hir;
use portugol_hir::{Diagnostic, Span};

use super::constants::*;
use super::{FunctionSignature, Parser, Symbol};
use crate::lexer::Kind;

impl Parser {
    pub(super) fn begin_function(
        &mut self,
        signature: &FunctionSignature,
    ) -> Result<(), Vec<Diagnostic>> {
        self.bindings.clear();
        self.scopes = vec![HashMap::new()];
        self.loop_depth = 0;
        self.function_has_syntax_error = false;
        self.return_type = signature.return_type;
        self.in_function = true;
        for parameter in &signature.parameters {
            if self.scopes[0].contains_key(&parameter.name) {
                return self.fail(
                    "pors.redeclared",
                    format!("`{}` já foi declarado nesta função", parameter.name),
                    parameter.span,
                );
            }
            let id = hir::BindingId(self.bindings.len() as u32);
            let mutability = if parameter.mutable {
                hir::Mutability::Mutable
            } else {
                hir::Mutability::Immutable
            };
            self.bindings.push(hir::Binding {
                id,
                source_name: parameter.name.clone(),
                ty: parameter.ty,
                mutability,
                is_const: !parameter.mutable,
                span: to_hir_span(parameter.span),
            });
            self.scopes[0].insert(
                parameter.name.clone(),
                Symbol {
                    id,
                    ty: parameter.ty,
                    mutability,
                },
            );
        }
        Ok(())
    }

    pub(super) fn block(
        &mut self,
        root: bool,
        allow_tail: bool,
    ) -> Result<(Vec<hir::Stmt>, Option<hir::Expr>, Span), Vec<Diagnostic>> {
        let open = self.expect(Kind::LBrace, "Esperado `{` para iniciar bloco")?;
        if !root {
            self.scopes.push(HashMap::new());
        }
        let mut statements = Vec::new();
        let mut tail = None;
        while !matches!(self.current().kind, Kind::RBrace | Kind::Eof)
            && !is_top_level_start(&self.current().kind)
        {
            if let Some(expression) = self.try_tail()? {
                if !allow_tail {
                    return self.fail(
                        "pors.implicit-return-without-type",
                        "Expressão final sem `;` só é retorno no corpo de uma função",
                        syntax_span(expression.span),
                    );
                }
                tail = Some(expression);
                break;
            }
            let statement_start = self.cursor;
            let binding_count = self.bindings.len();
            let scopes = self.scopes.clone();
            let loop_depth = self.loop_depth;
            match self.statement() {
                Ok(statement) => statements.push(statement),
                Err(errors) => {
                    let syntax_errors: Vec<_> = errors
                        .iter()
                        .filter(|diagnostic| is_recoverable_syntax(diagnostic.code))
                        .cloned()
                        .collect();
                    if !syntax_errors.is_empty() {
                        append_unique(&mut self.diagnostics, syntax_errors);
                        self.function_has_syntax_error = true;
                    } else if !self.function_has_syntax_error {
                        return Err(errors);
                    }

                    // A failed statement must not leave partial bindings or scopes
                    // visible to later statements while the parser synchronizes.
                    self.bindings.truncate(binding_count);
                    self.scopes = scopes;
                    self.loop_depth = loop_depth;
                    self.synchronize_statement(statement_start);
                }
            }
        }
        let close = self.expect(Kind::RBrace, "Bloco não foi fechado com `}`")?;
        if !root {
            self.scopes.pop();
        }
        Ok((statements, tail, open.span.join(close.span)))
    }

    /// Reconhece uma expressão final sem `;`. Restaura a stream quando o trecho
    /// não é uma cauda, para que o comando normal seja analisado em seguida.
    fn try_tail(&mut self) -> Result<Option<hir::Expr>, Vec<Diagnostic>> {
        if !can_start_expression(&self.current().kind) {
            return Ok(None);
        }
        let cursor = self.cursor;
        let binding_count = self.bindings.len();
        let scopes = self.scopes.clone();
        let parsed = self.expression(0);
        match parsed {
            Ok(expression) if matches!(self.current().kind, Kind::RBrace | Kind::Eof) => {
                Ok(Some(expression))
            }
            Ok(_) => {
                self.cursor = cursor;
                self.bindings.truncate(binding_count);
                self.scopes = scopes;
                Ok(None)
            }
            Err(_) => {
                self.cursor = cursor;
                self.bindings.truncate(binding_count);
                self.scopes = scopes;
                Ok(None)
            }
        }
    }

    pub(super) fn synchronize_statement(&mut self, statement_start: usize) {
        let mut braces = 0usize;
        let mut parentheses = 0usize;
        let mut brackets = 0usize;
        for token in &self.tokens[statement_start..self.cursor.min(self.tokens.len())] {
            match token.kind {
                Kind::LBrace => braces += 1,
                Kind::RBrace => braces = braces.saturating_sub(1),
                Kind::LParen => parentheses += 1,
                Kind::RParen => parentheses = parentheses.saturating_sub(1),
                Kind::LBracket => brackets += 1,
                Kind::RBracket => brackets = brackets.saturating_sub(1),
                _ => {}
            }
        }
        if self.cursor > statement_start
            && matches!(self.tokens[self.cursor - 1].kind, Kind::Semicolon)
            && braces == 0
            && parentheses == 0
            && brackets == 0
        {
            return;
        }

        loop {
            match self.current().kind {
                Kind::Eof => return,
                Kind::Semicolon if braces == 0 && parentheses == 0 && brackets == 0 => {
                    self.advance();
                    return;
                }
                Kind::RBrace if braces == 0 => return,
                Kind::RBrace => {
                    self.advance();
                    braces -= 1;
                    if braces == 0 && parentheses == 0 && brackets == 0 {
                        if self.is_keyword("senao") {
                            continue;
                        }
                        return;
                    }
                }
                _ if braces == 0
                    && parentheses == 0
                    && brackets == 0
                    && is_top_level_start(&self.current().kind) =>
                {
                    return;
                }
                Kind::LBrace => {
                    braces += 1;
                    self.advance();
                }
                Kind::LParen => {
                    parentheses += 1;
                    self.advance();
                }
                Kind::RParen => {
                    parentheses = parentheses.saturating_sub(1);
                    self.advance();
                }
                Kind::LBracket => {
                    brackets += 1;
                    self.advance();
                }
                Kind::RBracket => {
                    brackets = brackets.saturating_sub(1);
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(super) fn statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        if self.is_keyword("seja") {
            return self.declaration();
        }
        if self.is_keyword("escreva") {
            return self.write(false);
        }
        if self.is_keyword("escreval") {
            return self.write(true);
        }
        if self.is_keyword("leia") {
            return self.read();
        }
        if self.is_keyword("se") {
            return self.conditional();
        }
        if self.is_keyword("enquanto") {
            return self.while_loop();
        }
        if self.is_keyword("para") {
            return self.for_loop();
        }
        if self.is_keyword("retorne") {
            return self.return_statement();
        }
        if self.is_keyword("pare") || self.is_keyword("continue") {
            return self.loop_control();
        }
        if self.is_keyword("adicionar") {
            return self.vector_push_statement();
        }
        if self.is_keyword("remover") {
            return self.vector_pop_statement();
        }
        if let Kind::Ident(keyword) = &self.current().kind {
            if keyword == "escolha" {
                return self.choose();
            }
            if keyword == "faca" {
                return self.do_while_loop();
            }
            if self.tokens.get(self.cursor + 1).is_some_and(|token| {
                matches!(
                    token.kind,
                    Kind::Assign
                        | Kind::PlusEq
                        | Kind::MinusEq
                        | Kind::StarEq
                        | Kind::SlashEq
                        | Kind::PercentEq
                        | Kind::AmpEq
                        | Kind::PipeEq
                        | Kind::CaretEq
                )
            }) {
                return self.assignment();
            }
            if self
                .tokens
                .get(self.cursor + 1)
                .is_some_and(|token| matches!(token.kind, Kind::Dot))
            {
                if matches!(
                    keyword.as_str(),
                    "Util" | "Internet" | "Graficos" | "Teclado" | "Mouse" | "Sons"
                ) && self.lookup(keyword).is_none()
                {
                    return self.stdlib_statement_call();
                }
                return self.field_assignment();
            }
            if self
                .tokens
                .get(self.cursor + 1)
                .is_some_and(|token| matches!(token.kind, Kind::LBracket))
            {
                return self.index_assignment();
            }
            if self
                .tokens
                .get(self.cursor + 1)
                .is_some_and(|token| matches!(token.kind, Kind::LParen))
            {
                let expression = self.expression(0)?;
                let end = self.expect(Kind::Semicolon, "Esperado `;` após chamada")?;
                let hir::ExprKind::Call {
                    function,
                    arguments,
                } = expression.kind
                else {
                    return self.fail(
                        "pors.expected-statement",
                        "Esperada chamada de função",
                        syntax_span(expression.span),
                    );
                };
                return Ok(hir::Stmt {
                    kind: hir::StmtKind::Call {
                        function,
                        arguments,
                    },
                    span: to_hir_span(syntax_span(expression.span).join(end.span)),
                });
            }
        }
        self.fail(
            "pors.expected-statement",
            "Esperada declaração, atribuição, escreva ou se",
            self.current().span,
        )
    }

    pub(super) fn declaration(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.current().span;
        self.keyword("seja")?;
        let mutable = self.eat_keyword("mutavel");
        let (name, name_span) = self.identifier()?;
        if is_reserved(&name) {
            return self.fail(
                "pors.reserved-name",
                format!("`{name}` é reservado e não pode ser usado como nome"),
                name_span,
            );
        }
        let declared_ty = self
            .eat(&Kind::Colon)
            .then(|| self.type_name())
            .transpose()?;
        self.expect(
            Kind::Assign,
            "Toda variável deve receber um valor inicial (`=`)",
        )?;
        let initializer = self.expression_with_expected(0, declared_ty)?;
        if let Some(ty) = declared_ty
            && initializer.ty != ty
        {
            return self.fail(
                "pors.type-mismatch",
                format!(
                    "O valor inicial de `{name}` não corresponde ao tipo `{}`",
                    type_label(ty)
                ),
                syntax_span(initializer.span),
            );
        }
        let ty = initializer.ty;
        let semi = self.expect(Kind::Semicolon, "Esperado `;` após declaração")?;
        if self.scopes.last().unwrap().contains_key(&name) {
            return self.fail(
                "pors.redeclared",
                format!("`{name}` já foi declarado neste bloco"),
                name_span,
            );
        }
        let id = hir::BindingId(self.bindings.len() as u32);
        let mutability = if mutable {
            hir::Mutability::Mutable
        } else {
            hir::Mutability::Immutable
        };
        self.bindings.push(hir::Binding {
            id,
            source_name: name.clone(),
            ty,
            mutability,
            is_const: !mutable,
            span: to_hir_span(name_span),
        });
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name, Symbol { id, ty, mutability });
        Ok(hir::Stmt {
            kind: hir::StmtKind::Declare {
                binding: id,
                initializer: Some(initializer),
            },
            span: to_hir_span(start.join(semi.span)),
        })
    }

    pub(super) fn assignment(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        self.assignment_with_terminator(true)
    }

    pub(super) fn field_assignment(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let (name, name_span) = self.identifier()?;
        let symbol = self.lookup(&name).ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.unknown-name",
                format!("Nome não declarado: {name}"),
                name_span,
            )]
        })?;
        let hir::Type::Record(mut type_id) = symbol.ty else {
            return self.fail(
                "pors.field-non-record",
                "Atribuição de campo exige um registro",
                name_span,
            );
        };
        // Caminho de campos: `t.dados.mensagem` desce um registro por `.`.
        let mut fields = Vec::new();
        let mut path = name.clone();
        let mut field_ty;
        loop {
            self.expect(Kind::Dot, "Esperado `.` após o registro")?;
            let (field_name, field_span) = self.identifier()?;
            let definition = self
                .record_types
                .iter()
                .find(|definition| definition.id == type_id)
                .expect("registro possui definição");
            let Some((field_index, ty)) = definition
                .fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == field_name)
                .map(|(index, field)| (index, field.ty))
            else {
                return self.fail(
                    "pors.unknown-field",
                    format!(
                        "Campo `{field_name}` não existe no registro `{}`",
                        definition.name
                    ),
                    field_span,
                );
            };
            fields.push(field_index);
            path.push('.');
            path.push_str(&field_name);
            field_ty = ty;
            if !matches!(self.current().kind, Kind::Dot) {
                break;
            }
            let hir::Type::Record(inner) = ty else {
                return self.fail(
                    "pors.field-non-record",
                    format!("`{path}` não é um registro; não tem campos"),
                    field_span,
                );
            };
            type_id = inner;
        }
        let op_token = self.advance();
        let op = match op_token.kind {
            Kind::Assign => hir::AssignOp::Set,
            Kind::PlusEq => hir::AssignOp::Add,
            Kind::MinusEq => hir::AssignOp::Subtract,
            Kind::StarEq => hir::AssignOp::Multiply,
            Kind::SlashEq => hir::AssignOp::Divide,
            Kind::PercentEq => hir::AssignOp::Remainder,
            Kind::AmpEq => hir::AssignOp::BitAnd,
            Kind::PipeEq => hir::AssignOp::BitOr,
            Kind::CaretEq => hir::AssignOp::Xor,
            _ => {
                return self.fail(
                    "pors.expected-assignment",
                    "Esperado operador de atribuição",
                    op_token.span,
                );
            }
        };
        let expected = (op == hir::AssignOp::Set).then_some(field_ty);
        let value = self.expression_with_expected(0, expected)?;
        if value.ty != field_ty {
            return self.fail(
                "pors.type-mismatch",
                format!("Atribuição incompatível para `{path}`"),
                syntax_span(value.span),
            );
        }
        if symbol.mutability == hir::Mutability::Immutable {
            return self.fail(
                "pors.immutable-assignment",
                format!("`{name}` é imutável; declare com `seja mutavel` para alterar seus campos"),
                name_span,
            );
        }
        let end = self.expect(Kind::Semicolon, "Esperado `;` após atribuição")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::FieldAssign {
                binding: symbol.id,
                fields,
                op,
                value,
            },
            span: to_hir_span(name_span.join(end.span)),
        })
    }

    pub(super) fn index_assignment(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let (name, name_span) = self.identifier()?;
        let symbol = self.lookup(&name).ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.unknown-name",
                format!("Nome não declarado: {name}"),
                name_span,
            )]
        })?;
        let Some(element_type) = symbol.ty.vector_element() else {
            return self.fail(
                "pors.index-non-vector",
                format!("`{name}` não é um vetor"),
                name_span,
            );
        };
        self.expect(Kind::LBracket, "Esperado `[` após o vetor")?;
        let index = self.expression(0)?;
        if index.ty != hir::Type::Integer {
            return self.fail(
                "pors.index-type",
                "O índice do vetor deve ter tipo inteiro",
                syntax_span(index.span),
            );
        }
        self.expect(Kind::RBracket, "Esperado `]` após índice")?;
        self.expect(
            Kind::Assign,
            "A atribuição em um índice de vetor deve usar `=`",
        )?;
        let value = self.expression_with_expected(0, Some(element_type))?;
        if value.ty != element_type {
            return self.fail(
                "pors.vector-element-type",
                "O valor atribuído não corresponde ao tipo dos elementos do vetor",
                syntax_span(value.span),
            );
        }
        let end = self.expect(Kind::Semicolon, "Esperado `;` após atribuição")?;
        if symbol.mutability == hir::Mutability::Immutable {
            return self.fail(
                "pors.immutable-assignment",
                format!(
                    "`{name}` é imutável; declare com `seja mutavel` para alterar seus elementos"
                ),
                name_span,
            );
        }
        Ok(hir::Stmt {
            kind: hir::StmtKind::IndexAssign {
                binding: symbol.id,
                index,
                value,
            },
            span: to_hir_span(name_span.join(end.span)),
        })
    }

    pub(super) fn assignment_with_terminator(
        &mut self,
        needs_semicolon: bool,
    ) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let (name, name_span) = self.identifier()?;
        let symbol = self.lookup(&name).ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.unknown-name",
                format!("Nome não declarado: {name}"),
                name_span,
            )]
        })?;
        let op_token = self.advance();
        let op = match op_token.kind {
            Kind::Assign => hir::AssignOp::Set,
            Kind::PlusEq => hir::AssignOp::Add,
            Kind::MinusEq => hir::AssignOp::Subtract,
            Kind::StarEq => hir::AssignOp::Multiply,
            Kind::SlashEq => hir::AssignOp::Divide,
            Kind::PercentEq => hir::AssignOp::Remainder,
            Kind::AmpEq => hir::AssignOp::BitAnd,
            Kind::PipeEq => hir::AssignOp::BitOr,
            Kind::CaretEq => hir::AssignOp::Xor,
            _ => {
                return self.fail(
                    "pors.expected-assignment",
                    "Esperado operador de atribuição",
                    op_token.span,
                );
            }
        };
        let expected = (op == hir::AssignOp::Set).then_some(symbol.ty);
        let value = self.expression_with_expected(0, expected)?;
        let end = if needs_semicolon {
            self.expect(Kind::Semicolon, "Esperado `;` após atribuição")?
                .span
        } else {
            syntax_span(value.span)
        };
        if symbol.mutability == hir::Mutability::Immutable {
            return self.fail(
                "pors.immutable-assignment",
                format!(
                    "`{name}` é imutável; declare com `seja mutavel` para permitir atribuições"
                ),
                name_span,
            );
        }
        if symbol.ty != value.ty {
            return self.fail(
                "pors.type-mismatch",
                format!("Atribuição incompatível para `{name}`"),
                syntax_span(value.span),
            );
        }
        let valid = match op {
            hir::AssignOp::Set => true,
            hir::AssignOp::Add => matches!(
                symbol.ty,
                hir::Type::Integer | hir::Type::Real | hir::Type::String
            ),
            hir::AssignOp::Subtract | hir::AssignOp::Multiply | hir::AssignOp::Divide => {
                matches!(symbol.ty, hir::Type::Integer | hir::Type::Real)
            }
            hir::AssignOp::Remainder
            | hir::AssignOp::BitAnd
            | hir::AssignOp::BitOr
            | hir::AssignOp::Xor => symbol.ty == hir::Type::Integer,
        };
        if !valid {
            return self.fail(
                "pors.invalid-assignment-operator",
                "Este operador não é válido para o tipo da variável",
                op_token.span,
            );
        }
        Ok(hir::Stmt {
            kind: hir::StmtKind::Assign {
                binding: symbol.id,
                op,
                value,
            },
            span: to_hir_span(name_span.join(end)),
        })
    }

    pub(super) fn write(&mut self, newline: bool) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.expect(Kind::LParen, "Esperado `(` após escreva")?;
        let mut values = Vec::new();
        if !matches!(self.current().kind, Kind::RParen) {
            loop {
                let value = self.expression(0)?;
                if value.ty == hir::Type::Unit {
                    return self.fail(
                        "pors.void-value",
                        "Uma função sem retorno não pode ser usada como valor",
                        syntax_span(value.span),
                    );
                }
                if value.ty.is_vector() {
                    return self.fail(
                        "pors.vector-value",
                        "Um vetor precisa ser acessado por índice antes de ser escrito",
                        syntax_span(value.span),
                    );
                }
                if matches!(value.ty, hir::Type::Record(_) | hir::Type::Enum(_)) {
                    return self.fail("pors.compound-value", "Registro ou enum precisa ser acessado por campos ou escolha antes de ser escrito", syntax_span(value.span));
                }
                values.push(value);
                if !self.eat(&Kind::Comma) {
                    break;
                }
            }
        }
        let close = self.expect(Kind::RParen, "Esperado `)` após valores de escreva")?;
        if newline {
            values.push(hir::Expr {
                kind: hir::ExprKind::String("\n".to_owned()),
                ty: hir::Type::String,
                span: hir::Span::new(close.span.start, close.span.start),
            });
        }
        let end = self.expect(Kind::Semicolon, "Esperado `;` após escreva")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::Write(values),
            span: to_hir_span(start.join(end.span)),
        })
    }

    pub(super) fn return_statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        let value = if matches!(self.current().kind, Kind::Semicolon) {
            None
        } else {
            Some(self.expression_with_expected(0, self.return_type)?)
        };
        let end = self.expect(Kind::Semicolon, "Esperado `;` após retorne")?;
        if !self.in_function {
            return self.fail(
                "pors.return-outside-function",
                "`retorne` só pode ser usado dentro de uma função",
                start,
            );
        }
        match (self.return_type, value.as_ref()) {
            (Some(expected), Some(actual)) if expected == actual.ty => {}
            (Some(_), Some(actual)) => {
                return self.fail(
                    "pors.return-type-mismatch",
                    "O valor retornado não corresponde ao tipo da função",
                    syntax_span(actual.span),
                );
            }
            (Some(_), None) => {
                return self.fail(
                    "pors.return-value-required",
                    "Esta função precisa retornar um valor",
                    start,
                );
            }
            (None, Some(actual)) => {
                return self.fail(
                    "pors.unexpected-return-value",
                    "Esta função não declara tipo de retorno",
                    syntax_span(actual.span),
                );
            }
            (None, None) => {}
        }
        Ok(hir::Stmt {
            kind: hir::StmtKind::Return { value },
            span: to_hir_span(start.join(end.span)),
        })
    }

    pub(super) fn read(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.expect(Kind::LParen, "Esperado `(` após leia")?;
        let (name, name_span) = self.identifier()?;
        let symbol = self.lookup(&name).ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.unknown-name",
                format!("A variável `{name}` não foi declarada"),
                name_span,
            )]
        })?;
        if symbol.ty.is_vector() {
            return self.fail(
                "pors.read-vector-unsupported",
                "A entrada direta de um vetor ainda não é suportada; leia cada elemento por índice",
                name_span,
            );
        }
        if symbol.mutability != hir::Mutability::Mutable {
            return self.fail(
                "pors.immutable-assignment",
                format!("A variável `{name}` é imutável e não pode receber entrada"),
                name_span,
            );
        }
        self.expect(Kind::RParen, "Esperado `)` após a variável de leia")?;
        let end = self.expect(Kind::Semicolon, "Esperado `;` após leia")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::Read {
                binding: symbol.id,
                name_span: to_hir_span(name_span),
            },
            span: to_hir_span(start.join(end.span)),
        })
    }

    pub(super) fn vector_push_statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.expect(Kind::LParen, "Esperado `(` após adicionar")?;
        let (name, name_span) = self.identifier()?;
        let symbol = self.lookup(&name).ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.unknown-name",
                format!("Nome não declarado: {name}"),
                name_span,
            )]
        })?;
        let Some(element_type) = symbol.ty.vector_element() else {
            return self.fail(
                "pors.vector-operation-type",
                format!("`adicionar` espera um vetor, mas `{name}` não é um vetor"),
                name_span,
            );
        };
        if symbol.mutability != hir::Mutability::Mutable {
            return self.fail(
                "pors.immutable-assignment",
                format!("`{name}` é imutável; declare com `seja mutavel` para adicionar elementos"),
                name_span,
            );
        }
        self.expect(
            Kind::Comma,
            "Esperado `,` entre o vetor e o valor em `adicionar`",
        )?;
        let value = self.expression_with_expected(0, Some(element_type))?;
        if value.ty != element_type {
            return self.fail(
                "pors.vector-element-type",
                "O valor adicionado não corresponde ao tipo dos elementos do vetor",
                syntax_span(value.span),
            );
        }
        self.expect(Kind::RParen, "Esperado `)` após argumentos de `adicionar`")?;
        let end = self.expect(Kind::Semicolon, "Esperado `;` após `adicionar`")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::VectorPush {
                binding: symbol.id,
                value,
            },
            span: to_hir_span(start.join(end.span)),
        })
    }

    pub(super) fn vector_pop_statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.expect(Kind::LParen, "Esperado `(` após remover")?;
        let (name, name_span) = self.identifier()?;
        let symbol = self.lookup(&name).ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.unknown-name",
                format!("Nome não declarado: {name}"),
                name_span,
            )]
        })?;
        if !symbol.ty.is_vector() {
            return self.fail(
                "pors.vector-operation-type",
                format!("`remover` espera um vetor, mas `{name}` não é um vetor"),
                name_span,
            );
        }
        if symbol.mutability != hir::Mutability::Mutable {
            return self.fail(
                "pors.immutable-assignment",
                format!("`{name}` é imutável; declare com `seja mutavel` para remover elementos"),
                name_span,
            );
        }
        self.expect(Kind::RParen, "Esperado `)` após o vetor de `remover`")?;
        let end = self.expect(Kind::Semicolon, "Esperado `;` após `remover`")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::VectorPop { binding: symbol.id },
            span: to_hir_span(start.join(end.span)),
        })
    }

    pub(super) fn choose(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.current().span;
        self.keyword("escolha")?;
        let value = self.expression(0)?;
        let hir::Type::Enum(type_id) = value.ty else {
            return self.fail(
                "pors.choose-type",
                "`escolha` exige um valor enum",
                syntax_span(value.span),
            );
        };
        let definition = self
            .enum_types
            .iter()
            .find(|definition| definition.id == type_id)
            .cloned()
            .expect("enum tipado possui definição");
        self.expect(Kind::LBrace, "Esperado `{` após valor de escolha")?;
        let mut arms = Vec::new();
        let mut seen = HashMap::new();
        let mut has_wildcard = false;
        while self.is_keyword("caso") {
            let arm_start = self.current().span;
            self.advance();
            if has_wildcard {
                return self.fail(
                    "pors.unreachable-case",
                    "Este caso é inalcançável após o ramo coringa `_`",
                    arm_start,
                );
            }
            let (enum_name, enum_span) = self.identifier()?;
            if enum_name == "_" && !matches!(self.current().kind, Kind::ColonColon) {
                if seen.len() == definition.variants.len() {
                    return self.fail(
                        "pors.unreachable-case",
                        "O ramo coringa `_` é inalcançável porque todas as variantes já foram tratadas",
                        enum_span,
                    );
                }
                self.expect(Kind::FatArrow, "Esperado `=>` após `_`")?;
                self.scopes.push(HashMap::new());
                let (body, _, arm_end) = self.block(true, false)?;
                self.scopes.pop();
                has_wildcard = true;
                arms.push(hir::ChooseArm {
                    variant_index: None,
                    payload_bindings: Vec::new(),
                    body,
                    span: to_hir_span(arm_start.join(arm_end)),
                });
                continue;
            }
            if enum_name != definition.name {
                return self.fail(
                    "pors.choose-enum-mismatch",
                    format!("Esperada variante de `{}`", definition.name),
                    enum_span,
                );
            }
            self.expect(Kind::ColonColon, "Esperado `::` antes da variante")?;
            let (variant_name, variant_span) = self.identifier()?;
            let Some((variant_index, variant)) = definition
                .variants
                .iter()
                .enumerate()
                .find(|(_, variant)| variant.name == variant_name)
                .map(|(index, variant)| (index, variant.clone()))
            else {
                return self.fail(
                    "pors.unknown-variant",
                    format!(
                        "Variante `{variant_name}` não existe em `{}`",
                        definition.name
                    ),
                    variant_span,
                );
            };
            if seen.insert(variant_index, ()).is_some() {
                return self.fail(
                    "pors.duplicate-case",
                    format!("A variante `{variant_name}` aparece mais de uma vez"),
                    variant_span,
                );
            }
            let mut payload_names = Vec::new();
            if self.eat(&Kind::LParen) {
                if !matches!(self.current().kind, Kind::RParen) {
                    loop {
                        payload_names.push(self.identifier()?);
                        if !self.eat(&Kind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(Kind::RParen, "Esperado `)` após bindings do caso")?;
            }
            if payload_names.len() != variant.payload_types.len() {
                return self.fail(
                    "pors.case-payload-arity",
                    format!(
                        "A variante `{variant_name}` exige {} binding(s) de payload",
                        variant.payload_types.len()
                    ),
                    variant_span,
                );
            }
            self.expect(Kind::FatArrow, "Esperado `=>` após a variante")?;
            self.scopes.push(HashMap::new());
            let mut payload_bindings = Vec::new();
            for ((name, name_span), ty) in payload_names
                .into_iter()
                .zip(variant.payload_types.iter().copied())
            {
                if self.scopes.last().unwrap().contains_key(&name) || is_reserved(&name) {
                    self.scopes.pop();
                    return self.fail(
                        "pors.redeclared",
                        format!("Binding de payload inválido: `{name}`"),
                        name_span,
                    );
                }
                let id = hir::BindingId(self.bindings.len() as u32);
                self.bindings.push(hir::Binding {
                    id,
                    source_name: name.clone(),
                    ty,
                    mutability: hir::Mutability::Immutable,
                    is_const: true,
                    span: to_hir_span(name_span),
                });
                self.scopes.last_mut().unwrap().insert(
                    name,
                    Symbol {
                        id,
                        ty,
                        mutability: hir::Mutability::Immutable,
                    },
                );
                payload_bindings.push(id);
            }
            let (body, _, arm_end) = self.block(true, false)?;
            self.scopes.pop();
            arms.push(hir::ChooseArm {
                variant_index: Some(variant_index),
                payload_bindings,
                body,
                span: to_hir_span(arm_start.join(arm_end)),
            });
        }
        let close = self.expect(Kind::RBrace, "Esperado `}` após casos de escolha")?;
        if !has_wildcard
            && let Some(missing) =
                (0..definition.variants.len()).find(|index| !seen.contains_key(index))
        {
            return self.fail(
                "pors.non-exhaustive-choice",
                format!(
                    "A variante `{}` não foi contemplada",
                    definition.variants[missing].name
                ),
                close.span,
            );
        }
        Ok(hir::Stmt {
            kind: hir::StmtKind::Choose { value, arms },
            span: to_hir_span(start.join(close.span)),
        })
    }

    pub(super) fn conditional(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.expect(Kind::LParen, "Esperado `(` após se")?;
        let condition = self.expression(0)?;
        self.expect(Kind::RParen, "Esperado `)` após condição")?;
        if condition.ty != hir::Type::Boolean {
            return self.fail(
                "pors.condition-type",
                "A condição de se deve ser logica",
                syntax_span(condition.span),
            );
        }
        let (then_branch, _, then_span) = self.block(false, false)?;
        let (else_branch, end) = if self.eat_keyword("senao") {
            if self.is_keyword("se") {
                let nested = self.conditional()?;
                let end = Span::new(nested.span.start, nested.span.end);
                (Some(vec![nested]), end)
            } else {
                let (body, _, block_span) = self.block(false, false)?;
                (Some(body), block_span)
            }
        } else {
            (None, then_span)
        };
        Ok(hir::Stmt {
            kind: hir::StmtKind::If {
                condition,
                then_branch,
                else_branch,
            },
            span: to_hir_span(start.join(end)),
        })
    }

    pub(super) fn loop_control(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let token = self.advance();
        let (keyword, control) = match token.kind {
            Kind::Ident(word) if word == "pare" => (word, hir::LoopControl::Break),
            Kind::Ident(word) if word == "continue" => (word, hir::LoopControl::Continue),
            _ => unreachable!("loop_control só é chamado para keywords conhecidas"),
        };
        if self.loop_depth == 0 {
            return self.fail(
                "pors.loop-control-outside-loop",
                format!("`{keyword}` só pode ser usado dentro de enquanto, faca-enquanto ou para"),
                token.span,
            );
        }
        let end = self.expect(Kind::Semicolon, "Esperado `;` após controle do laço")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::LoopControl { control },
            span: to_hir_span(token.span.join(end.span)),
        })
    }

    pub(super) fn do_while_loop(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.loop_depth += 1;
        let body_result = self.block(false, false);
        self.loop_depth -= 1;
        let (body, _, _) = body_result?;
        self.keyword("enquanto")?;
        self.expect(Kind::LParen, "Esperado `(` após enquanto")?;
        let condition = self.expression(0)?;
        self.expect(Kind::RParen, "Esperado `)` após condição")?;
        if condition.ty != hir::Type::Boolean {
            return self.fail(
                "pors.condition-type",
                "A condição de enquanto deve ser logica",
                syntax_span(condition.span),
            );
        }
        let end = self.expect(Kind::Semicolon, "Esperado `;` após faca-enquanto")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::DoWhile { body, condition },
            span: to_hir_span(start.join(end.span)),
        })
    }

    pub(super) fn while_loop(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.expect(Kind::LParen, "Esperado `(` após enquanto")?;
        let condition = self.expression(0)?;
        self.expect(Kind::RParen, "Esperado `)` após condição")?;
        if condition.ty != hir::Type::Boolean {
            return self.fail(
                "pors.condition-type",
                "A condição de enquanto deve ser logica",
                syntax_span(condition.span),
            );
        }
        self.loop_depth += 1;
        let body_result = self.block(false, false);
        self.loop_depth -= 1;
        let (body, _, end) = body_result?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::While { condition, body },
            span: to_hir_span(start.join(end)),
        })
    }

    pub(super) fn for_loop(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        self.expect(Kind::LParen, "Esperado `(` após para")?;
        self.scopes.push(HashMap::new());
        let initializer = self.declaration()?;
        let control = match initializer.kind {
            hir::StmtKind::Declare { binding, .. } => self.bindings[binding.0 as usize].clone(),
            _ => unreachable!("declaration() sempre retorna uma declaração"),
        };
        if !matches!(control.ty, hir::Type::Integer | hir::Type::Real) {
            return self.fail(
                "pors.for-control-type",
                "O controle de para deve ter tipo inteiro ou real",
                syntax_span(control.span),
            );
        }

        let condition = self.expression(0)?;
        if condition.ty != hir::Type::Boolean {
            return self.fail(
                "pors.condition-type",
                "A condição de para deve ser logica",
                syntax_span(condition.span),
            );
        }
        self.expect(Kind::Semicolon, "Esperado `;` após condição de para")?;
        let step = self.assignment_with_terminator(false)?;
        let (step_binding, step_op) = match step.kind {
            hir::StmtKind::Assign { binding, op, .. } => (binding, op),
            _ => unreachable!("assignment_with_terminator sempre retorna atribuição"),
        };
        if step_binding != control.id {
            return self.fail(
                "pors.for-step-binding",
                "A atualização de para deve alterar a variável declarada no inicializador",
                syntax_span(step.span),
            );
        }
        if step_op == hir::AssignOp::Set {
            return self.fail(
                "pors.for-step-operator",
                "Use um operador composto para atualizar o controle de para (por exemplo, `+= 1`)",
                syntax_span(step.span),
            );
        }
        self.expect(Kind::RParen, "Esperado `)` após atualização de para")?;
        self.loop_depth += 1;
        let body_result = self.block(false, false);
        self.loop_depth -= 1;
        let (body, _, end) = body_result?;
        self.scopes.pop();
        Ok(hir::Stmt {
            kind: hir::StmtKind::For {
                initializer: Box::new(initializer),
                condition,
                step: Box::new(step),
                body,
            },
            span: to_hir_span(start.join(end)),
        })
    }
}
