# Workspace Project Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement FT-10 by copying the user's selected workspace files to a new local project folder.

**Architecture:** Keep selection in the existing frontend, use a typed Tauri command with the existing workspace session and native folder picker, and copy through scoped Rust filesystem capabilities. No ZIP library, shell command, or upload service.

**Tech Stack:** Existing Tauri v2 dialog plugin, Rust `cap-std`, TypeScript, Node test runner.

**Spec:** `docs/superpowers/specs/2026-11-21-workspace-project-export-design.md`

## Global Constraints

- Preserve the existing Rust + Tauri architecture and workspace session contract.
- Do not introduce shell execution, dependencies, IPC permissions, or cloud publishing.
- Accept only validated workspace-relative `.por`, `.pors`, `.rs`, and `.pex` file paths.
- Preserve relative directory paths, never overwrite an existing export directory, and remove partial output on failure.
- Report success only after the filesystem operation completes.

## Review Focus

- Empty selection must not open the folder picker; frontend test pins the guard.
- Traversal, absolute paths, unsupported extensions, and symlinks must not read/copy files; backend tests exercise each case.
- Existing destination project folder must remain unchanged; backend test pins exclusive directory creation.
- Failure partway through copying must not leave partial output; backend test forces a source failure and checks cleanup.
- Canceling the native destination picker must not report success; API/handler contract test checks nullable result.

---

### Task 1: Model selectable workspace files

**Files:**
- Modify: `apps/desktop/src/workspace-explorer.ts`
- Modify: `apps/desktop/tests/workspace-explorer.test.mjs`

**Interfaces:**
- Produces `collectWorkspaceFiles(entries): FileEntry[]`, recursively ordered and containing only file nodes.

- [x] Add failing cases for nested files, stable order, and empty trees.
- [x] Implement the recursive helper with no mutation of the source tree.
- [x] Run the focused and full frontend tests (full suite: 12 files, all pass).

### Task 2: Copy selected files within workspace/destination capabilities

**Files:**
- Modify: `apps/desktop/src-tauri/src/workspace.rs`
- Modify: `apps/desktop/src-tauri/src/lib.rs`

**Interfaces:**
- Produces `workspace_export(workspace_id, relative_paths) -> Result<Option<WorkspaceExportResult>, String>`.
- Produces a pure helper taking source root, destination parent, and selected relative paths; result contains project name and copied file count.

- [x] Add backend tests for successful nested copies, unsupported/traversal/symlink rejection, collision, and cleanup after failure.
- [x] Implement scoped copy, exclusive project directory creation, rollback of partial output, and native folder picker with cancel as `None`.
- [x] Register the command; Rust test run is BLOCKED by missing Linux system libraries `webkit2gtk-4.1` and `javascriptcoregtk-4.1` before crate compilation.

### Task 3: Add export selection dialog and report

**Files:**
- Modify: `apps/desktop/src/api.ts`
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/main.ts`
- Create: `apps/desktop/src/workspace-export-state.ts`
- Modify: `apps/desktop/src/style.css`
- Create: `apps/desktop/tests/workspace-export.test.mjs`
- Modify: `docs/PARIDADE.md`
- Create: `docs/reports/FT-10-project-export.md`

**Interfaces:**
- Frontend calls `api.exportWorkspaceFiles(workspaceId, relativePaths)` and receives `WorkspaceExportResult | null`.
- Dialog selection defaults all supported files to checked and returns only checked relative paths.

- [x] Add failing tests for entry point, selection, empty state, error/status copy, no upload claim, and Tauri API contract.
- [x] Implement the accessible selection dialog, busy/cancel/error/success states, and disabled action without a workspace.
- [x] Frontend tests (12 files), production build, Rust format, and diff checks pass; Rust tests are BLOCKED by missing system libraries as recorded above.
- [x] Record actual validation and platform limitations in the FT-10 report.
- [x] Commit the verified feature; review fixes are being added to the same PR.
