# Seleção de alvos Cargo — Implementation Plan

> **For agentic workers:** execute each step with TDD; preserve the existing Tauri architecture and Cargo execution consent.

## Interfaces

- Serviço Cargo retorna lista tipada de binários, cada alvo com `package` e `name`, além de eventual `defaultRun` do pacote.
- A execução recebe um alvo opcional tipado. `None` preserva `cargo run`; `Some(target)` acrescenta `--package` e `--bin` com nomes verificados.
- UI guarda seleção na aba e usa um `<dialog>` nativo acessível para escolha múltipla.

## Steps

- [ ] **1. RED: teste de descoberta de bins** em `crates/portugol-service/tests/cargo_project.rs`: gerar workspace com dois pacotes, três bins e `default-run`; verificar nomes/pacotes e padrão. Rodar teste e confirmar ausência da API.
- [ ] **2. GREEN: implementar `cargo_run_targets_with_cancel`** em `crates/portugol-service/src/cargo_project.rs`: invocar `cargo metadata --no-deps --offline --format-version 1 --manifest-path ...` por supervisor, parsear apenas targets `bin`, preservar ordenação determinística e default-run.
- [ ] **3. RED/GREEN: validar alvo de execução**: testar bin selecionado entre pacotes, bin inexistente e caminho de manifesto symlink. Implementar argumento opcional no runner, comparar seleção à metadata do projeto antes de chamar Cargo e passar argumentos fixos.
- [ ] **4. RED/GREEN: contrato Tauri/API** em `workspace.rs`, `lib.rs` e `api.ts`: expor descoberta via workspace validado e encaminhar alvo tipado ao runner/cancelamento.
- [ ] **5. RED/GREEN: seletor UI** em `index.html`, `main.ts` e testes `apps/desktop/tests/cargo-target-selection.test.mjs`: após o clique em Executar, descobrir metadata sob cancelamento, abrir antes do consentimento quando múltiplos bins não têm padrão, oferecer pacote/bin em labels, reusar seleção da aba e permitir cancelar sem compilar; pedir consentimento antes de build/run.
- [ ] **6. Atualizar documentação**: ADR 0008, Portugol-Rust, paridade e snapshot de pesquisa para a base merged `6fa03ec` e os resultados atuais de teste.
- [ ] **7. Validar**: `cargo fmt --all --check`; serviço test/clippy; desktop tests/clippy quando a quota permitir; `npm test`, frontend build; revisão read-only; PR, CI verde, merge e remoção da branch.

## Review Focus

- Não executar metadata antes de o usuário iniciar a execução Cargo.
- Metadata discovery é offline, não executa código de dependência e rejeita caminhos/manifestos fora da raiz registrada.
- Não selecionar alvos por texto de entrada livre; validar pacote/binário novamente no backend.
- Fechar/cancelar o seletor não registra processo nem grava consentimento.
