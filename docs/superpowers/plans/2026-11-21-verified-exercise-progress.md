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
- Produces `createExerciseProgress(storage: StorageLike)` with `read()`, `record(identity, source, result)`, and `hasPassed(identity)`.

- [ ] Write tests for stable built-in/imported identities, exact revision hashes, only complete real evaluations passing, failed attempts, malformed storage, and unavailable storage.
- [ ] Run `cd apps/desktop && node --experimental-strip-types tests/exercise-progress.test.mjs`; confirm the missing module/API fails.
- [ ] Implement the versioned local store with guarded WebCrypto and storage access; store metadata only.
- [ ] Run the focused tests and confirm all pass.

### Task 2: Connect real exercise grading

**Files:**
- Modify: `apps/desktop/src/exercises.ts`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/tests/exercise-progress.test.mjs`

**Interfaces:** consumes Task 1 store and the existing `ExerciseDocument` and `checkExercise` contract.

- [ ] Add tests proving browse/open does not record, complete evaluation records the tested revision, partial evaluation is incomplete, and rejected grading records nothing.
- [ ] Run the new focused tests and verify the integration assertions fail before implementation.
- [ ] Record after `checkExercise` resolves; label the result with tested revision status and annotate completed built-in exercises in the catalog.
- [ ] Keep user-facing language clear that completion applies to an evaluated revision; leave imported exercise identity distinct from catalog IDs.
- [ ] Run focused tests and TypeScript build.

### Task 3: Full verification and traceability

**Files:**
- Modify: `docs/PARIDADE.md`
- Modify: `docs/superpowers/specs/2026-11-21-verified-exercise-progress-design.md`

- [ ] Update parity notes to distinguish existing auto-grading from the new validated local progress.
- [ ] Run `cd apps/desktop && npm test`.
- [ ] Run `cd apps/desktop && npm run build`.
- [ ] Run `cargo test --workspace`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Review changed files and commit the feature.
