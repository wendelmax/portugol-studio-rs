# FERRO Contracts and Acceptance Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publicar o ADR mínimo de tipos e as RFCs 001–006, e deixar testes de aceite vermelhos que provem o programa motivador em frontend/HIR, VM, Rust gerado e WASM antes de qualquer implementação de sintaxe.

**Architecture:** Artefatos documentais ficam em `docs/adr/` e `docs/superpowers/specs/`; o fonte canônico do programa motivador fica em `tests/fixtures/` e é incluído por `include_str!` nos testes de cada crate. Os testes de editor não são duplicados aqui: são os testes das Tasks 1, 2 e 4 de `docs/superpowers/plans/2026-11-27-ferro-syntax-highlighting.md`.

**Tech Stack:** Markdown, Rust (`node:test` não usado neste plano), `include_str!`, `cargo test`.

**Spec:** `docs/superpowers/specs/2026-11-26-ferro-syntax-evolution-design.md`

## Global Constraints

- `inteiro` continua `i32` com `wrapping_add` e `__ferro_index(index: i32, ...)` até ADR posterior.
- `leia()` infere o tipo do contexto e resulta em `cadeia` sem contexto tipado; a forma legada com variável-alvo converte ao tipo da variável; entrada inválida é erro de execução.
- `escreval(a, b);` escreve `a`, `b` e uma quebra final como `escreva`; nenhum opcode novo. `escreval` é builtin não reservado, conforme RFC 003.
- Interpolação: `\{`/`\}` literais; `{...}` não vazio e fechado é expressão; `{}` e `{` aberto permanecem literais; conteúdo inválido gera diagnóstico com span UTF-8 exato.
- Retorno implícito vale somente para a última expressão sem `;` do corpo de função com tipo de retorno declarado; blocos aninhados exigem `retorne`.
- Nenhuma API WASM nova, nenhuma VM nova, nenhum rename de crate/produto, nenhuma pasta `specs/` paralela.
- RFCs e design docs datados em `docs/superpowers/specs/`; ADR em `docs/adr/`.
- Commits sem `Co-Authored-By`, sem menção a Claude/IA/Anthropic, sem emoji.

## Review Focus

- `seja idade: inteiro = leia();` deve compilar com conversão implícita a `inteiro`; entrada não numérica falha em execução, não no frontend — teste em `crates/ferro-rust-frontend/tests/semantics.rs` e no runner na Task 8.
- `{1 +}` deve falhar com diagnóstico no span exato do conteúdo entre chaves, não no span da cadeia inteira — teste em `crates/ferro-rust-frontend/tests/semantics.rs` na Task 8.
- `{}`, `"abri {"` e `"\\{nome}"` devem continuar cadeia literal, sem token `interpolation` — testes nas Tasks 2 e 4 de `docs/superpowers/plans/2026-11-27-ferro-syntax-highlighting.md`, confirmados vermelhos na Task 8.
- Expressão sem `;` no corpo de função sem retorno declarado deve ser erro localizado, sem retorno implícito — teste em `crates/ferro-rust-frontend/tests/semantics.rs` na Task 8.
- A forma com variável-alvo de `leia` e `escreva` atuais não podem regredir — teste preexistente `reads_typed_input_through_bufferro_io_in_minimal_vm` é preservado e executado na Task 8.

---

### Task 1: ADR 0019 — contrato mínimo de tipos

**Files:**
- Create: `docs/adr/0019-contrato-minimo-de-tipos-ferro.md`
- Modify: `docs/superpowers/specs/2026-11-26-ferro-syntax-evolution-design.md:4` (`Status: proposto` → `Status: aprovado`)

**Interfaces:**
- Consumes: decisões em `2026-11-26-ferro-syntax-evolution-design.md:96-105` e `:180-191`.
- Produces: ADR referenciado pelas RFCs 001–006 como `../adr/0019-contrato-minimo-de-tipos-ferro.md`.

- [ ] **Step 1: Escrever o ADR**

