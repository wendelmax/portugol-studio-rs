# Tests Panel Empty State Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the FT-06 Tests tab with a truthful empty state and no fabricated test execution.

**Architecture:** Extend the existing bottom-panel tab union and HTML tab/panel pair. Keep this UI-only because the repository has no TestService or test-discovery contract.

**Tech Stack:** Existing TypeScript, HTML, CSS, Node test runner, Tauri v2 (unchanged).

**Spec:** `docs/superpowers/specs/2026-11-21-tests-panel-empty-state-design.md`

## Global Constraints

- Preserve the existing Rust + Tauri architecture and current execution behavior.
- Never show fake test counts or outcomes.
- Keep unsupported execution capabilities explicit.
- Do not introduce a new dependency or backend command.

## Review Focus

- Missing test runner must remain an empty state, not a passing result; static contract test pins the exact message and rejects fake count copy.
- Selecting the Testes tab must show only its panel; source contract test checks tab/panel association and dispatch wiring.
- Panel must remain reachable and named to assistive technology; source contract test checks tab and status roles.
- Tabs must support arrow/Home/End keyboard navigation; unit tests verify navigation and wrapping.

---

### Task 1: Add the Tests tab and honest empty state

**Files:**
- Modify: `apps/desktop/src/panel-tabs.ts`
- Modify: `apps/desktop/index.html`
- Modify: `apps/desktop/src/main.ts`
- Create: `apps/desktop/tests/tests-panel.test.mjs`

**Interfaces:**
- Consumes: `PanelName` and `panelContentId` used by the current panel switcher.
- Produces: `PanelName` includes `"tests"`; `panelContentId("tests")` resolves to the `tests` panel.

- [x] **Step 1: Write failing tests**

Check the Testes tab has `role=tab`, controls `tests`, and the panel has `role=tabpanel` with a status containing “Nenhum teste configurado”; assert the message explains the runner is unavailable and no pass/fail counter is rendered. Check `showPanel("tests")` is wired to the tab click handler. Verify arrow navigation wraps and Home/End select boundaries.

- [x] **Step 2: Run the focused test and confirm it fails**

Run `node --experimental-strip-types --test tests/tests-panel.test.mjs` from `apps/desktop`.

- [x] **Step 3: Implement the tab/panel wiring**

Add `tests` to `PanelName`, the bottom tablist and panel iteration; retain the existing panel visibility and ARIA selection behavior.

- [x] **Step 4: Run focused and full frontend tests**

Run the focused test, then `npm test`; both must pass.

- [x] **Step 5: Verify production build and formatting**

Run `npm run build`, `cargo fmt --all -- --check`, and `git diff --check`.

- [x] **Step 6: Commit**

Commit the verified UI-only FT-06 slice.
