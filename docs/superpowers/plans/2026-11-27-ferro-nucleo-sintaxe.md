# FERRO Core Syntax Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fazer o programa motivador compilar e produzir `Olá, Ada!\n` em frontend/HIR, bytecode/VM e Rust gerado, com `escreval`, inferência em `seja`, interpolação com partes, `leia()` como expressão e retorno implícito no corpo da função.

**Architecture:** As cinco sintaxes são açúcares ou nós novos sobre a arquitetura existente: `escreval` vira `Write`, `leia()` vira `ExprKind::Read` + `Op::Read`, interpolação vira `ExprKind::InterpolatedString` com partes e lowering para `Op::Add`/`__ferro_concat`, e retorno implícito vira `Function.tail_expression` + `Op::Return`/`return`. Nenhuma VM nova, nenhum opcode novo e nenhuma API WASM nova.

**Tech Stack:** Rust 2024, `ferro-hir`, `ferro-rust-frontend`, `ferro-bytecode`, `ferro-vm`, `ferro-rust`, `ferro-service`, `ferro-wasm`.

**Spec:** `docs/superpowers/specs/2026-11-26-ferro-syntax-evolution-design.md` e RFCs 001–005 em `docs/superpowers/specs/`.

**Precondition:** `docs/superpowers/plans/2026-11-27-ferro-contratos-e-testes-de-aceite.md` executado até a fase vermelha; o fixture `tests/fixtures/ferro-syntax-evolution.fer` existe.

## Global Constraints

- `escreval(a, b);` escreve `a`, `b` e uma quebra final como `escreva`; nenhum `Op`/`StmtKind` novo para ele. `escreval` é builtin não reservado, conforme RFC 003.
- `leia()` infere o tipo do contexto e resulta em `cadeia` sem contexto tipado; a forma legada com variável-alvo converte ao tipo da variável.
- Interpolação: `\{`/`\}` literais; `{...}` não vazio e fechado é expressão; `{}` e `{` aberto são literais; números em interpolação geram diagnóstico nesta fatia.
- Retorno implícito apenas na última posição do corpo de função com retorno declarado; blocos aninhados exigem `retorne`.
- Spans são offsets UTF-8 semiabertos; cada parte de interpolação e cada nó de leitura carrega o seu.
- `inteiro` permanece `i32` com `wrapping`; nenhuma decisão de largura/overflow/aliases nesta fatia.
- `cargo fmt --check` ao final de cada task que altera Rust.
- Commits sem `Co-Authored-By`, sem menção a Claude/IA/Anthropic, sem emoji.

## Review Focus

- `escreval("a");` não pode duplicar newline quando o programa já termina com `"\n"` — teste na Task 1.
- `seja x = [];` continua erro de inferência, mesmo com `:` opcional — teste na Task 2.
- `\{nome}`, `{}` e `{` sem fechamento continuam literais, e `{1 +}` falha no span do conteúdo — testes na Task 3.
- `leia()` em contexto `inteiro` converte para `inteiro` e falha em execução quando a entrada não converte; `has_read` enxerga a leitura dentro da expressão — testes na Task 4.
- Bloco aninhado sem `;` não vira retorno; `seja` sem `;` no corpo de `inicio` continua erro — teste na Task 5.

---

### Task 1: `escreval` como açúcar de `Write`

**Files:**
- Modify: `crates/ferro-rust-frontend/src/parser/stmt.rs:175-181`, `:582-618` (despacho de `escreval` como builtin não reservado)
- Test: `crates/ferro-rust-frontend/tests/parser.rs`, `crates/ferro-vm/tests/ferro_core.rs`

