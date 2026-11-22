# ADR 0008 — Selecionar binários em projetos Cargo

- **Status:** proposta para implementação
- **Data:** 2026-11-22

## Contexto

O fluxo atual executa `cargo run` no manifesto do workspace. Projetos com mais de um binário e sem `default-run` não conseguem iniciar, embora cada binário seja executável. A IDE já diferencia projetos Cargo de arquivos Rust standalone e solicita consentimento antes de build, macros ou execução.

## Decisão proposta

Descobrir alvos binários com `cargo metadata --no-deps --offline` apenas após a pessoa iniciar uma execução Cargo. Preservar o comportamento atual quando Cargo tem um alvo padrão. Quando há vários binários sem padrão explícito, mostrar uma escolha acessível com pacote e nome do binário; a escolha fica associada à aba/workspace durante a sessão. Invocar Cargo com `--package` e `--bin` usando valores obtidos do metadata, nunca texto livre ou shell.

## Consequências

- Projetos Cargo com múltiplos executáveis passam a poder ser executados pela IDE.
- A IDE deve distinguir ausência de binários, alvo padrão existente e escolha necessária.
- A metadata discovery usa modo offline e não executa build scripts; a execução continua coberta pelo consentimento Cargo existente.
- Uma execução explicitamente selecionada usa o pacote/binário resolvido. Argumentos do programa são enviados somente depois de `--`; o campo não aceita flags do próprio Cargo.
