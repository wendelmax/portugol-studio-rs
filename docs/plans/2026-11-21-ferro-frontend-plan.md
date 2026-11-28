# Ferro Frontend Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Parse the initial Ferro subset into the shared HIR and generate equivalent Rust through the existing backend.

**Architecture:** Add a standalone frontend crate that lexes/parses the proposed syntax, resolves scopes/types/mutability while lowering to `ferro-hir`, and expose a Rust export API that calls the shared code generator. The classic parser, VM, UI file dialogs, and CLI remain unchanged in this slice.

**Tech Stack:** Rust 2024 workspace, `ferro-hir`, existing `ferro-syntax::Diagnostic`, `ferro-rust` codegen, Rust unit/integration tests.

**Spec:** `docs/superpowers/specs/2026-11-21-ferro-frontend-design.md`

## Global Constraints

- Preserve `.por` parsing, semantic analysis, runtime and export behavior.
- HIR contains no target-language source strings.
- `.fer` is not registered in UI/CLI before end-to-end file flows exist.
- Never report generated output as successful if parsing or semantic checks failed.
- Unsupported syntax returns a diagnostic with a source span.

## Review Focus

- Unicode text before an error must not shift byte spans; test a UTF-8 prefix and asserted source slice.
- Nested shadowing must bind references to the nearest declaration; assert `BindingId`s.
- Immutable bindings reject assignment; mutable bindings accept only type-compatible assignments.
- Operator precedence and parentheses must survive lowering and generated code; compile/output test.
- Malformed strings/comments and EOF must return diagnostics without panics; malformed-input tests.

---

### Task 1: Frontend lexer and parser

**Files:** Create `crates/ferro-rust-frontend/Cargo.toml`, `src/lib.rs`, `src/lexer.rs`, `src/parser.rs`, `tests/parser.rs`; modify workspace `Cargo.toml` and `Cargo.lock`.

**Interfaces:** `pub fn lower(source: &str) -> Result<ferro_hir::Program, Vec<Diagnostic>>`.

- [x] Add failing parse tests for declarations, mutability, expressions, `escreva`, `se/senao`, Unicode spans and malformed input.
- [x] Run `cargo test -p ferro-rust-frontend` and confirm failures before implementation.
- [x] Implement a byte-span lexer and Pratt parser for only the grammar in the design spec.
- [x] Run parser tests and `cargo fmt --all --check`.

### Task 2: Semantic lowering to HIR

**Files:** `crates/ferro-rust-frontend/src/lower.rs`, `src/parser.rs`, `tests/semantics.rs`.

**Interfaces:** `lower` resolves declarations into HIR `BindingId`s and typed expressions; diagnostics use stable `ferro.*` codes and source spans.

- [x] Add failing tests for unknown names, assignment to immutable, type mismatch, shadowing and uninitialized declarations (all declarations require initializers in this subset).
- [x] Run focused tests and confirm failures.
- [x] Implement scoped symbol resolution and type checking while constructing HIR; do not mutate classic semantic analysis.
- [x] Run frontend tests and Clippy for the new crate.

### Task 3: Shared Rust code generation

**Files:** `crates/ferro-rust/Cargo.toml`, `src/lib.rs`, tests in `src/lib.rs`.

**Interfaces:** `pub fn export_ferro_rs(source: &str) -> Result<String, Vec<Diagnostic>>`.

- [x] Add failing tests that generate the user's `idade` example, compile it using `rustc`, and assert output `17`.
- [x] Add a negative test asserting the `.por` export API behavior remains unchanged.
- [x] Implement the API by lowering with `ferro-rust-frontend` and reusing `codegen::render`.
- [x] Run focused tests, `cargo test --offline --locked`, Clippy and format checks.

### Task 4: Evidence and handoff

**Files:** update `docs/plans/hir-compartilhada-ferro-rust.md`, create `docs/reports/FEAT-LANG-002-ferro-frontend.md`.

- [x] Record exact commands/results, supported grammar, blocked UI `.fer` integration and rollback.
- [x] Verify `git diff --check` and report the PR dependency on the shared-HIR PR.
