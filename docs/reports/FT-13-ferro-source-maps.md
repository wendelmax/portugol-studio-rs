# FT-13 — source maps e diagnósticos do Ferro

## Entrega

- O backend HIR gera Rust junto com segmentos de source map, mantendo os
  exports existentes compatíveis. A API expõe as posições do editor em UTF-16
  para `.por` e `.fer`.
- A execução `.fer` usa `rustc --error-format=json` e retorna um resultado
  discriminado: `executed` com saída/código de retorno, ou `compile_error` com
  diagnósticos estruturados e o stderr bruto de `rustc`.
- Diagnósticos primários usam o menor segmento de origem que contém o span. A
  qualidade `exact`, `approximate` ou `generated_only` explicita a confiança;
  dados JSON malformados são ignorados e o stderr bruto permanece disponível.
- A prévia Rust transforma os trechos mapeados em controles de navegação
  acessíveis, priorizando o menor segmento quando há sobreposição. Mensagens
  compiladas podem voltar ao trecho original; localizações aproximadas são
  rotuladas. Erros sem origem aparecem no console como detalhe
  do Rust gerado. Respostas de execução são descartadas se a aba ou o documento
  mudarem durante a compilação.

## Compatibilidade e limites

O comando Tauri existente `run_ferro_rs` e o export de código continuam com
os mesmos nomes. A rejeição de `rustc` não é mais confundida com falha IPC.
O Ferro ainda compila apenas o subset aceito pelo frontend, requer a
toolchain Rust e não ganha entrada interativa, depurador ou suporte a execução
de arquivos `.rs` com esta entrega.

## Validação

- `cargo test --locked -p ferro-service`: 42 testes (29 unitários, 3 do
  runner e 10 de integração), todos passaram.
- `cargo test --locked -p ferro-rust`: 27 testes passaram, incluindo
  equivalência de saída legada e intervalos de mapeamento.
- `cargo test --locked --workspace --exclude ferro-desktop`: toda a suíte
  passou com acesso ao socket local, incluindo o corpus de regressão.
- `PKG_CONFIG_PATH=/tmp/ft10-pkgconfig cargo check --locked -p ferro-desktop --tests`:
  passou com o metadata local de GTK/WebKit. O `cargo test --workspace` completo
  não conseguiu ligar o binário desktop porque este ambiente não tem as
  bibliotecas WebKitGTK/JavaScriptCoreGTK instaladas; os `.pc` disponíveis são
  placeholders sem bibliotecas.
- `cargo clippy --locked -p ferro-service --all-targets -- -D warnings`:
  passou.
- `cargo fmt --all -- --check` e `git diff --check`: passaram.
- `cd apps/desktop && npm test`: passou nos 15 arquivos de teste, incluindo os
  casos de diagnósticos mapeados, destino sem origem e navegação da prévia.
- Build TypeScript/Vite ainda depende de `@codemirror/lang-rust`, ausente do
  `node_modules` disponível neste ambiente. O build foi tentado e parou na
  resolução desse pacote; o pacote consta nas dependências e no lockfile.
- `tsc --noEmit` passou usando uma declaração temporária só para o módulo Rust
  ausente; o arquivo e o symlink de dependências temporários foram removidos.