Seguir o formato de `docs/adr/0017-modelo-execucao-e-estado-ui-visual-ferro.md`: título `ADR 0019: Contrato mínimo de tipos do FERRO`, `Status: Aceita`, `Data: 2026-11-27`, `Responsável: mantenedor do produto`, relacionados (ADR 0002, ADR 0016, design doc). Nas seções `Contexto`, `Decisão` e `Alternativas consideradas`, registrar literalmente: `inteiro` = `i32` com `wrapping`; `leia()` converte implícita ao tipo esperado pelo contexto (`cadeia` sem contexto), sem `leia<T>()` nesta fatia; nomes `caracter`, `registro`, `enum`, `Opcao`, `Resultado`, `escolha` preservados; spans UTF-8 em bytes; `vazio` apenas como tipo de retorno; adiamento explícito de larguras, overflow, aliases (`caractere`, `estrutura`, `opcional`, `resultado`), `nunca`, Unicode e heap/GC.

- [ ] **Step 2: Verificar o conteúdo**

Run: `grep -nE "i32|cadeia|UTF-8|wrapping|aliases|heap|nunca" docs/adr/0019-contrato-minimo-de-tipos-ferro.md && grep -n "Status: aprovado" docs/superpowers/specs/2026-11-26-ferro-syntax-evolution-design.md`
Expected: todos os termos presentes; status atualizado.

- [ ] **Step 3: Commit**

```bash
git add docs/adr/0019-contrato-minimo-de-tipos-ferro.md docs/superpowers/specs/2026-11-26-ferro-syntax-evolution-design.md
git commit -m "docs: registrar contrato mínimo de tipos do FERRO"
```

---

### Task 2: RFC 001 — inferência em `seja`

**Files:**
- Create: `docs/superpowers/specs/2026-11-27-rfc-001-inferencia-em-seja.md`

**Interfaces:**
- Consumes: ADR 0019 (Task 1); `ferro.declaration()` em `crates/ferro-rust-frontend/src/parser/stmt.rs:284-346`; `ferro.empty-vector-type-inference` em `expr.rs:843-852`.
- Produces: regra `:` opcional com `=` obrigatório; nenhum tipo inferido muda por contexto.

- [ ] **Step 1: Escrever a RFC**

Formato: `RFC 001: Inferência de tipo em seja`, `Data: 2026-11-27`, `Status: Aceita`, relacionados (ADR 0019, RFC 002). Incluir: sintaxe `seja nome = expressao;`; o tipo vem de literal, chamada, construtor ou operação binária; tipo explícito continua canônico e é conferido com `ferro.type-mismatch`; `[]` sem contexto continua `ferro.empty-vector-type-inference`; inicializador continua obrigatório; exemplos válidos e inválidos com spans.

- [ ] **Step 2: Verificar**

Run: `grep -nE "seja nome = expressao|ferro.type-mismatch|empty-vector|obrigat" docs/superpowers/specs/2026-11-27-rfc-001-inferencia-em-seja.md`
Expected: 4 linhas encontradas.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-11-27-rfc-001-inferencia-em-seja.md
git commit -m "docs: especificar inferência de tipo em seja"
```

---

### Task 3: RFC 002 — `leia()` como expressão

**Files:**
- Create: `docs/superpowers/specs/2026-11-27-rfc-002-leia-como-expressao.md`

**Interfaces:**
- Consumes: RFC 001 (Task 2), ADR 0019; `Op::Read(ValueType)` em `crates/ferro-bytecode/src/op.rs:166-167`; `__ferro_read_line` em `crates/ferro-rust/src/codegen/mod.rs:50`.
- Produces: `ExprKind::Read { target }` com alvo inferido do contexto (`None` → `cadeia`); `StmtKind::Read` preservado; `has_read` deve enxergar expressões.

- [ ] **Step 1: Escrever a RFC**

Incluir: sintaxe `leia()` em posição de expressão, com conversão implícita ao tipo esperado pelo contexto e `cadeia` sem contexto; leitura com argumento em expressão gera `ferro.read-arguments-unsupported`; forma legada com variável-alvo inalterada e com o tipo da variável; entrada inválida gera erro de execução; `StmtKind::Read` mantido como açúcar de compatibilidade; `has_read` e `FERRO_INPUT:{expected}` no Rust gerado; `leia<T>()` e validação tipada explícita em P1; nenhuma mudança na API WASM.

- [ ] **Step 2: Verificar**

Run: `grep -nE "cadeia|convers|read-arguments|erro de execu|has_read|FERRO_INPUT|P1" docs/superpowers/specs/2026-11-27-rfc-002-leia-como-expressao.md`
Expected: todos os termos presentes.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-11-27-rfc-002-leia-como-expressao.md
git commit -m "docs: especificar leia() com conversão implícita"
```

---

### Task 4: RFC 003 — `escreval`

