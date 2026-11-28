# FERRO Examples Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fazer `leia()` converter implicitamente ao tipo esperado e migrar todas as superfícies de exemplos FERRO para a sintaxe evoluída, com validação e documentação coerente.

**Architecture:** O HIR `Read` passa a carregar um tipo de destino opcional; o frontend preenche esse tipo a partir do contexto já disponível, o bytecode emite `Op::Read(tipo)` e a VM reaproveita `parse_input` para erro de execução. Um par de testes de aceite (Rust + Node) compila os 211 `.fer` e varre fontes embutidos com allowlist explícita; os exemplos são migrados em lotes e a documentação normativa é atualizada no mesmo PR.

**Tech Stack:** Rust 2024 (workspace Cargo), Node 22 (`node --test`, TypeScript via `--experimental-strip-types`), Markdown/HTML.

**Spec:** `docs/superpowers/specs/2026-11-27-ferro-examples-migration-design.md`

## Global Constraints

- 211 arquivos `.fer`: 77 em `apps/desktop/src/help-assets/recursos/exemplos-ferro`, 133 em `crates/ferro-service/assets/exemplos-ferro`, 1 em `tests/fixtures`.
- `leia()` sem contexto tipado retorna `cadeia`; com contexto tipado converte para `inteiro`, `real`, `caracter` ou `logico`.
- `leia(variavel)` continua válido e converte ao tipo declarado da variável; exemplos migrados preferem `leia()` (`seja x: tipo = leia();` ou `x = leia();`), com `leia(variavel)` reservado a compatibilidade e laços com leitura repetida.
- Entrada que não converte produz erro de execução claro; nunca valor padrão silencioso.
- `Tipos.para_*` + `Resultado` permanece obrigatório apenas em exemplos de campo de texto (`UI.entrada`).
- RFC 004: somente `cadeia` e `caracter` podem ser interpolados; números, lógicos, vetores, registros, enums e `vazio` geram `ferro.interpolation-non-string`.
- `escreval` é builtin não reservado.
- Preservar stdout exato de 13 exemplos nativos, 7 clássicos, 14 exercícios (44 casos) e do fixture canônico (`Olá, Ada!\n`).
- Não alterar `apps/desktop/package-lock.json`.
- Não tocar nos PRs #248, #246 e #247; abrir exatamente um PR contra `main` a partir de `ferro-examples-migration`.

## Review Focus

1. `leia()` em contexto sem tipo (`escreval(leia())`) ainda deve ser `cadeia` — teste `keeps_untyped_read_expression_as_cadeia` na Task 1.
2. `leia()` tipado com entrada inválida deve falhar na execução com mensagem do tipo esperado — teste `reports_invalid_input_for_a_typed_read_expression` na Task 2.
3. `leia(variavel)` ainda deve funcionar e os exemplos devem preferir `leia()` — teste `parses_reading_into_a_mutable_binding` na Task 1 e varredura `leia(` na Task 4.
4. RFC 007/008 não podem interpol valores não textuais e precisam do aviso de não execução no `main` — verificação `docs_no_invalid_interpolation_and_warning` na Task 4 e edições na Task 9.
5. A migração não pode mudar stdout dos exemplos/exercícios — testes `every_ferro_example_parses_formats_compiles_and_executes_with_expected_behavior` (Task 5) e `evaluates_all_builtin_ferro_exercises_with_passing_and_failing_solutions` (Task 8).
6. `escreval` continua utilizável como identificador — teste `escreval_can_still_be_used_as_an_identifier` preservado na Task 9.

---

### Task 1: HIR e frontend: `leia()` com tipo implícito

**Files:**
- Modify: `crates/ferro-hir/src/lib.rs:392-393` (variant `Read`),`:116-146` (helper em `impl Type`)
- Modify: `crates/ferro-rust-frontend/src/parser/expr.rs:137-206` (braço `leia()` e helper)
- Modify: `crates/ferro-rust/src/lib.rs:268`, `crates/ferro-rust/src/codegen/types.rs:103,500`, `crates/ferro-service/src/rename.rs:303,742` (ajuste mecânico de pattern)
- Test: `crates/ferro-rust-frontend/tests/semantics.rs:399-422`

**Interfaces:**
- Consumes: `Parser::expression_with_expected(min_precedence, expected)` já existente em `expr.rs:13-18`.
- Produces: `hir::ExprKind::Read { target: Option<hir::Type> }`, `hir::Type::is_implicit_read_target() -> bool`. `Expr.ty` é `target.unwrap_or(Type::String)`. `target: None` preserve o comportamento atual.

