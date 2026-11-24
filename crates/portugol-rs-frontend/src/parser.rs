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
    record_types: Vec<hir::RecordDef>,
    record_ids: HashMap<String, hir::TypeId>,
    enum_types: Vec<hir::EnumDef>,
    enum_ids: HashMap<String, hir::TypeId>,
    builtin_enums: HashMap<hir::BuiltinEnum, hir::TypeId>,
    declared_types: HashMap<String, hir::TypeId>,
    next_type_id: u32,
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
    span: Span,
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
            record_types: vec![],
            record_ids: HashMap::new(),
            enum_types: vec![],
            enum_ids: HashMap::new(),
            builtin_enums: HashMap::new(),
            declared_types: HashMap::new(),
            next_type_id: 0,
        }
    }

    pub(super) fn program(mut self) -> Result<hir::Program, Vec<Diagnostic>> {
        self.collect_signatures()?;
        let start = self.current().span.start;
        let mut bindings = None;
        let mut body = None;
        let mut functions = Vec::new();
        while !matches!(self.current().kind, Kind::Eof) {
            if self.eat_keyword("registro") {
                self.skip_record_definition()?;
                continue;
            }
            if self.eat_keyword("enum") {
                self.skip_enum_definition()?;
                continue;
            }
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
        let (bindings, body) = match (bindings, body) {
            (Some(bindings), Some(body)) => (bindings, body),
            _ if self.has_mvu_entry_triple() => (Vec::new(), Vec::new()),
            _ => {
                return Err(vec![Diagnostic::error(
                    "pors.entry-function",
                    "O programa precisa declarar uma única função inicio()",
                    self.current().span,
                )]);
            }
        };
        let end = self.previous_span().end;
        Ok(hir::Program {
            record_types: std::mem::take(&mut self.record_types),
            enum_types: std::mem::take(&mut self.enum_types),
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
            if self.eat_keyword("registro") {
                self.collect_record_definition()?;
                continue;
            }
            if self.eat_keyword("enum") {
                self.collect_enum_definition()?;
                continue;
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
            self.skip_function_body()?;
        }
        if !self.signatures.contains_key("inicio") {
            if self.has_mvu_entry_triple() {
                self.validate_mvu_entry_signatures()?;
            } else {
                return self.fail(
                    "pors.entry-function",
                    "O programa precisa declarar uma função inicio()",
                    self.current().span,
                );
            }
        }
        self.cursor = 0;
        Ok(())
    }

    fn has_mvu_entry_triple(&self) -> bool {
        self.signatures.contains_key("iniciar")
            && self.signatures.contains_key("atualizar")
            && self.signatures.contains_key("visualizar")
    }

    fn validate_mvu_entry_signatures(&self) -> Result<(), Vec<Diagnostic>> {
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
                span: header_span,
            },
        ))
    }

    fn intern_builtin_enum(&mut self, builtin: hir::BuiltinEnum, span: Span) -> hir::Type {
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

    fn builtin_enum_of(&self, ty: hir::Type) -> Option<(hir::TypeId, hir::BuiltinEnum)> {
        let hir::Type::Enum(id) = ty else {
            return None;
        };
        let definition = self.enum_types.iter().find(|item| item.id == id)?;
        definition.builtin.map(|builtin| (id, builtin))
    }

    fn type_name(&mut self) -> Result<hir::Type, Vec<Diagnostic>> {
        let (name, span) = self.identifier()?;
        if name == "vetor" {
            self.expect(Kind::Less, "Esperado `<` após `vetor`")?;
            let element = self.type_name()?;
            let close = self.expect(Kind::Greater, "Esperado `>` após o tipo do vetor")?;
            return hir::Type::vector_of(element).ok_or_else(|| {
                vec![Diagnostic::error(
                    "pors.vector-element-type",
                    "O tipo de elemento não pode ser um vetor",
                    span.join(close.span),
                )]
            });
        }
        if name == "Opcao" {
            self.expect(Kind::Less, "Esperado `<` após `Opcao`")?;
            let inner = self.type_name()?;
            let close = self.expect(Kind::Greater, "Esperado `>` após o tipo de `Opcao`")?;
            return Ok(
                self.intern_builtin_enum(hir::BuiltinEnum::Option(inner), span.join(close.span))
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
            let close = self.expect(Kind::Greater, "Esperado `>` após os tipos de `Resultado`")?;
            return Ok(self
                .intern_builtin_enum(hir::BuiltinEnum::Result { ok, err }, span.join(close.span)));
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

    fn collect_record_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
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

    fn skip_record_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
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

    fn collect_enum_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
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

    fn skip_enum_definition(&mut self) -> Result<(), Vec<Diagnostic>> {
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
                .is_some_and(|token| matches!(token.kind, Kind::Dot))
            {
                if keyword == "Util" && self.lookup("Util").is_none() {
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
        let ty = self.type_name()?;
        self.expect(
            Kind::Assign,
            "Toda variável deve receber um valor inicial (`=`)",
        )?;
        let initializer = self.expression_with_expected(0, Some(ty))?;
        if initializer.ty != ty {
            return self.fail(
                "pors.type-mismatch",
                format!(
                    "O valor inicial de `{name}` não corresponde ao tipo `{}`",
                    type_label(ty)
                ),
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

    fn field_assignment(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let (name, name_span) = self.identifier()?;
        let symbol = self.lookup(&name).ok_or_else(|| {
            vec![Diagnostic::error(
                "pors.unknown-name",
                format!("Nome não declarado: {name}"),
                name_span,
            )]
        })?;
        let hir::Type::Record(type_id) = symbol.ty else {
            return self.fail(
                "pors.field-non-record",
                "Atribuição de campo exige um registro",
                name_span,
            );
        };
        self.expect(Kind::Dot, "Esperado `.` após o registro")?;
        let (field_name, field_span) = self.identifier()?;
        let definition = self
            .record_types
            .iter()
            .find(|definition| definition.id == type_id)
            .expect("registro possui definição");
        let Some((field_index, field_ty)) = definition
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
        let expected = (op == hir::AssignOp::Set).then_some(field_ty);
        let value = self.expression_with_expected(0, expected)?;
        if value.ty != field_ty {
            return self.fail(
                "pors.type-mismatch",
                format!("Atribuição incompatível para `{name}.{field_name}`"),
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
                field_index,
                op,
                value,
            },
            span: to_hir_span(name_span.join(end.span)),
        })
    }

    fn index_assignment(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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

    fn vector_push_statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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

    fn vector_pop_statement(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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

    fn choose(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
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
        while self.is_keyword("caso") {
            let arm_start = self.current().span;
            self.advance();
            let (enum_name, enum_span) = self.identifier()?;
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
            let (body, arm_end) = self.block(true)?;
            self.scopes.pop();
            arms.push(hir::ChooseArm {
                variant_index,
                payload_bindings,
                body,
                span: to_hir_span(arm_start.join(arm_end)),
            });
        }
        let close = self.expect(Kind::RBrace, "Esperado `}` após casos de escolha")?;
        if let Some(missing) =
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
        self.expression_with_expected(min_precedence, None)
    }

    fn expression_with_expected(
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
                        "pors.field-non-record",
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
                        "pors.unknown-field",
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
                                    "pors.try-return-mismatch",
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
                                    "pors.try-return-mismatch",
                                    "O operador `?` em `Resultado` exige que a função retorne `Resultado<..., E>` com o mesmo tipo de erro",
                                    question_span,
                                );
                            }
                        }
                    }
                    None => {
                        return self.fail(
                            "pors.try-invalid-type",
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

    fn prefix(&mut self, expected: Option<hir::Type>) -> Result<hir::Expr, Vec<Diagnostic>> {
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
                if name == "UI"
                    && matches!(self.current().kind, Kind::Dot)
                    && self.lookup("UI").is_none()
                {
                    return self.ui_host_call(span);
                }
                if matches!(
                    name.as_str(),
                    "Matematica" | "Texto" | "Tipos" | "Util" | "Calendario" | "Arquivos"
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
                            "pors.unknown-type",
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
                            "pors.unknown-variant",
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
                            "pors.enum-payload-arity",
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
                            "pors.enum-payload-type",
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
                                "pors.unknown-field",
                                format!("Campo `{field_name}` não existe em `{name}`"),
                                field_span,
                            );
                        };
                        if values[index].is_some() {
                            return self.fail(
                                "pors.field-redeclared",
                                format!("Campo `{field_name}` foi informado mais de uma vez"),
                                field_span,
                            );
                        }
                        self.expect(Kind::Colon, "Esperado `:` após o nome do campo")?;
                        let value = self.expression_with_expected(0, Some(field.ty))?;
                        if value.ty != field.ty {
                            return self.fail(
                                "pors.field-type-mismatch",
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
                            "pors.missing-field",
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
                                "pors.length-type",
                                "`tamanho` espera um vetor",
                                syntax_span(argument.span),
                            );
                        };
                        let hir::ExprKind::Binding(binding) = argument.kind else {
                            return self.fail(
                                "pors.length-binding",
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
                                "pors.unknown-name",
                                format!("Nome não declarado: {target_name}"),
                                target_span,
                            )]
                        })?;
                        let Some(element_type) = symbol.ty.vector_element() else {
                            return self.fail(
                                "pors.vector-operation-type",
                                format!(
                                    "`remover` espera um vetor, mas `{target_name}` não é um vetor"
                                ),
                                target_span,
                            );
                        };
                        if symbol.mutability != hir::Mutability::Mutable {
                            return self.fail(
                                "pors.immutable-assignment",
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
                                "pors.unknown-name",
                                format!("Nome não declarado: {target_name}"),
                                target_span,
                            )]
                        })?;
                        let Some(element_type) = symbol.ty.vector_element() else {
                            return self.fail(
                                "pors.vector-operation-type",
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
                                "pors.index-type",
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
                            "pors.unknown-function",
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
                if matches!(self.current().kind, Kind::LBracket) {
                    let symbol = self.lookup(&name).ok_or_else(|| {
                        vec![Diagnostic::error(
                            "pors.unknown-name",
                            format!("Nome não declarado: {name}"),
                            span,
                        )]
                    })?;
                    let Some(element_type) = symbol.ty.vector_element() else {
                        return self.fail(
                            "pors.index-non-vector",
                            format!("`{name}` não é um vetor"),
                            span,
                        );
                    };
                    self.advance();
                    let index = self.expression(0)?;
                    if index.ty != hir::Type::Integer {
                        return self.fail(
                            "pors.index-type",
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

    fn ui_host_call(&mut self, start: Span) -> Result<hir::Expr, Vec<Diagnostic>> {
        self.advance(); // '.'
        let (component, component_span) = self.identifier()?;
        self.expect(Kind::LParen, "Esperado `(` após componente de UI")?;
        let mut arguments = Vec::new();
        if !matches!(self.current().kind, Kind::RParen) {
            loop {
                arguments.push(self.expression(0)?);
                if !self.eat(&Kind::Comma) {
                    break;
                }
            }
        }
        let close = self.expect(Kind::RParen, "Esperado `)` após argumentos de UI")?;
        let whole_span = start.join(close.span);

        match component.as_str() {
            "texto" => {
                if arguments.is_empty() {
                    return self.fail(
                        "pors.ui-call-arity",
                        "`UI.texto` espera pelo menos 1 argumento",
                        whole_span,
                    );
                }
                for argument in &arguments {
                    if !matches!(
                        argument.ty,
                        hir::Type::String
                            | hir::Type::Integer
                            | hir::Type::Real
                            | hir::Type::Boolean
                            | hir::Type::Character
                    ) {
                        return self.fail(
                            "pors.ui-call-type-mismatch",
                            format!(
                                "`UI.texto` espera valores escalares (cadeia, inteiro, real, logico ou caracter), recebeu `{}`",
                                type_label(argument.ty)
                            ),
                            syntax_span(argument.span),
                        );
                    }
                }
            }
            "botao" => {
                if arguments.len() != 2 {
                    return self.fail(
                        "pors.ui-call-arity",
                        format!(
                            "`UI.botao` espera 2 argumentos (id, rotulo), recebeu {}",
                            arguments.len()
                        ),
                        whole_span,
                    );
                }
                for (index, label) in [(0, "id"), (1, "rotulo")] {
                    if arguments[index].ty != hir::Type::String {
                        return self.fail(
                            "pors.ui-call-type-mismatch",
                            format!("O argumento `{label}` de `UI.botao` deve ter tipo `cadeia`"),
                            syntax_span(arguments[index].span),
                        );
                    }
                }
            }
            "entrada" => {
                if arguments.len() != 3 {
                    return self.fail(
                        "pors.ui-call-arity",
                        format!(
                            "`UI.entrada` espera 3 argumentos (id, valor, dica), recebeu {}",
                            arguments.len()
                        ),
                        whole_span,
                    );
                }
                for (index, label) in [(0, "id"), (1, "valor"), (2, "dica")] {
                    if arguments[index].ty != hir::Type::String {
                        return self.fail(
                            "pors.ui-call-type-mismatch",
                            format!("O argumento `{label}` de `UI.entrada` deve ter tipo `cadeia`"),
                            syntax_span(arguments[index].span),
                        );
                    }
                }
            }
            "coluna" | "linha" | "lista" => {
                if arguments.len() != 1 {
                    return self.fail(
                        "pors.ui-call-arity",
                        format!(
                            "`UI.{component}` espera 1 argumento (vetor<inteiro>), recebeu {}",
                            arguments.len()
                        ),
                        whole_span,
                    );
                }
                if arguments[0].ty != hir::Type::VectorInteger {
                    return self.fail(
                        "pors.ui-call-type-mismatch",
                        format!(
                            "`UI.{component}` espera `vetor<inteiro>`, recebeu `{}`",
                            type_label(arguments[0].ty)
                        ),
                        syntax_span(arguments[0].span),
                    );
                }
            }
            _ => {
                return self.fail(
                    "pors.unknown-ui-component",
                    format!("Componente visual desconhecido: UI.{component}"),
                    component_span,
                );
            }
        }

        Ok(hir::Expr {
            kind: hir::ExprKind::HostCall {
                module: "UI".into(),
                name: component,
                arguments,
            },
            ty: hir::Type::Integer,
            span: to_hir_span(whole_span),
        })
    }

    fn stdlib_statement_call(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let (module, start) = self.identifier()?;
        let expr = self.stdlib_member_or_call(&module, start)?;
        let end = self.expect(Kind::Semicolon, "Esperado `;` após chamada de biblioteca")?;
        let hir::ExprKind::HostCall {
            module,
            name,
            arguments,
        } = expr.kind
        else {
            return self.fail(
                "pors.expected-statement",
                "Esperada chamada de função",
                syntax_span(expr.span),
            );
        };
        Ok(hir::Stmt {
            kind: hir::StmtKind::HostCall {
                module,
                name,
                arguments,
            },
            span: to_hir_span(start.join(end.span)),
        })
    }

    fn stdlib_member_or_call(
        &mut self,
        module: &str,
        start: Span,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        enum StdlibReturn {
            Scalar(hir::Type),
            Option(hir::Type),
            ResultString(hir::Type),
        }

        self.advance(); // '.'
        let (member, member_span) = self.identifier()?;
        if module == "Matematica"
            && !matches!(self.current().kind, Kind::LParen)
            && let Some(value) = match member.as_str() {
                "PI" => Some(std::f64::consts::PI),
                "E" => Some(std::f64::consts::E),
                _ => None,
            }
        {
            return Ok(hir::Expr {
                kind: hir::ExprKind::Real(value),
                ty: hir::Type::Real,
                span: to_hir_span(start.join(member_span)),
            });
        }
        if module == "Calendario"
            && !matches!(self.current().kind, Kind::LParen)
            && let Some(value) = calendario_constant(&member)
        {
            return Ok(hir::Expr {
                kind: hir::ExprKind::Integer(value),
                ty: hir::Type::Integer,
                span: to_hir_span(start.join(member_span)),
            });
        }
        self.expect(
            Kind::LParen,
            &format!("Esperado `(` após `{module}.{member}`"),
        )?;
        let mut arguments = Vec::new();
        if !matches!(self.current().kind, Kind::RParen) {
            loop {
                arguments.push(self.expression(0)?);
                if !self.eat(&Kind::Comma) {
                    break;
                }
            }
        }
        let close = self.expect(
            Kind::RParen,
            &format!("Esperado `)` após argumentos de `{module}.{member}`"),
        )?;
        let whole_span = start.join(close.span);

        use StdlibReturn::{Option as RetOpt, ResultString as RetRes, Scalar as Ret};
        use hir::Type::{Boolean as B, Character as C, Integer as I, Real as R, String as S};

        let signature: Option<(&[hir::Type], StdlibReturn)> = match (module, member.as_str()) {
            ("Matematica", "potencia") => Some((&[R, R], Ret(R))),
            ("Matematica", "raiz") => Some((&[R, R], Ret(R))),
            ("Matematica", "raiz_quadrada") => Some((&[R], Ret(R))),
            ("Matematica", "arredondar") => Some((&[R, I], Ret(R))),
            ("Matematica", "piso") => Some((&[R], Ret(R))),
            ("Matematica", "teto") => Some((&[R], Ret(R))),
            ("Matematica", "logaritmo") => Some((&[R, R], Ret(R))),
            ("Matematica", "seno") => Some((&[R], Ret(R))),
            ("Matematica", "cosseno") => Some((&[R], Ret(R))),
            ("Matematica", "tangente") => Some((&[R], Ret(R))),
            ("Matematica", "valor_absoluto") => Some((&[R], Ret(R))),
            ("Matematica", "valor_absoluto_inteiro") => Some((&[I], Ret(I))),
            ("Matematica", "maior_numero") => Some((&[R, R], Ret(R))),
            ("Matematica", "menor_numero") => Some((&[R, R], Ret(R))),
            ("Matematica", "maior_inteiro") => Some((&[I, I], Ret(I))),
            ("Matematica", "menor_inteiro") => Some((&[I, I], Ret(I))),

            ("Texto", "numero_caracteres") => Some((&[S], Ret(I))),
            ("Texto", "caixa_alta") => Some((&[S], Ret(S))),
            ("Texto", "caixa_baixa") => Some((&[S], Ret(S))),
            ("Texto", "aparar") => Some((&[S], Ret(S))),
            ("Texto", "contem") => Some((&[S, S], Ret(B))),
            ("Texto", "comeca_com") => Some((&[S, S], Ret(B))),
            ("Texto", "termina_com") => Some((&[S, S], Ret(B))),
            ("Texto", "substituir") => Some((&[S, S, S], Ret(S))),
            ("Texto", "preencher_a_esquerda") => Some((&[C, I, S], Ret(S))),
            ("Texto", "obter_caracter") => Some((&[S, I], RetOpt(C))),
            ("Texto", "posicao_texto") => Some((&[S, S, I], RetOpt(I))),
            ("Texto", "extrair_subtexto") => Some((&[S, I, I], RetRes(S))),
            ("Texto", "dividir") => Some((&[S, S], Ret(hir::Type::VectorString))),

            ("Tipos", "para_inteiro") => Some((&[S], RetRes(I))),
            ("Tipos", "para_inteiro_base") => Some((&[S, I], RetRes(I))),
            ("Tipos", "para_real") => Some((&[S], RetRes(R))),
            ("Tipos", "para_logico") => Some((&[S], RetRes(B))),
            ("Tipos", "para_caracter") => Some((&[S], RetRes(C))),
            ("Tipos", "inteiro_para_real") => Some((&[I], Ret(R))),
            ("Tipos", "real_para_inteiro") => Some((&[R], Ret(I))),
            ("Tipos", "inteiro_para_cadeia") => Some((&[I], Ret(S))),
            ("Tipos", "inteiro_para_cadeia_base") => Some((&[I, I], RetRes(S))),
            ("Tipos", "real_para_cadeia") => Some((&[R], Ret(S))),
            ("Tipos", "logico_para_cadeia") => Some((&[B], Ret(S))),
            ("Tipos", "caracter_para_cadeia") => Some((&[C], Ret(S))),
            ("Tipos", "logico_para_inteiro") => Some((&[B], Ret(I))),
            ("Tipos", "caracter_para_inteiro") => Some((&[C], RetRes(I))),
            ("Tipos", "inteiro_para_caracter") => Some((&[I], RetRes(C))),
            ("Tipos", "cadeia_e_inteiro") => Some((&[S], Ret(B))),
            ("Tipos", "cadeia_e_real") => Some((&[S], Ret(B))),
            ("Tipos", "cadeia_e_logico") => Some((&[S], Ret(B))),
            ("Tipos", "cadeia_e_caracter") => Some((&[S], Ret(B))),

            ("Util", "sorteia") => Some((&[I, I], RetRes(I))),
            ("Util", "definir_semente") => Some((&[I], Ret(hir::Type::Unit))),
            ("Util", "tempo_decorrido") => Some((&[], Ret(I))),
            ("Util", "aguarde") => Some((&[I], Ret(hir::Type::Unit))),

            ("Calendario", "ano_atual") => Some((&[], Ret(I))),
            ("Calendario", "mes_atual") => Some((&[], Ret(I))),
            ("Calendario", "dia_mes_atual") => Some((&[], Ret(I))),
            ("Calendario", "dia_semana_atual") => Some((&[], Ret(I))),
            ("Calendario", "hora_atual") => Some((&[B], Ret(I))),
            ("Calendario", "minuto_atual") => Some((&[], Ret(I))),
            ("Calendario", "segundo_atual") => Some((&[], Ret(I))),
            ("Calendario", "milisegundo_atual") => Some((&[], Ret(I))),
            ("Calendario", "dia_semana_completo") => Some((&[I, B, B], RetRes(S))),
            ("Calendario", "dia_semana_curto") => Some((&[I, B, B], RetRes(S))),
            ("Calendario", "dia_semana_abreviado") => Some((&[I, B, B], RetRes(S))),
            ("Calendario", "nome_mes") => Some((&[I], RetRes(S))),
            ("Calendario", "eh_bissexto") => Some((&[I], Ret(B))),
            ("Calendario", "dias_no_mes") => Some((&[I, I], RetRes(I))),
            ("Calendario", "data_iso_atual") => Some((&[], Ret(S))),

            ("Arquivos", "ler_texto") => Some((&[S], RetRes(S))),
            ("Arquivos", "ler_linhas") => Some((&[S], RetRes(hir::Type::VectorString))),
            ("Arquivos", "escrever_texto") => Some((&[S, S], RetRes(B))),
            ("Arquivos", "acrescentar_texto") => Some((&[S, S], RetRes(B))),
            ("Arquivos", "existe") => Some((&[S], RetRes(B))),
            ("Arquivos", "remover") => Some((&[S], RetRes(B))),
            ("Arquivos", "listar") => Some((&[S], RetRes(hir::Type::VectorString))),
            _ => None,
        };

        let Some((param_types, return_spec)) = signature else {
            return self.fail(
                "pors.unknown-stdlib-function",
                format!("Função desconhecida na biblioteca padrão: {module}.{member}"),
                member_span,
            );
        };
        if arguments.len() != param_types.len() {
            return self.fail(
                "pors.stdlib-call-arity",
                format!(
                    "`{module}.{member}` espera {} argumento(s), recebeu {}",
                    param_types.len(),
                    arguments.len()
                ),
                whole_span,
            );
        }
        for (index, (argument, expected_ty)) in arguments.iter().zip(param_types).enumerate() {
            if argument.ty != *expected_ty {
                return self.fail(
                    "pors.stdlib-call-type-mismatch",
                    format!(
                        "O argumento {} de `{module}.{member}` deve ter tipo `{}`, recebeu `{}`",
                        index + 1,
                        type_label(*expected_ty),
                        type_label(argument.ty)
                    ),
                    syntax_span(argument.span),
                );
            }
        }
        let ty = match return_spec {
            Ret(scalar) => scalar,
            RetOpt(inner) => self.intern_builtin_enum(hir::BuiltinEnum::Option(inner), whole_span),
            RetRes(ok) => self.intern_builtin_enum(
                hir::BuiltinEnum::Result {
                    ok,
                    err: hir::Type::String,
                },
                whole_span,
            ),
        };
        Ok(hir::Expr {
            kind: hir::ExprKind::HostCall {
                module: module.into(),
                name: member,
                arguments,
            },
            ty,
            span: to_hir_span(whole_span),
        })
    }

    fn parse_builtin_option_constructor(
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
                        "pors.enum-payload-arity",
                        "A variante `Algum` exige 1 valor(es)",
                        variant_span,
                    );
                }
                if payload[0].ty == hir::Type::Unit {
                    return self.fail(
                        "pors.void-value",
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
                        "pors.enum-payload-arity",
                        "A variante `Nenhum` exige 0 valor(es)",
                        variant_span,
                    );
                }
                let Some((type_id, hir::BuiltinEnum::Option(_))) =
                    expected.and_then(|ty| self.builtin_enum_of(ty))
                else {
                    return self.fail(
                        "pors.option-type-inference",
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
                "pors.unknown-variant",
                format!("Variante `{variant_name}` não existe em `Opcao`"),
                variant_span,
            ),
        }
    }

    fn parse_builtin_result_constructor(
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
                    "pors.unknown-variant",
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
                "pors.enum-payload-arity",
                format!("A variante `{variant_name}` exige 1 valor(es)"),
                variant_span,
            );
        }
        if payload[0].ty == hir::Type::Unit {
            return self.fail(
                "pors.void-value",
                "Uma função sem retorno não pode ser usada como valor",
                syntax_span(payload[0].span),
            );
        }
        let Some((type_id, expected_ok, expected_err)) = expected_result else {
            return self.fail(
                "pors.result-type-inference",
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
                "pors.enum-payload-type",
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

    fn vector_literal(
        &mut self,
        start: Span,
        expected: Option<hir::Type>,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        if matches!(self.current().kind, Kind::RBracket) {
            let close = self.advance();
            let whole = start.join(close.span);
            let Some(ty) = expected.filter(|ty| ty.is_vector()) else {
                return self.fail(
                    "pors.empty-vector-type-inference",
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
                "pors.vector-element-type",
                "Vetores aninhados e valores sem retorno não podem ser elementos",
                syntax_span(elements[0].span),
            );
        }
        if let Some(incompatible) = elements.iter().find(|element| element.ty != element_type) {
            return self.fail(
                "pors.vector-element-type",
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

fn calendario_constant(name: &str) -> Option<i32> {
    const DIAS: [&str; 7] = [
        "DIA_DOMINGO",
        "DIA_SEGUNDA_FEIRA",
        "DIA_TERCA_FEIRA",
        "DIA_QUARTA_FEIRA",
        "DIA_QUINTA_FEIRA",
        "DIA_SEXTA_FEIRA",
        "DIA_SABADO",
    ];
    const MESES: [&str; 12] = [
        "MES_JANEIRO",
        "MES_FEVEREIRO",
        "MES_MARCO",
        "MES_ABRIL",
        "MES_MAIO",
        "MES_JUNHO",
        "MES_JULHO",
        "MES_AGOSTO",
        "MES_SETEMBRO",
        "MES_OUTUBRO",
        "MES_NOVEMBRO",
        "MES_DEZEMBRO",
    ];
    DIAS.iter()
        .position(|day| *day == name)
        .or_else(|| MESES.iter().position(|month| *month == name))
        .map(|index| index as i32 + 1)
}

fn to_hir_span(value: Span) -> hir::Span {
    hir::Span::new(value.start, value.end)
}
fn scalar_type(name: &str) -> Option<hir::Type> {
    Some(match name {
        "inteiro" => hir::Type::Integer,
        "real" => hir::Type::Real,
        "logico" => hir::Type::Boolean,
        "cadeia" => hir::Type::String,
        "caracter" => hir::Type::Character,
        _ => return None,
    })
}
fn type_label(ty: hir::Type) -> &'static str {
    match ty {
        hir::Type::Unit => "vazio",
        hir::Type::Integer => "inteiro",
        hir::Type::Real => "real",
        hir::Type::Boolean => "logico",
        hir::Type::String => "cadeia",
        hir::Type::Character => "caracter",
        hir::Type::VectorInteger => "vetor<inteiro>",
        hir::Type::VectorReal => "vetor<real>",
        hir::Type::VectorBoolean => "vetor<logico>",
        hir::Type::VectorString => "vetor<cadeia>",
        hir::Type::VectorCharacter => "vetor<caracter>",
        hir::Type::Record(_) => "registro",
        hir::Type::VectorRecord(_) => "vetor<registro>",
        hir::Type::Enum(_) => "enum",
        hir::Type::VectorEnum(_) => "vetor<enum>",
    }
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
            | "vetor"
            | "tamanho"
            | "adicionar"
            | "remover"
            | "obter"
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
            | "registro"
            | "enum"
            | "caso"
            | "Opcao"
            | "Resultado"
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
        hir::StmtKind::Choose { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| guarantees_return(&arm.body))
        }
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
    if left.is_vector() || right.is_vector() {
        return None;
    }
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
        hir::BinaryOp::Equal | hir::BinaryOp::NotEqual
            if left == right && !matches!(left, Record(_) | Enum(_)) =>
        {
            Boolean
        }
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
