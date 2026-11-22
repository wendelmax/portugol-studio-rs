# Fixar tópico no inspetor — Implementation Plan

> **For agentic workers:** Execute na worktree com TDD e revisão independente antes do merge.

**Goal:** Manter um conceito didático escolhido visível enquanto a pessoa explora outra seleção do código.

**Architecture:** Um helper puro mantém pins por modo; cada aba contém esse estado temporário. O renderer prioriza um pin válido sobre o contexto automático e o botão indica o estado acessível atual.

**Tech Stack:** TypeScript, DOM nativo, Node `node:test`; sem dependência nova ou comando Tauri.

**Spec:** `docs/superpowers/specs/2026-11-22-pin-learning-topic-design.md`

## Global Constraints

- Não persistir pins ao fechar aba nem reiniciar a IDE.
- Não alterar editor, seleção, source maps, fonte, tradução ou progresso.
- “Tópico fixado” indica escolha explícita; não alegar detecção contextual.
- Preservar o fallback atual para contexto mapeado e seleção manual.

## Review Focus

- Desafixar em Bridge deve devolver imediatamente o conceito do source map exato, se existir.
- Pins de `beginner`, `bridge` e `rust` não podem se sobrescrever.
- Trocar de aba deve alternar o pin junto com o documento.
- Mudar o seletor de conceitos enquanto fixado deve atualizar o pin em vez de renderizar o conceito antigo.
- Modo sem a toolchain Rust mantém o comportamento atual e não exibe pin inválido.

### Task 1: Estado puro de pins

**Files:** `apps/desktop/src/learning-pins.ts`, `apps/desktop/tests/learning-pins.test.mjs`.

- [x] RED: testar pin, desafixar o mesmo conceito, substituir por outro e isolamento entre modos.
- [x] Implementar `toggleLearningPin(pins, mode, conceptId)` e `setLearningPin(pins, mode, conceptId)` como funções imutáveis.
- [x] Rodar teste focado.

### Task 2: Estado por aba e comportamento do inspetor

**Files:** `apps/desktop/src/main.ts`, `apps/desktop/index.html`, `apps/desktop/src/style.css`, `apps/desktop/tests/learning-content.test.mjs`.

- [x] Adicionar `learningPins` à aba e priorizar pin válido sobre contexto automático.
- [x] Adicionar botão com `aria-pressed`, rótulo alternável e proveniência “Tópico fixado”.
- [x] Ao trocar o seletor com pin ativo, atualizar o pin do modo atual; ao desafixar, restaurar resolução contextual/manual.
- [x] Adicionar testes de contrato sem alegar que cobrem interação DOM/Tauri.

### Task 3: Validação e integração

**Files:** `docs/research/REPOSITORY_DISCOVERY_CURRENT.md`, este plano.

- [x] Atualizar o estado de UI-023/024.
- [x] Rodar `npm test` (183 testes), `npm run build`, `cargo fmt --all --check` e `git diff --check`.
- [ ] Revisão, PR e merge somente após CI verde; excluir branch remota e atualizar main local.

## Self-review

- Fluxo, persistência fora de escopo, acessibilidade e limites de contexto têm testes e arquivos associados.
- Rollback remove helper, estado por aba e controle sem alterar dados persistidos.