- [ ] **Step 1: Escrever os testes que falham**

Em `crates/ferro-rust-frontend/tests/semantics.rs`, adicionar `use ferro_hir::{self as hir, ExprKind, Type};`, remover `rejects_read_expression_in_an_integer_context_at_its_span` e escrever:

```rust
#[test]
fn typed_read_expression_is_accepted_in_annotated_scalar_initializers() {
    for (type_name, expected) in [
        ("inteiro", Type::Integer),
        ("real", Type::Real),
        ("caracter", Type::Character),
        ("logico", Type::Boolean),
    ] {
        let source = format!("funcao inicio() {{ seja n: {type_name} = leia(); }}");
        let program = lower(&source).unwrap_or_else(|e| panic!("{type_name}: {e:?}"));
        let hir::StmtKind::Declare { initializer: Some(expr), .. } = &program.body[0].kind else {
            panic!("initializer ausente para {type_name}");
        };
        assert_eq!(expr.ty, expected);
        assert!(matches!(expr.kind, ExprKind::Read { target: Some(_) }));
    }
}

#[test]
fn typed_read_expression_is_accepted_as_typed_function_argument() {
    let source = "funcao f(n: inteiro) -> inteiro { retorne n; } funcao inicio() { escreval(f(leia())); }";
    assert!(lower(source).is_ok(), "{source}");
}

#[test]
fn keeps_untyped_read_expression_as_cadeia() {
    let source = "funcao inicio() { escreval(leia()); }";
    let program = lower(source).unwrap();
    let hir::StmtKind::Write(values) = &program.body[0].kind else { panic!("Write ausente") };
    assert_eq!(values[0].ty, Type::String);
    assert!(matches!(values[0].kind, ExprKind::Read { target: None }));
}

#[test]
fn keeps_explicit_tipos_conversion_as_resultado() {
    let source = "funcao inicio() { escolha Tipos.para_inteiro(leia()) { caso Resultado::Ok(n) => { escreval(n); } caso Resultado::Erro(_) => { escreval(\"erro\"); } } }";
    assert!(lower(source).is_ok());
}
```

- [ ] **Step 2: Rodar os testes para confirmar a falha**

Run: `cargo test -p ferro-rust-frontend --test semantics typed_read_expression -- --nocapture`
Expected: FAIL com `ferro.type-mismatch` / `ferro.call-type-mismatch`.

- [ ] **Step 3: Implementar a mudança de HIR e parser**

Em `ferro-hir/src/lib.rs`, trocar `Read` por `Read { target: Option<Type> }` e adicionar `Type::is_implicit_read_target()`, que aceita somente `Integer`, `Real`, `Boolean` e `Character`.

Em `parser/expr.rs`, adicionar `fn implicit_read_target(expected: Option<hir::Type>) -> Option<hir::Type> { expected.filter(|ty| ty.is_implicit_read_target()) }` e, no braço `leia()` (linhas 177-206), usar esse helper para construir `ExprKind::Read { target }` com `ty: target.unwrap_or(Type::String)`. Manter o diagnóstico `ferro.read-arguments-unsupported`.

Nos quatro arquivos listados em **Files**, trocar os patterns `ExprKind::Read` por `ExprKind::Read { .. }` apenas para restaurar a compilação; comportamento de codegen vem na Task 3.

- [ ] **Step 4: Rodar os testes do frontend e do workspace**

Run: `cargo test -p ferro-rust-frontend --test semantics && cargo test -p ferro-rust-frontend --test parser && cargo test`
Expected: PASS; `read_expression_rejects_arguments` e `escreval_can_still_be_used_as_an_identifier` continuam passando.

- [ ] **Step 5: Commit**

```bash
git add crates/ferro-hir/src/lib.rs crates/ferro-rust-frontend/src/parser/expr.rs crates/ferro-rust-frontend/tests/semantics.rs crates/ferro-rust/src/lib.rs crates/ferro-rust/src/codegen/types.rs crates/ferro-service/src/rename.rs
git commit -m "feat: infere tipo de leia() pelo contexto"
```

---

### Task 2: Bytecode e VM: emissão e erro de execução

**Files:**
- Modify: `crates/ferro-bytecode/src/ferro.rs:788-794`
- Test: `crates/ferro-bytecode/tests/ferro_compiler.rs:268-278`
- Test: `crates/ferro-vm/tests/ferro_core.rs:1015-1022`

