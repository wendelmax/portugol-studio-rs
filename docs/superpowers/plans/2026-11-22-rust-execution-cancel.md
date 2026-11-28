# Plano: cancelar execução Rust standalone

**Goal:** habilitar Interromper/Shift+F7 para `.rs`, encerrando o processo no backend e exibindo cancelamento apenas após confirmação.

**Architecture:** compartilhar o contador de IDs `Runs`; registrar `Arc<AtomicBool>` por run Rust; o supervisor observa o token enquanto compila/executa. A UI registra ID, aguarda resultado, e envia o mesmo ID para `stop`.

**Spec:** `docs/superpowers/specs/2026-11-22-rust-execution-cancel-design.md`; decisão `docs/adr/0004-cancelamento-execucao-rust.md`.

## Task 1 — Serviço de execução cancelável

**Arquivos:** `crates/ferro-service/src/ferro_run.rs`, `crates/ferro-service/src/lib.rs`, `crates/ferro-service/tests/rust_source_runner.rs`.

- [x] Escrever testes reais para cancelar runtime infinito via token e resultado tipado `Cancelled` com fase `Runtime`.
- [x] Cobrir fase de compilação de forma determinística com token pré-cancelado; mantida a decisão de não depender do tempo de compilação do `rustc` real.
- [x] Estender `ProcessLimitReason`/supervisor para observar `AtomicBool`, matar/recolher grupo e coletar stdout/stderr restantes.
- [x] Adicionar `RustSourceRunPhase::{Compilation, Runtime}` e `RustSourceRunResult::Cancelled`.
- [x] Exportar `run_rust_source_with_cancel`; manter `run_rust_source` usando token local falso.
- [x] O teste de runtime falhou inicialmente por API/variante ausentes (RED) e passou após a implementação; suíte completa também passou.

## Task 2 — Estado/IPC Tauri com ID e Stop

**Arquivos:** `apps/desktop/src-tauri/src/lib.rs`, teste de contrato em `apps/desktop/tests/source-map.test.mjs` (ou teste de contrato Rust existente).

- [x] Adicionar contrato frontend/Tauri e testes unitários reais de roteamento por ID e aceite de Stop.
- [x] Adicionar `rust_source_cancellations: Mutex<HashMap<u32, Arc<AtomicBool>>>` a `Runs`.
- [x] Implementar `register_rust_source_run` com o contador `Runs.next_id`; retorna ID.
- [x] Alterar `run_rust_source(id, source)` para buscar token, chamar serviço em `spawn_blocking`, remover registro após retorno/tarefa falha e devolver resultado tipado.
- [x] Alterar `stop(id)` para parar `RunHandle` clássico ou setar token Rust; retornar `bool` de aceite.
- [x] Registrar o comando novo sem ampliar capabilities; 37 testes Tauri passaram.

## Task 3 — Fluxo de execução na UI

**Arquivos:** `apps/desktop/src/api.ts`, `apps/desktop/src/main.ts`, `apps/desktop/tests/source-map.test.mjs` e teste puro pequeno para estado de cancelamento se necessário.

- [x] Adicionar API de registro Rust e assinatura `runRustSource(id, source)`; tipar stop como `Promise<boolean>`.
- [x] Após consentimento, registrar ID, atribuir `runId`, ativar Stop e aguardar compilação/execução.
- [x] Se Stop for aceito, anunciar pedido pendente; “cancelado” só vem de `RustSourceRunResult::Cancelled`.
- [x] Renderizar stdout/stderr capturados e estado terminal sem marcar execução cancelada como sucesso.
- [x] Liberar `runId`, `runTabId` e busy state em todos os caminhos; preservar descarte de respostas obsoletas.
- [x] Cobrir por contrato que o consentimento antecede o registro/execução.

## Task 4 — Validação, reporte e integração

- [x] `cargo fmt --all -- --check`, `cargo test -p ferro-service --offline --locked`, `cargo test -p ferro-desktop --lib --offline --locked`.
- [x] `cd apps/desktop && npm test` (135), `npm run build`, `git diff --check`.
- [x] Atualizar `docs/PARIDADE.md` delimitando `.fer` e `.rs`.
- [x] Criar relatório em `docs/reports/FEAT-RUST-EXECUTION-CANCEL.md`.
- [ ] Commit, PR, aguardar CI Linux/Windows/macOS, corrigir vermelho e mesclar/deletar a branch somente após verde.

## Review focus

- Pedido Stop antes do comando de execução chegar ao serviço ainda cancela a compilação.
- Execução natural que termina ao mesmo tempo que Stop não é rotulada como cancelada sem confirmação.
- Registro é removido após compile error, rustc spawn error, timeout, limite de saída, runtime exit ou cancelamento.
- IDs antigos nunca cancelam uma execução nova; stop em ID inválido retorna `false`.
- Stale-tab/document result não altera Console ou Mensagens da aba ativa.