**Files:**
- Create: `docs/superpowers/specs/2026-11-27-rfc-003-escreval.md`

**Interfaces:**
- Consumes: `StmtKind::Write(Vec<Expr>)` em `crates/ferro-hir/src/lib.rs:282`; `write()` em `crates/ferro-rust-frontend/src/parser/stmt.rs:582-618`.
- Produces: equivalência observável: `escreval(a, b);` escreve `a`, `b` e uma quebra final como `escreva`, com newline sintético de span zero.

- [ ] **Step 1: Escrever a RFC**

Incluir: sintaxe, equivalência com `escreva` acrescido de uma quebra final, `escreval()` sem argumentos produzindo apenas a quebra, nenhum opcode/HIR novo, preservação de `Op::Write`, do `print!` e dos limites de saída, `escreval` como builtin não reservado na IDE e no playground, e exemplo com `stdout` esperado terminando em `\n`.

- [ ] **Step 2: Verificar**

Run: `grep -nE 'quebra final|Op::Write|builtin|stdout' docs/superpowers/specs/2026-11-27-rfc-003-escreval.md`
Expected: todos os termos presentes.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-11-27-rfc-003-escreval.md
git commit -m "docs: especificar escreval como açúcar de escreva"
```

---

### Task 5: RFC 004 — interpolação de cadeias

**Files:**
- Create: `docs/superpowers/specs/2026-11-27-rfc-004-interpolacao-de-cadeias.md`

**Interfaces:**
- Consumes: RFC 003 (Task 4); regra de concatenação em `crates/ferro-rust-frontend/src/parser/constants.rs:318-323`; `Op::Add` e `__ferro_concat`.
- Produces: contrato HIR `StringPart::Literal(String, Span)` / `StringPart::Expression(Expr, Span)` e `ExprKind::InterpolatedString { parts }`; código `ferro.interpolation-invalid` e `ferro.interpolation-non-string`.

- [ ] **Step 1: Escrever a RFC**

Incluir: gramática exata do design (`\{`, `\}`, `{...}` não vazio e fechado, `{}` e `{` aberto literais, primeiro `}` fecha), partes HIR com spans próprios e `ty = cadeia`, conversão apenas de `cadeia`/`caracter`, diagnóstico `ferro.interpolation-invalid` para `{1 +}` no span do conteúdo e `ferro.interpolation-non-string` para números, lowering via `Op::Add`/`__ferro_concat` com source map por parte, formatação verbatim, rename/completion interno em P1, e ausência de especificadores de formato.

- [ ] **Step 2: Verificar**

Run: `grep -nE "StringPart|ferro.interpolation-invalid|ferro.interpolation-non-string|verbatim|Op::Add" docs/superpowers/specs/2026-11-27-rfc-004-interpolacao-de-cadeias.md`
Expected: todos os termos presentes.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-11-27-rfc-004-interpolacao-de-cadeias.md
git commit -m "docs: especificar interpolação de cadeias com partes na HIR"
```

---

### Task 6: RFC 005 — retorno implícito

**Files:**
- Create: `docs/superpowers/specs/2026-11-27-rfc-005-retorno-implicito.md`

**Interfaces:**
- Consumes: `Function.return_type: Option<Type>` e `Function.body` em `crates/ferro-hir/src/lib.rs:220-230`; `guarantees_return()` em `parser/constants.rs:270-283`.
- Produces: campo `Function.tail_expression: Option<Expr>` e código `ferro.implicit-return-without-type`.

- [ ] **Step 1: Escrever a RFC**

Incluir: expressão final sem `;` apenas no corpo de função com retorno declarado; conversão para `StmtKind::Return { value }` nos backends; blocos aninhados (`se`, `senao`, `escolha`, laços) continuam exigindo `retorne`; `retorne` explícito continua válido; funções sem retorno e blocos vazios não ganham retorno implícito; erro `ferro.implicit-return-without-type` com span da expressão; exemplos com `;` e sem `;`.

- [ ] **Step 2: Verificar**

