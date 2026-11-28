# Rust Source Execution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable bounded, explicit compilation and execution of standalone `.rs` editor buffers.

**Architecture:** Add a service API beside the existing `.fer` runner, reusing the fixed `rustc` invocation, temporary directory and process supervision patterns. Add one typed Tauri command; the frontend uses existing language capability and Console/Messages views, with session-scoped consent and stale-buffer checks.

**Tech Stack:** Rust, Tauri 2 IPC, TypeScript, Node `node:test`, real `rustc` integration tests.

**Spec:** `docs/superpowers/specs/2026-11-22-rust-source-execution-design.md`

## Global Constraints

- Do not change the classic Ferro interpreter or `.fer` language semantics.
- Do not expose shell, Cargo, executable paths or compiler arguments to the frontend.
- Use fixed `rustc --edition=2021` arguments and a single temporary source/binary.
- Apply 1 MiB source, 20 second compilation, 3 second runtime and 1 MiB combined-output limits.
- Clearly state that native Rust execution uses local account permissions and is not sandboxed.
- No execution before explicit user action and first-session confirmation.
- Ignore results if the active tab or editor document changed while the IPC request was running.

## Review Focus

- Compiler source with non-ASCII text before an error maps Rust byte spans to CodeMirror UTF-16 positions.
- A compiler timeout/output overflow kills and reaps the process before temporary files are removed.
- Runtime timeout and output overflow display failure status with captured output, never success.
- Canceling the first-run safety prompt starts no compiler process.
- Editing or switching tabs during compilation discards the response from the old revision.

---

### Task 1: Implement the standalone Rust service runner

**Files:**
- Modify: `crates/ferro-service/src/ferro_run.rs`
- Modify: `crates/ferro-service/src/lib.rs`
- Create: `crates/ferro-service/tests/rust_source_runner.rs`

**Interfaces:**
- Produce `pub fn run_rust_source(source: &str) -> Result<RustSourceRunResult, String>`.
- `RustSourceRunResult` has `executed`, `compile_error`, `compile_limit`, and `runtime_limit` variants; source diagnostics carry UTF-16 `Position` spans.
- Reuse existing `rust_toolchain_available`, fixed-edition compiler invocation and `supervise` process collection; compiler and runtime receive separate time limits.

- [x] Write real-rustc integration tests for successful stdout/stderr/exit code, compile error positions after an emoji, infinite-loop timeout, output cap, and source-size rejection.
- [x] Run `cargo test -p ferro-service --test rust_source_runner` and verify failures identify the missing API/behavior.
- [x] Implement the result types, direct-source diagnostic parser and bounded compile/run path.
- [x] Rerun focused tests (6 pass), including regressions where child processes inherit stdout/stderr on Unix and Windows. The RAII temporary directory cleans up on all return paths.

### Task 2: Add the typed Tauri command and frontend adapter

**Files:**
- Modify: `apps/desktop/src-tauri/src/lib.rs`
- Modify: `apps/desktop/src/api.ts`

**Interfaces:**
- Expose only `run_rust_source({ source })`; backend chooses compiler and arguments.
- Return the service result through `spawn_blocking`, following `run_ferro_rs`.

- [x] Register the command and add a frontend contract test confirming API/handler registration without changing capabilities.
- [x] Export the serialized result union and `runRustSource(source)` in the frontend API.
- [x] `cargo test -p ferro-desktop --lib --offline --locked` (35 pass).

### Task 3: Enable explicit Rust execution in the IDE

**Files:**
- Modify: `apps/desktop/src/runtime-capabilities.ts`
- Modify: `apps/desktop/src/main.ts`
- Create: `apps/desktop/src/rust-execution-consent.ts`
- Create: `apps/desktop/tests/rust-execution-consent.test.mjs`
- Modify: `apps/desktop/tests/runtime-capabilities.test.mjs`

**Interfaces:**
- `.rs` is runnable only when `rustcAvailable === true`; debugger remains unavailable.
- `runProgram` captures the active tab and exact `EditorState`, awaits session consent, invokes the typed command and discards stale results.
- Compilation diagnostics are converted to existing `DiagnosticInfo`, rendered in Mensagens and navigable in the editor.
- Consent helper reads/writes only a session storage key after affirmative confirmation.

- [x] Add tests for `.rs` capability and affirmative/negative session-consent behavior.
- [x] Focused tests passed after implementation (5 pass).
- [x] Connect Executar to compile/run output, stderr, exit code, compile diagnostics, compile/runtime limits and rustc-unavailable messages.
- [x] Add an explicit warning dialog before the first Rust run in each IDE session; cancel must return before IPC.
- [x] Keep stop/debug appropriately unavailable for bounded standalone execution; do not silently report cancellation.
- [x] Rerun focused tests and full frontend suite (134 pass).

### Task 4: Document the install and runtime contract

**Files:**
- Modify: `docs/FERRO-RUST.md`
- Modify: `docs/DISTRIBUICAO.md`
- Create: `docs/reports/FEAT-RUST-SOURCE-EXECUTION.md`

- [x] Document standalone `.rs` behavior, Rust 2021 edition, requirements, limits, no-sandbox warning, and exclusions.
- [x] Run `npm test`, `npm run build`, `cargo fmt --all --check`, `cargo test -p ferro-service`, `cargo test -p ferro-desktop --lib` where available, and `git diff --check`.
- [x] Record actual results and any platform-local block in the feature report.
- [ ] Commit the verified feature, open a PR, and merge/delete the branch only after CI is green.
