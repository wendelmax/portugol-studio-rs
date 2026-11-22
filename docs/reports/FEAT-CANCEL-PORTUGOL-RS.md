# Cancelamento de execução Portugol-RS

## Resultado

`.pors` usa o registro de ID/token de cancelamento já disponível no estado Tauri. Parar e Shift+F7 podem cancelar tanto `rustc` quanto o binário gerado; o serviço retorna o estágio confirmado e captura a saída parcial. A UI mostra o pedido somente após aceite e limpa o ID em todos os caminhos.

A compilação gerada passa a ser supervisionada, limitada a 20 segundos e à saída combinada de 1 MiB. O runtime mantém 3 segundos, teto de saída de 1 MiB e orçamento de strings de 16 MiB. A gramática e o sandbox não mudaram.

## Limites

- Entrada interativa e depuração de `.pors` continuam ausentes.
- `.rs` mantém seu fluxo de consentimento separado; a VM clássica continua usando `RunHandle`.
- CI Linux/macOS/Windows e verificação manual no desktop devem validar o fluxo integrado antes do merge.

## Validação local

- `cargo fmt --all -- --check`
- `cargo test -p portugol-service --offline --locked` — 30 unitários, 11 PORS, 8 Rust standalone e 10 de integração passaram.
- `cargo test -p portugol-desktop --lib --offline --locked` — 37 testes passaram.
- `cd apps/desktop && npm test` — 135 testes passaram.
- `cd apps/desktop && npm run build` — build concluído; Vite avisou que o bundle principal excede 500 kB.
- `cargo clippy -p portugol-service -p portugol-desktop --all-targets --offline --locked -- -D warnings`
- `git diff --check`

O teste focado de cancelamento inicia um worker PORS real, sinaliza cancelamento após 1 segundo e verifica retorno tipado antes do limite de runtime. CI Linux/macOS/Windows e validação manual desktop aguardam PR.
