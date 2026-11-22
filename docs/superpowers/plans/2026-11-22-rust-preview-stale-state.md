# Rust Preview Stale State Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve the last generated Rust preview safely while a newer source revision is translating.

**Architecture:** Add a pure state helper for a per-tab artifact containing its source snapshot, Rust source and source map. Integrate that state into the existing tab lifecycle and existing Tauri export requests; keep request-generation checks and never add a second translator.

**Tech Stack:** Rust/Tauri 2, TypeScript, CodeMirror, Node `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-22-rust-preview-stale-state.md`

## Global Constraints

- Preserve the existing app architecture, export commands, file formats and translation semantics.
- Never clear a prior artifact when a newer request starts or fails.
- Do not show stale mappings as if they belonged to the active source.
- Disable copy, save and detach while stale or pending.
- Discard results for inactive tabs or superseded source snapshots.
- Do not add dependencies, Tauri commands, execution claims or runtime capabilities.

## Review Focus

- Switching from one source tab to another during an export must not show the wrong artifact; cover with per-tab state tests and generation checks.
- Editing then failing a translation must keep the old text marked stale; cover both rejected IPC and an export diagnostic.
- Old source maps must not remain clickable while stale; cover the `renderMappingGutter` stale state.
- Copy/save/detach actions must remain disabled until a successful current result; cover markup/action state.
- Editing in beginner mode must still mark an existing bridge artifact stale; cover the document-change path.

---

### Task 1: Add testable per-tab preview state

**Files:**
- Create: `apps/desktop/src/rust-preview-state.ts`
- Create: `apps/desktop/tests/rust-preview-state.test.mjs`

**Interfaces:** `beginRustPreview(state, source)`, `acceptRustPreview(state, source, result)`, and `failRustPreview(state, message)` return immutable state containing an optional artifact `{ source, code, sourceMap }`, `pending`, `stale`, and `error`.

- [x] Write tests for first generation, retaining an artifact when the source changes, accepting fresh output, and preserving stale output on diagnostic or rejected request.
- [x] Run `node --experimental-strip-types --test tests/rust-preview-state.test.mjs` and confirm the expected missing-module failure.
- [x] Implement the pure transition helpers with no Tauri or DOM dependency.
- [x] Rerun the focused state tests.

### Task 2: Integrate preview state with editor tabs

**Files:**
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/src/style.css`
- Test: `apps/desktop/tests/source-map.test.mjs`
- Test: `apps/desktop/tests/design-as-code-shell.test.mjs`

**Interfaces:** Add optional `rustPreview` state to `Tab`; render a tab's artifact before scheduling its next export; accept a result only if generation, active tab and source snapshot still match. Use the existing export adapter.

- [x] Write static regression checks for stale labeling, per-tab state rendering, and hidden/disabled stale actions.
- [x] Run focused shell and source-map tests and confirm they fail before integration.
- [x] Preserve old Rust text on regeneration and failure; show `Tradução desatualizada` when its source snapshot differs.
- [x] Hide old mappings while stale; re-enable actions and mappings only after a current successful result.
- [x] Mark existing artifacts stale after source edits even when the Bridge panel is hidden.
- [x] Rerun focused tests, all `npm test`, and `npm run build`.

### Task 3: Verify native behavior and report

**Files:**
- Modify: `docs/reports/FEAT-UI-NEXT-SHELL.md` or create `docs/reports/FEAT-RUST-PREVIEW-STALE.md`

- [x] Run `cargo test --workspace`, `git diff --check`, and native Tauri smoke.
- [x] Record what was verified and any screen-capture limitation.
- [ ] Commit the feature in the isolated worktree and open a PR against `main`.
