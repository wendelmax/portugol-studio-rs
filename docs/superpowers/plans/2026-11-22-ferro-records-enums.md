# FERRO Records and Enums Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add typed nominal records, payload enums, exhaustive `escolha`, field access, and vectors of compound values to FERRO across VM/debugger and generated Rust.

**Architecture:** Extend the shared HIR and frontend with nominal type IDs, record/enum constructors, field expressions/assignments, and match arms. Lower the same typed constructs to VM values/opcodes and to Rust structs/enums/match, preserving value-copy, string budgets, spans, and editor UTF-16 boundaries.

**Tech Stack:** Rust workspace; `ferro-hir`, `ferro-rust-frontend`, `ferro-bytecode`, `ferro-vm`, `ferro-rust`, `ferro-service`; Tauri/CodeMirror tests.

**Spec:** `docs/superpowers/specs/2026-11-22-ferro-records-enums-design.md`.

## Global Constraints

- Preserve Rust + Tauri, `.fer`, product identity, and existing `.por`/`.pex` behavior.
- Keep byte offsets in HIR; convert to UTF-16 only at service/editor boundaries.
- Run equivalent FERRO semantics in VM/debugger and generated Rust; no new dependency.
- Records, enums, parameters, returns, field mutation, and vector elements have independent value semantics.
- Match every enum variant exactly once; no wildcard, guards, references, methods, generics, or recursive type cycles in this slice.
- Charge copied `cadeia` values to the existing cumulative string budgets in both backends.

## Review Focus

- Incomplete/duplicate/wrongly typed record construction: point at the offending field/value.
- Missing/duplicate enum arms and wrong payload arity/type: reject before execution.
- Mutating a field through an immutable binding: reject in frontend semantics.
- Copying nested records/enums/vectors then mutating either copy: never alias values.
- Unicode before field/variant tokens: retain correct byte spans and UTF-16 diagnostics.

---

### Task 1: Nominal definitions and typed HIR

**Files:** `crates/ferro-hir/src/lib.rs`, `crates/ferro-hir/src/tests.rs`, `crates/ferro-rust-frontend/src/lexer.rs`, `crates/ferro-rust-frontend/src/parser.rs`, frontend parser/semantics tests.

**Interfaces:**
- Add `TypeId(u32)`, `RecordDef { id, name, fields }`, `EnumDef { id, name, variants }`, `RecordField { name, ty, span }`, and `EnumVariant { name, payload_types, span }` to `ferro-hir`.
- Add `Program.record_types` and `Program.enum_types`; `Type` adds `Record(TypeId)`, `Enum(TypeId)`, `VectorRecord(TypeId)`, and `VectorEnum(TypeId)`.
- Add HIR `ExprKind::RecordLiteral { type_id, fields: Vec<Expr> }`, `Field { value: Box<Expr>, field_index }`, `EnumConstructor { type_id, variant_index, payload: Vec<Expr> }`; add `StmtKind::FieldAssign { binding, field_index, op, value }` and `Choose { value, arms }`.
- Each `ChooseArm` stores the enum variant index, payload binding IDs, and body. `Program` type definitions resolve before function signatures/bodies; each type may refer only to earlier definitions.
- Surface grammar: `registro R { campo: Tipo, ... }`; `enum E { Vazio, ComDados(Tipo, ...) }`; `R { campo: valor, ... }`; `E::Variante(...)`; `valor.campo`; and `escolha enum { caso E::Variante(bindings...) => { statements } ... }`.

- [ ] Add failing HIR/parser tests for definitions, record construction, nested field reads, field assignment, enum payload construction, exhaustive arms, bindings/scopes, and spans.
- [ ] Verify each fails for the expected missing syntax/type behavior.
- [ ] Implement top-level type collection, nominal resolution, constructor/type checking, and new HIR nodes.
- [ ] Run frontend parser/semantic tests and format.

### Task 2: Static semantics and diagnostics

**Files:** frontend parser/semantics implementation and tests.

**Interfaces:** consume Task 1 definitions; produce exact field/variant lookup, exhaustiveness, mutability, arity/type, declaration-order and unsupported-cycle diagnostics.

- [ ] Add failing tests for omitted/duplicate fields, unknown fields, duplicate/omitted match arms, wrong payloads, immutable writes, and recursive record definitions.
- [ ] Confirm expected diagnostics and source spans.
- [ ] Implement semantic checks and per-arm lexical scopes.
- [ ] Run all frontend tests.

### Task 3: VM values, bytecode, and debugger

**Files:** `crates/ferro-bytecode/src/op.rs`, `crates/ferro-bytecode/src/ferro.rs`, `crates/ferro-vm/src/value.rs`, `crates/ferro-vm/src/vm.rs`, bytecode/service debugger tests.

**Interfaces:** consume typed HIR; produce deep-copyable record/enum `Value`s, field load/store and variant-match/payload bytecode, expanded debugger snapshots.

- [ ] Add failing execution tests for field reads/writes, `escolha`, vector-of-record values, copy isolation, payload scope, and string budgets.
- [ ] Confirm bytecode/VM lacks the operations.
- [ ] Implement value representation, deep clone, opcodes, compiler lowering, scopes, and debugger expansion.
- [ ] Run VM, bytecode, and debugger tests.

### Task 4: Rust generation and differential behavior

**Files:** `crates/ferro-rust/src/codegen.rs`, service runner tests.

**Interfaces:** consume same HIR; emit Rust record/enum definitions, constructors, field access, match statements, and independent copies.

- [ ] Add failing real-`rustc` runner tests for nested construction, event updates, vector-of-record, and string-copy budgets.
- [ ] Implement code generation and budget-aware deep-copy helpers.
- [ ] Compare VM and generated Rust output and runtime errors for shared fixtures.
- [ ] Run runner and rustfmt/source-map tests.

### Task 5: Service/editor integration and docs

**Files:** `crates/ferro-service/src/lib.rs`, `crates/ferro-service/src/rename.rs`, service integration tests, `docs/research/ferro_MIGRATION_MATRIX.md`, `docs/plans/ferro-only-migration.md`.

- [ ] Test symbol inspection/navigation/rename for type, field, variant, and pattern bindings where supported.
- [ ] Implement service type labels and safe symbol occurrences.
- [ ] Mark B1 partial or complete with exclusions and evidence.
- [ ] Run `cargo fmt --all -- --check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, frontend tests/build, and `git diff --check`.
- [ ] Review full diff for product identity, `.por` compatibility, and user-owned untracked files.
- [ ] Commit without coauthor/generated-by trailers, open PR, fix CI to green, merge, and delete branch/worktree.
