mod constants;
mod expr;
mod items;
mod stdlib;
mod stmt;

use std::collections::HashMap;

use ferro_hir as hir;
use ferro_hir::{Diagnostic, Span};

use crate::lexer::{Kind, Token};
use constants::*;

pub(super) struct Parser {
    tokens: Vec<Token>,
    source: String,
    cursor: usize,
    invalid_items: Vec<(usize, usize)>,
    diagnostics: Vec<Diagnostic>,
    function_has_syntax_error: bool,
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
    external_records: Vec<crate::lower::ExternalRecord>,
    require_entry: bool,
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
    pub(super) fn new(tokens: Vec<Token>, source: String) -> Self {
        Self {
            tokens,
            source,
            cursor: 0,
            invalid_items: Vec::new(),
            diagnostics: Vec::new(),
            function_has_syntax_error: false,
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
            external_records: Vec::new(),
            require_entry: true,
        }
    }

    pub(super) fn with_options(mut self, options: &crate::lower::LowerOptions) -> Self {
        self.external_records = options.records.clone();
        self.require_entry = options.require_entry;
        self
    }

    pub(super) fn program(mut self) -> Result<hir::Program, Vec<Diagnostic>> {
        let mut diagnostics = self.collect_signatures();
        let start = self.current().span.start;
        let mut bindings = None;
        let mut body = None;
        let mut functions = Vec::new();
        while !matches!(self.current().kind, Kind::Eof) {
            if let Some((_, end)) = self
                .invalid_items
                .iter()
                .find(|(item_start, _)| *item_start == self.cursor)
            {
                self.cursor = *end;
                continue;
            }
            let item_start = self.cursor;
            if self.eat_keyword("registro") {
                if let Err(errors) = self.skip_record_definition() {
                    append_unique(&mut diagnostics, errors);
                    self.synchronize_top_level(item_start);
                }
                continue;
            }
            if self.eat_keyword("enum") {
                if let Err(errors) = self.skip_enum_definition() {
                    append_unique(&mut diagnostics, errors);
                    self.synchronize_top_level(item_start);
                }
                continue;
            }
            let (name, header_span, _) = match self.function_header() {
                Ok(header) => header,
                Err(errors) => {
                    append_unique(&mut diagnostics, errors);
                    self.synchronize_top_level(item_start);
                    continue;
                }
            };
            let Some(signature) = self.signatures.get(&name).cloned() else {
                append_unique(
                    &mut diagnostics,
                    vec![Diagnostic::error(
                        "pors.invalid-function",
                        format!("A função `{name}` não pôde ser analisada"),
                        header_span,
                    )],
                );
                self.synchronize_top_level(item_start);
                continue;
            };
            if let Err(errors) = self.begin_function(&signature) {
                append_unique(&mut diagnostics, errors);
                self.reset_function_state();
                self.synchronize_top_level(item_start);
                continue;
            }
            let (function_body, tail_expression, body_span) = match self.block(true, true) {
                Ok(block) => block,
                Err(errors) => {
                    append_unique(&mut diagnostics, errors);
                    self.reset_function_state();
                    self.synchronize_top_level(item_start);
                    continue;
                }
            };
            if let Some(tail) = &tail_expression {
                if signature.return_type.is_none() {
                    append_unique(
                        &mut diagnostics,
                        vec![Diagnostic::error(
                            "pors.implicit-return-without-type",
                            "Função sem tipo de retorno não pode terminar em expressão",
                            syntax_span(tail.span),
                        )],
                    );
                    self.reset_function_state();
                    continue;
                }
                if Some(tail.ty) != signature.return_type {
                    append_unique(
                        &mut diagnostics,
                        vec![Diagnostic::error(
                            "pors.return-type-mismatch",
                            "A expressão final não corresponde ao tipo de retorno declarado",
                            syntax_span(tail.span),
                        )],
                    );
                    self.reset_function_state();
                    continue;
                }
            } else if !self.function_has_syntax_error
                && signature.return_type.is_some()
                && !guarantees_return(&function_body)
            {
                append_unique(
                    &mut diagnostics,
                    vec![Diagnostic::error(
                        "pors.missing-return",
                        format!("A função `{name}` precisa retornar um valor em todos os caminhos"),
                        body_span,
                    )],
                );
                self.reset_function_state();
                continue;
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
                    tail_expression,
                    span: to_hir_span(function_span),
                });
            }
            self.reset_function_state();
        }
        append_unique(&mut diagnostics, std::mem::take(&mut self.diagnostics));
        if !diagnostics.is_empty() {
            return Err(diagnostics);
        }
        let (bindings, body) = match (bindings, body) {
            (Some(bindings), Some(body)) => (bindings, body),
            _ if self.has_mvu_entry_triple() || !self.require_entry => (Vec::new(), Vec::new()),
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
