# Plano: cancelar execução `.fer`

**Goal:** completar Stop/Shift+F7 para Ferro com cancelamento de compilação e runtime confirmado.

**Design:** `docs/superpowers/specs/2026-11-22-cancel-ferro-design.md`; **ADR:** `docs/adr/0005-cancelamento-execucao-ferro.md`.

## Task 1 — serviço

- [x] Adicionar `Cancelled` e fase ao resultado FERRO e API cancelável.
- [x] Compilar via `spawn` + `supervise` com `COMPILATION_LIMIT` e token.
- [x] Aplicar token ao runtime e manter limites atuais.
- [x] Testar cancelamento e regressões dos executores FERRO/Rust.

## Task 2 — Tauri/UI

- [x] Reutilizar registro Rust-source por ID em `run_ferro_rs`.
- [x] Registrar `.fer` na UI e ativar Stop após o ID.
- [x] Mostrar pedido pendente só após aceite e cancelar apenas após retorno tipado.
- [x] Limpar registros e estado de execução em todos os caminhos.

## Task 3 — validação/integrar

- [x] Atualizar contratos frontend e `docs/PARIDADE.md`.
- [x] Atualizar relatório com cobertura e pendências.
- [x] Rodar fmt, testes service/Tauri/frontend, build, Clippy e diff check.
- [ ] Abrir PR; corrigir CI até verde, mesclar e apagar branch.
