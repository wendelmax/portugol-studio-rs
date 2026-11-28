# Renomear arquivos no Explorador — Implementation Plan

> **For agentic workers:** implement each task in order and verify it before continuing.

**Goal:** Rename `.por`, `.fer`, or `.pex` files from the workspace tree without overwriting data or losing open edits.

**Architecture:** Keep filesystem operations in the Rust workspace service. A typed rename result returns the refreshed tree and canonical renamed file path; the current tab model is updated only after backend success.

**Tech Stack:** Rust, Tauri 2, TypeScript, Node test runner.

**Spec:** `docs/superpowers/specs/2026-11-21-workspace-rename-file.md`

## Global Constraints

- Every operation is scoped by workspace ID and relative path.
- Preserve the original extension and file contents.
- Existing destinations are never overwritten; source and destination symlinks are rejected.
- Dirty buffers are saved before rename; cancel/failure leaves the old path intact.

## Review Focus

- Destination collision does not change either file's bytes.
- Symlink/traversal input cannot escape the workspace.
- Save cancellation/failure leaves file path and tab association unchanged.
- Rename of open clean `.por`, `.fer`, and `.pex` keeps matching tabs attached.
- Keyboard focus can reach rename actions and return to the renamed tree item.

---

### Task 1: Filename and filesystem contracts

**Files:** `apps/desktop/src/workspace-explorer.ts`, `apps/desktop/tests/workspace-explorer.test.mjs`, `apps/desktop/src-tauri/src/workspace.rs`

- [x] Add frontend name tests and Rust tests for valid extensions, collision, traversal, symlink, and content preservation.
- [x] Run the frontend helper tests red and the Rust test attempt (record WebKitGTK limitation locally).
- [x] Implement same-extension name validation and exclusive rename helper.

### Task 2: Tauri command

**Files:** `apps/desktop/src-tauri/src/workspace.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/api.ts`

- [x] Return refreshed `WorkspaceListing` plus canonical/relative file paths.
- [x] Register and type `workspace_rename_file`.
- [ ] Run formatting and Rust checks in CI.

### Task 3: Explorer and document integration

**Files:** `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/tests/workspace-explorer.test.mjs`

- [x] Add accessible file-row action and rename dialog contract tests.
- [x] Before renaming, save dirty source buffers or keep operation canceled.
- [x] Update open tabs, recent list, selection, and explorer only after backend success.
- [x] Run frontend tests and production build.

### Task 4: Quality and delivery

**Files:** `docs/PARIDADE.md`, `docs/reports/FEAT-WORKSPACE-004.md`

- [x] Record validation and remaining file-action gaps.
- [x] Run applicable local checks and `git diff --check`.
- [ ] Open PR, fix failures until CI is green, merge, and delete its branch.