**Interfaces:**
- Consumes: `write(&mut self)` atual, `StmtKind::Write(Vec<Expr>)`.
- Produces: `write(&mut self, newline: bool)`; `escreval` é builtin não reservado, conforme RFC 003; newline sintético com `Span::new(close.span.start, close.span.start)`.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/ferro-rust-frontend/tests/parser.rs` (o arquivo já importa `ExprKind`, `StmtKind` e `Type` de `ferro_hir`):

```rust
#[test]
fn escreval_appends_a_newline_expression_to_write() {
    let source = "funcao inicio() { escreval(\"a\"); }";
    let program = lower(source).expect("escreval deve compilar");
    let StmtKind::Write(values) = &program.body[0].kind else {
        panic!("esperava Write");
    };
    assert_eq!(values.len(), 2);
    assert!(matches!(&values[1].kind, ExprKind::String(text) if text == "\n"));
    assert!(values[1].span.is_empty());
}
```

Em `crates/ferro-vm/tests/ferro_core.rs`:

```rust
#[test]
fn escreval_emits_one_newline_without_duplication() {
    let source = "funcao inicio() { escreval(\"a\"); escreval(); }";
    let module = compile_ferro_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução de escreval");
    assert_eq!(io.output, "a\n\n");
}
```

- [ ] **Step 2: Rodar e confirmar a falha**

Run: `cargo test -p ferro-rust-frontend --test parser escreval_appends_a_newline_expression_to_write && cargo test -p ferro-vm --test ferro_core escreval_emits_one_newline_without_duplication`
Expected: FAIL — `lower()` retorna `ferro.expected-statement`.

- [ ] **Step 3: Implementar o açúcar**

Manter `escreval` fora da lista de reservadas (`is_reserved()`): ele é builtin não reservado, conforme RFC 003 e o teste `escreval_can_still_be_used_as_an_identifier`. Em `statement()`, despachar `escreval` para `self.write(true)` e `escreva` para `self.write(false)`. Em `write(newline)`: manter o laço atual de argumentos, capturando o token de fechamento com `let close = self.expect(Kind::RParen, "Esperado `)` após valores de escreva")?;`, após `let end = self.expect(Kind::Semicolon, ...)?`, quando `newline`, acrescentar a `values` a expressão `hir::Expr { kind: hir::ExprKind::String("\n".to_owned()), ty: hir::Type::String, span: hir::Span::new(close.span.start, close.span.start) }`. Não alterar HIR, bytecode, VM nem Rust gerado. Este passo implementa o RFC 003, linha 24: o `"\n"` sintético tem span `[p, p)` com `p` imediatamente antes do `)` de fechamento.

- [ ] **Step 4: Rodar e confirmar o passe**

Run: `cargo test -p ferro-rust-frontend --test parser escreval_appends_a_newline_expression_to_write && cargo test -p ferro-vm --test ferro_core escreval_emits_one_newline_without_duplication && cargo fmt --check`
Expected: PASS nos dois testes; `cargo fmt --check` limpo.

- [ ] **Step 5: Commit**

```bash
git add crates/ferro-rust-frontend/src/parser/stmt.rs crates/ferro-rust-frontend/tests/parser.rs crates/ferro-vm/tests/ferro_core.rs
git commit -m "feat(ferro): adicionar escreval como açúcar de escreva"
```

---

### Task 2: Inferência de tipo em `seja`

**Files:**
- Modify: `crates/ferro-rust-frontend/src/parser/stmt.rs:284-346`
- Test: `crates/ferro-rust-frontend/tests/parser.rs`, `crates/ferro-rust-frontend/tests/semantics.rs`

**Interfaces:**
- Consumes: `expression_with_expected(min_precedence: u8, expected: Option<Type>)`, `Binding.ty: Type`.
- Produces: `:` opcional; tipo do binding = `initializer.ty`; `=` continua obrigatório.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/ferro-rust-frontend/tests/parser.rs`:

```rust
#[test]
fn infers_binding_type_from_the_initializer_without_a_colon() {
    let source = "funcao inicio() { seja nome = \"Ada\"; seja idade = 42; }";
    let program = lower(source).expect("declarações sem tipo devem compilar");
    assert_eq!(program.bindings[0].ty, hir::Type::String);
    assert_eq!(program.bindings[1].ty, hir::Type::Integer);
}
```

Em `crates/ferro-rust-frontend/tests/semantics.rs`:

```rust
#[test]
fn still_requires_an_initializer_and_an_explicit_type_for_an_empty_vector() {
    assert!(lower("funcao inicio() { seja x: inteiro; }")
        .unwrap_err()
        .iter()
        .any(|d| d.code == "ferro.unexpected-token"));
    assert!(lower("funcao inicio() { seja x = []; }")
        .unwrap_err()
        .iter()
        .any(|d| d.code == "ferro.empty-vector-type-inference"));
}
```

