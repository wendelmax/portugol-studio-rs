# Ferro `para` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Aceitar e executar o loop contado `para` no subset Ferro com semântica tipada, escopo local e source maps reais.

**Architecture:** Acrescentar um nó `For` na HIR compartilhada e validar os três campos no frontend FERRO. O gerador emite bloco Rust com inicialização, `while` e atualização pós-corpo, preservando o runner limitado existente.

**Tech Stack:** Rust workspace, `ferro-rust-frontend`, `ferro-hir`, `ferro-rust`, `rustc` runner existente.

**Spec:** `docs/superpowers/specs/2026-11-21-ferro-for-loop-design.md`

## Global Constraints

- Não alterar a execução nem a semântica do pseudocódigo clássico.
- Não adicionar dependências, comandos Tauri ou IPC.
- Usar HIR tipada, binding IDs e source maps, sem substituir palavras-chave.
- Manter os limites atuais do runner FERRO.
- Toda atualização deve atingir o binding mutável declarado no inicializador.

## Review Focus

- Um loop interno que sombreia o binding externo deve resolver e atualizar o binding correto — cobrir nos testes de parser/semântica.
- Resolver/renomear um binding citado no cabeçalho deve incluir todas as referências e não capturar o controle sombreado — cobrir no teste do serviço.
- Uma condição que se torna falsa após a atualização não pode executar uma iteração adicional — cobrir na execução real com `rustc`.
- `/= 0` e `%= 0` no step devem conservar a falha real de runtime, sem erro de tradução — cobrir pelo runner.
- `generated_only` do preâmbulo não pode receber mapeamento de origem do loop — cobrir em teste de invariantes dos mapas.
- Condição, atualização e corpo com UTF-8 devem manter offsets de origem válidos — cobrir no teste de source map.

---

### Task 1: Parser FERRO e HIR para loop contado

**Files:**
- Modify: `crates/ferro-hir/src/lib.rs`
- Modify: `crates/ferro-rust-frontend/src/parser.rs`
- Test: `crates/ferro-rust-frontend/tests/parser.rs`
- Test: `crates/ferro-rust-frontend/tests/semantics.rs`

**Interfaces:**
- Consumes: `Token` e expressões/atribuições tipadas existentes do frontend.
- Produces: `StmtKind::For { initializer: Stmt, condition: Expr, step: Stmt, body: Vec<Stmt> }` com `BindingId` do inicializador presente em `Program.bindings`.

- [x] Escrever teste do parser para inicializador mutável, condição lógica, atualização composta e corpo.
- [x] Rodar `cargo test -p ferro-rust-frontend parses_counted_loop` e confirmar falha por variante `For` ausente na HIR.
- [x] Adicionar variante `For` à HIR e parser dos três campos obrigatórios, com escopo do controle e bloco do corpo.
- [x] Validar tipo numérico, condição lógica, mutabilidade e binding da atualização; reportar erros localizados.
- [x] Adicionar testes para binding fora de escopo, condição inválida, controle imutável, update de binding incorreto, loop aninhado e shadowing.
- [x] Rodar `cargo test --locked -p ferro-rust-frontend` (18 testes passaram); a suíte `ferro-hir` será incluída na validação workspace.
- [x] Ruling: a nova variante HIR exige atualização do visitante de símbolos em `ferro-service/src/rename.rs`; incluído na Task 2 porque ignorar os campos do `For` quebra a renomeação semântica em FERRO.

### Task 2: Gerar Rust, source maps e execução equivalente

**Files:**
- Modify: `crates/ferro-rust/src/codegen.rs`
- Modify: `crates/ferro-rust/src/lib.rs`
- Modify: `crates/ferro-service/src/rename.rs`
- Test: `crates/ferro-service/src/rename.rs`
- Modify: `docs/PARIDADE.md`
- Create: `docs/reports/FT-16-ferro-for-loop.md`

**Interfaces:**
- Consumes: `StmtKind::For` da Task 1.
- Produces: bloco Rust equivalente com mappings exatos para os trechos FERRO mapeáveis.

- [x] Escrever teste de exportação/execução real que espera saída `012` para o loop `i < 3`.
- [x] Rodar o teste focal e confirmar falha por variante HIR ainda não tratada.
- [x] Gerar declaração local, `while` com condição e instrução de atualização dentro do bloco; usar os helpers de overflow do backend.
- [x] Mapear spans de inicializador, condição, atualização, corpo e statement completo sem sobreposição inválida.
- [x] Adicionar teste com loop aninhado/shadowing e fonte UTF-8; confirmar maps e saída.
- [x] Preservar recusa explícita de `Stmt::For` no lowering clássico.
- [x] Percorrer initializer, condition, step e body no visitor semântico de rename; teste valida declaração, atualização e uso dentro do loop.
- [x] Regressão de execução confirma que uma declaração do corpo que sombreia o controle não altera o binding usado pelo step.
- [x] Testar recusa explícita do cabeçalho sem `seja` e falhas reais para `/= 0` e `%= 0`.
- [x] Atualizar paridade e relatório com comando/resultado, limites e semânticas suportadas.
- [x] Rodar `cargo fmt --all -- --check`, `cargo test --locked --workspace --exclude ferro-desktop`, `cargo clippy --locked --workspace --exclude ferro-desktop --all-targets -- -D warnings`, check Tauri com tests e `git diff --check`.

### Task 3: Revisão e integração

- [x] Revisão independente concluiu; achados Important (cabeçalho sem `seja`; escopo do corpo vazando ao step) corrigidos com testes RED→GREEN. Achados Minor de UTF-8/maps, zero-divisores, pipe Markdown e frase sobre bloco vazio também foram fechados.
- [ ] Abrir PR para `main`; aguardar CI multiplataforma verde.
- [ ] Squash-merge e remover a branch remota e a worktree após integração.