**Interfaces:**
- Consumes: `hir::ExprKind::Read { target }` da Task 1; `ferro_bytecode::value_type_hir()` em `ferro.rs:1060`.
- Produces: `Op::Read(ValueType)` com o tipo do destino; nenhuma mudança de API pública.

- [ ] **Step 1: Escrever os testes que falham**

Em `ferro_compiler.rs`:

```rust
#[test]
fn compiles_typed_read_expression_with_the_expected_value_type() {
    let source = "funcao inicio() { seja n: inteiro = leia(); }";
    let program = ferro_rs_frontend::lower(source).expect("fonte válida");
    let module = ferro_bytecode::compile_ferro(&program).expect("bytecode válido");
    let entry = &module.functions[module.entry as usize];
    assert!(entry.code.iter().any(|op| matches!(
        op,
        ferro_bytecode::Op::Read(ferro_bytecode::ValueType::Inteiro)
    )));
    assert!(entry.nodes.iter().flatten().any(|n| n.node == "Read"));
}
```

Em `ferro_core.rs`:

```rust
#[test]
fn reads_typed_expressions_without_a_target_binding_in_minimal_vm() {
    for (source, input, expected) in [
        (
            "funcao inicio() { seja n: inteiro = leia(); escreval(n + 1); }",
            "36\n",
            "37\n",
        ),
        (
            "funcao inicio() { seja n: real = leia(); escreval(n + 1.0); }",
            "1.5\n",
            "2.5\n",
        ),
        (
            "funcao inicio() { seja n: caracter = leia(); escreval(n); }",
            "A\n",
            "A\n",
        ),
        (
            "funcao inicio() { seja n: logico = leia(); escreval(n); }",
            "verdadeiro\n",
            "verdadeiro\n",
        ),
    ] {
        let module = compile_ferro_source(source);
        let mut io = BufferIo::new(input);
        run(&module, &mut io).expect("leitura tipada na VM mínima");
        assert_eq!(io.output, expected);
    }
}

#[test]
fn reports_invalid_input_for_a_typed_read_expression() {
    let module = compile_ferro_source("funcao inicio() { seja n: inteiro = leia(); escreval(n); }");
    let mut io = BufferIo::new("abc\n");
    let err = run(&module, &mut io).expect_err("entrada inválida deve falhar");
    assert!(format!("{err:?}").contains("inteiro"), "{err:?}");
}
```

- [ ] **Step 2: Rodar os testes para confirmar a falha**

Run: `cargo test -p ferro-bytecode --test ferro_compiler compiles_typed_read_expression -- --exact && cargo test -p ferro-vm --test ferro_core reports_invalid_input_for_a_typed_read_expression -- --exact`
Expected: FAIL — bytecode emite `Op::Read(Cadeia)` e a execução não falha.

- [ ] **Step 3: Emitir `Op::Read` com o tipo do destino**

Substituir o braço `E::Read` em `crates/ferro-bytecode/src/ferro.rs` por:

```rust
E::Read { target } => {
    let ty = target.map(value_type_hir).unwrap_or(ValueType::Cadeia);
    self.emit_node(Op::Read(ty), expr.span, mapping("Read", expr.span));
}
```

Não alterar a VM: `parse_input` em `vm.rs:1592-1619` já produz o erro de execução. Completar o teste de sucesso com quatro fontes `seja n: tipo = leia(); escreval(n);` e entradas/stdout correspondentes, sem `?` em `inicio`.

- [ ] **Step 4: Rodar bytecode, VM e job de FERRO**

Run: `cargo test -p ferro-bytecode --test ferro_compiler && cargo test -p ferro-vm --test ferro_core && mise run corpus`
Expected: PASS, incluindo `reads_a_string_expression_without_a_target_binding` e `paused_info_exposes_the_instruction_ip_and_its_hir_mapping`.

- [ ] **Step 5: Commit**

```bash
git add crates/ferro-bytecode/src/ferro.rs crates/ferro-bytecode/tests/ferro_compiler.rs crates/ferro-vm/tests/ferro_core.rs
git commit -m "feat: emite leitura tipada para leia() no bytecode FERRO"
```

---

### Task 3: Backend Rust gerado: leitura tipada

**Files:**
- Modify: `crates/ferro-rust/src/codegen/expr.rs:302-309`
- Modify: `crates/ferro-rust/src/codegen/types.rs:100-133`
- Test: `crates/ferro-service/tests/ferro_rs_runner.rs:170-218`

