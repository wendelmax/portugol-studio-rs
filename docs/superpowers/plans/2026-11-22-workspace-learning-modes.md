# Modos de aprendizagem por workspace — Implementation Plan

> **For agentic workers:** implement in the current Tauri architecture with TDD and preserve global preference fallback.

## Interfaces

- `WorkspaceListing` adds serialized `preferenceKey: string`, computed by `workspace_preference_key(root: &Path) -> String` from canonical path bytes via SHA-256.
- `Tab` adds optional `workspacePreferenceKey`; `newTab` accepts the key for workspace-backed source tabs.
- `loadLearningMode(storage?, workspaceKey?)` and `saveLearningMode(mode, storage?, workspaceKey?)` use the per-workspace key when present, otherwise the existing global key. A missing workspace-specific value falls back to the existing global key.

## Steps

- [x] **1. RED: key storage tests** in `apps/desktop/tests/learning-mode.test.mjs`: save independent modes for two workspace keys, verify each restores separately, global fallback remains, and standalone mode still writes the legacy key.
- [x] **2. GREEN: implement storage keys** in `apps/desktop/src/learning-mode.ts`; use a namespaced key derived from the backend preference key and preserve fallback behavior.
- [x] **3. RED/GREEN: stable workspace preference identity** in `apps/desktop/src-tauri/src/workspace.rs`: test identical canonical roots produce the same SHA-256 key and different roots differ; add direct `sha2` dependency only if not already direct, and attach key in every `WorkspaceListing` constructor.
- [x] **4. RED/GREEN: connect active tabs** in `api.ts` and `main.ts`: carry key on workspace source and exercise tabs; restore preference during `activate`; save preference for active workspace; preserve global behavior without workspace.
- [x] **5. Update docs**: ADR 0009, design/plan, repository discovery.
- [ ] **6. Verify**: `cargo fmt --all --check` passed; `npm test` passed (31 top-level test files); frontend build passed. Tauri unit tests could not finish because `/tmp` exceeded its disk quota; require CI verification. PR review/CI, merge green and remove remote branch pending.

## Review Focus

- Switching the active tab in another workspace loads that workspace's mode before rendering preview/inspector controls.
- Open/create/rename/delete/restore responses preserve `preferenceKey`.
- First use without a workspace-specific value falls back to the legacy global setting; changing one workspace never overwrites another.
- Toggling learning mode never writes to CodeMirror source or changes runtime.
