# Rust Standalone Formatter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Format `.rs` editor buffers with installed `rustfmt`, without modifying stale or failed documents.

**Architecture:** Add a bounded Rust service operation using a fixed `rustfmt` command and a temporary source/config file. Expose availability and formatting through Tauri, then route the existing format action by language while retaining its document identity guard.

**Tech Stack:** Rust, Tauri 2 IPC, TypeScript, CodeMirror, Node `node:test`, installed `rustfmt` integration tests.

**Spec:** `docs/superpowers/specs/2026-11-22-rustfmt-standalone-design.md`

## Global Constraints

- Keep classic Portugol and Portugol-RS formatting behavior unchanged.
- Never run a shell, Cargo, student-provided command, or file path.
- Format only the active `.rs` buffer using Rust 2021 and a deterministic empty rustfmt config.
- Apply formatted text only after successful formatting and only if tab/document identity is unchanged.
- Limits: source 1 MiB, combined output 1 MiB, 5 seconds; use the existing process supervisor.
- Enable the control only when the independent `rustfmt` capability is detected.

## Review Focus

- Invalid Rust returns an error without changing the source buffer; assert service error and UI stale guard.
- Missing rustfmt disables `.rs` formatting with actionable text; assert capability mapping.
- A source over 1 MiB is rejected before starting a process; assert backend boundary.
- Unicode and long lines survive actual rustfmt output; assert exact formatted output.
- stdout/stderr output and timeout remain bounded by the process supervisor; assert the operation uses that path.

---

### Task 1: Rust service formatter

**Files:** modify `crates/portugol-service/src/pors_run.rs` and `crates/portugol-service/src/lib.rs`; create/update `crates/portugol-service/tests/rust_format.rs`.

**Interfaces:** add `rustfmt_available() -> bool` and `format_rust_source(source: &str) -> Result<String, String>`.

- [x] Add real-rustfmt tests for formatting Unicode Rust and rejecting invalid/oversized input.
- [x] Run focused tests and verify the missing API fails to compile.
- [x] Implement fixed `rustfmt --quiet --emit stdout --edition 2021` with empty config in a temp directory; supervise at 5 seconds and return output only on exit 0.
- [x] Run service tests serially and Clippy.

### Task 2: Tauri and UI integration

**Files:** modify `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/api.ts`, `apps/desktop/src/runtime-capabilities.ts`, `apps/desktop/src/main.ts`; update `apps/desktop/tests/runtime-capabilities.test.mjs` and add focused formatter routing tests if existing static test patterns apply.

**Interfaces:** Tauri commands `rustfmt_available` and async `format_rust_source(source) -> Result<String, String>`; typed frontend wrappers of the same names; `canFormatSource(language, rustfmtAvailable)`.

- [x] Test that classic Portugol remains formattable, `.rs` depends on rustfmt, and Portugol-RS remains unavailable.
- [x] Run the new capability test and confirm it fails before implementation.
- [x] Add command adapters and asynchronously refresh `.rs` format availability with a clear unavailable reason.
- [x] Route Rust formatting through service output, preserve the active tab/document stale check, apply one undoable full-document transaction, and preserve classic formatter response handling.
- [x] Run frontend tests/build and Tauri check.

### Task 3: Traceability and evidence

**Files:** modify `docs/PARIDADE.md` and the integration kit traceability/gap report; add `docs/portugol-studio-next-integration/docs/agents/FEATURE_REPORT_RUST_FORMAT.md`.

- [x] Record `FEAT-RUSTFMT-001`, platform/toolchain constraints, rollback, actual test commands, and E2E status.
- [x] Run workspace tests, app tests/build, fmt, Clippy, Tauri check, and `git diff --check`.
- [x] Review security, stale-result handling, and whether errors leave editor content untouched.