Run: `grep -nE "tail_expression|ferro.implicit-return-without-type|guarantees_return|retorne" docs/superpowers/specs/2026-11-27-rfc-005-retorno-implicito.md`
Expected: todos os termos presentes.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-11-27-rfc-005-retorno-implicito.md
git commit -m "docs: especificar retorno implícito no corpo da função"
```

---

### Task 7: RFC 006 — contrato de realce compartilhado

**Files:**
- Create: `docs/superpowers/specs/2026-11-27-rfc-006-realce-de-sintaxe.md`

**Interfaces:**
- Consumes: `2026-11-26-ferro-syntax-evolution-design.md:154-165`; RFC 003 (Task 4), RFC 004 (Task 5).
- Produces: taxonomia `keyword`, `type`, `builtin`, `string`, `interpolation`, `number`, `comment`, `operator`; implementação já planejada em `docs/superpowers/plans/2026-11-27-ferro-syntax-highlighting.md`.

- [ ] **Step 1: Escrever a RFC**

Incluir: taxonomia compartilhada, `escreval` e `leia` como `builtin`, regras de `{...}` idênticas à RFC 004 (o realce não valida expressão), IDE com nó interno `variableName` exclusivo, playground com `pg-tok-interpolation` para o segmento inteiro, variáveis `--syntax-interpolation`/`--pg-interpolation`, `<pre>` exatamente igual ao `<textarea>` e proibição de `innerHTML`. Referenciar o plano de implementação sem duplicar suas tarefas.

- [ ] **Step 2: Verificar**

Run: `grep -nE "interpolation|variableName|pg-tok-interpolation|--syntax-interpolation|innerHTML|textarea" docs/superpowers/specs/2026-11-27-rfc-006-realce-de-sintaxe.md`
Expected: todos os termos presentes.

- [ ] **Step 3: Commit**

```bash
git add docs/superpowers/specs/2026-11-27-rfc-006-realce-de-sintaxe.md
git commit -m "docs: especificar contrato compartilhado de realce FERRO"
```

---

### Task 8: Testes de aceite vermelhos

**Files:**
- Create: `tests/fixtures/ferro-syntax-evolution.fer`
- Modify: `crates/ferro-rust-frontend/tests/parser.rs`, `crates/ferro-rust-frontend/tests/semantics.rs`, `crates/ferro-vm/tests/ferro_core.rs`, `crates/ferro-wasm/tests/wasm_facade.rs`, `crates/ferro-service/tests/ferro_rs_runner.rs`
- Testes dos editores: executar somente os Steps 1 e 2 das Tasks 1, 2 e 4 de `docs/superpowers/plans/2026-11-27-ferro-syntax-highlighting.md` (escrever os testes e confirmar a falha); não implementar as features.

**Interfaces:**
- Consumes: RFCs 001–006; `lower(source) -> Result<Program, Vec<Diagnostic>>`; `compile_ferro(&Program) -> Result<Module, FerroCompileError>`; `run(&Module, &mut BufferIo)`; `run_ferro_json(source, input, RunBudget)`; `run_ferro_rs_with_io(source, &AtomicBool, handler, observer)`.
- Produces: programa canônico `tests/fixtures/ferro-syntax-evolution.fer` com entrada `Ada\n` e `stdout` `Olá, Ada!\n`; nenhum teste pode referenciar variantes novas da HIR nesta fase.

- [ ] **Step 1: Criar o fixture canônico**

```ferro
funcao saudar(nome: cadeia) -> cadeia {
    "Olá, {nome}!"
}

funcao inicio() {
    seja nome = leia();
    escreval(saudar(nome));
}
```

- [ ] **Step 2: Adicionar os testes de frontend**

Em `crates/ferro-rust-frontend/tests/parser.rs`:

```rust
#[test]
fn accepts_the_canonical_syntax_evolution_program() {
    let source = include_str!("../../../tests/fixtures/ferro-syntax-evolution.fer");
    let program = lower(source).expect("programa motivador deve compilar no frontend");
    assert_eq!(program.functions.len(), 1);
}
```

Em `crates/ferro-rust-frontend/tests/semantics.rs`:

```rust
#[test]
fn infers_integer_target_for_read_expression_in_annotated_declaration() {
    let source = "funcao inicio() { seja idade: inteiro = leia(); }";
    let program = lower(source).expect("leia() infere o tipo anotado");
    assert_eq!(program.bindings[0].ty, hir::Type::Integer);
}

#[test]
fn rejects_invalid_interpolation_content_at_the_inner_span() {
    let source = r#"funcao inicio() { escreva("{1 +}"); }"#;
    let errors = lower(source).unwrap_err();
    let error = errors.iter().find(|d| d.code == "ferro.interpolation-invalid").unwrap();
    assert_eq!(&source[error.span.start as usize..error.span.end as usize], "1 +");
}

