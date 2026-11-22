# Vite Dev License Access Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Tauri development WebView load the repository license without broadening Vite filesystem access beyond the project.

**Architecture:** Keep `LICENSE` as the single source of truth and add only the repository root to the Vite development server's explicit filesystem allowlist. The production bundler and Tauri IPC remain unchanged.

**Tech Stack:** Vite 8, Node test runner, TypeScript configuration.

**Spec:** `docs/superpowers/specs/2026-11-22-vite-dev-license-access-design.md`

## Global Constraints

- Do not disable Vite filesystem access checks.
- Do not allow directories outside the repository root.
- Do not change production build, Tauri permissions, or the license content.

## Review Focus

- From `apps/desktop/vite.config.ts`, resolve the repository root correctly on Linux, Windows, and macOS.
- Ensure the allowlist does not include a user's home directory or a broad parent directory.
- Keep production builds and path resolution independent of the process working directory.

---

### Task 1: Allow the root license in the Tauri development server

**Files:**
- Modify: `apps/desktop/vite.config.ts`
- Create: `apps/desktop/tests/vite-config.test.mjs`
- Create: `docs/reports/FT-21-vite-dev-license-access.md`

**Interfaces:**
- The contract test reads the Vite config source and verifies that its single explicit allowlist entry is the repository root derived from `import.meta.url`; the native smoke test checks the resolved Vite behavior.
- `server.fs.allow` must include the normalized repository root derived from `import.meta.url`; it must not include the parent of that root.

- [x] **Step 1: Write the failing test** `Vite dev filesystem allowlist includes only the repository root needed for LICENSE`
  - Check the config source uses a root derived from `import.meta.url` and configures only `[repositoryRoot]`.
- [x] **Step 2: Run the test and confirm RED**
  - Run `node --experimental-strip-types --test tests/vite-config.test.mjs` from `apps/desktop`.
  - Expected: fail because `server.fs.allow` is absent.
- [x] **Step 3: Implement the narrow allowlist**
  - Derive the repo root from `import.meta.url` in `vite.config.ts` and set `server.fs.allow` to that one normalized path.
- [x] **Step 4: Verify GREEN and production build**
  - Run the focused Node test, `npm test`, and `npm run build` from `apps/desktop`.
  - Run `npm run tauri dev` and confirm Vite no longer rejects `/LICENSE?raw` and the Tauri WebView starts.
- [x] **Step 5: Record evidence and commit**
  - Write FT-21 with actual commands/results, then commit the config, test, report, spec, and plan.

### Ruling

Ruling: use a static source-contract test plus a real Tauri/Vite smoke test instead of importing the Vite config inside Node's test worker — direct config import behaved inconsistently under the test runner, while the actual desktop launch exercises Vite's config loader and filesystem serving path. Cost if wrong: a future refactor could preserve the regex shape but change runtime semantics; the Tauri smoke test and repository-root endpoint check reduce that risk.
