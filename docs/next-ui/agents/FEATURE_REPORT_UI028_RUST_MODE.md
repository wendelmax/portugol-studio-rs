# Relatório de implementação — FEAT-LEARN-003

- Branch / base: `feat/ferro-rust-learning-mode` / `264d11c`.
- Escopo: UI-028/023/024/027; UC-05/13/15; FT-02/05/07.
- Estado anterior: `LearningMode` era só `beginner | bridge` e o seletor afirmava que editor e compilador Rust não estavam configurados. A IDE já tinha CodeMirror Rust, abrir/salvar `.rs`, `rust_toolchain_available` e execução real `run_rust_source` com consentimento.
- Arquivos alterados: `apps/desktop/index.html`, `src/main.ts`, `src/learning-mode.ts`, `src/learning-content.ts`, testes de modo e conteúdo, `docs/PARIDADE.md`, matriz de rastreabilidade, gap report, discovery e este relatório.
- Contratos IPC/Rust/Tauri: nenhum contrato ou comando novo. Consome `rust_toolchain_available`, `register_rust_source_run` e `run_rust_source` existentes.
- Permissões/capabilities alteradas: não.
- Compatibilidade: valores desconhecidos de preferência continuam no Beginner; modo Rust só habilita com toolchain detectada. Alternar modo não edita nem troca o arquivo. `.por` e `.fer` preservam o comportamento anterior.
- Interface: modo Rust mostra conceitos de mutabilidade, ownership e borrowing. Em `.rs`, explica o caminho de execução real; em outro tipo de arquivo, orienta abrir/criar `.rs` sem alterar o documento. Exemplos são selecionados manualmente e identificados como editoriais. O laboratório segue rotulado como simulação didática verificada.
- Segurança: o fluxo de execução `.rs` existente continua solicitando consentimento por sessão e informa que não há sandbox; limites de processo permanecem iguais.

## Testes e validação

- `cd apps/desktop && npm test` — PASS, 25 test suites/arquivos.
- `cd apps/desktop && npm run build` — PASS, TypeScript e Vite; permanece o aviso Vite de chunk IDE acima de 500 kB.
- `cargo fmt --all --check` — PASS.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo test --workspace --offline --locked` — PASS, inclui testes de crates, corpus e doctests. A primeira tentativa sem acesso de rede falhou somente no teste Internet; com rede autorizada, o teste isolado e a suíte completa passaram.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo clippy --workspace --all-targets --offline --locked -- -D warnings` — PASS.
- `git diff --check` — PASS.
- E2E visual da janela Tauri — NOT_RUN; não há runner de navegação para esse fluxo.
- Smoke manual de inicialização Tauri em KDE/Wayland — app iniciou com Vite local em `localhost:5175`; GTK registrou aviso sobre titlebar customizada, sem encerrar o app. A interação no seletor não foi automatizada.

## Limitações e próximos passos

- Preferência continua global em `localStorage` até existir `WorkspaceStore`.
- Formatação, argumentos e depuração Rust continuam ausentes.
- Revisão docente do novo texto de borrowing e screenshot pareado ainda pendentes.

## Rollback

Retirar o terceiro valor de modo e a seção de conceitos Rust não altera os recursos `.rs` existentes nem o executor Rust.
