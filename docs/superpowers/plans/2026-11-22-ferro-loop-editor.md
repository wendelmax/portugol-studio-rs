# FERRO Loop Editor Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax for tracking.

**Goal:** Realçar e sugerir `enquanto` e `para` em arquivos Ferro.

**Architecture:** Atualizar as listas locais de keywords e completions do CodeMirror. Nenhuma alteração ao parser Rust ou ao protocolo Tauri.

**Tech Stack:** TypeScript, CodeMirror 6, Node `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-22-ferro-loop-editor-design.md`

## Global Constraints

- Preserve o editor clássico.
- Só expor no editor construções implementadas pelo dialeto FERRO.
- Não adicionar dependências.

## Review Focus

- `enquanto` e `para` devem ser marcados como keywords no parser de realce FERRO.
- As mesmas palavras devem aparecer no autocomplete FERRO.
- `faca` e `pare` não devem ser sugeridos como suporte inexistente.

---

### Task 1: Add loop syntax to the FERRO editor

**Files:**
- Modify: `apps/desktop/src/ferro.ts`
- Test: `apps/desktop/tests/ferro-editor-loops.test.mjs`

**Interfaces:**
- Consumes: `ferroRust()` CodeMirror support.
- Produces: highlighting and completions for supported loop keywords.

- [x] Write tests for real CodeMirror keyword nodes and autocomplete options.
- [x] Run the tests and confirm they fail because the loop keywords were classified as variables and absent from completions.
- [x] Add `enquanto` and `para` to FERRO keyword and completion lists.
- [x] Run the focused and complete frontend suites plus build (17/17; Vite build passes with the existing chunk-size warning).
- [x] Record delivery and update the FERRO parity note.
- [x] Commit the implementation, tests, plan, and feature report.
- [ ] Open PR, repair CI failures, squash-merge when green, then delete branch/worktree.
