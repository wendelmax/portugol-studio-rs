# Relatório de implementação — FEAT-LANG-002

- Branch / base: `feat/ferro-rust-frontend` sobre `main` (`70b8c99`, merge do PR #39).
- Escopo: primeiro subset de Ferro, lowering direto à HIR e geração `.fer` → Rust por API Rust. Sem UI/IPC/Tauri nesta fatia.
- Arquitetura anterior: `ferro-hir` e `ferro-rust::export` já existiam após PR #39; não havia parser Ferro nem API de exportação do dialeto.
- Arquivos alterados: `Cargo.toml`, `Cargo.lock`, `crates/ferro-rust-frontend/**`, `crates/ferro-rust/Cargo.toml`, `crates/ferro-rust/src/lib.rs`, design, plano, plano de evolução e este relatório.
- API: `ferro_rs_frontend::lower(&str) -> Result<Program, Vec<Diagnostic>>`; `ferro_rust::export_ferro_rs(&str) -> Result<String, Vec<Diagnostic>>`.
- Gramática implementada: `funcao inicio()`, blocos, `seja` imutável, `seja mutavel`, tipos primitivos, inicializador obrigatório, atribuições compatíveis, `escreva`, `se/senao`, expressões tipadas. Spans são bytes UTF-8; diagnósticos `ferro.*` cobrem erro lexical/sintático/semântico e construções não suportadas.
- Compatibilidade: parser, semântica, VM, extensão `.por` e API `export` clássica preservados.
- Extensão `.fer`: definida como extensão proposta, ainda não registrada em abrir/salvar/associação do Tauri, CodeMirror ou CLI. O subset ainda não é uma integração de arquivo end-to-end.
- Testes executados:
  - `cargo test -p ferro-rust-frontend -p ferro-rust --offline --locked` — PASS (36 testes; inclui execução de Rust gerado pelo exemplo `idade`, que imprime `17`).
  - `CARGO_TARGET_DIR=/home/wendel/.codex/worktrees/fb26/ferro-studio/target cargo test --offline --locked` — PASS nos membros padrão, inclui corpus clássico, frontend e exportador.
  - `cargo clippy -p ferro-rust-frontend -p ferro-rust --offline --all-targets -- -D warnings` — PASS.
  - `cargo fmt --all --check` e `git diff --check` — PASS.
  - `cargo test --workspace` incluindo Tauri — não executado nesta fatia; neste ambiente falta `javascriptcoregtk-4.1.pc`/WebKitGTK 4.1, limitação confirmada na fatia anterior. A CI do PR fará o gate completo.
- UI/E2E/screenshot: NOT_RUN; nenhuma tela ou contrato Tauri foi alterado.
- Limitações: sem loops, funções adicionais, parâmetros/retornos, referências/ownership, vetores, bibliotecas, execução `.fer` na VM ou file-open/save UI. Declarações precisam de inicializador. Esses casos retornam diagnóstico ou seguem pendentes, nunca simulam suporte.
- Rollback: remover `ferro-rust-frontend`, sua dependência e API `export_ferro_rs`; o frontend e exportação clássicos não dependem do novo crate.
- Próxima fatia: integrar detecção de linguagem e documentos `.fer` no serviço de arquivos/CLI/IDE, proteger buffers dirty e configurar CodeMirror; depois ampliar subset e execução real.
