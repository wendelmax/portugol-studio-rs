# Relatório de implementação — FEAT-RUST-ARGS-001

- Branch / base: `feat/rust-program-arguments` / `808a866`.
- Escopo e IDs: UI-003/021/028; UC-06; FT-03.
- Estado anterior: Rust standalone usava o executor real, mas `run_rust_source` recebia apenas fonte; campo de parâmetros era desabilitado para `.rs`.
- Arquivos alterados: `apps/desktop/index.html`, `src/main.ts`, `src/api.ts`, parser novo `src/process-arguments.ts`, testes Node do parser, cobertura `source-map.test.mjs`/`learning-mode.test.mjs`; `apps/desktop/src-tauri/src/lib.rs`; `crates/portugol-service/src/pors_run.rs`, `src/lib.rs`, testes `rust_source_runner.rs`; `docs/PARIDADE.md`, matriz, gap report, design/plano e este relatório.
- Contrato Rust/Tauri/IPC: `run_rust_source` agora recebe `args: Vec<String>` e chama `run_rust_source_with_args_and_cancel`. Args opcionais no frontend, limitados a 256 itens, 64 KiB UTF-8 acumulados em Linux/macOS e 8 KiB no Windows para respeitar o limite de linha de comando, e sem NUL.
- Segurança: `Command::args` passa cada item como `argv`; nenhum shell é iniciado. `rustc` continua fixo e argumentos só são enviados ao binário Rust compilado.
- Permissões/capabilities: nenhuma alteração.
- Compatibilidade: `.por` mantém parser atual; `.pors` continua com argumentos desabilitados. Rust sem argumentos usa array vazio e preserva execução anterior.
- Parser UI: whitespace separa itens fora de aspas duplas; `""` cria item vazio; `\"`, `\\` e barra antes de whitespace escapam caracteres. Aspas não fechadas mostram erro antes do consentimento.
- Consentimento, cancelamento, limites de execução, captura de saída e diagnósticos existentes continuam no mesmo fluxo.

## Testes

- `cd apps/desktop && npm test` — PASS, 26 suítes.
- `cd apps/desktop && npm run build` — PASS; Vite avisa que o chunk principal excede 500 kB.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo test --workspace --offline --locked` — PASS com acesso de rede autorizado; um teste de bibliotecas Internet/serviços web não pode passar no sandbox sem rede.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo test -p portugol-service --test rust_source_runner --offline --locked` — PASS, 12 testes, incluindo `argv` literal e rejeição de limites.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo check -p portugol-desktop --offline --locked` — PASS.
- `cargo fmt --all --check`, `git diff --check` e `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` — PASS.
- E2E visual/interação na janela Tauri — NOT_RUN; sem runner para preencher o campo.

## Limitações e rollback

- Aspas simples não são delimitadores; não há glob, expansão de variável, pipe, redirecionamento ou shell.
- Rollback remove `args` do contrato Tauri e volta a desabilitar o campo `.rs`; wrappers de execução sem argumentos permanecem disponíveis.

## Próxima fatia

Rust formatting e debugger permanecem pendentes; seleção de arquivos Cargo/projeto é uma decisão separada.