**Interfaces:**
- Consumes: `hir::ExprKind::Read { target }`.
- Produces: `read_target_label(ty: Option<hir::Type>) -> &'static str` em `codegen/types.rs`, reaproveitado pelo gerador; protocolo `FERRO_INPUT:<label>`.

- [ ] **Step 1: Escrever os testes que falham**

```rust
#[test]
fn generated_rust_requests_typed_input_for_read_expression() {
    let cancel = AtomicBool::new(false);
    let mut requests = Vec::new();
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: inteiro = leia(); escreval(n); }",
        &cancel,
        |expected| {
            requests.push(expected.to_owned());
            Ok("36".to_owned())
        },
        |_| {},
    )
    .unwrap();
    assert_eq!(requests, ["inteiro"]);
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "36\n"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_reports_invalid_typed_expression_input() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: inteiro = leia(); escreval(n); }",
        &cancel,
        |_| Ok("abc".to_owned()),
        |_| {},
    )
    .unwrap();
    assert!(
        matches!(
            result,
            FerroRustRunResult::Executed { ref stderr, exit_code: Some(code), .. }
                if code != 0 && stderr.to_lowercase().contains("inteiro")
        ),
        "{result:?}"
    );
}
```

Os imports `FerroRustRunResult`, `run_ferro_rs_with_io` e `AtomicBool` já existem no topo do arquivo; não introduzir novo helper de teste público.

- [ ] **Step 2: Rodar os testes para confirmar a falha**

Run: `cargo test -p ferro-service --test ferro_rs_runner generated_rust_requests_typed_input -- --exact`
Expected: FAIL — gerador pede `cadeia`.

- [ ] **Step 3: Gerar parse tipado**

Em `codegen/expr.rs`, substituir o braço `E::Read` para usar `read_target_label(target)` e gerar `__ferro_parse::<T>(&line, label)` para `Integer`/`Real`/`Character`, tratamento explícito de `verdadeiro`/`falso` para `Boolean`, e manter `__ferro_read_line("cadeia")` com `__ferro_string` quando `target` é `None`/`String`. Em `codegen/types.rs`, atualizar `expression_has_read` para `E::Read { .. }` e adicionar `read_target_label`.

- [ ] **Step 4: Rodar o suite do runner**

Run: `cargo test -p ferro-service --test ferro_rs_runner && cargo test -p ferro-service --test ferro_rs_debugger`
Expected: PASS; `generated_rust_requests_a_string_when_read_is_an_expression` continua válido para o caso sem tipo.

- [ ] **Step 5: Commit**

```bash
git add crates/ferro-rust/src/codegen/expr.rs crates/ferro-rust/src/codegen/types.rs crates/ferro-service/tests/ferro_rs_runner.rs
git commit -m "feat: gera Rust com leitura tipada para leia()"
```

---

### Task 4: Testes de aceite: compilar 211 `.fer` e varrer superfícies

**Files:**
- Create: `crates/ferro-service/tests/ferro_examples_migration.rs`
- Create: `apps/desktop/tests/ferro-examples-migration.test.mjs`
- Create: `tests/fixtures/ferro-migration-allowlist.json`
- Test: os dois arquivos criados

**Interfaces:**
- Consumes: `lower`, `ferro_rs_frontend::format`, `ferro_bytecode::compile_ferro`, `PLAYGROUND_EXAMPLES`, `FERRO_STDLIB_CATALOG`, `FERRO_HELP`, `FERRO_UI_EXAMPLES`, `FERRO_TEMPLATE`.
- Produces: allowlist compartilhada no formato `[{ "path": "<relativo à raiz>", "markers": ["escreva("], "reason": "..." }]`; comandos `cargo test -p ferro-service --test ferro_examples_migration` e `cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs`.

- [ ] **Step 1: Escrever os testes que falham**

No teste Rust: percorrer recursiveamente com `std::fs` (sem nova dependência) os diretórios `apps/desktop/src/help-assets/recursos/exemplos-ferro`, `crates/ferro-service/assets/exemplos-ferro` e `tests/fixtures`; assertar contagens `77`, `133`, `1`; para cada arquivo, `assert!(lower(&source).is_ok())`, `assert!(ferro_rs_frontend::format(&source).is_some())` e `assert!(ferro_bytecode::compile_ferro(&program).is_ok())`; assertar que nenhum contém `programa {` ou `inclua biblioteca`.

