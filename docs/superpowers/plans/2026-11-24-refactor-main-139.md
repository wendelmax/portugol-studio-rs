# IDE main.ts Refactoring Implementation Plan

> **For agentic workers:** implement each extraction in a separate, reviewable PR. Keep behavior unchanged and validate each step.

**Goal:** Reduce `apps/desktop/src/main.ts` below 300 lines by extracting IDE responsibilities into focused modules while preserving runtime behavior.

**Architecture:** Keep `main.ts` as composition/bootstrap. Extract domain and UI responsibilities into existing module areas (`files/`, `state/`, `editor/`, `run/`, `learning/`, `commands/`, `shell/`) and pass shared state through explicit callbacks or small stores. Keep Tauri calls behind `api.ts`.

**Tech Stack:** TypeScript, Tauri 2, CodeMirror 6, Node's built-in test runner, Vite.

**Spec:** [Issue #139](https://github.com/wendelmax/portugol-studio-rs/issues/139)

## Global Constraints

- Preserve existing behavior and visuals during refactoring.
- Keep each new module below approximately 500 lines.
- Keep `main.ts` below 300 lines when the issue is complete.
- UI modules call Tauri through `api.ts`, not direct `invoke` calls.
- Add focused tests for extracted state and behavior.
- Follow `CONTRIBUTING.md`; deliver each coherent group as a separate PR with `Parte de #139` until the acceptance criteria are met.

## Review Focus

- Saving a file changes its language only after a successful write; test save failure and extension detection.
- Opening an already-open file activates the existing tab without reading it again; test duplicate path behavior.
- Switching tabs retains each document's editor state; test state-store selection and replacement.
- Execution events for stale run IDs do not update the active editor; preserve and test event routing.
- Extracted modules do not bypass `api.ts` for Tauri calls; inspect module imports and run frontend tests/build.

---

### Task 1: File opening and saving

**Files:**
- Create: `apps/desktop/src/files/io.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/files-io.test.mjs`
- Modify: `apps/desktop/tests/source-map.test.mjs`

- [x] Write focused tests for duplicate opens, pristine-tab replacement, dialog cancellation, save-as extension, workspace saves, and failed writes.
- [x] Run the new test and confirm the missing module failure.
- [x] Extract file dialog, read, write, extension, and recent-file orchestration behind injected dependencies.
- [x] Run the new test, all frontend tests, typecheck, and build.
- [ ] Commit the file-I/O group.

### Task 2: Execution and debugger orchestration

**Files:** `apps/desktop/src/main.ts`, `apps/desktop/src/run/`, related tests.

- [ ] Extract run state, execution flows, and debugger orchestration in cohesive groups.
- [ ] Add tests for run lifecycle and stale event behavior.
- [ ] Validate frontend tests, typecheck, and build after each group.

### Task 3: Editor setup, tabs, panels, and remaining shell wiring

**Files:** `apps/desktop/src/main.ts`, `apps/desktop/src/editor/`, `apps/desktop/src/state/`, `apps/desktop/src/shell/`, related tests.

- [ ] Extract editor setup and tab state using explicit boundaries.
- [ ] Extract exercise/test panels and remaining shell initialization.
- [ ] Add tests for extracted state and capability behavior.
- [ ] Verify `main.ts` is below 300 lines and no new module exceeds approximately 500 lines.
- [ ] Run all required frontend validations and review the final diff.

### Task 4: Learning-mode controller

**Files:**
- Create: `apps/desktop/src/learning/mode-controller.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/learning-mode-controller.test.mjs`
- Modify: `apps/desktop/tests/learning-mode.test.mjs`

- [x] Write and run failing tests for unavailable Rust, workspace persistence, and preview scheduling/cancellation.
- [x] Extract mode selection effects behind callbacks and keep the shell as the composition layer.
- [x] Run the full frontend tests, typecheck, and build.
- [ ] Commit the learning-mode group.
