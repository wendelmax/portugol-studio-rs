# Ferro Studio Next Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Recompose the existing Tauri IDE into the Ferro Studio Next desktop shell while preserving current language/runtime behavior.

**Architecture:** Keep the HTML, TypeScript, CodeMirror, Tauri commands, and Rust services already in `apps/desktop`. Move existing controls into the proposed visual regions, add only view state needed to navigate those regions, and reuse existing stores/adapters.

**Tech Stack:** Tauri 2, Vite 8, TypeScript, vanilla HTML/CSS, CodeMirror, Node `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-22-ferro-studio-next-shell-design.md`

## Global Constraints

- Preserve the current Rust + Tauri architecture, CodeMirror editor, IPC, file formats, keyboard shortcuts, themes, and existing Ferro workflows.
- Do not fabricate execution, translation, test, compiler, or memory-trace results.
- Rust learning mode and `.rs` execution remain disabled until their real capabilities exist.
- Do not add frontend frameworks, runtime dependencies, Tauri permissions, or arbitrary shell access.
- Keep `docs/ferro-studio-next-integration/` and other user-provided untracked docs untouched.

## Review Focus

- Moving `#tabs` or action controls must preserve the existing handlers and dirty indicators; test stable IDs and keyboard actions.
- The editor must remain usable at 1280×800 with the inspector collapsed; test responsive layout rules.
- Hiding the inspector or explorer must not lose its state or break pane sizing; test toggle and keyboard access.
- Learning and debug views must continue to report their actual provenance and capabilities; test the accessible labels and disabled states.
- Existing themes and theme persistence must survive the new default palette; test theme selection and stored preferences.

---

### Task 1: Add the Next visual theme tokens

**Files:**
- Modify: `apps/desktop/src/theme-presets.ts`
- Modify: `apps/desktop/src/style.css`
- Modify: `apps/desktop/tests/theme-presets.test.mjs`

**Interfaces:** Add a named `ferro-studio-next` theme preset using semantic shell/editor/console colors. Preserve all existing preset IDs and storage behavior.

- [x] Write a failing test that the preset exists, has the exact core colors from the spec, and can be selected without removing existing presets.
- [x] Run `node --experimental-strip-types --test tests/theme-presets.test.mjs` and confirm the new assertions fail.
- [x] Add the preset tokens and option to the existing theme selector; keep legacy themes intact.
- [x] Rerun the focused theme test and `npm test`.
- [ ] Commit `feat(ui): add Ferro Studio Next theme tokens`.

### Task 2: Recompose the application shell and activity rail

**Files:**
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/src/style.css`
- Create: `apps/desktop/tests/design-as-code-shell.test.mjs`

**Interfaces:** Preserve existing IDs (`tabs`, `btn-run`, `btn-debug`, `btn-stop`, `btn-explorer`, save/open/help) while moving them into the app bar, tab row, or rail. Add `data-activity="project|learn|examples|debug|settings"` controls and route each to the existing project, learning, home examples, debug, or editor-settings behavior. Do not add unsupported Share behavior.

- [ ] Write failing static DOM/contract tests for branded title bar, functional activity rail, central editor tabs, and existing run/debug IDs.
- [ ] Run `node --experimental-strip-types --test tests/design-as-code-shell.test.mjs` and confirm it fails on missing structure.
- [ ] Reorder the existing DOM into app bar, activity rail, project explorer, central editor workspace, right inspector, and bottom panel without duplicating IDs.
- [ ] Connect every rail control to an existing action and keep all current run/save/debug shortcuts unchanged.
- [ ] Rerun the focused test, `npm test`, and `npm run build`.
- [ ] Commit `feat(ui): compose desktop shell and activity rail`.

### Task 3: Integrate learning and variable inspection in the right panel

**Files:**
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/src/style.css`
- Test: `apps/desktop/tests/design-as-code-shell.test.mjs`

**Interfaces:** Reuse `learning-content.ts`, existing manual concept selection, `#variable-inspector`, and real VM debug values. Keep teaching provenance visible; retain library/model and outline controls in a discoverable inspector section.

- [x] Extend shell tests to require the learning card, variable panel, and simulation provenance under the inspector.
- [x] Place existing learning panel in the right inspector slot, wire rail navigation, and preserve variable/debug behavior.
- [x] Verify no learning progress is inferred from opening the panel and no ownership simulation is labeled runtime state.
- [x] Rerun focused tests and full `npm test`.
- [x] Rerun `npm run build` after final shell integration.
- [ ] Commit `feat(ui): integrate learning inspector in desktop shell`.

### Task 4: Match desktop proportions and responsive behavior

**Files:**
- Modify: `apps/desktop/src/style.css`
- Modify: `apps/desktop/src/main.ts` only if initial panel visibility needs viewport-aware setup
- Test: `apps/desktop/tests/design-as-code-shell.test.mjs`

**Interfaces:** Use the existing `PaneLayout` persistence and splitter code. At widths ≥1440 px, show the inspector by default; at 1200–1439 px, start it collapsed; keep the source editor and Run control visible at 1280×800.

- [x] Add failing CSS/contract assertions for the 1440 px breakpoint, 56 px activity rail, and 300–380 px inspector.
- [x] Implement desktop proportions (56 px rail, 232 px explorer ratio, 300–380 px inspector) and compact startup without overriding user-controlled panel state after startup.
- [ ] Review 1280 px editor width and horizontal overflow in native UI.
- [x] Run `npm test`, production build, `cargo test --workspace`, and `git diff --check` (native screenshot pending; see report).
- [ ] Verify `PaneLayout` resize and restore tests still pass.
- [ ] Run `npm test`, `npm run build`, `git diff --check`, and native `npm run tauri dev` smoke at 1536×960 and 1280×800 where the local desktop environment allows.
- [ ] Record exact screenshot/visual evidence and any automation limits in `docs/reports/FEAT-UI-NEXT-SHELL.md`.
- [ ] Commit `feat(ui): align shell layout with Next design contract`.