- [ ] **Step 2: Rodar e confirmar a falha**

Run: `cargo test -p ferro-rust-frontend --test parser infers_binding_type_from_the_initializer_without_a_colon && cargo test -p ferro-rust-frontend --test semantics still_requires_an_initializer_and_an_explicit_type_for_an_empty_vector`
Expected: FAIL — `Esperado ':' entre nome e tipo`.

- [ ] **Step 3: Implementar a inferência**

Em `declaration()`: substituir as linhas 296–302 por `let ty = self.eat(&Kind::Colon).then(|| self.type_name()).transpose()?;`, manter `expect(Kind::Assign, ...)`, chamar `self.expression_with_expected(0, ty)?` e usar `initializer.ty` como `Binding.ty`. A checagem de compatibilidade (`ferro.type-mismatch`) continua apenas quando `ty` é `Some`. Não tocar em `for_loop()` nem em `vector_literal()`.

- [ ] **Step 4: Rodar e confirmar o passe**

Run: `cargo test -p ferro-rust-frontend --test parser infers_binding_type_from_the_initializer_without_a_colon && cargo test -p ferro-rust-frontend --test semantics still_requires_an_initializer_and_an_explicit_type_for_an_empty_vector && cargo test -p ferro-rust-frontend --test parser && cargo fmt --check`
Expected: PASS, sem regressão nos testes de `para`.

- [ ] **Step 5: Commit**

```bash
git add crates/ferro-rust-frontend/src/parser/stmt.rs crates/ferro-rust-frontend/tests/parser.rs crates/ferro-rust-frontend/tests/semantics.rs
git commit -m "feat(ferro): inferir tipo de seja a partir do inicializador"
```

---

### Task 3: Interpolação de cadeias com partes na HIR

**Files:**
- Modify: `crates/ferro-hir/src/lib.rs:331-396`
- Modify: `crates/ferro-rust-frontend/src/lexer.rs:8`, `:235-288`; `src/parser/mod.rs:15-34`, `:60-81`; `src/lower.rs:7-13`; `src/parser/expr.rs:137-156`; `src/format.rs:53-67`
- Modify: `crates/ferro-bytecode/src/ferro.rs:641-820`
- Modify: `crates/ferro-rust/src/codegen/expr.rs:21-460`, `src/codegen/types.rs:330-427`
- Modify: `crates/ferro-service/src/rename.rs:158-300`, `:471-731`
- Test: `crates/ferro-rust-frontend/tests/parser.rs`, `crates/ferro-rust-frontend/tests/semantics.rs`, `crates/ferro-vm/tests/ferro_core.rs`, `crates/ferro-rust/src/lib.rs`

