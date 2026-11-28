# ADR 0013 — Argumentos do programa em projetos Cargo

- **Status:** aceito
- **Data:** 2026-11-22

## Contexto

O campo de parâmetros já converte uma notação simples em `argv` para `.rs` standalone, sem shell. Arquivos Rust dentro de projetos Cargo desativam o campo e executam sem argumentos, embora o programa compilado possa depender de `std::env::args()`.

## Decisão

Habilitar o mesmo campo para `.rs` standalone e Cargo. Reutilizar `parseProcessArguments`, validar no serviço Rust os limites já aplicados a programas Rust (256 itens, rejeição de NUL, 64 KiB em Linux/macOS e 8 KiB em Windows), e encaminhar os itens com `Command::args` após `--` em `cargo run`. A descoberta de alvos via `cargo metadata` não recebe nem valida argumentos do programa. Não aceitar flags de Cargo ou shell do campo.

## Consequências

- Parâmetros com espaços, Unicode, aspas e valor vazio chegam como argumentos separados ao programa.
- Projeto Cargo continua usando consentimento e supervisão existentes; o build não recebe os argumentos do programa.
- pseudocódigo clássico e Ferro mantêm seus próprios contratos de entrada/parâmetros.
