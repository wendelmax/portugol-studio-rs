# Verified Exercise Progress Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist local exercise attempts and completion only when real checker cases pass for a specific source revision.

**Architecture:** A pure TypeScript store hashes exercise identity and source revisions with WebCrypto and persists versioned metadata in localStorage. `exercises.ts` records only successful checker responses and annotates the existing exercise selector; Rust, Tauri commands, and the `.pex` contract remain unchanged.

**Tech Stack:** TypeScript, Web Crypto SHA-256, browser localStorage, Node `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-21-verified-exercise-progress-design.md`

## Global Constraints

- Completion requires real test cases: `total > 0` and `passed === total`.
- Progress must remain tied to the exact code revision tested.
- Opening a concept or exercise never records progress.
- Keep progress local; do not store source or personal data.
- Checker infrastructure failures do not produce attempts.
- Preserve `checkExercise` and its Rust/Tauri response contract.

## Review Focus

- Empty/static-only assessments must not mark completion: test `total === 0`.
- Partial success must remain incomplete: test `0 < passed < total`.
- A changed source revision must not masquerade as the revision that passed: test distinct hashes.
- Invalid or older local storage must recover without throwing: test malformed/version-mismatched values.
- Persistence or WebCrypto unavailability must not block exercise correction: test guarded storage/hash failures.

---

### Task 1: Local progress store

**Files:**
- Create: `apps/desktop/src/exercise-progress.ts`
- Create: `apps/desktop/tests/exercise-progress.test.mjs`

**Interfaces:**
- Produces async `exerciseIdentity(id: number, contents?: string): Promise<string | null>`.
- Produces async `sourceRevision(source: string): Promise<string | null>`.
- Produces `createExerciseProgress(storage?: StorageLike)` with `read()`, `record(identity, source, result)`, and `hasPassed(identity)`.

- [x] Write tests for stable built-in/imported identities, exact revision hashes, only complete real evaluations passing, failed attempts, malformed storage, and unavailable storage.
- [x] Run `cd apps/desktop && node --experimental-strip-types tests/exercise-progress.test.mjs`; confirm the missing module/API fails.
- [x] Implement the versioned local store with guarded WebCrypto and storage access; store metadata only.
- [x] Run the focused tests and confirm all pass.

### Task 2: Connect real exercise grading

**Files:**
- Modify: `apps/desktop/src/exercises.ts`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/tests/exercise-progress.test.mjs`

**Interfaces:** consumes Task 1 store and the existing `ExerciseDocument` and `checkExercise` contract.

- [x] Add tests proving complete evaluation records the tested revision, partial evaluation is incomplete, and rejected grading records nothing.
- [x] Run the new focused tests and verify missing APIs/feedback assertions fail before implementation.
- [x] Record after `checkExercise` resolves; label the result with tested revision status and annotate completed built-in exercises in the catalog.
- [x] Keep user-facing language clear that completion applies to an evaluated revision; leave imported exercise identity distinct from catalog IDs.
- [x] Run focused tests and TypeScript build.

### Task 3: Full verification and traceability

**Files:**
- Modify: `docs/PARIDADE.md`
- Modify: `docs/superpowers/plans/2026-11-21-verified-exercise-progress.md`

- [x] Update parity notes to distinguish existing auto-grading from the new validated local progress.
- [x] Run `cd apps/desktop && npm test` (4 test files pass).
- [x] Run `cd apps/desktop && npm run build` (TypeScript and Vite pass).
- [ ] Run `cargo test --workspace`; local run is blocked while compiling Tauri by missing `webkit2gtk-4.1` and `javascriptcoregtk-4.1` development libraries. `cargo test --workspace --exclude portugol-desktop` passed all remaining Rust tests after enabling the test's loopback listener.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`; local run is blocked by the same missing WebKitGTK development library. `cargo clippy --workspace --exclude portugol-desktop --all-targets -- -D warnings` passed.
- [x] Review changed files and commit the feature documentation; CI remains the gate for full workspace validation.