**Interfaces:**
- Consumes: `Kind::String`, `Op::PushStr`, `Op::Add`, `__ferro_concat`, `ferro_string_view(value, ty)`.
- Produces: `StringPart::Literal(String, Span)` / `StringPart::Expression(Expr, Span)`; `ExprKind::InterpolatedString { parts: Vec<StringPart> }`; `Kind::String { decoded: String, raw: String }`; `ferro.interpolation-invalid`; `ferro.interpolation-non-string`.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/ferro-rust-frontend/tests/parser.rs`, acrescentando `StringPart` ao `use` existente:

```rust
#[test]
fn interpolation_parts_keep_their_own_spans_and_escaped_braces_are_literal() {
    let source = r#"funcao inicio() { escreva("Olá, {nome}!"); }"#;
    let program = lower(source).expect("interpolação válida");
    let StmtKind::Write(values) = &program.body[0].kind else { panic!("esperava Write") };
    let ExprKind::InterpolatedString { parts } = &values[0].kind else {
        panic!("esperava cadeia interpolada");
    };
    assert_eq!(parts.len(), 3);
    assert!(matches!(&parts[0], StringPart::Literal(text, _) if text == "Olá, "));
    assert!(matches!(&parts[1], StringPart::Expression(expr, _) if expr.ty == Type::String));
    let literal = r#"funcao inicio() { escreva("\{nome}"); }"#;
    let program = lower(literal).expect("chaves escapadas são literais");
    let StmtKind::Write(values) = &program.body[0].kind else { panic!("esperava Write") };
    assert!(matches!(&values[0].kind, ExprKind::String(text) if text == "{nome}"));

    let escaped_and_interpolated = r#"funcao inicio() { escreva("\{{nome}"); }"#;
    let program = lower(escaped_and_interpolated).expect("escape e interpolação");
    let StmtKind::Write(values) = &program.body[0].kind else { panic!("esperava Write") };
    let ExprKind::InterpolatedString { parts } = &values[0].kind else {
        panic!("esperava cadeia interpolada");
    };
    assert!(matches!(&parts[0], StringPart::Literal(text, span) if text == "{" && span.end - span.start == 2), "{parts:?}");
}
```

Em `crates/ferro-rust-frontend/tests/semantics.rs`:

```rust
#[test]
fn interpolation_rejects_invalid_content_numbers_and_unclosed_braces() {
    let source = r#"funcao inicio() { escreva("{1 +}"); }"#;
    let error = lower(source).unwrap_err().into_iter().find(|d| d.code == "ferro.interpolation-invalid").unwrap();
    assert_eq!(&source[error.span.start as usize..error.span.end as usize], "1 +");

    let number = r#"funcao inicio() { escreva("{42}"); }"#;
    assert!(lower(number).unwrap_err().iter().any(|d| d.code == "ferro.interpolation-non-string"));

    for literal in [r#"funcao inicio() { escreva("{}"); }"#, r#"funcao inicio() { escreva("abri {"); }"#] {
        assert!(lower(literal).is_ok(), "{literal}");
    }
}
```

Em `crates/ferro-vm/tests/ferro_core.rs`:

```rust
#[test]
fn executes_interpolated_string_through_bytecode_and_vm() {
    let source = r#"funcao inicio() { seja nome = "Ada"; escreva("Olá, {nome}!"); }"#;
    let module = compile_ferro_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução de interpolação");
    assert_eq!(io.output, "Olá, Ada!");
}
```

- [ ] **Step 2: Rodar e confirmar a falha**

Run: `cargo test -p ferro-rust-frontend --test parser interpolation_parts_keep_their_own_spans_and_escaped_braces_are_literal && cargo test -p ferro-rust-frontend --test semantics interpolation_rejects_invalid_content_numbers_and_unclosed_braces && cargo test -p ferro-vm --test ferro_core executes_interpolated_string_through_bytecode_and_vm`
Expected: FAIL — HIR não tem `StringPart`/`InterpolatedString`.

- [ ] **Step 3: Definir os nós na HIR**

Em `crates/ferro-hir/src/lib.rs`, acrescentar `#[derive(Debug, Clone, PartialEq)] pub enum StringPart { Literal(String, Span), Expression(Expr, Span) }` e a variante `InterpolatedString { parts: Vec<StringPart> }` em `ExprKind`, ambos ao lado de `String(String)`.

- [ ] **Step 4: Transportar o texto bruto da cadeia até o parser**

Mudar `Kind::String(String)` para `Kind::String { decoded: String, raw: String }` em `lexer.rs:8`; `quoted()` continua decodificando `decoded` e passa `raw` = conteúdo entre aspas. Ajustar os três pontos que casam o token: `lexer.rs:286`, `format.rs:65`, `expr.rs:155`. Mudar `Parser::new(tokens: Vec<Token>)` para `Parser::new(tokens: Vec<Token>, source: String)` e `lower()` para passar `source.to_owned()`. O campo `raw` é a única fonte para decidir `{`/`\}`; `decoded` continua sendo usado para cadeias simples.

- [ ] **Step 5: Implementar o parser de partes**

Em `expr.rs`, acrescentar `fn string_parts(&mut self, token: &Token) -> Result<Vec<StringPart>, ...>` e chamar da arm `Kind::String` quando `raw` contém interpolação válida. Regras exatas: percorrer `raw` com índices de byte; `\{`/`\}` viram `Literal("{"/"}", span_de_2_bytes)` cobrindo os dois bytes do escape no fonte, enquanto o valor decodificado é um caractere; demais escapes seguem a tabela atual do lexer; `{` não escapado só vira expressão se existir `}` não escapado depois dele com conteúdo não vazio na mesma cadeia; `{}` e `{` sem fechamento viram `Literal`. Para o conteúdo, trocar temporariamente `self.tokens`/`self.cursor` pelos tokens de `lexer::lex(conteudo)` com spans deslocados pelo offset do conteúdo, chamar `self.expression(0)`, restaurar a stream e devolver `StringPart::Expression(expr, span_da_parte)`. Conforme a RFC 004, o `Span` da parte cobre `{...}` incluindo as chaves, enquanto `expr.span` e os diagnósticos continuam no conteúdo sem chaves. Erros de parse viram `ferro.interpolation-invalid` no span do conteúdo; expressão com `ty` diferente de `String`/`Character` vira `ferro.interpolation-non-string` no mesmo span. A expressão inteira tem `ty: Type::String` e `span` do token original.

