# Relatório de implementação — FEAT-RUSTFMT-001

- Branch / base: `feat/rustfmt-standalone` / `9b762ed`.
- Escopo: habilitar formatação de Rust standalone no editor com o componente rustfmt existente.
- Estado anterior: `syncLanguageControls()` desabilitava `editor-format` em arquivos `.rs` com a mensagem `Formatador Rust não configurado.`
- Arquitetura preservada: Rust/Tauri 2, TypeScript, CodeMirror e IPC atual.
- Comportamento: comando fixo `rustfmt --quiet --emit stdout --edition 2021`; fonte e configuração vazia ficam em diretório temporário. Saída CRLF do Windows é normalizada para LF conforme o documento CodeMirror. Não altera arquivos de usuário, não usa shell e não usa Cargo.
- Limites: fonte 1 MiB, saída combinada 1 MiB pelo supervisor existente, cinco segundos.
- UI: disponibilidade `rustfmt_available` é detectada independentemente de `rustc`; `.rs` só habilita formatar quando verdadeiro. O retorno só é aplicado se a aba e o documento capturados ainda forem os mesmos; alteração é uma etapa de desfazer.
- Compatibilidade: Portugol mantém `format_source`; Portugol-RS continua sem formatação. Erro da ferramenta não altera o buffer.

## Evidências

- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo test -p portugol-service --test rust_format --offline --locked -- --test-threads=1` — PASS, 4 testes com rustfmt real (Unicode e literal longo, sintaxe inválida e limite de fonte).
- `npm test --prefix apps/desktop` — PASS, 27 arquivos de teste.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo check -p portugol-desktop --offline --locked` — PASS.
- `cargo fmt --all --check` e `git diff --check` — PASS.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo test --workspace --offline --locked -- --test-threads=1` — PASS após liberar rede ao teste existente de serviços Internet.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo clippy --workspace --all-targets --offline --locked -- -D warnings` — PASS.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo fmt --all --check`, `git diff --check` e `cargo check -p portugol-desktop --offline --locked` — PASS.
- `npm run build --prefix apps/desktop` — PASS; Vite mantém aviso do chunk principal acima de 500 kB.
- CI multiplataforma — pendente.
- E2E visual no Tauri — NOT_RUN; o workspace não tem runner para clicar no editor.

## Riscos e rollback

- Sem `rustfmt`, a interface explica como instalar `rustup component add rustfmt` e mantém o controle desabilitado.
- Rollback remove os comandos `format_rust_source`/`rustfmt_available` e a ramificação Rust de `formatCode`; o formatador clássico e sua semântica permanecem.