No teste Node: ler o JSON de allowlist; importar os arrays de exemplos embutidos e ler os arquivos Markdown/HTML listados no plano; para cada fonte, verificar ausência dos markers `escreva(`, `leia(`, `: inteiro = 0;`, `+ "\\n"` e `Tipos.para_inteiro(leia())`, ignorando apenas entradas da allowlist; verificar que `docs/superpowers/specs/2026-11-27-rfc-007-tratamento-de-erros.md` e `...rfc-008-coalescencia-e-navegacao-segura.md` contêm `"não executam no \`main\`"` e não contêm `"{valor}"`, `"{numero}"`, `"{erro}"`; verificar que `docs/superpowers/plans/2026-11-27-ferro-nucleo-sintaxe.md` não contém `"escreval reservado"` nem `"Adicionar \`escreval\` à lista de reservadas"`.

Criar o JSON com `[]` inicialmente; a allowlist será preenchida com entradas explícitas e motivo durante as Tasks 5-9.

- [ ] **Step 2: Rodar os testes para confirmar a falha**

Run: `cargo test -p ferro-service --test ferro_examples_migration && cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs`
Expected: FAIL nos markers legados, ainda antes da migração dos exemplos.

- [ ] **Step 3: Ajustar o teste para aceitar o estado migrado**

Nenhuma implementação de produção aqui. À medida que os lotes das Tasks 5-9 migrarem, adicionar ao JSON apenas exceções reais: `leia(variavel)` em laços com leitura repetida e trechos de documentação que deliberadamente mostram compatibilidade. Cada entrada precisa de `reason` curto.

- [ ] **Step 4: Rodar os dois testes**

Run: `cargo test -p ferro-service --test ferro_examples_migration && cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs`
Expected: PASS somente depois das Tasks 5-9; durante o desenvolvimento, rodar os comandos após cada lote para medir o progresso.

- [ ] **Step 5: Commit**

```bash
git add crates/ferro-service/tests/ferro_examples_migration.rs apps/desktop/tests/ferro-examples-migration.test.mjs tests/fixtures/ferro-migration-allowlist.json
git commit -m "test: cerca todos os exemplos FERRO contra sintaxe legada"
```

---

### Task 5: Catálogo de serviços: 133 exemplos `.fer`

**Files:**
- Modify: `crates/ferro-service/assets/exemplos-ferro/**/*.fer` (133 arquivos)
- Modify: `tests/fixtures/ferro-migration-allowlist.json` (se restarem `leia(variavel)` em laços)
- Test: `crates/ferro-service/src/ferro_examples.rs:1537-1576`

**Interfaces:**
- Consumes: gate da Task 4 e conversão implícita das Tasks 1-3.
- Produces: exemplos do catálogo na sintaxe evoluída, com stdout inalterado.

- [ ] **Step 1: Rodar o teste de aceite para listar os arquivos pendentes**

Run: `cargo test -p ferro-service --test ferro_examples_migration -- --nocapture`
Expected: FAIL com a lista de arquivos que ainda contêm `escreva(`, `leia(variavel)` ou concatenação de `\n`.

- [ ] **Step 2: Migrar em lotes**

Aplicar, por arquivo: `escreva(..., "\n")` → `escreval(...)`; `seja mutavel x: tipo = literal;` com tipo inferível → `seja mutavel x = literal;` quando seguro, mantendo tipo explícito em vetores vazios e variáveis reatribuídas com tipo diferente; concatenação manual de texto com `+` → interpolação somente quando todos os valores são `cadeia`/`caracter`; `leia(variavel)` → `seja x: tipo = leia();` na declaração ou `x = leia();` em atribuição, exceto em laços com leitura repetida; `Tipos.para_*` de leitura direta removido em favor da conversão implícita.

- [ ] **Step 3: Rodar o teste de comportamento do catálogo**

Run: `cargo test -p ferro-service --lib ferro_examples::tests::every_ferro_example_parses_formats_compiles_and_executes_with_expected_behavior -- --exact`
Expected: PASS com os stdout exatos listados na spec (por exemplo `ferro-07-entrada-tipada` com entrada `Ada\n36\n` produz `Olá, Ada! Próxima idade: 37`).

- [ ] **Step 4: Rodar o gate e o commit parcial**

Run: `cargo test -p ferro-service --test ferro_examples_migration`
Expected: FAIL apenas para as demais superfícies; nenhum arquivo deste lote deve aparecer.

```bash
git add crates/ferro-service/assets/exemplos-ferro tests/fixtures/ferro-migration-allowlist.json
git commit -m "refactor: migra exemplos do catálogo FERRO"
```