- [ ] **Step 6: Lowering nos backends**

Em `crates/ferro-bytecode/src/ferro.rs`, `expr()`: para `InterpolatedString`, emitir `Op::PushStr` de cada literal com o span da parte; para cada parte seguinte, avaliar a expressão e emitir `Op::Add` com o span da parte. Para uma única parte literal, emitir somente `PushStr`. A parte `caracter` entra na dobra já como valor `Str` de um caractere pelo caminho de concatenação existente na VM; não há opcode novo nem `Op::Convert` para `cadeia`. Em `crates/ferro-rust/src/codegen/expr.rs`, `render_expr()`: montar a mesma dobra com `__ferro_concat(acc, ferro_string_view(parte, ty))`, usando `__ferro_string(...)` para literais; `ferro_string_view` já materializa `caracter` via `__ferro_char`, sem helper novo; registrar `SourceMapSegment` por parte via `push_expr`. Em `types.rs` (`expr_uses_host_module`) e `crates/ferro-service/src/rename.rs`, incluir as novas variantes: `Read` não usa host nem rename; `InterpolatedString` percorre as partes. Todos os `match` exaustivos sobre `ExprKind` devem compilar.

- [ ] **Step 7: Preservar a formatação**

Atualizar apenas o casamento de `Kind::String` em `format.rs:65` para o novo payload. `format()` continua reinserindo `&source[start..end]` verbatim. Acrescentar teste em `format.rs::tests` com `escreva("Olá, { nome }!");` assertando que o conteúdo entre chaves permanece exatamente `{ nome }`.

- [ ] **Step 8: Rodar e confirmar o passe**

Run: `cargo test -p ferro-hir -p ferro-rust-frontend -p ferro-bytecode -p ferro-vm -p ferro-rust && cargo fmt --check`
Expected: PASS, sem regressão nos testes de concatenação e formatação.

- [ ] **Step 9: Commit**

```bash
git add crates/ferro-hir/src/lib.rs crates/ferro-rust-frontend crates/ferro-bytecode/src/ferro.rs crates/ferro-rust/src/codegen crates/ferro-service/src/rename.rs
git commit -m "feat(ferro): representar interpolação com partes na HIR"
```

---

### Task 4: `leia()` como expressão com conversão implícita

**Files:**
- Modify: `crates/ferro-hir/src/lib.rs:338-396`
- Modify: `crates/ferro-rust-frontend/src/parser/expr.rs:137-631`
- Modify: `crates/ferro-bytecode/src/ferro.rs:641-820`
- Modify: `crates/ferro-rust/src/codegen/expr.rs:21-460`, `src/codegen/types.rs:49-66`, `:330-427`
- Modify: `crates/ferro-service/src/rename.rs`
- Test: `crates/ferro-rust-frontend/tests/semantics.rs`, `crates/ferro-vm/tests/ferro_core.rs`, `crates/ferro-wasm/tests/wasm_facade.rs`, `crates/ferro-service/tests/ferro_rs_runner.rs`

