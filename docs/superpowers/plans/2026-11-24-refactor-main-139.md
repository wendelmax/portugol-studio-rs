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

### Task 43: Editor theme lifecycle

**Files:**
- Create: `apps/desktop/src/editor/theme-controller.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/editor-theme-controller.test.mjs`

- [x] Add tests for applying a selected theme and refreshing both editors and help on system theme changes.
- [x] Move editor theme refresh/application orchestration into a controller.
- [x] Run frontend tests, typecheck, and build.

### Task 42: Editor navigation bindings

**Files:**
- Create: `apps/desktop/src/editor/navigation-bindings.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/editor-navigation-bindings.test.mjs`
- Modify: `apps/desktop/tests/editor-extensions.test.mjs`

- [x] Add tests for Rust exclusion, click modifiers/button/coordinates, quiet navigation, and F12 cursor navigation.
- [x] Move editor click and F12 handlers behind an injected navigation binding.
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
- [x] Commit the language capability group (#171).

### Task 17: Tab activation lifecycle

**Files:**
- Create: `apps/desktop/src/state/tab-activation.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/tab-activation.test.mjs`

- [x] Add tests for invalid IDs, home/help navigation, active document preservation, and file activation sequencing.
- [x] Extract tab activation orchestration while keeping editor, preview, workspace, and DOM integrations in the shell.
- [x] Run the full frontend tests (64/64), typecheck, and build.
- [x] Commit the tab activation group (#172).

### Task 18: System file intake

**Files:**
- Create: `apps/desktop/src/files/system-files.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/system-files.test.mjs`

- [x] Add tests for supported `.pex`, `.por`, `.pors`, and `.rs` paths, unsupported-file warnings, per-file errors, and continued processing.
- [x] Extract incoming path dispatch while leaving Tauri event subscriptions in the shell.
- [x] Run the full frontend tests (65/65), typecheck, and build.
- [x] Commit the system file intake group (#173).

### Task 19: Symbol navigation

**Files:**
- Create: `apps/desktop/src/editor/symbol-navigation.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/symbol-navigation.test.mjs`

- [x] Add tests for Portugol/PORS resolution, selection and focus, Rust diagnostics, stale responses, and quiet failures.
- [x] Extract asynchronous declaration navigation while preserving the editor callback contract.
- [x] Run the full frontend tests (66/66), typecheck, and build.
- [x] Commit the symbol navigation group (#174).

### Task 20: Editor formatting

**Files:**
- Create: `apps/desktop/src/editor/format.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/editor-format.test.mjs`

- [x] Add tests for language-specific formatting, no-op output, runtime/rustfmt guards, and stale asynchronous results.
- [x] Extract format orchestration while preserving edit annotations, status messages, and focus behavior.
- [x] Run the full frontend tests (67/67), typecheck, and build.
- [x] Commit the editor formatting group (#175).

### Task 21: IDE shortcut catalog

**Files:**
- Create: `apps/desktop/src/commands/ide-shortcuts.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/ide-shortcuts.test.mjs`

- [x] Add tests for editor, file, navigation, debugging, and palette shortcut matching and dispatch.
- [x] Move the IDE shortcut catalog and shortcut-specific filtering loops into the commands module.
- [x] Run the full frontend tests (68/68), typecheck, and build.
- [x] Commit the shortcut catalog group (#176).

### Task 22: Editor update lifecycle

**Files:**
- Create: `apps/desktop/src/editor/update-controller.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/editor-update-controller.test.mjs`

- [x] Add tests for diagnostic clearing, exercise invalidation, preview staleness, selection updates, and guarded centering.
- [x] Extract CodeMirror update-event orchestration behind explicit callbacks.
- [x] Run the full frontend tests (69/69), typecheck, and build.
- [x] Commit the editor update group (#177).

### Task 23: CodeMirror extension setup

**Files:**
- Create: `apps/desktop/src/editor/extensions.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/editor-extensions.test.mjs`

- [x] Add tests for language-specific extensions and the editor's keyboard and mouse navigation hooks.
- [x] Extract extension assembly while keeping language services and shell actions injected.
- [x] Run the full frontend tests (70/70), typecheck, and build.
- [x] Commit the CodeMirror setup group (#178).

### Task 24: Rust source linter

**Files:**
- Create: `apps/desktop/src/editor/rust-linter.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/rust-linter.test.mjs`

- [x] Add tests for availability guards, stale document/tab results, compiler failures, and diagnostic conversion.
- [x] Extract Rust source lint orchestration while keeping shell state and rendering injected.
- [x] Run the full frontend tests (71/71), typecheck, and build.
- [x] Commit the Rust linter group (#179).

### Task 25: Portugol and PORS source linter

**Files:**
- Create: `apps/desktop/src/editor/portugol-linter.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/portugol-linter.test.mjs`

- [x] Add tests for language routing, Rust clearing, diagnostic aggregation, and stale inspection responses.
- [x] Extract Portugol/PORS lint orchestration with inspection and UI callbacks injected.
- [x] Run the full frontend tests (72/72), typecheck, and build.
- [x] Commit the source linter group (#180).

### Task 26: Execution dispatcher

**Files:**
- Create: `apps/desktop/src/run/dispatcher.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/run-dispatcher.test.mjs`

- [x] Add tests for busy/no-tab guards, language dispatch, pause/step routing, and stop delegation.
- [x] Extract run command orchestration while injecting the language execution controllers.
- [x] Run the full frontend tests (73/73), typecheck, and build.
- [x] Commit the execution dispatcher group (#181).

### Task 27: Exercise tab creation

**Files:**
- Create: `apps/desktop/src/state/exercise-tab.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/exercise-tab.test.mjs`

- [x] Add tests for starter source, PORS language, clean-tab replacement, and workspace reactivation.
- [x] Extract exercise-to-tab orchestration with tab and workspace operations injected.
- [x] Run the full frontend tests (74/74), typecheck, and build.
- [x] Commit the exercise tab group (#182).

### Task 28: Paused debugger presentation

**Files:**
- Create: `apps/desktop/src/run/pause-presentation.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/pause-presentation.test.mjs`

- [x] Add tests for paused-state updates, current run selection, stack-frame navigation, panel visibility, and language-specific status.
- [x] Extract pause presentation orchestration with editor/shell callbacks injected.
- [x] Run the full frontend tests (75/75), typecheck, and build.
- [x] Commit the pause presentation group (#183).

### Task 29: Panel navigation

**Files:**
- Create: `apps/desktop/src/layout/panel-navigation.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/panel-navigation.test.mjs`

- [x] Add tests for learning mode, bottom-panel selection, tab semantics, and panel-specific controls.
- [x] Extract panel navigation and visibility orchestration with shell callbacks injected.
- [x] Run the full frontend tests (76/76), typecheck, and build.
- [x] Commit the panel navigation group (#184).

### Task 30: Symbol rename workflow

**Files:**
- Create: `apps/desktop/src/editor/symbol-rename.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/symbol-rename.test.mjs`

- [x] Add tests for language routing, stale requests, submission, change application, and error display.
- [x] Extract rename target and dialog orchestration with editor and UI dependencies injected.
- [x] Run the full frontend tests (77/77), typecheck, and build.
- [x] Commit the symbol rename group (#185).

### Task 31: Center-source preference

**Files:**
- Create: `apps/desktop/src/editor/center-source.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/center-source.test.mjs`

- [x] Add tests for preference restore, inactive-tab guard, center toggling, focus, and storage failures.
- [x] Extract center-source preference state and UI orchestration.
- [x] Run the full frontend tests (78/78), typecheck, and build.
- [x] Commit the editor centering group (#186).

### Task 32: Toolchain capability startup

**Files:**
- Create: `apps/desktop/src/shell/toolchain-status.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/toolchain-status.test.mjs`

- [x] Add tests for Rust, Cargo, and rustfmt success/failure and active-tab updates.
- [x] Extract startup capability checks and status UI orchestration.
- [x] Run the full frontend tests (79/79), typecheck, and build.
- [x] Commit the toolchain status group (#187).

### Task 33: Language controls view

**Files:**
- Create: `apps/desktop/src/state/language-controls-view.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/language-controls-view.test.mjs`
- Modify: `apps/desktop/tests/language-capabilities.test.mjs`

Ruling: keep capability derivation in `language-capabilities.ts` and extract only DOM presentation into an injected view controller — this separates rules from their rendering without changing language behavior; if this boundary is too narrow, it adds one small state view module.

- [x] Add tests for default-title preservation and language-dependent controls.
- [x] Extract DOM updates for run, format, argument, library, and symbol controls.
- [x] Run the full frontend tests (80/80), typecheck, and build.
- [x] Commit the language controls view group (#188).

### Task 34: Tab collection state

**Files:**
- Create: `apps/desktop/src/state/tab-collection.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/tab-collection.test.mjs`

Ruling: extract the mutable tab list and active ID into a small store while retaining the existing factory, activation, closing, and view modules — centralizes shared state without bundling unrelated lifecycle behavior; if the store boundary is wrong, the risk is one extra indirection for tab reads.

- [x] Add tests for initial active tab, add/remove/replace, and defensive list snapshots.
- [x] Route shared tab reads and writes through the collection store.
- [x] Run the full frontend tests (81/81), typecheck, and build.
- [x] Commit the tab collection group (#189).

### Task 35: Editor utility insertions

**Files:**
- Create: `apps/desktop/src/editor/utility-insertions.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/utility-insertions.test.mjs`

Ruling: group library insertion, snippet buttons, and utility panel tabs behind one injected editor utility controller — they are the three controls in the same utilities panel; if the grouping is too broad, the cost is a slightly wider controller API.

- [x] Add tests for stale library requests, running/PORS guards, snippet insertion, and utility tab selection.
- [x] Extract the utilities panel controls and keep library calls behind `api.ts` injection.
- [x] Run the full frontend tests (82/82), typecheck, and build.
- [x] Commit the utility insertions group (#190).

### Task 36: Run toolbar and console bindings

**Files:**
- Create: `apps/desktop/src/run/toolbar.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/run-toolbar.test.mjs`

Ruling: group run/debug buttons, panel shortcuts, console copy/clear, and runtime input submission in one toolbar controller — they all wire the run/console surface; if the grouping is too broad, the cost is one controller with several injected UI actions.

- [x] Add tests for run/debug dispatch, pause language guard, console copy/error, panel selection, and runtime input submission.
- [x] Extract run toolbar and console event bindings.
- [x] Run the full frontend tests (83/83), typecheck, and build.
- [x] Commit the toolbar group.

### Task 37: Activity bar bindings

**Files:**
- Create: `apps/desktop/src/shell/activity-bar.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/activity-bar.test.mjs`

Ruling: move the activity bar's five navigation bindings behind a small injected controller. Keep its `aria-pressed` updates, explorer toggle state, and current panel/view routing unchanged; if this boundary is too small, the cost is one simple shell adapter.

- [x] Add tests for selection state and each activity destination.
- [x] Extract activity bar event bindings and explorer/settings adapters.
- [x] Run the full frontend tests (84/84), typecheck, and build.
- [x] Commit the activity bar group.

### Task 38: Keyboard shortcut dispatch

**Files:**
- Create: `apps/desktop/src/commands/keyboard-dispatch.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/keyboard-dispatch.test.mjs`

Ruling: extract the global keydown listener and tab-cycle helper into a focused command/navigation module. Preserve dialog and Escape-menu guards, default prevention, shortcut dispatch, and tab wrapping; if the boundary is too narrow, the cost is one small adapter around the existing shortcut catalog.

- [x] Add tests for guards, menu close/focus, shortcut dispatch, and tab cycling.
- [x] Extract keydown listener and tab-cycle helper.
- [x] Run the full frontend tests (85/85), typecheck, and build.
- [x] Commit the keyboard dispatch group.

### Task 39: Tab shell composition

**Files:**
- Create: `apps/desktop/src/state/tabs-shell.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/tabs-shell.test.mjs`
- Update: `apps/desktop/tests/tabs-state.test.mjs` and `apps/desktop/tests/tabs-view.test.mjs`

Ruling: compose the tab collection, factory, dirty-state helper, closer, and tab view in a `tabs-shell` factory. Keep tab activation effects in the existing activation controller; this removes lifecycle wiring from `main.ts` without creating a second source of tab behavior.

- [x] Add tests for tab creation, selection, dirty state, rendering, and close behavior through the shell composition.
- [x] Move tab lifecycle composition behind `createTabsShell`.
- [x] Run the full frontend tests (86/86), typecheck, and build.
- [x] Commit the tab shell group.

### Task 40: Recent-files view

**Files:**
- Create: `apps/desktop/src/files/recent-view.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/recent-view.test.mjs`

Ruling: extract only recent-files rendering and open/error click behavior into an injected view. Keep persistence and ordering in `files/recent.ts`; if the view module is too narrow, the cost is one small adapter.

- [x] Add tests for empty state, path labels, opening files, and error status.
- [x] Move recent-files DOM rendering to `recent-view.ts`.
- [x] Run the full frontend tests (87/87), typecheck, and build.
- [x] Commit the recent-files view group.

### Task 41: Shared editor setup

**Files:**
- Create: `apps/desktop/src/editor/setup.ts`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/editor-setup.test.mjs`
- Modify: `apps/desktop/tests/rust-source-check.test.mjs`

- [x] Add a contract test for the editor setup boundary and verify it fails before extraction.
- [x] Move shared CodeMirror compartments, centering, breakpoint gutter, and language linters into an injected setup factory.
- [x] Keep CodeMirror execution services routed through existing controllers and `api.ts`.
- [x] Run frontend tests, typecheck, and build.