---

### Task 6: Ajuda da IDE: 77 `.fer` e blocos HTML

**Files:**
- Modify: `apps/desktop/src/help-assets/recursos/exemplos-ferro/**/*.fer` (77 arquivos)
- Modify: `apps/desktop/src/help-assets/topicos/linguagem_ferro/entrada_saida/{escreva,index,leia}.html`
- Test: `apps/desktop/tests/help-examples-ferro.test.mjs`, `apps/desktop/tests/ferro-examples-migration.test.mjs`

**Interfaces:**
- Consumes: mesmo conjunto de regras da Task 5.
- Produces: ajuda integrada sem `leia(variavel)` para números e sem `escreva` com `\n` explícito.

- [ ] **Step 1: Rodar os testes da ajuda e do gate**

Run: `cd apps/desktop && node --test tests/help-examples-ferro.test.mjs tests/ferro-examples-migration.test.mjs`
Expected: FAIL nos markers dos 77 arquivos e no texto de `leia.html` que orienta manter `leia(variavel)`.

- [ ] **Step 2: Migrar os arquivos e o texto**

Aplicar as regras da Task 5 aos 77 `.fer`. Em `entrada_saida/leia.html`, substituir a orientação “Para ler números, mantenha `leia(variavel)`” pela recomendação `seja n: inteiro = leia();` e documentar que `leia(variavel)` segue válido para laços. Nos 13 blocos inline de `escreva.html`, `index.html` e `leia.html`, trocar `escreva(..., "\n")` por `escreval(...)` e `leia(variavel)` por `leia()` quando não for laço.

- [ ] **Step 3: Rodar os testes da ajuda**

Run: `cd apps/desktop && node --test tests/help-examples-ferro.test.mjs tests/ferro-examples-migration.test.mjs`
Expected: PASS, mantendo as contagens 77 arquivos / 48 tópicos / 77 links.

- [ ] **Step 4: Commit**

```bash
git add apps/desktop/src/help-assets apps/desktop/tests/ferro-examples-migration.test.mjs tests/fixtures/ferro-migration-allowlist.json
git commit -m "refactor: migra exemplos da ajuda FERRO"
```

---

### Task 7: Fontes embutidos da IDE e do playground

**Files:**
- Modify: `apps/desktop/src/ferro-stdlib-catalog.ts` (13 `source:`), `apps/desktop/src/ferro-help.ts` (8), `apps/desktop/src/ferro-ui-examples.ts` (2), `apps/desktop/src/learning-content.ts` (10), `apps/desktop/src/ferro.ts` (11 snippets + docs de `escreva`/`escreval`/`leia`), `apps/desktop/src/tips.ts` (3 SVG), `apps/desktop/src/state/tabs.ts` (template), `apps/desktop/index.html` (1), `apps/web-playground/playground-core.mjs` (4)
- Test: `apps/desktop/tests/ferro-examples-migration.test.mjs`, `apps/desktop/tests/web-playground.test.mjs`, `apps/desktop/tests/ferro-completions.test.mjs`

**Interfaces:**
- Consumes: arrays exportados pelos módulos TS e `PLAYGROUND_EXAMPLES` de `playground-core.mjs`.
- Produces: snippets e exemplos padrão na sintaxe nova; assinaturas de builtin atualizadas em `ferro.ts`.

- [ ] **Step 1: Rodar os testes das superfícies embutidas**

Run: `cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs tests/web-playground.test.mjs tests/ferro-completions.test.mjs`
Expected: FAIL nos `source:` que ainda usam `escreva(`/`leia(variavel)`.

- [ ] **Step 2: Migrar os snippets**

Aplicar as regras da Task 5 aos `source:` completos e aos trechos parciais. Em `ferro.ts`, manter `leia()` documentado como expressão com conversão implícita e `leia(variavel_mutavel)` como forma válida para laços; `escreval` continua `builtin`, nunca reservado. Em `ferro-ui-examples.ts`, preservar `UI.entrada` e o valor bruto `valor: cadeia`; não introduzir `Tipos.para_*` onde o campo é textual.

- [ ] **Step 3: Rodar testes e build**

