# Ferro Document Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Open, edit, save, analyze and preview `.fer` files in the existing Tauri IDE without affecting `.por` behavior.

**Architecture:** Use a per-tab language ID detected from file extension, route service inspection/export through the Ferro frontend, and add `.fer` to current Tauri file flows. Rust execution/debug remains disabled with an explanation until a safe runtime exists.

**Tech Stack:** Rust workspace/service, Tauri v2 commands/file associations, TypeScript, CodeMirror 6, Node test runner.

**Spec:** `docs/superpowers/specs/2026-11-21-ferro-document-flow-design.md`

## Global Constraints

- Preserve `.por` parser, VM, debugger and file behavior.
- `.fer` dispatches to the real `ferro-rust-frontend`; never rewrite keywords or route to the classic parser.
- Never overwrite dirty buffers or change a tab's language before a successful save.
- No arbitrary shell process, Rust compiler, or fake execution output.
- Drop stale analysis/translation results when the active document changes.

## Review Focus

- Uppercase `.fer` suffix and paths with non-ASCII characters; add to file-detection tests.
- Save As cancellation/failure must retain path, language and dirty state; test state transition.
- UTF-8 prefix before an error must produce CodeMirror UTF-16 positions; test service spans.
- Switching between `.por` and `.fer` tabs must restore syntax/linter for each; test editor language mapping.
- Run/debug shortcuts on `.fer` must not invoke the classic VM; test disabled/action guard.

---

### Task 1: Document language model and service

**Files:** Create `apps/desktop/src/document-language.ts` and `tests/document-language.test.mjs`; modify `crates/ferro-service/Cargo.toml`, `src/lib.rs`, service tests.

**Interfaces:** `type SourceLanguage = 'ferro' | 'ferro-rust'`; `detectSourceLanguage(path: string): SourceLanguage`; service functions `inspect_ferro_rs(source)` and `export_ferro_rs(source)`.

- [x] Add tests for `.por`, `.fer`, paths with spaces/Unicode and unsupported extensions.
- [x] Add service tests for real diagnostics, UTF-16 conversion and HIR symbols.
- [x] Implement language detection and service routing; use `LineIndex` for all spans.
- [x] Run focused Node/Rust tests; Clippy remains pending.

### Task 2: Tauri file contract

**Files:** Modify `apps/desktop/src-tauri/src/open_files.rs`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`, add Tauri tests as needed.

- [x] Add test coverage that accepted OS file paths include `.por`, `.fer`, `.pex` and reject unrelated files.
- [x] Add `.fer` Tauri commands for inspect/export through the service; register commands.
- [x] Add `.fer` to open-file filtering, dialog filters and platform associations.
- [ ] Run native Tauri test and CI (local native test requires WebKitGTK 4.1; CI pending).

### Task 3: Per-tab UI language

**Files:** Modify `apps/desktop/src/main.ts`, `src/api.ts`, `src/ferro.ts`, `index.html`, `style.css` if needed; add frontend tests.

- [x] Add a “Novo Ferro” action and language ID to each tab.
- [x] Route linter and Rust preview by tab language; preserve stale-result guard.
- [x] Reconfigure CodeMirror syntax/completion when activating a `.fer` tab.
- [x] Add `.fer` to open/save/drag-drop flows; switch language only on successful Save As.
- [x] Disable run/debug and classic formatting/library actions for `.fer` with accessible reasons.
- [x] Test file language/model and run `npm test`, `npm run build`.

### Task 4: Evidence

**Files:** create `docs/reports/FEAT-LANG-003-ferro-document-flow.md`; update the shared evolution plan.

- [ ] Record commands, statuses, CI, limitations, and rollback.
- [x] Verify all available tests and `git diff --check`.
