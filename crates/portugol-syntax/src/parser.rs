//! Parser recursivo descendente para o Portugol 2.0.
//!
//! Escrito à mão (em vez de gerado a partir do ANTLR) por três motivos:
//! 1. diagnósticos e recuperação de erro melhores, essenciais em uma IDE;
//! 2. spans precisos em todos os nós, para highlighting e navegação;
//! 3. evolução incremental (o parser da IDE precisa ser rápido e tolerante).
//!
//! A gramática de referência é `Portugol.g4` do projeto original. Pequenas
//! extensões deliberadas: o parser aceita arquivos sem o bloco `programa { }`
//! (os exemplos de ajuda do Portugol Studio são trechos soltos).

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub struct Parser<'a> {
    source: &'a str,
    tokens: &'a [Token],
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: &'a [Token], diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            source,
            tokens,
            pos: 0,
            diagnostics,
        }
    }

    pub fn parse_file(mut self) -> (Program, Vec<Diagnostic>) {
        let mut includes = Vec::new();
        let mut items = Vec::new();
        let mut has_program_block = false;
        let mut guard = 0usize;

        while !self.at(TokenKind::Eof) {
            guard += 1;
            if guard > 1_000_000 {
                self.diagnostics.push(Diagnostic::error(
                    "syntax.internal-loop",
                    "parser interrompido após muitas iterações",
                    self.peek_span(),
                ));
                break;
            }

            if self.at(TokenKind::Programa) {
                has_program_block = true;
                self.bump();
                self.parse_program_body(&mut includes, &mut items);
            } else if !self.parse_item(&mut includes, &mut items) {
                self.recover_top_level();
            }
        }

        let program = Program {
            includes,
            items,
            span: Span::new(0, self.source.len() as u32),
            has_program_block,
        };
        (program, self.diagnostics)
    }

    // ---------------------------------------------------------------------
    // Estrutura do arquivo
    // ---------------------------------------------------------------------

    fn parse_program_body(&mut self, includes: &mut Vec<Include>, items: &mut Vec<Item>) {
        if self.expect(TokenKind::LBrace).is_none() {
            return;
        }

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            if !self.parse_item(includes, items) {
                self.recover_top_level();
            }
        }

        self.expect(TokenKind::RBrace);
    }

    /// Retorna `false` quando o token atual não inicia um item e nada foi
    /// consumido (o chamador decide como recuperar).
    fn parse_item(&mut self, includes: &mut Vec<Include>, items: &mut Vec<Item>) -> bool {
        match self.peek() {
            TokenKind::Inclua => {
                if let Some(include) = self.parse_include() {
                    includes.push(include);
                }
                true
            }
            TokenKind::Funcao => {
                if let Some(function) = self.parse_function() {
                    items.push(Item::Function(function));
                }
                true
            }
            TokenKind::Const => {
                if let Some(decl) = self.parse_declarations() {
                    items.push(Item::Declarations(decl));
                }
                true
            }
            kind if kind.is_type() => {
                if let Some(decl) = self.parse_declarations() {
                    items.push(Item::Declarations(decl));
                }
                true
            }
            kind if self.can_start_expr(kind)
                || matches!(
                    kind,
                    TokenKind::Se
                        | TokenKind::Enquanto
                        | TokenKind::Faca
                        | TokenKind::Para
                        | TokenKind::Escolha
                        | TokenKind::Retorne
                        | TokenKind::Pare
                ) =>
            {
                match self.parse_stmt() {
                    Some(stmt) => {
                        items.push(Item::Stmt(stmt));
                        true
                    }
                    None => false,
                }
            }
            _ => false,
        }
    }

    fn parse_include(&mut self) -> Option<Include> {
        let start = self.expect(TokenKind::Inclua)?;
        self.expect(TokenKind::Biblioteca);
        let library = self.expect_ident("nome da biblioteca")?;

        let alias = if self.eat(TokenKind::Arrow) {
            self.expect_ident("apelido da biblioteca")
        } else {
            None
        };

        Some(Include {
            library,
            alias,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_function(&mut self) -> Option<Function> {
        let start = self.bump();
        let return_type = if self.peek().is_type() {
            self.parse_type()
        } else {
            None
        };
        let name = self.expect_ident("nome da função")?;
        let params = self.parse_params();
        let body = self.parse_block();

        Some(Function {
            return_type,
            name,
            params,
            body,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_params(&mut self) -> Vec<Param> {
        let mut params = Vec::new();
        self.expect(TokenKind::LParen);

        if !self.at(TokenKind::RParen) {
            while let Some(param) = self.parse_param() {
                params.push(param);
                if !self.eat(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(TokenKind::RParen);
        params
    }

    fn parse_param(&mut self) -> Option<Param> {
        let start = self.peek_span();
        let ty = self.parse_type()?;
        let by_ref = self.eat(TokenKind::Amp);
        let name = self.expect_ident("nome do parâmetro")?;

        let mut dims = 0;
        if self.eat(TokenKind::LBracket) {
            self.expect(TokenKind::RBracket);
            dims = 1;
            if self.eat(TokenKind::LBracket) {
                self.expect(TokenKind::RBracket);
                dims = 2;
            }
        }

        Some(Param {
            ty,
            by_ref,
            name,
            dims,
            span: start.join(self.last_span()),
        })
    }

    fn parse_declarations(&mut self) -> Option<Declarations> {
        let start = self.peek_span();
        let is_const = self.eat(TokenKind::Const);
        let ty = self.parse_type()?;
        let mut declarators = Vec::new();

        while let Some(declarator) = self.parse_declarator() {
            declarators.push(declarator);
            if !self.eat(TokenKind::Comma) {
                break;
            }
        }

        Some(Declarations {
            is_const,
            ty,
            declarators,
            span: start.join(self.last_span()),
        })
    }

    fn parse_declarator(&mut self) -> Option<Declarator> {
        let start = self.peek_span();
        let name = self.expect_ident("nome da variável")?;

        if self.eat(TokenKind::LBracket) {
            let first_size = if self.at(TokenKind::RBracket) {
                None
            } else {
                self.parse_expr().map(Box::new)
            };
            self.expect(TokenKind::RBracket);

            if self.eat(TokenKind::LBracket) {
                let second_size = if self.at(TokenKind::RBracket) {
                    None
                } else {
                    self.parse_expr().map(Box::new)
                };
                self.expect(TokenKind::RBracket);

                let init = if self.eat(TokenKind::Assign) {
                    self.parse_matrix_init()
                } else {
                    None
                };

                Some(Declarator::Matrix {
                    name,
                    rows: first_size,
                    cols: second_size,
                    init,
                    span: start.join(self.last_span()),
                })
            } else {
                let init = if self.eat(TokenKind::Assign) {
                    self.parse_array_init()
                } else {
                    None
                };

                Some(Declarator::Array {
                    name,
                    size: first_size,
                    init,
                    span: start.join(self.last_span()),
                })
            }
        } else {
            let init = if self.eat(TokenKind::Assign) {
                self.parse_expr()
            } else {
                None
            };

            Some(Declarator::Variable {
                name,
                init,
                span: start.join(self.last_span()),
            })
        }
    }

    fn parse_array_init(&mut self) -> Option<Vec<Expr>> {
        self.expect(TokenKind::LBrace)?;

        let mut values = Vec::new();
        if !self.at(TokenKind::RBrace) {
            loop {
                match self.parse_expr() {
                    Some(expr) => values.push(expr),
                    None => self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]),
                }
                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }

        self.expect(TokenKind::RBrace);
        Some(values)
    }

    fn parse_matrix_init(&mut self) -> Option<Vec<Vec<Expr>>> {
        self.expect(TokenKind::LBrace)?;

        let mut rows = Vec::new();
        if !self.at(TokenKind::RBrace) {
            loop {
                match self.parse_array_init() {
                    Some(row) => rows.push(row),
                    None => self.recover_until(&[TokenKind::Comma, TokenKind::RBrace]),
                }
                if self.eat(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }

        self.expect(TokenKind::RBrace);
        Some(rows)
    }

    // ---------------------------------------------------------------------
    // Comandos
    // ---------------------------------------------------------------------

    fn parse_block(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        if self.expect(TokenKind::LBrace).is_none() {
            return stmts;
        }

        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let before = self.pos;
            match self.parse_stmt() {
                Some(stmt) => stmts.push(stmt),
                None => self.recover_stmt(),
            }
            if self.pos == before {
                self.bump();
            }
        }

        self.expect(TokenKind::RBrace);
        stmts
    }

    /// `listaComandos`: um bloco entre chaves ou um único comando.
    fn parse_command_list(&mut self) -> Vec<Stmt> {
        if self.at(TokenKind::LBrace) {
            self.parse_block()
        } else {
            match self.parse_stmt() {
                Some(stmt) => vec![stmt],
                None => {
                    self.recover_stmt();
                    Vec::new()
                }
            }
        }
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            TokenKind::Const => self.parse_declarations().map(Stmt::Declarations),
            kind if kind.is_type() => self.parse_declarations().map(Stmt::Declarations),
            TokenKind::Se => self.parse_if(),
            TokenKind::Enquanto => self.parse_while(),
            TokenKind::Faca => self.parse_do_while(),
            TokenKind::Para => self.parse_for(),
            TokenKind::Escolha => self.parse_switch(),
            TokenKind::Retorne => self.parse_return(),
            TokenKind::Pare => {
                let token = self.bump();
                Some(Stmt::Break { span: token.span })
            }
            TokenKind::LBrace => {
                self.diagnostics.push(Diagnostic::error(
                    "syntax.unexpected-block",
                    "bloco de comandos inesperado neste ponto",
                    self.peek_span(),
                ));
                None
            }
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_if(&mut self) -> Option<Stmt> {
        let start = self.bump();
        self.expect(TokenKind::LParen);
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RParen);
        let then_branch = self.parse_command_list();

        let else_branch = if self.eat(TokenKind::Senao) {
            Some(self.parse_command_list())
        } else {
            None
        };

        Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_while(&mut self) -> Option<Stmt> {
        let start = self.bump();
        self.expect(TokenKind::LParen);
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RParen);
        let body = self.parse_command_list();

        Some(Stmt::While {
            condition,
            body,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_do_while(&mut self) -> Option<Stmt> {
        let start = self.bump();
        let body = self.parse_command_list();
        self.expect(TokenKind::Enquanto);
        self.expect(TokenKind::LParen);
        let condition = self.parse_expr()?;
        self.expect(TokenKind::RParen);

        Some(Stmt::DoWhile {
            body,
            condition,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_for(&mut self) -> Option<Stmt> {
        let start = self.bump();
        self.expect(TokenKind::LParen);

        let init = if self.at(TokenKind::Const) || self.peek().is_type() {
            self.parse_declarations().map(ForInit::Declarations)
        } else if self.at(TokenKind::Semi) {
            None
        } else {
            self.parse_expr_or_assign()
                .map(|(expr, assignment)| match assignment {
                    Some(assignment) => ForInit::Assignment(assignment),
                    None => ForInit::Expression(expr),
                })
        };

        self.expect(TokenKind::Semi);

        let condition = if self.at(TokenKind::Semi) {
            None
        } else {
            self.parse_expr()
        };
        self.expect(TokenKind::Semi);

        let step = if self.at(TokenKind::RParen) {
            None
        } else {
            self.parse_expr_or_assign()
                .map(|(expr, assignment)| match assignment {
                    Some(assignment) => ForStep::Assignment(assignment),
                    None => ForStep::Expression(expr),
                })
        };
        self.expect(TokenKind::RParen);
        let body = self.parse_command_list();

        Some(Stmt::For {
            init,
            condition,
            step,
            body,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_switch(&mut self) -> Option<Stmt> {
        let start = self.bump();
        self.expect(TokenKind::LParen);
        let value = self.parse_expr()?;
        self.expect(TokenKind::RParen);
        self.expect(TokenKind::LBrace);

        let mut cases = Vec::new();
        while self.at(TokenKind::Caso) {
            let case_start = self.bump();
            let case_value = if self.eat(TokenKind::Contrario) {
                None
            } else {
                Some(self.parse_expr()?)
            };
            self.expect(TokenKind::Colon);

            let body = if self.at(TokenKind::LBrace) {
                self.parse_block()
            } else {
                let mut stmts = Vec::new();
                while !self.at(TokenKind::Caso)
                    && !self.at(TokenKind::Contrario)
                    && !self.at(TokenKind::RBrace)
                    && !self.at(TokenKind::Eof)
                {
                    let before = self.pos;
                    match self.parse_stmt() {
                        Some(stmt) => stmts.push(stmt),
                        None => self.recover_stmt(),
                    }
                    if self.pos == before {
                        self.bump();
                    }
                }
                stmts
            };

            cases.push(Case {
                value: case_value,
                body,
                span: case_start.span.join(self.last_span()),
            });
        }

        self.expect(TokenKind::RBrace);

        Some(Stmt::Switch {
            value,
            cases,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_return(&mut self) -> Option<Stmt> {
        let start = self.bump();
        let value = if self.can_start_expr(self.peek()) {
            self.parse_expr()
        } else {
            None
        };

        Some(Stmt::Return {
            value,
            span: start.span.join(self.last_span()),
        })
    }

    fn parse_expr_or_assign_stmt(&mut self) -> Option<Stmt> {
        let (target, assignment) = self.parse_expr_or_assign()?;

        match assignment {
            Some(assignment) => Some(Stmt::Assign(assignment)),
            None => {
                let span = target.span();
                Some(Stmt::Expr { expr: target, span })
            }
        }
    }

    fn parse_expr_or_assign(&mut self) -> Option<(Expr, Option<Assignment>)> {
        let start = self.peek_span();
        let target = self.parse_expr()?;

        if let Some(op) = assign_op(self.peek()) {
            self.bump();
            let value = self.parse_expr()?;
            let assignment = Assignment {
                target: target.clone(),
                op,
                value,
                span: start.join(self.last_span()),
            };
            Some((target, Some(assignment)))
        } else {
            Some((target, None))
        }
    }

    // ---------------------------------------------------------------------
    // Expressões
    // ---------------------------------------------------------------------

    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_binary(1)
    }

    fn parse_binary(&mut self, min_precedence: u8) -> Option<Expr> {
        let mut left = self.parse_unary()?;

        while let Some((op, precedence)) = binary_op(self.peek()) {
            if precedence < min_precedence {
                break;
            }

            self.bump();
            let right = self.parse_binary(precedence + 1)?;
            let span = left.span().join(right.span());
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }

        Some(left)
    }

    fn parse_unary(&mut self) -> Option<Expr> {
        match self.peek() {
            TokenKind::Minus | TokenKind::Plus | TokenKind::Nao | TokenKind::Tilde => {
                let token = self.bump();
                let op = match token.kind {
                    TokenKind::Minus => UnaryOp::Neg,
                    TokenKind::Plus => UnaryOp::Plus,
                    TokenKind::Nao => UnaryOp::Not,
                    TokenKind::Tilde => UnaryOp::BitNot,
                    _ => unreachable!(),
                };
                let expr = self.parse_unary()?;
                let span = token.span.join(expr.span());
                Some(Expr::Unary {
                    op,
                    expr: Box::new(expr),
                    span,
                })
            }
            TokenKind::Inc | TokenKind::Dec => {
                let token = self.bump();
                let target = self.parse_unary()?;
                let span = token.span.join(target.span());
                Some(Expr::IncDec {
                    target: Box::new(target),
                    delta: if token.kind == TokenKind::Inc { 1 } else { -1 },
                    prefix: true,
                    span,
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Option<Expr> {
        let mut expr = self.parse_primary()?;

        while self.at(TokenKind::Inc) || self.at(TokenKind::Dec) {
            let token = self.bump();
            let span = expr.span().join(token.span);
            expr = Expr::IncDec {
                target: Box::new(expr),
                delta: if token.kind == TokenKind::Inc { 1 } else { -1 },
                prefix: false,
                span,
            };
        }

        Some(expr)
    }

    fn parse_primary(&mut self) -> Option<Expr> {
        let token = self.peek();

        match token {
            TokenKind::Int => {
                let token = self.bump();
                let raw = self.slice(token.span);
                let value = match raw.parse::<i64>() {
                    Ok(value) => value,
                    Err(_) => {
                        self.diagnostics.push(Diagnostic::error(
                            "syntax.integer-overflow",
                            format!("inteiro fora do intervalo de 64 bits: {}", raw),
                            token.span,
                        ));
                        0
                    }
                };
                Some(Expr::Int {
                    value,
                    span: token.span,
                })
            }
            TokenKind::Hex => {
                let token = self.bump();
                let raw = self.slice(token.span);
                let value = i64::from_str_radix(&raw[2..], 16).unwrap_or_else(|_| {
                    self.diagnostics.push(Diagnostic::error(
                        "syntax.invalid-hex",
                        format!("número hexadecimal inválido: {}", raw),
                        token.span,
                    ));
                    0
                });
                Some(Expr::Int {
                    value,
                    span: token.span,
                })
            }
            TokenKind::Real => {
                let token = self.bump();
                let raw = self.slice(token.span);
                let value = raw.parse::<f64>().unwrap_or(0.0);
                Some(Expr::Real {
                    value,
                    span: token.span,
                })
            }
            TokenKind::Verdadeiro | TokenKind::Falso => {
                let token = self.bump();
                Some(Expr::Bool {
                    value: token.kind == TokenKind::Verdadeiro,
                    span: token.span,
                })
            }
            TokenKind::Char => {
                let token = self.bump();
                let raw = self.slice(token.span);
                let value = decode_char(raw).unwrap_or('\0');
                Some(Expr::Char {
                    value,
                    span: token.span,
                })
            }
            TokenKind::Str => {
                let token = self.bump();
                let raw = self.slice(token.span);
                Some(Expr::Str {
                    value: decode_string(raw),
                    span: token.span,
                })
            }
            TokenKind::Ident => self.parse_ident_expr(),
            TokenKind::LParen => {
                let start = self.bump();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen);
                Some(Expr::Paren {
                    expr: Box::new(expr),
                    span: start.span.join(self.last_span()),
                })
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "syntax.expected-expression",
                    format!("esperava uma expressão, encontrado {}", token.describe()),
                    self.peek_span(),
                ));
                None
            }
        }
    }

    fn parse_ident_expr(&mut self) -> Option<Expr> {
        let first = self.expect_ident("identificador")?;
        let first_span = first.span;

        let (scope, name) = if self.eat(TokenKind::Dot) {
            let second = self.expect_ident("identificador após '.'")?;
            (Some(first.name), second)
        } else {
            (None, first)
        };

        if self.eat(TokenKind::LParen) {
            let args = self.parse_args();
            let span = first_span.join(self.last_span());
            return Some(Expr::Call {
                scope,
                name: name.name,
                args,
                span,
            });
        }

        let mut indices = Vec::new();
        while self.eat(TokenKind::LBracket) {
            match self.parse_expr() {
                Some(index) => indices.push(index),
                None => self.recover_until(&[TokenKind::RBracket]),
            }
            self.expect(TokenKind::RBracket);
        }

        if indices.is_empty() {
            Some(Expr::Ident(Ident {
                scope,
                name: name.name,
                span: first_span.join(name.span),
            }))
        } else {
            Some(Expr::Index {
                scope,
                name: name.name,
                indices,
                span: first_span.join(self.last_span()),
            })
        }
    }

    fn parse_args(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();

        if self.eat(TokenKind::RParen) {
            return args;
        }

        loop {
            match self.parse_expr_or_assign() {
                Some((_expr, Some(assignment))) => args.push(Expr::Assign {
                    target: Box::new(assignment.target),
                    op: assignment.op,
                    value: Box::new(assignment.value),
                    span: assignment.span,
                }),
                Some((expr, None)) => args.push(expr),
                None => {
                    self.recover_until(&[TokenKind::Comma, TokenKind::RParen]);
                    break;
                }
            }

            if self.eat(TokenKind::Comma) {
                continue;
            }
            break;
        }

        self.expect(TokenKind::RParen);
        args
    }

    // ---------------------------------------------------------------------
    // Utilidades
    // ---------------------------------------------------------------------

    fn parse_type(&mut self) -> Option<Type> {
        let ty = match self.peek() {
            TokenKind::TypeReal => Type::Real,
            TokenKind::TypeInteiro => Type::Inteiro,
            TokenKind::TypeVazio => Type::Vazio,
            TokenKind::TypeLogico => Type::Logico,
            TokenKind::TypeCadeia => Type::Cadeia,
            TokenKind::TypeCaracter => Type::Caracter,
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    "syntax.expected-type",
                    format!("esperava um tipo, encontrado {}", self.peek().describe()),
                    self.peek_span(),
                ));
                return None;
            }
        };

        self.bump();
        Some(ty)
    }

    fn expect_ident(&mut self, what: &str) -> Option<Ident> {
        if self.at(TokenKind::Ident) {
            let token = self.bump();
            Some(Ident {
                scope: None,
                name: self.slice(token.span).to_string(),
                span: token.span,
            })
        } else {
            self.diagnostics.push(Diagnostic::error(
                "syntax.expected-identifier",
                format!("esperava {}, encontrado {}", what, self.peek().describe()),
                self.peek_span(),
            ));
            None
        }
    }

    fn can_start_expr(&self, kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Int
                | TokenKind::Hex
                | TokenKind::Real
                | TokenKind::Char
                | TokenKind::Str
                | TokenKind::Verdadeiro
                | TokenKind::Falso
                | TokenKind::Ident
                | TokenKind::LParen
                | TokenKind::Minus
                | TokenKind::Plus
                | TokenKind::Nao
                | TokenKind::Tilde
                | TokenKind::Inc
                | TokenKind::Dec
        )
    }

    fn recover_top_level(&mut self) {
        // Chave de fechamento sem par: consome para não travar o laço principal.
        if self.at(TokenKind::RBrace) {
            self.bump();
            return;
        }

        if matches!(
            self.peek(),
            TokenKind::Eof
                | TokenKind::Programa
                | TokenKind::Inclua
                | TokenKind::Funcao
                | TokenKind::Const
        ) || self.peek().is_type()
        {
            return;
        }

        while !matches!(
            self.peek(),
            TokenKind::Eof
                | TokenKind::RBrace
                | TokenKind::Programa
                | TokenKind::Inclua
                | TokenKind::Funcao
                | TokenKind::Const
        ) && !self.peek().is_type()
        {
            self.bump();
        }
    }

    fn recover_stmt(&mut self) {
        if matches!(
            self.peek(),
            TokenKind::Eof | TokenKind::RBrace | TokenKind::Caso | TokenKind::Contrario
        ) {
            return;
        }

        while !matches!(
            self.peek(),
            TokenKind::Eof
                | TokenKind::RBrace
                | TokenKind::Caso
                | TokenKind::Contrario
                | TokenKind::Semi
        ) {
            self.bump();
        }

        if self.at(TokenKind::Semi) {
            self.bump();
        }
    }

    fn recover_until(&mut self, stops: &[TokenKind]) {
        while !self.at(TokenKind::Eof) && !stops.iter().any(|stop| self.at(*stop)) {
            self.bump();
        }
    }

    fn peek(&self) -> TokenKind {
        self.tokens[self.pos].kind
    }

    fn peek_span(&self) -> Span {
        self.tokens[self.pos].span
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Option<Token> {
        if self.at(kind) {
            Some(self.bump())
        } else {
            let found = self.peek().describe();
            self.diagnostics.push(
                Diagnostic::error(
                    "syntax.expected-token",
                    format!("esperado {}, encontrado {}", kind.describe(), found),
                    self.peek_span(),
                )
                .with_help(
                    "verifique se falta um fechamento de bloco ou parêntese antes deste ponto",
                ),
            );
            None
        }
    }

    fn last_span(&self) -> Span {
        if self.pos == 0 {
            self.tokens[0].span
        } else {
            self.tokens[self.pos - 1].span
        }
    }

    fn slice(&self, span: Span) -> &'a str {
        &self.source[span.start as usize..span.end as usize]
    }
}

fn assign_op(kind: TokenKind) -> Option<AssignOp> {
    match kind {
        TokenKind::Assign => Some(AssignOp::Assign),
        TokenKind::PlusEq => Some(AssignOp::Add),
        TokenKind::MinusEq => Some(AssignOp::Sub),
        TokenKind::StarEq => Some(AssignOp::Mul),
        TokenKind::SlashEq => Some(AssignOp::Div),
        TokenKind::PercentEq => Some(AssignOp::Mod),
        TokenKind::CaretEq => Some(AssignOp::Xor),
        TokenKind::AmpEq => Some(AssignOp::BitAnd),
        TokenKind::PipeEq => Some(AssignOp::BitOr),
        _ => None,
    }
}

/// Precedência conforme a ordem das alternativas em `Portugol.g4`
/// (a primeira alternativa tem precedência mais alta).
fn binary_op(kind: TokenKind) -> Option<(BinOp, u8)> {
    let value = match kind {
        TokenKind::Star => (BinOp::Mul, 10),
        TokenKind::Slash => (BinOp::Div, 10),
        TokenKind::Percent => (BinOp::Mod, 10),
        TokenKind::Plus => (BinOp::Add, 9),
        TokenKind::Minus => (BinOp::Sub, 9),
        TokenKind::Eq => (BinOp::Eq, 8),
        TokenKind::Ne => (BinOp::Ne, 8),
        TokenKind::Gt => (BinOp::Gt, 8),
        TokenKind::Lt => (BinOp::Lt, 8),
        TokenKind::Le => (BinOp::Le, 8),
        TokenKind::Ge => (BinOp::Ge, 8),
        TokenKind::ELogico => (BinOp::And, 7),
        TokenKind::OuLogico => (BinOp::Or, 6),
        TokenKind::Caret => (BinOp::Xor, 5),
        TokenKind::Shl => (BinOp::Shl, 4),
        TokenKind::Shr => (BinOp::Shr, 4),
        TokenKind::Amp => (BinOp::BitAnd, 3),
        TokenKind::Pipe => (BinOp::BitOr, 2),
        _ => return None,
    };
    Some(value)
}

fn decode_string(raw: &str) -> String {
    if raw.len() < 2 {
        return String::new();
    }

    let inner = &raw[1..raw.len() - 1];
    let mut out = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000C}'),
            Some('0') => out.push('\0'),
            Some('u') => {
                let mut hex = String::new();
                for _ in 0..4 {
                    if let Some(next) = chars.next() {
                        hex.push(next);
                    }
                }
                if let Ok(code) = u32::from_str_radix(&hex, 16)
                    && let Some(decoded) = char::from_u32(code)
                {
                    out.push(decoded);
                }
            }
            Some(first @ '0'..='7') => {
                let mut octal = String::from(first);
                for _ in 0..2 {
                    if let Some(next @ '0'..='7') = chars.peek().copied() {
                        octal.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(code) = u32::from_str_radix(&octal, 8)
                    && let Some(decoded) = char::from_u32(code)
                {
                    out.push(decoded);
                }
            }
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }

    out
}

fn decode_char(raw: &str) -> Option<char> {
    if raw.len() < 2 {
        return None;
    }
    decode_string(&format!("\"{}\"", &raw[1..raw.len() - 1]))
        .chars()
        .next()
}
