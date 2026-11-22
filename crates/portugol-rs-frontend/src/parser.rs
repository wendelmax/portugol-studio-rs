use std::collections::HashMap;

use portugol_hir as hir;
use portugol_syntax::{Diagnostic, Span};

use crate::lexer::{Kind, Token};

pub(super) struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    bindings: Vec<hir::Binding>,
    scopes: Vec<HashMap<String, Symbol>>,
    loop_depth: usize,
    signatures: HashMap<String, FunctionSignature>,
    return_type: Option<hir::Type>,
    in_function: bool,
}

#[derive(Clone)]
struct FunctionParameter {
    name: String,
    ty: hir::Type,
    mutable: bool,
    span: Span,
}

#[derive(Clone)]
struct FunctionSignature {
    id: Option<hir::FunctionId>,
    parameters: Vec<FunctionParameter>,
    return_type: Option<hir::Type>,
}

#[derive(Clone, Copy)]
struct Symbol {
    id: hir::BindingId,
    ty: hir::Type,
    mutability: hir::Mutability,
}

impl Parser {
    pub(super) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            bindings: vec![],
            scopes: vec![HashMap::new()],
            loop_depth: 0,
            signatures: HashMap::new(),
            return_type: None,
            in_function: false,
        }
    }

    pub(super) fn program(mut self) -> Result<hir::Program, Vec<Diagnostic>> {
        self.collect_signatures()?;
        let start = self.current().span.start;
        let mut bindings = None;
        let mut body = None;
        let mut functions = Vec::new();
        while !matches!(self.current().kind, Kind::Eof) {
            let (name, header_span, _) = self.function_header()?;
            let signature = self.signatures[&name].clone();
            self.begin_function(&signature)?;
            let (function_body, body_span) = self.block(true)?;
            if signature.return_type.is_some() && !guarantees_return(&function_body) {
                return self.fail(
                    "pors.missing-return",
                    format!("A função `{name}` precisa retornar um valor em todos os caminhos"),
                    body_span,
                );
            }
            let local_bindings = std::mem::take(&mut self.bindings);
            let function_span = header_span.join(body_span);
            if name == "inicio" {
                bindings = Some(local_bindings);
                body = Some(function_body);
            } else {
                functions.push(hir::Function {
                    id: signature.id.expect("função auxiliar tem ID"),
                    source_name: name,
                    parameters: (0..signature.parameters.len() as u32)
                        .map(hir::BindingId)
                        .collect(),
                    bindings: local_bindings,
                    return_type: signature.return_type,
                    body: function_body,
                    span: to_hir_span(function_span),
                });
            }
            self.scopes = vec![HashMap::new()];
            self.return_type = None;
            self.in_function = false;
            self.loop_depth = 0;
        }
        let bindings = bindings.ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.entry-function",
                "O programa precisa declarar uma única função inicio()",
                self.current().span,
            )]
        })?;
        let body = body.expect("bindings e body são definidos juntos para inicio");
        let end = self.previous_span().end;
        Ok(hir::Program {
            bindings,
            body,
            functions,
            span: hir::Span::new(start, end),
        })
    }

    fn collect_signatures(&mut self) -> Result<(), Vec<Diagnostic>> {
        self.cursor = 0;
        let mut next_id = 0;
        while !matches!(self.current().kind, Kind::Eof) {
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
            self.skip_function_body()?;
        }
        if !self.signatures.contains_key("inicio") {
            return self.fail(
                "pors.entry-function",
                "O programa precisa declarar uma função inicio()",
                self.current().span,
            );
        }
        self.cursor = 0;
        Ok(())
    }

    fn function_header(&mut self) -> Result<(String, Span, FunctionSignature), Vec<Diagnostic>> {
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
            },
        ))
    }

    fn type_name(&mut self) -> Result<hir::Type, Vec<Diagnostic>> {
        let (name, span) = self.identifier()?;
        match name.as_str() {
            "inteiro" => Ok(hir::Type::Integer),
            "real" => Ok(hir::Type::Real),
            "logico" => Ok(hir::Type::Boolean),
            "cadeia" => Ok(hir::Type::String),
            "caracter" => Ok(hir::Type::Character),
            _ => self.fail(
                "pors.unknown-type",
                format!("Tipo desconhecido: {name}"),
                span,
            ),
        }
    }

    fn skip_function_body(&mut self) -> Result<(), Vec<Diagnostic>> {
        self.expect(Kind::LBrace, "Esperado `{` para iniciar o corpo da função")?;
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

    fn begin_function(&mut self, signature: &FunctionSignature) -> Result<(), Vec<Diagnostic>> {
        self.bindings.clear();
        self.scopes = vec![HashMap::new()];
        self.loop_depth = 0;
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

    fn block(&mut self, root: bool) -> Result<(Vec<hir::Stmt>, Span), Vec<Diagnostic>> {
        let open = self.expect(Kind::LBrace, "Esperado `{` para iniciar bloco")?;
        if !root {
            self.scopes.push(HashMap::new());
        }
        let mut statements = Vec::new();
        while !matches!(self.current().kind, Kind::RBrace | Kind::Eof) {
            statements.push(self.statement()?);
        }
        let close = self.expect(Kind::RBrace, "Bloco não foi fechado com `}`")?;
        if !root {
            self.scopes.pop();
        }
        Ok((statements, open.span.join(close.span)))
    }

    fn statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        if self.is_keyword("seja") {
            return self.declaration();
        }
        if self.is_keyword("escreva") {
            return self.write();
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
        if let Kind::Ident(keyword) = &self.current().kind {
            if matches!(keyword.as_str(), "faca" | "escolha") {
                return self.fail(
                    "pors.unsupported-construct",
                    format!("`{keyword}` ainda não faz parte do subset Portugol-RS"),
                    self.current().span,
                );
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
                )
            }) {
                return self.assignment();
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

    fn declaration(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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
        self.expect(Kind::Colon, "Esperado `:` entre nome e tipo")?;
        let (type_name, type_span) = self.identifier()?;
        let ty = match type_name.as_str() {
            "inteiro" => hir::Type::Integer,
            "real" => hir::Type::Real,
            "logico" => hir::Type::Boolean,
            "cadeia" => hir::Type::String,
            "caracter" => hir::Type::Character,
            _ => {
                return self.fail(
                    "pors.unknown-type",
                    format!("Tipo desconhecido: {type_name}"),
                    type_span,
                );
            }
        };
        self.expect(
            Kind::Assign,
            "Toda variável deve receber um valor inicial (`=`)",
        )?;
        let initializer = self.expression(0)?;
        if initializer.ty != ty {
            return self.fail(
                "pors.type-mismatch",
                format!("O valor inicial de `{name}` não corresponde ao tipo `{type_name}`"),
                syntax_span(initializer.span),
            );
        }
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

    fn assignment(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        self.assignment_with_terminator(true)
    }

    fn assignment_with_terminator(
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
            _ => {
                return self.fail(
                    "pors.expected-assignment",
                    "Esperado operador de atribuição",
                    op_token.span,
                );
            }
        };
        let value = self.expression(0)?;
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
            hir::AssignOp::Remainder => symbol.ty == hir::Type::Integer,
            _ => false,
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

    fn write(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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
                values.push(value);
                if !self.eat(&Kind::Comma) {
                    break;
                }
            }
        }
        self.expect(Kind::RParen, "Esperado `)` após valores de escreva")?;
        let end = self.expect(Kind::Semicolon, "Esperado `;` após escreva")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::Write(values),
            span: to_hir_span(start.join(end.span)),
        })
    }

    fn return_statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let start = self.advance().span;
        let value = if matches!(self.current().kind, Kind::Semicolon) {
            None
        } else {
            Some(self.expression(0)?)
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

    fn read(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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

    fn conditional(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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
        let (then_branch, then_span) = self.block(false)?;
        let (else_branch, end) = if self.eat_keyword("senao") {
            let (body, block_span) = self.block(false)?;
            (Some(body), block_span)
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

    fn loop_control(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let token = self.advance();
        let (keyword, control) = match token.kind {
            Kind::Ident(word) if word == "pare" => (word, hir::LoopControl::Break),
            Kind::Ident(word) if word == "continue" => (word, hir::LoopControl::Continue),
            _ => unreachable!("loop_control só é chamado para keywords conhecidas"),
        };
        if self.loop_depth == 0 {
            return self.fail(
                "pors.loop-control-outside-loop",
                format!("`{keyword}` só pode ser usado dentro de enquanto ou para"),
                token.span,
            );
        }
        let end = self.expect(Kind::Semicolon, "Esperado `;` após controle do laço")?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::LoopControl { control },
            span: to_hir_span(token.span.join(end.span)),
        })
    }

    fn while_loop(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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
        let body_result = self.block(false);
        self.loop_depth -= 1;
        let (body, end) = body_result?;
        Ok(hir::Stmt {
            kind: hir::StmtKind::While { condition, body },
            span: to_hir_span(start.join(end)),
        })
    }

    fn for_loop(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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
        let body_result = self.block(false);
        self.loop_depth -= 1;
        let (body, end) = body_result?;
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

    fn expression(&mut self, min_precedence: u8) -> Result<hir::Expr, Vec<Diagnostic>> {
        let mut left = self.prefix()?;
        while let Some((op, precedence)) = binary(&self.current().kind) {
            if precedence < min_precedence {
                break;
            }
            self.advance();
            let right = self.expression(precedence + 1)?;
            let ty = binary_type(op, left.ty, right.ty).ok_or_else(|| {
                vec![Diagnostic::error(
                    "pors.type-mismatch",
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

    fn prefix(&mut self) -> Result<hir::Expr, Vec<Diagnostic>> {
        let token = self.advance();
        let span = token.span;
        let (kind, ty) = match token.kind {
            Kind::Integer(n) => (
                hir::ExprKind::Integer(i32::try_from(n).map_err(|_| {
                    vec![Diagnostic::error(
                        "pors.integer-range",
                        "Literal inteiro fora do intervalo de 32 bits",
                        span,
                    )]
                })?),
                hir::Type::Integer,
            ),
            Kind::Real(n) => (hir::ExprKind::Real(n), hir::Type::Real),
            Kind::String(value) => (hir::ExprKind::String(value), hir::Type::String),
            Kind::Character(value) => (hir::ExprKind::Character(value), hir::Type::Character),
            Kind::Ident(word) if word == "verdadeiro" => {
                (hir::ExprKind::Boolean(true), hir::Type::Boolean)
            }
            Kind::Ident(word) if word == "falso" => {
                (hir::ExprKind::Boolean(false), hir::Type::Boolean)
            }
            Kind::Ident(name) => {
                if matches!(self.current().kind, Kind::LParen) {
                    let signature = self.signatures.get(&name).cloned().ok_or_else(|| {
                        vec![Diagnostic::error(
                            "pors.unknown-function",
                            format!("Função não declarada: {name}"),
                            span,
                        )]
                    })?;
                    self.advance();
                    let mut arguments = Vec::new();
                    if !matches!(self.current().kind, Kind::RParen) {
                        loop {
                            arguments.push(self.expression(0)?);
                            if !self.eat(&Kind::Comma) {
                                break;
                            }
                        }
                    }
                    let close = self.expect(Kind::RParen, "Esperado `)` após argumentos")?;
                    if arguments.len() != signature.parameters.len() {
                        return self.fail(
                            "pors.call-arity",
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
                                "pors.call-type-mismatch",
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
                            "pors.entry-call",
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
                let symbol = self.lookup(&name).ok_or_else(|| {
                    vec![Diagnostic::error(
                        "pors.unknown-name",
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
            Kind::Minus | Kind::Plus | Kind::Bang => {
                let op = match token.kind {
                    Kind::Minus => hir::UnaryOp::Negate,
                    Kind::Plus => hir::UnaryOp::Positive,
                    _ => hir::UnaryOp::Not,
                };
                let operand = self.expression(7)?;
                let ty = match (op, operand.ty) {
                    (
                        hir::UnaryOp::Negate | hir::UnaryOp::Positive,
                        hir::Type::Integer | hir::Type::Real,
                    ) => operand.ty,
                    (hir::UnaryOp::Not, hir::Type::Boolean) => hir::Type::Boolean,
                    _ => {
                        return self.fail(
                            "pors.invalid-unary-operator",
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
                let inner = self.expression(0)?;
                let close = self.expect(Kind::RParen, "Esperado `)` para fechar expressão")?;
                let ty = inner.ty;
                return Ok(hir::Expr {
                    kind: hir::ExprKind::Parenthesized(Box::new(inner)),
                    ty,
                    span: to_hir_span(span.join(close.span)),
                });
            }
            other => {
                return self.fail(
                    "pors.expected-expression",
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

    fn lookup(&self, name: &str) -> Option<Symbol> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }
    fn is_keyword(&self, word: &str) -> bool {
        matches!(&self.current().kind, Kind::Ident(name) if name == word)
    }
    fn eat_keyword(&mut self, word: &str) -> bool {
        if self.is_keyword(word) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn keyword(&mut self, word: &str) -> Result<(), Vec<Diagnostic>> {
        if self.eat_keyword(word) {
            Ok(())
        } else {
            self.fail(
                "pors.expected-keyword",
                format!("Esperado `{word}`"),
                self.current().span,
            )
        }
    }
    fn identifier(&mut self) -> Result<(String, Span), Vec<Diagnostic>> {
        let token = self.advance();
        if let Kind::Ident(name) = token.kind {
            Ok((name, token.span))
        } else {
            self.fail("pors.expected-identifier", "Esperado nome", token.span)
        }
    }
    fn expect(&mut self, expected: Kind, message: &str) -> Result<Token, Vec<Diagnostic>> {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(&expected) {
            Ok(self.advance())
        } else {
            self.fail("pors.unexpected-token", message, self.current().span)
        }
    }
    fn eat(&mut self, expected: &Kind) -> bool {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn advance(&mut self) -> Token {
        let token = self.current().clone();
        if !matches!(token.kind, Kind::Eof) {
            self.cursor += 1;
        }
        token
    }
    fn current(&self) -> &Token {
        &self.tokens[self.cursor.min(self.tokens.len() - 1)]
    }
    fn previous_span(&self) -> Span {
        self.tokens[self.cursor.saturating_sub(1)].span
    }
    fn fail<T>(
        &self,
        code: &'static str,
        message: impl Into<String>,
        span: Span,
    ) -> Result<T, Vec<Diagnostic>> {
        Err(vec![Diagnostic::error(code, message, span)])
    }
}

fn to_hir_span(value: Span) -> hir::Span {
    hir::Span::new(value.start, value.end)
}
fn syntax_span(value: hir::Span) -> Span {
    Span::new(value.start, value.end)
}
fn join_hir(a: hir::Span, b: hir::Span) -> hir::Span {
    hir::Span::new(a.start.min(b.start), a.end.max(b.end))
}

fn is_reserved(name: &str) -> bool {
    matches!(
        name,
        "funcao"
            | "inicio"
            | "seja"
            | "mutavel"
            | "inteiro"
            | "real"
            | "logico"
            | "cadeia"
            | "caracter"
            | "verdadeiro"
            | "falso"
            | "escreva"
            | "se"
            | "senao"
            | "enquanto"
            | "para"
            | "pare"
            | "continue"
            | "faca"
            | "escolha"
            | "leia"
            | "retorne"
    )
}

fn guarantees_return(statements: &[hir::Stmt]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        hir::StmtKind::Return { .. } => true,
        hir::StmtKind::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => guarantees_return(then_branch) && guarantees_return(else_branch),
        _ => false,
    })
}

fn binary(kind: &Kind) -> Option<(hir::BinaryOp, u8)> {
    Some(match kind {
        Kind::Or => (hir::BinaryOp::Or, 1),
        Kind::And => (hir::BinaryOp::And, 2),
        Kind::Equal => (hir::BinaryOp::Equal, 3),
        Kind::NotEqual => (hir::BinaryOp::NotEqual, 3),
        Kind::Greater => (hir::BinaryOp::Greater, 4),
        Kind::Less => (hir::BinaryOp::Less, 4),
        Kind::GreaterEqual => (hir::BinaryOp::GreaterEqual, 4),
        Kind::LessEqual => (hir::BinaryOp::LessEqual, 4),
        Kind::Plus => (hir::BinaryOp::Add, 5),
        Kind::Minus => (hir::BinaryOp::Subtract, 5),
        Kind::Star => (hir::BinaryOp::Multiply, 6),
        Kind::Slash => (hir::BinaryOp::Divide, 6),
        Kind::Percent => (hir::BinaryOp::Remainder, 6),
        _ => return None,
    })
}

fn binary_type(op: hir::BinaryOp, left: hir::Type, right: hir::Type) -> Option<hir::Type> {
    use hir::Type::*;
    Some(match op {
        hir::BinaryOp::Add
            if left == String && right == String || left == Character && right == Character =>
        {
            String
        }
        hir::BinaryOp::Add
        | hir::BinaryOp::Subtract
        | hir::BinaryOp::Multiply
        | hir::BinaryOp::Divide
            if left == right && matches!(left, Integer | Real) =>
        {
            left
        }
        hir::BinaryOp::Remainder if left == Integer && right == Integer => Integer,
        hir::BinaryOp::And | hir::BinaryOp::Or if left == Boolean && right == Boolean => Boolean,
        hir::BinaryOp::Equal | hir::BinaryOp::NotEqual if left == right => Boolean,
        hir::BinaryOp::Greater
        | hir::BinaryOp::Less
        | hir::BinaryOp::GreaterEqual
        | hir::BinaryOp::LessEqual
            if left == right && matches!(left, Integer | Real | Character) =>
        {
            Boolean
        }
        _ => return None,
    })
}