**Interfaces:**
- Consumes: `ExprKind::Read { target }` (Task 3 estabeleceu os pontos exaustivos), `Op::Read(ValueType)` com o tipo inferido, `__ferro_read_line` com o rótulo do tipo esperado.
- Produces: `ExprKind::Read` com `target` vindo do contexto (`None` → `cadeia`); erro de execução em entrada inválida; `ferro.read-arguments-unsupported`; `statements_have_read` passa a enxergar expressões.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/ferro-vm/tests/ferro_core.rs`:

```rust
#[test]
fn reads_a_string_expression_without_a_target_binding() {
    let source = "funcao inicio() { escreva(leia()); }";
    let module = compile_ferro_source(source);
    let mut io = BufferIo::new("Ada\n");
    run(&module, &mut io).expect("leia() como expressão");
    assert_eq!(io.output, "Ada");
}
```

Em `crates/ferro-service/tests/ferro_rs_runner.rs`:

```rust
#[test]
fn generated_rust_requests_a_string_when_read_is_an_expression() {
    let cancel = AtomicBool::new(false);
    let mut requests = Vec::new();
    let result = run_ferro_rs_with_io(
        "funcao inicio() { escreva(leia()); }",
        &cancel,
        |expected| { requests.push(expected.to_owned()); Ok("Ada".to_owned()) },
        |_| {},
    )
    .unwrap();
    assert_eq!(requests, ["cadeia"]);
    assert!(matches!(result, FerroRustRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "Ada"));
}
```

Em `crates/ferro-rust-frontend/tests/semantics.rs`:

```rust
#[test]
fn read_expression_infers_integer_from_context() {
    let source = "funcao inicio() { seja idade: inteiro = leia(); }";
    let program = lower(source).expect("leia() infere o tipo anotado");
    assert_eq!(program.bindings[0].ty, hir::Type::Integer);
}
```

A rejeição de argumentos na forma de expressão (`ferro.read-arguments-unsupported`) continua coberta em `crates/ferro-rust-frontend/tests/semantics.rs`; a grafia legada não é repetida neste plano.

- [ ] **Step 2: Rodar e confirmar a falha**

Run: `cargo test -p ferro-vm --test ferro_core reads_a_string_expression_without_a_target_binding && cargo test -p ferro-service --test ferro_rs_runner generated_rust_requests_a_string_when_read_is_an_expression`
Expected: FAIL — `leia()` exige identificador.

- [ ] **Step 3: Implementar o nó e o parser**

Acrescentar `ExprKind::Read { target }` na HIR. Em `prefix()` (`expr.rs`), na arm `Kind::Ident(name)`, antes da cadeia de builtins, tratar `name == "leia"` seguido de `LParen`: sem argumentos, consumir `RParen`, tomar o tipo esperado de `expression_with_expected` (ou `None` para `cadeia`) e devolver `Expr { kind: ExprKind::Read { target }, ty, span: início..fechamento }`; com argumentos, falhar com `ferro.read-arguments-unsupported` no span dos argumentos. Não alterar `read()` em `stmt.rs`: a forma com variável-alvo continua comando com o tipo da variável.

- [ ] **Step 4: Atualizar os backends e `has_read`**

Em `ferro.rs`, `expr()`: `ExprKind::Read { target }` emite `Op::Read(ValueType)` com o tipo inferido e o span da expressão (o valor fica na pilha, sem `StoreLocal`). Em `codegen/expr.rs`, `render_expr()`: `ExprKind::Read` devolve `__ferro_read_line` com o rótulo do tipo esperado (`cadeia` quando `target` é `None`) e mapeamento no span da chamada. Em `codegen/types.rs`, estender `statements_have_read()` com um auxiliar `expression_has_read(&Expr)` aplicado a todos os campos de expressão dos comandos (`Declare.initializer`, `Assign.value`, `FieldAssign.value`, `Write`, condições e corpos de `If`/laços/`Choose`, `Return.value`, `IndexAssign`, `VectorPush`); estender `expr_uses_host_module()` para percorrer `InterpolatedString`. Fazer o mesmo percurso em `ferro-service/src/rename.rs`.

- [ ] **Step 5: Rodar e confirmar o passe**

Run: `cargo test -p ferro-hir -p ferro-rust-frontend -p ferro-bytecode -p ferro-vm -p ferro-rust -p ferro-wasm -p ferro-service && cargo fmt --check`
Expected: PASS, incluindo `pauses_at_leia_requests_typed_input_and_resumes_the_program`.

- [ ] **Step 6: Commit**

```bash
git add crates/ferro-hir/src/lib.rs crates/ferro-rust-frontend/src crates/ferro-bytecode/src/ferro.rs crates/ferro-rust/src/codegen crates/ferro-service/src/rename.rs crates/ferro-rust-frontend/tests crates/ferro-vm/tests crates/ferro-service/tests
git commit -m "feat(ferro): aceitar leia() como expressão com conversão implícita"
```

---

### Task 5: Retorno implícito no corpo da função

**Files:**
- Modify: `crates/ferro-hir/src/lib.rs:220-230`
- Modify: `crates/ferro-rust-frontend/src/parser/stmt.rs:55-97`, `:620-664`; `src/parser/mod.rs:139-179`; `src/parser/constants.rs:270-283`
- Modify: `crates/ferro-bytecode/src/ferro.rs:165-204`
- Modify: `crates/ferro-rust/src/codegen/mod.rs:213-230`
- Test: `crates/ferro-rust-frontend/tests/parser.rs`, `crates/ferro-rust-frontend/tests/semantics.rs`, `crates/ferro-vm/tests/ferro_core.rs`

**Interfaces:**
- Consumes: `Function.return_type: Option<Type>`, `guarantees_return(&[Stmt]) -> bool`, `Op::Return`.
- Produces: `Function.tail_expression: Option<Expr>`; `block(&mut self, root: bool, allow_tail: bool) -> Result<(Vec<Stmt>, Option<Expr>, Span), ...>`; `ferro.implicit-return-without-type`.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/ferro-rust-frontend/tests/parser.rs`:

