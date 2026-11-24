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
- [x] Commit the file-I/O group.

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
- [x] Commit the learning-mode group.

### Task 5: Exercise and Cargo test panel

**Files:**
- Create: `apps/desktop/src/run/tests-panel.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/run-tests-panel-controller.test.mjs`

- [x] Write and run failing tests for rendering, completed results, stale documents, and Cargo run state.
- [x] Extract panel rendering and exercise-test orchestration behind view and service callbacks.
- [x] Run the full frontend tests, typecheck, and build.
- [x] Commit the test-panel group.

### Task 6: Cargo project orchestration

**Files:**
- Create: `apps/desktop/src/run/cargo-project.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/cargo-project-controller.test.mjs`

- [x] Write and run failing tests for dirty workspace guards, Cargo diagnostics, test results, and stale documents.
- [x] Extract `cargo check` and `cargo test` lifecycle orchestration behind callbacks; keep Tauri commands in `api.ts`.
- [x] Run the full frontend tests, typecheck, and build.
- [x] Commit the Cargo orchestration group.

### Task 7: Run controls

**Files:**
- Create: `apps/desktop/src/run/controls.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/run-controls.test.mjs`

- [x] Add failing tests for active, paused, and stopped run controls.
- [x] Extract button, input, and inspector state synchronization behind callbacks.
- [x] Run frontend tests, typecheck, and build.
- [x] Commit the run-controls group.

### Task 8: Debugger commands

**Files:**
- Create: `apps/desktop/src/run/debugger-commands.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/debugger-commands.test.mjs`

- [x] Add failing tests for active-run breakpoint updates, stale debug commands, and stop status.
- [x] Extract breakpoint synchronization, debugger commands, and stop orchestration behind callbacks.
- [x] Run frontend tests, typecheck, and build.
- [x] Commit the debugger-commands group.

### Task 9: Rust project execution

**Files:**
- Create: `apps/desktop/src/run/rust-execution.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/rust-execution-controller.test.mjs`
- Modify: Cargo, learning-mode, process-arguments, and source-map contract tests.

- [x] Add controller tests for toolchain guards, standalone execution, stale documents, dirty Cargo files, and selected Cargo targets.
- [x] Extract Rust and Cargo process orchestration; retain Tauri calls behind `api.ts`.
- [x] Update static contract tests to inspect the extracted module boundary.
- [x] Run frontend tests, typecheck, and build.
- [x] Commit the Rust execution group.

### Task 10: Portugol-RS execution

**Files:**
- Create: `apps/desktop/src/run/portugol-rs-execution.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/portugol-rs-execution-controller.test.mjs`
- Modify: `apps/desktop/tests/pors-input.test.mjs`, `run-event-router.test.mjs`, and `source-map.test.mjs`.

- [x] Add tests for VM startup, VM diagnostics, generated-Rust source mappings, and stale result cleanup.
- [x] Extract Portugol-RS VM and generated-Rust execution behind callbacks.
- [x] Keep backend calls through `api.ts` and preserve startup event replay.
- [x] Run frontend tests, typecheck, and build.
- [x] Commit the Portugol-RS execution group (#165).

### Task 11: Run event handling

**Files:**
- Create: `apps/desktop/src/run/run-events.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/run-events-controller.test.mjs`
- Modify: `apps/desktop/tests/run-event-router.test.mjs`, `pors-input.test.mjs`, and `tsconfig.json`.

- [x] Add focused tests for active/stale/startup events, input prompts, completion, and background runtime errors.
- [x] Extract run event handlers behind explicit UI and state callbacks.
- [x] Update contract tests for the new module boundary and allow the explicit TypeScript extension used by Node's test runner.
- [x] Run frontend tests, typecheck, and build.
- [x] Commit the run-event group (#166).

### Task 12: Classic Portugol execution

**Files:**
- Create: `apps/desktop/src/run/classic-execution.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/classic-execution-controller.test.mjs`

- [x] Add tests for run arguments, breakpoints, startup event replay, diagnostics, and cleanup after listener errors.
- [x] Extract classic execution orchestration behind callbacks while preserving the `api.ts` backend boundary.
- [x] Run the full frontend tests (59/59), typecheck, and build.
- [x] Commit the classic execution group (#167).

### Task 13: Tab strip view

**Files:**
- Create: `apps/desktop/src/state/tabs-view.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/tabs-view.test.mjs`

Ruling: extract the tab strip's rendering and event wiring as a view module, leaving tab lifecycle and shared-state ownership for a later isolated step. This yields independently verifiable UI behavior; if this boundary proves too small, it adds one module without moving the larger mutable tab state.

- [x] Add tests for strip structure, dirty indicators, callbacks, active title, and shell integration.
- [x] Move tab strip DOM creation into a view module with shell callbacks.
- [x] Run the full frontend tests (60/60), typecheck, and build.
- [x] Commit the tab view group (#168).

### Task 14: Tab factory and metadata

**Files:**
- Create: `apps/desktop/src/state/tabs.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/tabs-state.test.mjs`

- [x] Add tests for unique tab IDs, language detection, metadata preservation, naming, and active-buffer dirty state.
- [x] Extract tab construction and metadata helpers into the state module; keep lifecycle wiring in the shell.
- [x] Run the full frontend tests (61/61), typecheck, and build.
- [x] Commit the tab state group (#169).

### Task 15: Tab closing lifecycle

**Files:**
- Create: `apps/desktop/src/state/tab-closer.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/tab-closer.test.mjs`

- [x] Add tests for help/tab closing, dirty-buffer confirmation, cancellation, and active-tab selection.
- [x] Extract confirmation and collection mutation behind state callbacks.
- [x] Run the full frontend tests (62/62), typecheck, and build.
- [x] Commit the tab closer group (#170).

### Task 16: Language capability controls

**Files:**
- Create: `apps/desktop/src/state/language-capabilities.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/language-capabilities.test.mjs`

- [x] Add tests for Portugol/PORS/Rust capabilities, Cargo detection, execution state, formatting, libraries, and symbol actions.
- [x] Extract derived control state and leave DOM updates in the shell.
- [x] Run the full frontend tests (63/63), typecheck, and build.
- [ ] Commit the language capability group.
