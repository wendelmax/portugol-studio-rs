# Ferro `enquanto` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a typed `enquanto` loop to Ferro and bound generated Rust runtime, output, and string growth.

**Architecture:** Extend the shared HIR with a while statement and parse it in the existing Ferro frontend. Reuse HIR Rust codegen and source maps; add cumulative string-allocation guards only to FERRO output, then supervise the fixed Rust process with bounded concurrent pipe capture and typed outcomes for runtime limits.

**Tech Stack:** Rust workspace, serde, Tauri 2, TypeScript, CodeMirror 6, Node test runner.

**Spec:** `docs/superpowers/specs/2026-11-21-ferro-loops-design.md`

## Global Constraints

- Only Ferro gains `enquanto` in this slice; preserve classic `.por` lowering/runtime behavior.
- `enquanto` conditions must be `logico`; blocks retain normal lexical scope and mutability checks.
- Do not implement `pare`, `continue`, `faca-enquanto`, `para`, or labels.
- Run generated Rust with the existing fixed executable/arguments, a 3-second runtime deadline, 1 MiB combined stdout/stderr capture, and a 16 MiB cumulative generated-string allocation budget.
- Compile rejection remains `compile_error`; runtime limit is a typed `runtime_limit` response with reason `time`, `output`, or `string_memory`; process setup failures remain IPC errors.
- Add no dependencies, shell invocation, or arbitrary `.rs` input execution.

## Review Focus

- Nested loop bodies with shadowed names must retain lexical bindings and deterministic source maps.
- Non-boolean conditions must fail at their source span before invoking `rustc`.
- A non-terminating loop must be killed and reaped within a bounded interval.
- Simultaneous stdout/stderr output must never exceed the combined capture limit.
- String literals, cloning, and concatenation must pass through the FERRO allocation budget without changing classic output.
- Pipe readers must continue draining or terminate cleanly so a full pipe cannot deadlock timeout handling.

---

### Task 1: Parse `enquanto` into shared HIR

**Files:**
- Modify: `crates/ferro-hir/src/lib.rs`
- Modify: `crates/ferro-rust-frontend/src/parser.rs`
- Test: `crates/ferro-rust-frontend/tests/parser.rs`
- Test: `crates/ferro-rust-frontend/tests/semantics.rs`

**Interfaces:**
- Produce `StmtKind::While { condition: Expr, body: Vec<Stmt> }`.
- The parser consumes `enquanto ( <expression> ) { <statements> }`, requires a boolean condition and uses the loop body as a nested scope.

- [x] Write a failing parser test for a terminating loop that mutates a `seja mutavel` binding and reads it after the loop.
- [x] Run the focused frontend test and observe the missing HIR variant.
- [x] Implement the HIR node, parser branch, condition type check, and block scope.
- [x] Add failing semantic tests for non-logical conditions, immutable assignment in the body, and unsupported `pare`/`continue`.
- [x] Pin that `enquanto` body declarations do not leak into the enclosing scope and nested shadowing gets distinct binding IDs.
- [x] Run `cargo test --locked -p ferro-rust-frontend` and `cargo test --locked -p ferro-hir`.

### Task 2: Generate Rust and preserve loop source maps

**Files:**
- Modify: `crates/ferro-rust/src/codegen.rs`
- Modify: `crates/ferro-rust/src/lib.rs`
- Test: `crates/ferro-rust/src/lib.rs`

**Interfaces:**
- `StmtKind::While` emits `while <condition> { ... }` using existing expression generation.
- The full generated loop maps to its HIR span; nested condition/body mappings remain ordered and source ranges remain UTF-8 byte offsets.
- Only Ferro generation adds string-budget helpers; classic `.por` generated Rust and its output-equivalence fixtures remain unchanged.
- The classic AST-to-HIR lowering may continue rejecting classic loops in this slice.

- [x] Write a failing exporter test for a Ferro counter loop and assert generated Rust plus condition/body/statement mappings.
- [x] Write a failing test that FERRO generation emits string-allocation guards while classic generated Rust remains compatible with its legacy fixture.
- [x] Run the focused exporter test and observe the missing HIR codegen branch.
- [x] Implement loop emission with the existing mapped renderer.
- [x] Implement the FERRO-only 16 MiB cumulative string-allocation guard and pre-allocation concatenation length check.
- [x] Verify generated code compiles and produces the expected output with real `rustc`.
- [x] Run `cargo test --locked -p ferro-rust`.

### Task 3: Bound runtime duration and output capture

**Files:**
- Modify: `crates/ferro-service/src/ferro_run.rs`
- Modify: `crates/ferro-service/tests/ferro_rs_runner.rs`

**Interfaces:**
- Add tagged `FerroRustRunResult::RuntimeLimit { stdout, stderr, reason }`, where reason serializes as `time`, `output`, or `string_memory`.
- Runtime subprocess capture uses concurrent stdout/stderr readers sharing a strict 1 MiB byte budget; a 3-second deadline begins after successful compilation. Recognize the generated helper's fixed string-budget exit code.
- On a limit, kill and wait for the child, join readers, and return the captured output. Existing compile and ordinary execution variants remain unchanged.

- [x] Write a failing runner test for an infinite loop timing out.
- [x] Write failing tests for output beyond the combined stdout/stderr cap and generated string-allocation limit.
- [x] Run the focused runner test and confirm the missing typed limit outcome.
- [x] Implement child supervision, concurrent draining and typed `runtime_limit` serialization.
- [x] Test serialized tags/reasons, capture <= 1 MiB and string limit termination.
- [x] Run `cargo test --locked -p ferro-service` and `cargo clippy --locked -p ferro-service --all-targets -- -D warnings`.

### Task 4: Show loop execution limits in the IDE and document delivery

**Files:**
- Modify: `apps/desktop/src/api.ts`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/tests/source-map.test.mjs` or a focused runner-result UI test
- Modify: `docs/PARIDADE.md`
- Create: `docs/reports/FT-14-ferro-loops.md`

**Interfaces:**
- TypeScript mirrors the tagged `runtime_limit` variant and its camelCase fields.
- The IDE shows captured output and a Portuguese status naming the time/output/string limit; it clears stale run responses using the existing tab/document guard.
- Tauri command names and source-map response contracts do not change.

- [x] Write failing frontend contract tests for all runtime-limit reasons and user-facing messages.
- [x] Run frontend tests and confirm the missing result handling before implementation.
- [x] Render bounded output and clear limit status without marking the result as successful.
- [x] Update parity/report docs with supported grammar and fixed limits, and record ADR 0003.
- [x] Run frontend tests, `cargo fmt --all -- --check`, desktop Tauri check, service and exporter tests, and `git diff --check` (local `npm run build` could not start because `tsc` is absent from the worktree's `node_modules`; CI build remains required).
- [ ] Request independent review, fix Critical/Important findings, open a PR, wait for all checks, squash-merge when green, update local `main`, then remove the feature branch/worktree.
