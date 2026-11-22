# Exercise Test Panel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Executar no painel Testes os casos reais de um exercício `.pex` ativo e mostrar seu resultado associado à revisão.

**Architecture:** Reutilizar `check_exercise` e `createValidatedExerciseGrader`; manter estado de execução por aba e renderizar estados honestos no painel atual. Resultados obsoletos não atualizam UI nem progresso.

**Tech Stack:** TypeScript, CodeMirror `EditorState`, Tauri command já existente, Node `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-22-exercise-tests-panel-design.md`

## Global Constraints

- Nenhum progresso por abertura de painel.
- Conclusão exige todos os casos reais aprovados para a revisão exata.
- Não adicionar comandos, permissões ou dependências.

## Review Focus

- Arquivo sem exercício não oferece ação de execução nem inventa resultados.
- Editar ou remover a aba invalida resposta pendente e não grava progresso daquela resposta.
- Erro e contagem de casos vêm da resposta real do corretor.

---

### Task 1: Connect exercise cases to the Tests panel

**Files:**
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/src/exercises.ts`
- Create: `apps/desktop/src/exercise-test-panel.ts`
- Test: `apps/desktop/tests/exercise-test-panel.test.mjs`
- Modify: `apps/desktop/tests/tests-panel.test.mjs`
- Modify: `docs/PARIDADE.md`
- Create: `docs/reports/FT-19-exercise-tests-panel.md`

**Interfaces:**
- Consumes: `ExerciseDocument`, `checkExercise`, `createValidatedExerciseGrader`, active tab `EditorState`.
- Produces: panel view model for empty, ready, running, completed, and error states; evaluation API that returns the real result/progress outcome without forcing the modal.

- [x] Write tests for honest empty state and real result summary/failure rendering inputs.
- [x] Add tests for stale revision/tab results being discarded.
- [x] Run tests and verify they fail before the implementation (panel integration assertion failed before adding its markup and renderer).
- [x] Implement the view model, exercise evaluation path, and accessible panel action/results.
- [x] Run focused/full frontend tests, build, and `git diff --check`.
- [x] Update parity and add FT-19 evidence.
- [ ] Commit, review, open PR, fix CI until green, squash-merge, and clean branch/worktree.
