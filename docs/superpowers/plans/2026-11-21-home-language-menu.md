# Home page language menu Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore a compact classic start toolbar while exposing Portugol-RS and Rust creation through a keyboard-accessible language menu.

**Architecture:** Keep static markup in the existing desktop `index.html`, callback wiring in a small TypeScript view helper, and current language templates plus `newTab` integration in `main.ts`. Style the disclosure with the current CSS token system.

**Tech Stack:** Tauri 2, TypeScript, Vite, Node built-in test runner, existing CSS.

**Spec:** `docs/superpowers/specs/2026-11-21-home-language-menu-design.md`

## Global Constraints

- Preserve the existing architecture and classic Portugol default.
- Do not add dependencies, alter IPC, or change workspace file creation.
- Do not change `.por`, `.pors`, or `.rs` templates or save semantics.
- Do not report a browser-only capture as native Tauri visual validation.

## Review Focus

- Classic Novo Arquivo and Ctrl+N continue opening the classic template — test existing event wiring and stable primary markup.
- Portugol-RS and Rust options open their original templates and selected language — test callback selection and inspect `main.ts` integration.
- Alternate selection closes the menu — test event behavior.
- Escape closes and restores focus to the summary — test with a focus-tracking summary element.
- The disclosure remains readable in both built-in themes and does not inherit the large action icon layout — verify CSS token usage and build.

---

### Task 1: Test and implement the home language menu controller

**Files:**
- Create: `apps/desktop/src/home-new-file-menu.ts`
- Test: `apps/desktop/tests/home-new-file-menu.test.mjs`

**Interfaces:**
- Consumes: a `HTMLDetailsElement` containing `[data-home-new-language]` buttons and callbacks keyed by `portugol-rs` and `rust`.
- Produces: `mountHomeNewFileMenu(menu, actions)` wiring selection, disclosure close, and Escape close/refocus.

- [x] Write tests for each selection, menu close, Escape and focus restoration.
- [x] Run the test before implementing and confirm the expected failure because the controller is absent (module resolution error for `home-new-file-menu.ts`).
- [x] Implement the minimal controller using native button click and keydown events.
- [x] Run `node --experimental-strip-types apps/desktop/tests/home-new-file-menu.test.mjs` and verify all four behaviors pass.
- [x] Commit the controller and unit tests after they pass.

### Task 2: Wire accessible markup and preserve language templates

**Files:**
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/main.ts`
- Test: `apps/desktop/tests/workspace-explorer.test.mjs`

**Interfaces:**
- Consumes: `mountHomeNewFileMenu` and the existing `newTab` calls/templates.
- Produces: primary classic action plus a disclosure with the existing PORS and Rust action ids.

- [x] Add markup and CSS contract assertions for the primary classic action, disclosure label, controls, template choices, token colors and focus-visible states.
- [x] Run the markup test before changing the markup and confirm the expected failure because the disclosure is absent.
- [x] Replace dynamic standalone PORS/Rust buttons with the disclosure markup; preserve the classic click handler and connect alternate callbacks to the current templates.
- [x] Add CSS contract assertions, observe RED for the missing scoped selectors and token/focus styles, then implement those styles.
- [x] Run `node --experimental-strip-types tests/workspace-explorer.test.mjs` and `node --experimental-strip-types tests/home-new-file-menu.test.mjs`; all 18 checks pass.
- [x] Inspect the rendered start page and open menu using Playwright with UI-only Tauri fixtures; verify the classic toolbar, examples and both language choices render.
- [x] Commit the markup, callback wiring, styles and regression assertions after the focused tests pass.

### Task 3: Style, document and validate

**Files:**
- Modify: `apps/desktop/src/style.css`
- Modify: `docs/PARIDADE.md`
- Create: `docs/reports/FT-17-home-language-menu.md`

- [x] Scope the start toolbar’s large icon styles to direct action buttons; style the disclosure and menu using existing tokens, including hover/focus-visible and both built-in themes.
- [x] Run `npm --prefix apps/desktop test` (16/16) and `npm run build` (pass; one Vite chunk-size warning).
- [x] Run `cargo fmt --all -- --check`, `CARGO_TARGET_DIR=/home/wendel/Projetos/portugol-studio-rs/target cargo test --locked --workspace --exclude portugol-desktop`, `CARGO_TARGET_DIR=/home/wendel/Projetos/portugol-studio-rs/target cargo clippy --locked --workspace --exclude portugol-desktop --all-targets -- -D warnings`, `PKG_CONFIG_PATH=/tmp/ft10-pkgconfig CARGO_TARGET_DIR=/home/wendel/Projetos/portugol-studio-rs/target cargo check --locked --manifest-path apps/desktop/src-tauri/Cargo.toml --tests`, and `git diff --check` — all pass.
- [x] Record executed tests in FT-17 and keep the home-screen parity row partial pending equivalent native comparison screenshots.
- [x] Commit report and parity update before opening the PR.
- [ ] Open a PR; fix failures until CI is green; squash-merge and delete the remote branch and worktree.
