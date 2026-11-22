# Confirmar desvinculação de Rust gerado — Implementation Plan

> **For agentic workers:** Execute nativamente nesta worktree com TDD; revisão independente antes de merge.

**Goal:** Pedir confirmação antes de converter Rust vinculado em uma nova aba `.rs` e ignorar confirmações obsoletas.

**Architecture:** Reusar o diálogo nativo `ask`. Capturar snapshot (aba, `EditorState.doc`, artefato) antes do `await`; um helper puro valida o snapshot depois do diálogo antes de criar a aba.

**Tech Stack:** Tauri dialog plugin já instalado, TypeScript, Node `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-22-confirm-rust-detach-design.md`

## Global Constraints

- Não gravar arquivo automaticamente nem modificar fonte/preview.
- Reutilizar `ask`, sem nova dependência ou componente modal.
- Preservar as regras atuais de extensão `.rs` e texto puro.

## Review Focus

- Cancelar não cria aba nem altera conteúdo.
- Trocar aba enquanto `ask` está aberto não cria aba em destino errado.
- Fonte ou artefato mudarem durante o diálogo invalida o snapshot.
- Estado `pending` ou `stale` impede a desvinculação.
- Confirmação válida cria uma aba apenas com código Rust capturado no snapshot.

### Task 1: Guarda de snapshot

**Files:** `apps/desktop/src/generated-rust-detach.ts`, `apps/desktop/tests/generated-rust-detach.test.mjs`.

- [x] RED: testar snapshot válido, aba trocada, `EditorState.doc` trocado, artefato trocado, stale e pending.
- [x] Implementar `generatedRustDetachIsCurrent(snapshot, current) -> boolean` comparando identidade de aba/documento/artefato e estados pending/stale.
- [x] Rodar teste focado RED→GREEN.

### Task 2: Confirmação nativa

**Files:** `apps/desktop/src/main.ts`, `apps/desktop/tests/source-map.test.mjs`.

- [x] Substituir o handler síncrono por async com `ask`, botão “Desvincular e editar” e Cancelar.
- [x] Só criar nova aba depois da confirmação positiva e da guarda de snapshot; rejeitar dados obsoletos com status explicativo quando a aba original ainda está ativa.
- [x] Validar a guarda e os ramos de estado sem simular DOM inexistente; o diálogo Tauri real permanece sem E2E.

### Task 3: Validação e integração

**Files:** ADR 0012, spec/plan e inventário de lacunas.

- [x] Atualizar inventário rastreável UI-017 para fluxo confirmado e origem preservada.
- [x] Rodar `npm test` (167 testes), `npm run build` e `cargo fmt --all --check`; `git diff --check` segue no gate pré-commit. Revisão, PR e CI verde são gates de integração.

## Self-review

- O fluxo responde a cancelamento e mudanças durante `await`; qualquer documento novo continua explicitamente não salvo.
- O E2E do diálogo real segue pendente porque não há runner Tauri/DOM configurado.