#[test]
fn rejects_trailing_expression_in_a_function_without_return_type() {
    let source = "funcao inicio() { 1 + 1 }";
    let errors = lower(source).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "ferro.implicit-return-without-type"));
}
```

- [ ] **Step 3: Adicionar os testes de backend**

Em `crates/ferro-vm/tests/ferro_core.rs`:

```rust
#[test]
fn executes_canonical_syntax_evolution_program_in_minimal_vm() {
    let source = include_str!("../../../tests/fixtures/ferro-syntax-evolution.fer");
    let module = compile_ferro_source(source);
    let mut io = BufferIo::new("Ada\n");
    run(&module, &mut io).expect("execução do programa motivador na VM");
    assert_eq!(io.output, "Olá, Ada!\n");
}
```

Em `crates/ferro-wasm/tests/wasm_facade.rs`:

```rust
#[test]
fn executes_canonical_syntax_evolution_program_through_wasm_facade() {
    let source = include_str!("../../../tests/fixtures/ferro-syntax-evolution.fer");
    let json = run_ferro_json(source, "Ada\n", RunBudget::default());
    assert_eq!(json, "{\"ok\":true,\"stdout\":\"Olá, Ada!\\n\",\"diagnostics\":[]}");
}
```

Em `crates/ferro-service/tests/ferro_rs_runner.rs`:

```rust
#[test]
fn compiles_and_runs_canonical_syntax_evolution_program_with_real_rustc() {
    let source = include_str!("../../../tests/fixtures/ferro-syntax-evolution.fer");
    let cancel = AtomicBool::new(false);
    let mut requests = Vec::new();
    let result = run_ferro_rs_with_io(
        source,
        &cancel,
        |expected| {
            requests.push(expected.to_owned());
            Ok("Ada".to_owned())
        },
        |_| {},
    )
    .unwrap();
    assert_eq!(requests, ["cadeia"]);
    assert!(
        matches!(result, FerroRustRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "Olá, Ada!\n"),
        "{result:?}"
    );
}
```

- [ ] **Step 4: Rodar os testes e confirmar que falham pela sintaxe ausente**

Para os editores, executar somente os Steps 1 e 2 das Tasks 1, 2 e 4 de `docs/superpowers/plans/2026-11-27-ferro-syntax-highlighting.md` (escrever os testes e confirmar a falha); os Steps 3–5 ficam para a execução daquele plano. O arquivo `ferro-interpolation-highlight.test.mjs` é criado no Step 1 da Task 2 daquele plano.

Run (na raiz):

```bash
cargo test -p ferro-rust-frontend --test parser accepts_the_canonical_syntax_evolution_program
cargo test -p ferro-rust-frontend --test semantics infers_integer_target_for_read_expression_in_annotated_declaration
cargo test -p ferro-vm --test ferro_core executes_canonical_syntax_evolution_program_in_minimal_vm
cargo test -p ferro-wasm --test wasm_facade executes_canonical_syntax_evolution_program_through_wasm_facade
cargo test -p ferro-service --test ferro_rs_runner compiles_and_runs_canonical_syntax_evolution_program_with_real_rustc
node --experimental-strip-types --test apps/desktop/tests/ferro-completions.test.mjs apps/desktop/tests/ferro-interpolation-highlight.test.mjs apps/desktop/tests/web-playground.test.mjs
```

Expected: os cinco comandos `cargo test` falham em `lower()`/exportação (não por erro de compilação); os testes de editor das Tasks 1, 2 e 4 do plano de realce falham por `escreval`/`interpolation` ausentes. Nenhum teste existente passa a falhar: `cargo test -p ferro-vm --test ferro_core reads_typed_input_through_bufferro_io_in_minimal_vm` continua PASS.

- [ ] **Step 5: Commit**

```bash
git add tests/fixtures/ferro-syntax-evolution.fer crates/ferro-rust-frontend/tests/parser.rs crates/ferro-rust-frontend/tests/semantics.rs crates/ferro-vm/tests/ferro_core.rs crates/ferro-wasm/tests/wasm_facade.rs crates/ferro-service/tests/ferro_rs_runner.rs apps/desktop/tests/ferro-completions.test.mjs apps/desktop/tests/ferro-interpolation-highlight.test.mjs apps/desktop/tests/web-playground.test.mjs
git commit -m "test: adicionar aceite vermelho para evolução sintática do FERRO"
```
