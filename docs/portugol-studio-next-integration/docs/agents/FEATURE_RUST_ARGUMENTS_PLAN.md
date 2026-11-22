# Argumentos para programas Rust — plano

> **For agentic workers:** execução nativa tarefa por tarefa, TDD e revisão final.

**Goal:** encaminhar argumentos digitados na IDE como elementos `argv` de programas `.rs`.

**Architecture:** parser puro no frontend; array JSON tipado pelo IPC Tauri; validação no serviço Rust; aplicação com `std::process::Command::args`.

**Tech Stack:** TypeScript, Node `node:test`, Rust std, Tauri v2.

**Spec:** `docs/portugol-studio-next-integration/docs/agents/FEATURE_RUST_ARGUMENTS_DESIGN.md`.

## Restrições globais

- Nunca executar um shell ou aceitar nome de programa/comando do frontend.
- Executável é sempre o binário temporário compilado pelo `rustc` fixo.
- Limites do serviço: no máximo 256 argumentos e 64 KiB UTF-8 totais; NUL é inválido.
- Args não alteram fonte, extensão, consentimento, diretório temporário ou limites de execução.
- Array vazio deve manter compatibilidade.

## Foco de revisão

- Espaços/Unicode em token → preservação byte a byte no argumento recebido.
- `""` → argumento vazio preservado.
- `\"` e `\\` em token → aspas e barra literal, sem perder separação.
- Aspa não fechada → erro antes de pedir consentimento/iniciar processo.
- 257 tokens ou >64 KiB → rejeição backend independentemente do frontend.

## Arquivos

- Criar `apps/desktop/src/process-arguments.ts` e `apps/desktop/tests/process-arguments.test.mjs`.
- Modificar `apps/desktop/src/main.ts`, `src/api.ts`, `tests/source-map.test.mjs` e `tests/runtime-capabilities.test.mjs` apenas se necessário para o controle.
- Modificar `apps/desktop/src-tauri/src/lib.rs` para receber `Vec<String>` e encaminhar ao serviço.
- Modificar `crates/portugol-service/src/pors_run.rs`, `src/lib.rs` e `tests/rust_source_runner.rs` para validar argumentos e testar `std::env::args()`.
- Atualizar `docs/PARIDADE.md`, `docs/portugol-studio-next-integration/docs/quality/TRACEABILITY_MATRIX.yaml` e este relatório.

## Tarefas

### 1. Parser de tokens

- [x] Testar divisão por whitespace, aspas duplas, token vazio, `\"`, `\\` e erro de aspas não fechadas.
- [x] Executar teste Node focado e ver falha de módulo/comportamento ausente.
- [x] Implementar `parseProcessArguments(text: string): string[]` sem shell ou expansão.
- [x] Executar testes focados.

### 2. Serviço Rust e IPC

- [x] Testar end-to-end no serviço que `std::env::args()` recebe argumentos com espaços/Unicode/aspas corretamente.
- [x] Testar rejeição de NUL, 257 itens e total >64 KiB.
- [x] Executar `cargo test -p portugol-service --test rust_source_runner --offline --locked` para ver falha esperada antes da implementação.
- [x] Implementar funções com cancelamento compatíveis com wrappers existentes, validar limites e aplicar `.args(arguments)` somente ao binário.
- [x] Conectar `Vec<String>` ao comando Tauri e ao payload frontend.
- [x] Executar testes focados e Tauri app check.

### 3. UI, evidência e revisão

- [x] Habilitar campo de argumentos para Rust com sintaxe e erros de parse claros; manter Portugol-RS desabilitado.
- [x] Testar controle e API com cobertura frontend.
- [x] Atualizar paridade, matriz e relatório; registrar ausência de execução por shell.
- [x] Executar frontend, build, fmt, workspace tests, Clippy, Tauri check e `git diff --check`.
- [x] Revisar diffs de segurança, compatibilidade e limites.
