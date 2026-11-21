# Restaurar último workspace — Implementation Plan

> **For agentic workers:** implement the tasks in order with the existing Tauri workspace boundary.

**Goal:** Restore the last selected project folder on the next application launch.

**Architecture:** Rust owns persistence in Tauri `app_config_dir`, with only the canonical folder path serialized as JSON. A narrow command restores and revalidates the path into a fresh in-memory workspace session; the frontend adds the returned tree without opening or changing tabs.

**Tech Stack:** Rust, Tauri 2, serde_json, TypeScript.

**Spec:** `docs/superpowers/specs/2026-11-21-restore-last-workspace.md`

## Global Constraints

- Keep workspace reads/writes scoped by the Tauri-managed workspace ID.
- Persist only the selected folder path; never persist source or buffer contents.
- An unavailable preference must not prevent the editor from starting.

## Review Focus

- No saved preference returns `None` and leaves the app usable.
- Invalid JSON is discarded instead of crashing startup.
- A deleted/moved folder is forgotten after one restore attempt.
- Permission failure reading configuration is surfaced without replacing editor state.
- Restore completion must not override a workspace the user opened while the request was pending.

---

### Task 1: Rust persistence helpers

**Files:** `apps/desktop/src-tauri/src/workspace.rs`

- [ ] Add tests for save/load, no preference, stale directory cleanup, and restoration into a new session.
- [ ] Attempt targeted desktop tests; record the WebKitGTK limitation if they cannot start.
- [ ] Add atomic JSON save, safe read, and `restore_saved_workspace` helpers.
- [ ] Revalidate root and list entries before registering a restored session.

### Task 2: Tauri commands

**Files:** `apps/desktop/src-tauri/src/workspace.rs`, `apps/desktop/src-tauri/src/lib.rs`

- [ ] Persist the selected canonical root best-effort in `workspace_open`.
- [ ] Expose `workspace_restore` using the app-config directory and blocking filesystem work.
- [ ] Register the command and run formatting, desktop tests and Clippy in CI.

### Task 3: Startup integration

**Files:** `apps/desktop/src/api.ts`, `apps/desktop/src/main.ts`, `apps/desktop/tests/workspace-explorer.test.mjs`

- [ ] Add the typed invoke wrapper and a test for the restore API/startup contract.
- [ ] Restore tree state on startup without opening files; display restored folder name.
- [ ] Guard stale async results so a later user-selected workspace wins.
- [ ] Verify frontend tests and production build.

### Task 4: Quality and delivery

**Files:** `docs/PARIDADE.md`, `docs/reports/FEAT-WORKSPACE-003.md`

- [ ] Record persistence scope, privacy behavior, and validation results.
- [ ] Run available frontend/Rust checks and `git diff --check`.
- [ ] Open a PR, fix failures until CI is green, merge, and delete its branch.
