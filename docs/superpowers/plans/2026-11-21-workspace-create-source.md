# Criar programa no Explorador — Implementation Plan

> **For agentic workers:** execute the steps in order and keep each change testable.

**Goal:** Create a blank `.por` or `.pors` at the opened workspace root and open it in the existing editor.

**Architecture:** Keep the existing Tauri workspace boundary. Normalize the displayed filename in the frontend; independently validate and exclusively create it in Rust, then return the refreshed listing to the existing explorer state.

**Tech Stack:** Rust, Tauri 2, TypeScript, Node test runner.

**Spec:** `docs/superpowers/specs/2026-11-21-workspace-create-source.md`

## Global Constraints

- `.por` is Portugol classic; `.pors` is Portugol-RS.
- Workspace I/O stays scoped by workspace ID and relative path.
- Never overwrite an existing file or follow symlinks.
- Preserve current tabs, dirty buffers, keyboard access, and native individual-file flows.

## Review Focus

- Extension mismatch or case variants cannot silently create an unintended file.
- Names containing path traversal or separators cannot escape the workspace root.
- A file collision never truncates existing content.
- Canceling the UI dialog never mutates the filesystem.
- A creation/listing error does not replace the current workspace or active buffer.

---

### Task 1: Frontend name contract

**Files:** `apps/desktop/src/workspace-explorer.ts`, `apps/desktop/tests/workspace-explorer.test.mjs`

- [ ] Add tests for appended `.por`/`.pors`, matching extension, conflicting extension, empty name, and path separator.
- [ ] Run `node --experimental-strip-types --test apps/desktop/tests/workspace-explorer.test.mjs`; confirm failure because the helper is absent.
- [ ] Add `workspaceSourceFileName(name, language)` returning a filename or a Portuguese validation error.
- [ ] Run the targeted test and full `npm test`.

### Task 2: Safe Rust creation command

**Files:** `apps/desktop/src-tauri/src/workspace.rs`, `apps/desktop/src-tauri/src/lib.rs`

- [ ] Add unit tests for `.por`/`.pors` creation, invalid names/extensions, collisions preserving bytes, and unknown workspace IDs.
- [ ] Run the workspace module tests and confirm expected failures before implementation.
- [ ] Add exclusive root-only creation and `workspace_create_file`, returning an updated `WorkspaceListing`.
- [ ] Register the command; run `cargo fmt --all -- --check`, desktop tests, and Clippy.

### Task 3: Accessible dialog and editor integration

**Files:** `apps/desktop/index.html`, `apps/desktop/src/api.ts`, `apps/desktop/src/main.ts`, `apps/desktop/tests/workspace-explorer.test.mjs`

- [ ] Add failing markup/contract assertions for name input, language selection, cancel/create actions, and accessible labels.
- [ ] Add the dialog and Tauri API wrapper; keep the open-folder action available separately.
- [ ] On submit, normalize name, create file, replace only the current workspace listing, select and open the new file.
- [ ] Test dialog contract, run full frontend tests and `npm run build`.

### Task 4: Quality and delivery

**Files:** `docs/PARIDADE.md`, `docs/reports/FEAT-WORKSPACE-002.md`

- [ ] Record implemented parity and validated commands.
- [ ] Run all locally available Rust/frontend checks and `git diff --check`.
- [ ] Open a PR against `main`, fix failures until CI is green, merge, and delete its branch.
