# Contextual Learning from Source Maps Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Explain mutability when the current editor selection maps exactly to a current generated Rust binding.

**Architecture:** A pure frontend helper resolves an editor selection through the active tab's current Rust export source map and classifies only generated `let`/`let mut` bindings. The Learning inspector uses that result as contextual evidence, while manual concept selection remains available and takes precedence until the editor selection changes.

**Tech Stack:** TypeScript, CodeMirror selection/source-map types, Node `node:test`, existing Rust/Tauri export adapter.

**Spec:** `docs/superpowers/specs/2026-11-22-contextual-learning-source-maps.md`

## Global Constraints

- Never analyze Portugol text with a regex to claim semantic concepts.
- Ignore stale artifacts, source revision mismatches, approximate mappings, generated-only mappings and partial selections outside the mapped range.
- Do not change source code, translation semantics, Rust services, Tauri commands or exercise progress.
- Keep manual concept selection and explicit provenance.

## Review Focus

- Overlapping source maps: choose the narrowest exact declaration mapping; cover nested mappings.
- Stale or mismatched source snapshot: return no context and keep manual selection; cover both.
- Approximate/generated-only mappings: return no context; cover both quality variants.
- Manual picker choice while the caret remains on a mapped binding: manual choice stays selected until editor selection changes; cover the event path.
- Classic Portugol vs Portugol-RS: contextual explanation must describe the active source semantics accurately; cover both wording branches.

---

### Task 1: Resolve mapped mutability context

**Files:**
- Create: `apps/desktop/src/learning-context.ts`
- Create: `apps/desktop/tests/learning-context.test.mjs`

**Interfaces:**
- `learningContextAtSelection({ source, selectionFrom, selectionTo, artifact, stale })` returns a source-mapped mutability context or `null`.
- The context contains exact source/generated offsets, snippets, `conceptId: "mutability"`, binding mutability and source-map provenance.

- [x] Write tests for exact mutable/immutable declarations, caret and full-range selection, narrowest overlapping mapping, stale/mismatched revisions, approximate/generated-only maps and selections outside a map.
- [x] Run `node --experimental-strip-types --test tests/learning-context.test.mjs` and verify it fails because the helper is missing.
- [x] Implement the pure helper using only source-backed exact mappings and generated Rust binding syntax.
- [x] Rerun the focused test and verify all cases pass.

### Task 2: Connect the selection context to Learning

**Files:**
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/style.css`
- Test: `apps/desktop/tests/learning-context.test.mjs`

**Interfaces:**
- `Tab.rustPreview` is the only preview source; use the active document snapshot and CodeMirror UTF-16 selection offsets.
- Manual picker choice suppresses automatic context until editor selection changes.
- The contextual panel shows the selected source/Rust binding and provenance without replacing the existing editorial examples.

- [ ] Add regression tests for inspector context wiring, stale refresh, manual-choice precedence and source-language-specific explanation. (A helper test covers dialect wording; UI event wiring still lacks a component harness.)
- [ ] Run the new tests and confirm expected failures before integration. (Not isolated as a separate UI test; helper began with RED.)
- [x] Render contextual Mutability only for an exact current source-map result; keep the manual concept and label when no result exists.
- [x] Refresh context on editor selection/document revisions, tab activation, mode changes and successful export.
- [x] Rerun focused tests.

### Task 3: Verify and report

**Files:**
- Create: `docs/reports/FEAT-CONTEXTUAL-LEARNING.md`

- [x] Run all frontend tests, `npm run build` and `git diff --check`.
- [x] Record the implemented evidence rules, validation and remaining visual-review limits.
- [ ] Commit the isolated feature and open a PR against `main`; merge only after green CI and review.