```rust
#[test]
fn accepts_a_trailing_expression_as_an_implicit_return() {
    let source = "funcao saudar(nome: cadeia) -> cadeia { \"Olá, {nome}!\" }";
    let program = lower(source).expect("retorno implícito");
    let function = &program.functions[0];
    assert!(function.tail_expression.is_some());
    assert!(function.body.iter().all(|s| !matches!(s.kind, hir::StmtKind::Return { .. })));
}
```

Em `crates/ferro-rust-frontend/tests/semantics.rs`:

```rust
#[test]
fn implicit_return_is_only_allowed_in_a_typed_function_body() {
    assert!(lower("funcao f() -> inteiro { se (verdadeiro) { 1 } retorne 2; }")
        .unwrap_err()
        .iter()
        .any(|d| d.code == "ferro.implicit-return-without-type"));
    assert!(lower("funcao f() -> cadeia { 1 + 1 }")
        .unwrap_err()
        .iter()
        .any(|d| d.code == "ferro.return-type-mismatch"));
}
```

Em `crates/ferro-vm/tests/ferro_core.rs`:

```rust
#[test]
fn executes_an_implicit_return_in_minimal_vm() {
    let source = "funcao dobro(x: inteiro) -> inteiro { x * 2 } funcao inicio() { escreva(dobro(21)); }";
    let module = compile_ferro_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("retorno implícito na VM");
    assert_eq!(io.output, "42");
}
```

- [ ] **Step 2: Rodar e confirmar a falha**

Run: `cargo test -p ferro-rust-frontend --test parser accepts_a_trailing_expression_as_an_implicit_return && cargo test -p ferro-vm --test ferro_core executes_an_implicit_return_in_minimal_vm`
Expected: FAIL — campo `tail_expression` inexistente / `ferro.expected-statement`.

- [ ] **Step 3: Implementar a HIR e o parser**

Acrescentar `tail_expression: Option<Expr>` a `Function`. Mudar `block()` para `block(&mut self, root: bool, allow_tail: bool) -> Result<(Vec<Stmt>, Option<Expr>, Span), ...>` e atualizar os 8 chamadores: `allow_tail = true` somente em `mod.rs:139` (corpo de qualquer função, inclusive `inicio`), `false` nos blocos aninhados de `stmt.rs`. Fatorar a lookahead em `try_tail()`: guardar `cursor`, tentar `self.expression(0)`, aceitar somente se o token seguinte for `}`/`Eof` sem `;`; senão restaurar `cursor` e seguir com `statement()`. Em `block()`, quando `allow_tail=true`, registrar a expressão devolvida por `try_tail()` como cauda; quando `allow_tail=false` e `try_tail()` encontra expressão final sem `;`, emitir `ferro.implicit-return-without-type` no span da expressão sem registrar cauda — é isso que faz `se (verdadeiro) { 1 }` ser diagnosticado como retorno implícito fora do corpo da função, em vez de `ferro.expected-statement`. Se cauda existir no corpo raiz e `self.return_type.is_none()`, emitir o mesmo `ferro.implicit-return-without-type` no span da expressão — é isso que mantém `inicio` sem retorno implícito, com diagnóstico localizado. Em `mod.rs`, preencher `tail_expression` na construção de `Function`; exigir `guarantees_return()` apenas quando não há cauda; conferir `tail_expression.ty == signature.return_type`, senão `ferro.return-type-mismatch` no span da cauda. Atualizar `guarantees_return()` se necessário para o novo formato de bloco. Esta fatia não implementa `tente`; quando a RFC 007 for implementada, os ramos de valor de `tente` em expressão devem usar `try_tail()` para produzir valor de ramo, sem emitir esse diagnóstico.

