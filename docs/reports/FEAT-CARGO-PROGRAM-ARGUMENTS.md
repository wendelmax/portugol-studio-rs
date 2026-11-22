# FEAT — Argumentos de programa em projetos Cargo

- Requisito: executar binários Cargo selecionados com parâmetros de `argv` do programa.
- Implementação: habilita `Parâmetros de execução` para `.rs` em Cargo e reutiliza a notação já usada por Rust standalone. A UI envia `string[]`; o serviço Rust valida o tamanho/quantidade antes de descobrir alvos com `cargo metadata` e antes de executar `cargo run ... -- <argv>`, sem shell. O build Cargo não recebe esses argumentos. APIs Rust anteriores permanecem como wrappers sem argumentos.
- Segurança/limites: máximo de 256 argumentos; NUL rejeitado; bytes de argumentos limitados a 64 KiB em Linux/macOS e 8 KiB em Windows. Nenhuma flag arbitrária de Cargo é recebida pelo campo. `.pors` continua sem suporte a argumentos.

## Validação

- RED→GREEN: teste de integração com binário Cargo real; vazio, Unicode, espaços, aspas e string com metacaracteres chegam intactos a `std::env::args()`.
- Testes de NUL, quantidade máxima e excesso de bytes passam sem invocar Cargo, tanto na descoberta de alvos quanto na execução; manifestos deliberadamente inválidos confirmam que a validação vem antes de Cargo.
- `cargo test -p portugol-service --offline --locked` — PASS (inclui 14 testes Cargo).
- `cargo clippy -p portugol-service --all-targets --offline --locked -- -D warnings` — PASS.
- `cargo fmt --all -- --check` e `git diff --check` — PASS.
- `npm test` — PASS (36/36).
- `npm run build` — PASS; mantém o aviso conhecido do bundle IDE acima de 500 kB.
- `cargo check -p portugol-desktop --offline --locked` — PASS.
- `cargo test -p portugol-desktop --offline --locked --lib` — PASS (42/42).
- E2E visual/DOM Tauri — `NOT_RUN`; não há runner configurado no repositório. A execução Cargo foi exercitada por teste de integração Rust real.