Run: `cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs tests/web-playground.test.mjs tests/ferro-completions.test.mjs && npm run build`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add apps/desktop/src apps/desktop/index.html apps/web-playground/playground-core.mjs tests/fixtures/ferro-migration-allowlist.json
git commit -m "refactor: migra snippets embutidos da IDE e playground"
```

---

### Task 8: Exercícios e fontes Rust com exemplos inline

**Files:**
- Modify: `crates/ferro-service/assets/exercicios-ferro/*.fer` (14)
- Modify: `crates/ferro-service/src/ferro_exercises.rs:342-406` (soluções inline)
- Modify: `crates/ferro-service/src/ui_session.rs:488,563`, `crates/ferro-service/src/rename.rs:919`, `crates/ferro-service/tests/servico.rs:79-160`
- Modify: `README.md:116`
- Test: `cargo test -p ferro-service --lib ferro_exercises::tests::evaluates_all_builtin_ferro_exercises_with_passing_and_failing_solutions -- --exact`

**Interfaces:**
- Consumes: conversão implícita das Tasks 1-3.
- Produces: exercícios com o mesmo conjunto de saídas esperadas (44 casos).

- [ ] **Step 1: Rodar o teste dos exercícios**

Run: `cargo test -p ferro-service --lib ferro_exercises::tests::evaluates_all_builtin_ferro_exercises_with_passing_and_failing_solutions -- --exact`
Expected: PASS antes da edição; após a edição, continua PASS com as mesmas saídas (`42`, `erro`, `Ada:37`, `siga`, etc.).

- [ ] **Step 2: Migrar exercícios e fontes inline**

Nos 14 `.fer`, trocar `escreva(` por `escreval(` (removendo `"\n"` redundante) e `leia(variavel)` por declaração/atribuição com `leia()`, exceto em laços de leitura repetida. Em `ferro_exercises.rs`, aplicar a mesma regra às strings de solução sem alterar os casos esperados. Em `ui_session.rs`, `rename.rs` e `servico.rs`, atualizar apenas trechos que são exemplos — deixar intactas as strings que existem para testar compatibilidade com sintaxe legada.

- [ ] **Step 3: Atualizar README**

No bloco por volta da linha 116, usar `escreval`, `seja nome = leia();` e interpolação de `cadeia` onde aplicável; manter a descrição de tipos explícitos para casos em que a inferência não se aplica.

- [ ] **Step 4: Rodar os testes de serviço**

Run: `cargo test -p ferro-service --lib ferro_exercises::tests::evaluates_all_builtin_ferro_exercises_with_passing_and_failing_solutions && cargo test -p ferro-service --test servico && cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/ferro-service/assets/exercicios-ferro crates/ferro-service/src/ferro_exercises.rs crates/ferro-service/src/ui_session.rs crates/ferro-service/src/rename.rs crates/ferro-service/tests/servico.rs README.md tests/fixtures/ferro-migration-allowlist.json
git commit -m "refactor: migra exercícios e exemplos inline"
```

---

### Task 9: Documentação normativa: RFCs, ADR, planos e matriz

**Files:**
- Modify: `docs/adr/0019-contrato-minimo-de-tipos-ferro.md:17-18,54-58`
- Modify: `docs/superpowers/specs/2026-11-27-rfc-002-leia-como-expressao.md:11,26,49`
- Modify: `docs/superpowers/specs/2026-11-27-rfc-007-tratamento-de-erros.md:89,101,103,110,135,142,152`
- Modify: `docs/superpowers/specs/2026-11-27-rfc-008-coalescencia-e-navegacao-segura.md:95,107,108,112,125,127-129`
- Modify: `docs/superpowers/specs/2026-11-27-rfc-001-inferencia-em-seja.md:35`, `...rfc-003-escreval.md:52,60`, `...rfc-004-interpolacao-de-cadeias.md:83,97`, `...rfc-005-retorno-implicito.md:13,56,64`, `2026-11-26-ferro-syntax-evolution-design.md:11,24,119,125`, `2026-11-22-ferro-records-enums-design.md:67`, `2026-11-22-ferro-vectors-design.md:11`
- Modify: `docs/superpowers/plans/2026-11-27-ferro-nucleo-sintaxe.md:45,85`, `docs/superpowers/plans/2026-11-27-ferro-contratos-e-testes-de-aceite.md:237`, `docs/adr/0018-projetos-desktop-ferro-com-designer-visual.md:119`, `docs/next-ui/product/LANGUAGE_SEMANTICS_CONTRACT.md:38`
- Modify: `docs/research/ferro_MIGRATION_MATRIX.md`
- Test: `apps/desktop/tests/ferro-examples-migration.test.mjs`

**Interfaces:**
- Consumes: semântica implementada nas Tasks 1-3 e regras de interpolação da RFC 004.
- Produces: documentação sem contradições; matriz registra a diferença de comportamento de `caracter`/`logico` e as exceções da allowlist.

- [ ] **Step 1: Rodar a verificação documental**

Run: `cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs`
Expected: FAIL apontando `escreval reservado`, interpolação de `{valor}`/`{numero}`/`{erro}` e ausência do aviso de não execução.

- [ ] **Step 2: Atualizar RFC 002 e ADR 0019**

Reescrever as afirmações de que `leia()` tem tipo `cadeia` fixo e que `seja n: inteiro = leia();` deve falhar. Registrar que `leia()` infere o tipo do contexto, que `leia(variavel)` continua implícito ao tipo da variável e que entrada inválida gera erro de execução. Na ADR, mover a opção “Tipo de `leia()` dependente do contexto” de rejeitada para aceita nesta fatia, com a ressalva de que o contrato mínimo anterior foi superado.

- [ ] **Step 3: Corrigir RFC 007 e RFC 008**

Substituir interpolações de valores não textuais por argumentos de `escreval`, por exemplo `escreval("Resultado: ", valor);` e `escreval("Falha: ", erro_cadeia);`. Adicionar aviso explícito no início de cada RFC: “Estes exemplos não executam no `main`: `tente`/`capture`/`lance` e a navegação segura ainda não estão implementados.” Manter `Status: Proposta`.

- [ ] **Step 4: Corrigir o plano e os demais documentos**

Em `2026-11-27-ferro-nucleo-sintaxe.md`, trocar “`escreval` reservado” e o passo de adicioná-lo às reservadas por “`escreval` é builtin não reservado, conforme RFC 003”. Migrar os blocos `ferro` restantes nos documentos listados em **Files** para `escreval`, inferência e `leia()`; manter blocos que documentam compatibilidade legada apenas se marcados explicitamente como tal e incluídos na allowlist.

- [ ] **Step 5: Atualizar a matriz**

Em `docs/research/ferro_MIGRATION_MATRIX.md`, registrar: conversão implícita nas duas formas de `leia`, erro de execução em entrada inválida, preferência por `leia()` nos exemplos, exceção `UI.entrada`, e as diferenças observadas em `caracter` (exatamente um caractere) e `logico` (aceita espaços) em relação às antigas leituras tipadas.

- [ ] **Step 6: Rodar a verificação documental e o teste de compatibilidade**

Run: `cd apps/desktop && node --test tests/ferro-examples-migration.test.mjs && cd ../.. && cargo test -p ferro-rust-frontend --test parser escreval_can_still_be_used_as_an_identifier -- --exact`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add docs/adr docs/superpowers docs/research/ferro_MIGRATION_MATRIX.md tests/fixtures/ferro-migration-allowlist.json
git commit -m "docs: alinha RFCs e planos à sintaxe FERRO evoluída"
```

---

### Task 10: Verificação final e PR único

**Files:**
- Nenhum arquivo de código; verificação e publicação.

**Interfaces:**
- Consumes: todos os commits das Tasks 1-9 na branch `ferro-examples-migration`.
- Produces: único PR contra `main`.

- [ ] **Step 1: Rodar a suíte completa**

Run (na raiz):
```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
mise run test
mise run corpus
```
Expected: PASS, sem diffs de formatação e sem warnings.

- [ ] **Step 2: Rodar o frontend e o gate final**

Run:
```bash
cd apps/desktop && npm test && npm run build && node --test tests/ferro-examples-migration.test.mjs
```
Expected: PASS com 0 falhas; `npm run build` não altera arquivos rastreados.

- [ ] **Step 3: Conferir o estado do git**

Run: `git status --short && git diff --stat main...HEAD && git log --oneline main..HEAD`
Expected: nenhuma entrada para `apps/desktop/package-lock.json`; apenas arquivos das Tasks 1-9; nenhuma alteração em PRs #248/#246/#247.

- [ ] **Step 4: Publicar a branch e abrir o PR**

Run:
```bash
git push -u origin ferro-examples-migration
gh pr create --base main --head ferro-examples-migration --title "Migra exemplos FERRO para a sintaxe evoluída" --body "Migra os 211 exemplos .fer e os snippets embutidos, adiciona conversão implícita de leia() ao tipo esperado e corrige RFCs/planos. Inclui gate de compilação e varredura de sintaxe legada."
```
Expected: exatamente um PR aberto; registrar a URL na resposta final.
