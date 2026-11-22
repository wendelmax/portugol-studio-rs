# PORS Vectors Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add fixed-size homogeneous vectors to PORS with typed declarations, indexing, assignment, `tamanho`, function value semantics, generated Rust and debugger execution.

**Architecture:** Preserve the shared HIR and add copyable vector type variants for each primitive element type. Lower PORS vector syntax into typed vector literals/index operations; compile those operations to existing VM array values plus narrow length/deep-copy opcodes, and emit equivalent Rust `Vec<T>` operations with explicit bounds checks.

**Tech Stack:** Rust workspace; `portugol-rs-frontend`, `portugol-hir`, `portugol-bytecode`, `portugol-vm`, `portugol-rust`, `portugol-service`.

**Spec:** `docs/superpowers/specs/2026-11-22-pors-vectors-design.md`.

## Global Constraints

- Product name, Rust + Tauri architecture, `.pors` extension, and `.por` support stay unchanged in this feature.
- Only `vetor<T>` with the five current primitive element types is supported; no matrices, nested collections, empty literals, resize, slices, or references.
- Vector declarations and parameters require explicit `vetor<T>` annotations; literals must be nonempty and homogeneous.
- Vectors use value semantics: assignment, return, and by-value function arguments copy the vector; `mutavel` controls mutation of the local binding/copy.
- Indexes are zero-based `inteiro`; debugger errors carry the original index-expression span; generated Rust reports an out-of-bounds message and exits nonzero.
- Keep existing VM, debugger, and generated-Rust security/resource limits; add no dependency.

## Review Focus

- A heterogeneous or wrong-typed vector literal: frontend must report the first incompatible element span.
- A negative or too-large dynamic index: VM must produce the PORS runtime error at the index expression; Rust runner must exit nonzero with a readable message.
- Mutating an immutable vector or immutable vector parameter: reject in lowering; `mutavel` parameter changes only the callee's copy.
- Assigning, returning, or passing a vector then mutating one copy: caller and callee/other binding must remain independent in both backends.
- `vetor<cadeia>` copies: preserve string accounting limits and identical visible values in VM and Rust code generation.

---

### Task 1: Add vector types and PORS syntax to the frontend

**Files:**
- Modify: `crates/portugol-hir/src/lib.rs`
- Modify: `crates/portugol-rs-frontend/src/lexer.rs`
- Modify: `crates/portugol-rs-frontend/src/parser.rs`
- Test: `crates/portugol-rs-frontend/tests/parser.rs`
- Test: `crates/portugol-rs-frontend/tests/semantics.rs`
- Modify: `crates/portugol-rust/src/hir.rs` (classic HIR type conversion stays scalar)
- Modify: `crates/portugol-hir/src/tests.rs`

**Interfaces:**
- Add `Type::{VectorInteger, VectorReal, VectorBoolean, VectorString, VectorCharacter}` and helpers to retrieve each vector's scalar element type and test whether a type is a vector.
- Add HIR `ExprKind::VectorLiteral(Vec<Expr>)`, `ExprKind::Index { binding, index }`, `ExprKind::Length { binding }`, and `StmtKind::IndexAssign { binding, index, value }`.
- Parse `vetor<tipo>`, nonempty `[expr, ...]`, `binding[index]`, `binding[index] = expr`, and the builtin `tamanho(binding)`.
- Vector element mutation requires mutable binding; typed literal elements must match exactly; `tamanho` accepts only a vector and returns integer.

- [x] **Step 1: Write failing parser and semantic tests** for each primitive vector annotation/literal, index and length HIR, vector parameters/returns, heterogeneous literals, non-vector `tamanho`, wrong type, noninteger index, and immutable index assignment.
- [x] **Step 2: Run** `cargo test -p portugol-rs-frontend --test parser --test semantics`; confirm new behavior fails with expected missing-token/type diagnostics.
- [x] **Step 3: Implement** vector HIR types/expressions/statements, `[`/`]` tokens, typed type parsing, nonempty vector literals, index reads/writes and `tamanho` lowering.
- [x] **Step 4: Run** the frontend parser/semantics tests and format the Rust workspace.

### Task 2: Compile vectors in the debugger VM

**Files:**
- Modify: `crates/portugol-bytecode/src/op.rs`
- Modify: `crates/portugol-bytecode/src/pors.rs`
- Modify: `crates/portugol-vm/src/vm.rs`
- Test: `crates/portugol-bytecode/tests/pors_compiler.rs`
- Test: `crates/portugol-service/tests/portugol_rs_debugger.rs`

**Interfaces:**
- Reuse `ArrayLit`, `LoadElem`, and `StoreElem`; add only `ArrayLen` and `CloneArray` opcodes.
- Store PORS vectors in VM arrays with scalar `LocalInfo.ty` and `dims = 1` so existing debugger expansion shows vector elements.
- `CloneArray` makes a new array allocation and charges contained strings to the VM string-memory limit.

- [x] **Step 1: Write failing VM tests** for vector length/index writes, bounds error span, deep-copy calls, and debugger expansion/type of vector locals.
- [x] **Step 2: Run** focused debugger tests and confirm the bytecode lacks vector HIR support.
- [x] **Step 3: Implement** vector literal/index/length lowering, type mapping, function argument and return copies, `CloneArray`, `ArrayLen`, and local debug dimension metadata.
- [x] **Step 4: Run** the focused bytecode and service debugger tests.

### Task 3: Emit safe equivalent Rust

**Files:**
- Modify: `crates/portugol-rust/src/codegen.rs`
- Test: `crates/portugol-service/tests/portugol_rs_runner.rs`
- Test: `crates/portugol-service/tests/portugol_rs_debugger.rs`

**Interfaces:**
- Map PORS vector types to `Vec<i32>`, `Vec<f64>`, `Vec<bool>`, `Vec<String>`, and `Vec<char>`.
- Emit literals as `vec![...]`, `tamanho` as `.len() as i32`, checked indexing through `__pors_index(index, len)`, and indexed assignment only for mutable vector bindings.
- Clone vector function arguments, assignments and returns; for `cadeia`, clone each element through `__pors_string` to preserve the PORS generated-Rust memory counter.

- [x] **Step 1: Add failing runner cases** for dynamic indexing, out-of-bounds failure and independent vector copies.
- [x] **Step 2: Run** focused Rust runner tests; confirm generated Rust rejected new HIR and initially emitted an unhelpful bounds panic.
- [x] **Step 3: Implement** vector type/literal/index/length generation, deep-copy helpers, and a readable generated-Rust bounds check.
- [x] **Step 4: Run** focused tests and confirm VM/Rust output and independent copy behavior match.

### Task 4: Integrate docs and validate the workspace

**Files:**
- Modify: `docs/research/PORS_MIGRATION_MATRIX.md`
- Modify: `docs/plans/pors-only-migration.md` only if the current phase ordering needs a status update

- [x] Mark vectors as partial with this slice's exact limits and remaining matrices/empty-literal/resize work.
- [x] Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` on Linux. Frontend commands were not needed because this increment changes only Rust compiler/runtime/service layers; CI still provides the cross-platform Tauri builds.
- [x] Run `git diff --check`; inspect the complete diff for changes to product identity, `.por`, and user-owned untracked files.
- [ ] Commit this slice with no coauthor or generated-by trailer, request independent review, open a PR, and merge/delete the branch only after all CI jobs are green.
