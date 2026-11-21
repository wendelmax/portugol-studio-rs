# Temas Visuais Populares Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add six offline color themes inspired by popular VS Code themes while retaining original and custom themes.

**Architecture:** Define typed built-in palettes in `theme-presets.ts`, list them in the existing HTML selector, and resolve their colors through the current theme application path. Expand the editable color-token registry so presets and custom themes can set every existing UI token.

**Tech Stack:** TypeScript, HTML, CSS custom properties, Node test runner, Vite, Tauri.

**Spec:** `docs/superpowers/specs/2026-11-21-temas-visuais-populares.md`

## Global Constraints

- Do not add runtime dependencies or network access.
- Preserve the existing Portugol, Dark, System, and user-created themes.
- Support Linux, Windows, and macOS.
- Apply presets through the existing `localStorage` preference and CSS variables.

## Review Focus

- A preset ID stored from a previous run must still resolve at startup — add a test for every stable ID in the preset registry.
- Every CSS token used by the application must have a preset color — assert all registered fields exist and are valid hex values.
- Base colors must select the correct CodeMirror and native color scheme — assert each preset's light/dark base and inspect an app session.
- Invalid custom colors must remain rejected by existing validation — keep current custom-theme tests/validation behavior.
- The original and custom options must coexist with new options — inspect selector groups and run the theme switching smoke test.

---

### Task 1: Add palette data and unit tests

**Files:**
- Create: `apps/desktop/src/theme-presets.ts`
- Create: `apps/desktop/tests/theme-presets.test.mjs`
- Modify: `apps/desktop/src/themes.ts`
- Modify: `apps/desktop/package.json`

**Interfaces:**
- `theme-presets.ts` exports a `BUILTIN_THEMES` map keyed by stable IDs; each value has `name`, `base`, and a complete `colors` map.
- `themes.ts` exports the shared `Theme`, `Base`, and `Colors` types and a `themeProfile(choice)` lookup for built-in and custom themes.

- [x] **Step 1: Write a failing test** asserting the six stable IDs, their exact names and light/dark bases, valid hex colors for every theme token, readable text contrast, and selector options for all original and popular themes.
- [x] **Step 2: Run it with Node 22 type stripping and verify failure** because the preset registry does not exist yet.
- [x] **Step 3: Add the color tokens and typed palette registry** for GitHub Light, GitHub Dark, One Dark Pro, Dracula, Catppuccin Mocha, and Night Owl.
- [x] **Step 4: Add the npm test script and run the focused test**, expecting all six profiles to have a complete valid palette.

### Task 2: Connect presets to the IDE selector and application

**Files:**
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/themes.ts`
- Modify: `apps/desktop/src/main.ts`
- Modify: `apps/desktop/src/style.css`

**Interfaces:**
- Theme selection IDs match `BUILTIN_THEMES` keys; `themeProfile(choice)` returns built-in/custom theme or `undefined`.
- `applyTheme` continues persisting the selected stable ID and applies the matching base and complete color map.

- [x] **Step 1: Add six options under a Popular themes group** while retaining original options and the existing custom group.
- [x] **Step 2: Run the focused unit test and Vite build** to catch invalid IDs and type errors.
- [x] **Step 3: Connect selection/startup/custom-theme editing to `themeProfile`**, applying the selected base and preset colors; extend custom color fields to all theme tokens.
- [x] **Step 4: Run Node tests, frontend build, Rust desktop tests, and Clippy**; inspect each preset in the desktop app and verify switching persists after restart.

### Task 3: Document and submit

**Files:**
- Modify: `docs/PARIDADE.md`
- Modify: `docs/superpowers/specs/2026-11-21-temas-visuais-populares.md`
- Modify: `docs/superpowers/plans/2026-11-21-temas-visuais-populares.md`

- [ ] **Step 1: Document preset names, inspired references, and local/offline behavior** in the theme parity inventory.
- [ ] **Step 2: Review the full diff and run all project checks** before creating the commit and PR.
- [ ] **Step 3: Merge the PR only after every required CI check is green.**
