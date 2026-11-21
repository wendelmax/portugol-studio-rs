# Relatório de implementação — FEAT-LANG-002

- Branch / base: `feat/portugol-rs-frontend` sobre `main` (`70b8c99`, merge do PR #39).
- Escopo: primeiro subset de Portugol-RS, lowering direto à HIR e geração `.pors` → Rust por API Rust. Sem UI/IPC/Tauri nesta fatia.
- Arquitetura anterior: `portugol-hir` e `portugol-rust::export` já existiam após PR #39; não havia parser Portugol-RS nem API de exportação do dialeto.
- Arquivos alterados: `Cargo.toml`, `Cargo.lock`, `crates/portugol-rs-frontend/**`, `crates/portugol-rust/Cargo.toml`, `crates/portugol-rust/src/lib.rs`, design, plano, plano de evolução e este relatório.
- API: `portugol_rs_frontend::lower(&str) -> Result<Program, Vec<Diagnostic>>`; `portugol_rust::export_portugol_rs(&str) -> Result<String, Vec<Diagnostic>>`.
- Gramática implementada: `funcao inicio()`, blocos, `seja` imutável, `seja mutavel`, tipos primitivos, inicializador obrigatório, atribuições compatíveis, `escreva`, `se/senao`, expressões tipadas. Spans são bytes UTF-8; diagnósticos `pors.*` cobrem erro lexical/sintático/semântico e construções não suportadas.
- Compatibilidade: parser, semântica, VM, extensão `.por` e API `export` clássica preservados.
- Extensão `.pors`: definida como extensão proposta, ainda não registrada em abrir/salvar/associação do Tauri, CodeMirror ou CLI. O subset ainda não é uma integração de arquivo end-to-end.
- Testes executados:
  - `cargo test -p portugol-rs-frontend -p portugol-rust --offline --locked` — PASS (36 testes; inclui execução de Rust gerado pelo exemplo `idade`, que imprime `17`).
  - `CARGO_TARGET_DIR=/home/wendel/.codex/worktrees/fb26/portugol-studio-rs/target cargo test --offline --locked` — PASS nos membros padrão, inclui corpus clássico, frontend e exportador.
  - `cargo clippy -p portugol-rs-frontend -p portugol-rust --offline --all-targets -- -D warnings` — PASS.
  - `cargo fmt --all --check` e `git diff --check` — PASS.
  - `cargo test --workspace` incluindo Tauri — não executado nesta fatia; neste ambiente falta `javascriptcoregtk-4.1.pc`/WebKitGTK 4.1, limitação confirmada na fatia anterior. A CI do PR fará o gate completo.
- UI/E2E/screenshot: NOT_RUN; nenhuma tela ou contrato Tauri foi alterado.
- Limitações: sem loops, funções adicionais, parâmetros/retornos, referências/ownership, vetores, bibliotecas, execução `.pors` na VM ou file-open/save UI. Declarações precisam de inicializador. Esses casos retornam diagnóstico ou seguem pendentes, nunca simulam suporte.
- Rollback: remover `portugol-rs-frontend`, sua dependência e API `export_portugol_rs`; o frontend e exportação clássicos não dependem do novo crate.
- Próxima fatia: integrar detecção de linguagem e documentos `.pors` no serviço de arquivos/CLI/IDE, proteger buffers dirty e configurar CodeMirror; depois ampliar subset e execução real.
