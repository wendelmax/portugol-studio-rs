# Cargo Program Arguments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pass validated student `argv` to selected Cargo binaries through the existing parameters field.

**Architecture:** Reuse the frontend argument notation and Rust service validation. Add an explicit Cargo service entry point that inserts `--` before user arguments, and thread the vector through Tauri and the existing run flow.

**Tech Stack:** Rust, Tauri v2, TypeScript, Node `node:test`, Cargo integration tests.

**Spec:** `docs/superpowers/specs/2026-11-22-cargo-program-arguments-design.md`; ADR `docs/adr/0013-argumentos-de-programa-em-projetos-cargo.md`.

## Global Constraints

- Never invoke a shell or accept free-form Cargo flags from the UI.
- Revalidate limits in Rust: 256 arguments, NUL rejection, 64 KiB Unix and 8 KiB Windows.
- Keep arguments out of the Cargo build phase; preserve existing execution when the vector is empty.
- Preserve standalone `.rs`, `.por`, and `.fer` behavior.

## Review Focus

- Empty argument and Unicode/space quoting; cover in Cargo integration test.
- NUL, argument count, and byte limits; cover validation before spawn.
- Cargo target arguments must precede `--`; verify the generated command structure.
- Dirty-buffer consent and run cancellation must remain unchanged; run existing Cargo tests.
- Ferro and Ferro must not gain the Cargo parser; retain existing argument and FERRO tests.

---

### Task 1: Service API and real `argv` forwarding

**Files:**
- Modify: `crates/ferro-service/src/ferro_run.rs`
- Modify: `crates/ferro-service/src/cargo_project.rs`
- Modify: `crates/ferro-service/src/lib.rs`
- Test: `crates/ferro-service/tests/cargo_project.rs`

- [x] Add a failing integration test that invokes a Cargo binary with empty, Unicode, whitespace, and quote-containing arguments and compares its `std::env::args()` output. RED: missing service API; GREEN: real Cargo binary receives the selected target argv intact.
- [x] Add failing tests for NUL, argument count, and byte limit rejection before `cargo run` starts.
- [x] Expose the existing Rust argument validator within the service crate and add `run_cargo_project_target_with_args_and_cancel(root, target, args, cancel)`; preserve existing service functions as empty-args wrappers.
- [x] Append `--` and the validated arguments only to the `cargo run` command, never `cargo build`; focused integration tests and full service suite pass.
- [x] Run `cargo fmt --all -- --check` and Clippy for `ferro-service`.

### Task 2: Tauri and editor wiring

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/api.ts`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/tests/cargo-project.test.mjs`
- Modify: `apps/desktop/tests/cargo-target-selection.test.mjs`
- Verify existing: `apps/desktop/tests/process-arguments.test.mjs`

- [x] Add a failing contract test for the Tauri payload and ensure Cargo calls parse the same field used by standalone Rust.
- [x] Pass `Vec<String>` from the command to the service and include it in `api.runCargoProject`.
- [x] Enable the field for Cargo `.rs`, describe quoting/argv behavior, parse the field and pass arguments only to execution; keep target discovery independent and `.fer` disabled.
- [x] Run focused frontend tests, the complete frontend suite (36/36), and `npm run build` (pass; existing >500 kB chunk warning).
- [x] Run `git diff --check`; inspect that no user string is used as a shell or Cargo flag.

## Execution ledger

- Task 1: Cargo integration test observed RED at the absent service API; after implementation, argv round-trip and invalid-input tests passed. `cargo test -p ferro-service --offline --locked` and Clippy passed.
- Task 2: UI/Tauri contract test observed RED before threading argv. `npm test` passed 36/36; `npm run build` passed; `cargo check -p ferro-desktop --offline --locked` passed; `cargo test -p ferro-desktop --offline --locked --lib` passed 42/42.
- Formatting and `git diff --check` passed.
- UI E2E remains `NOT_RUN`: repository has no DOM/Tauri interaction runner; this change is additionally covered by a real Cargo subprocess integration test and Tauri compile/tests.