- [ ] **Step 4: Atualizar os backends**

Em `compile_function()`, acrescentar parâmetro `tail: Option<&hir::Expr>` e, quando presente, emitir `self.expr(tail)?` seguido de `Op::Return` (sem `Op::PushVoid`); quando ausente, manter `Op::PushVoid; Op::Return`. `compile_program()` passa `function.tail_expression.as_ref()` para auxiliares e `None` para `inicio`. Em `codegen/mod.rs`, após `render_statements(&function.body, ...)` e antes do `}` da função, emitir `return <expr>;` com `push_expr(..., owned = true, ferro)` e mapeamento no span da cauda.

- [ ] **Step 5: Rodar e confirmar o passe**

Run: `cargo test -p ferro-hir -p ferro-rust-frontend -p ferro-bytecode -p ferro-vm -p ferro-rust && cargo fmt --check`
Expected: PASS, sem regressão em `missing-return` e `return-type-mismatch` existentes.

- [ ] **Step 6: Commit**

```bash
git add crates/ferro-hir/src/lib.rs crates/ferro-rust-frontend/src crates/ferro-bytecode/src/ferro.rs crates/ferro-rust/src/codegen crates/ferro-rust-frontend/tests crates/ferro-vm/tests
git commit -m "feat(ferro): aceitar retorno implícito no corpo da função"
```

---

### Task 6: Aceite verde, documentação de uso e verificação final

**Files:**
- Modify: `docs/research/ferro_MIGRATION_MATRIX.md`
- Modify: Ajuda/exemplos FERRO existentes que ensinam entrada e saída (registrar `escreval`, `leia()` e interpolação sem remover a forma com variável-alvo de `leia` nem `escreva`)
- Test: fixture e testes das Tasks 1–5 do plano de aceite

**Interfaces:**
- Consumes: Tasks 1–5 deste plano; Tasks 1, 2 e 4 de `docs/superpowers/plans/2026-11-27-ferro-syntax-highlighting.md`; plano de debugger/source maps.
- Produces: suíte completa verde e matriz de migração atualizada.

- [ ] **Step 1: Rodar os testes de aceite**

Run:

```bash
cargo test -p ferro-rust-frontend --test parser accepts_the_canonical_syntax_evolution_program
cargo test -p ferro-vm --test ferro_core executes_canonical_syntax_evolution_program_in_minimal_vm
cargo test -p ferro-wasm --test wasm_facade executes_canonical_syntax_evolution_program_through_wasm_facade
cargo test -p ferro-service --test ferro_rs_runner compiles_and_runs_canonical_syntax_evolution_program_with_real_rustc
```

Expected: 4 PASS; `stdout` idêntico em VM, WASM e Rust gerado: `Olá, Ada!\n`.

- [ ] **Step 2: Rodar a verificação completa**

Run: `cargo fmt --check && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: 0 falhas; nenhum warning do clippy.

- [ ] **Step 3: Atualizar a matriz e a ajuda**

Na matriz, atualizar as linhas `Tipos`, `Expressões`, `Funções`, `Diagnósticos` e `Ajuda/autocomplete/formatador` com as cinco sintaxes e os testes que as cobrem, mantendo `🟡` onde decisões de largura/overflow/Unicode/heap continuam adiadas. Nos exemplos e tópicos de Ajuda de entrada/saída, acrescentar a forma nova ao lado da legada.

- [ ] **Step 4: Commit**

```bash
git add docs/research/ferro_MIGRATION_MATRIX.md
git commit -m "docs: registrar novas sintaxes FERRO na matriz de migração"
```
