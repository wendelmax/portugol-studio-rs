# Plan: semantic navigation and rename for Ferro

> Implementation work is test-first. Keep the classic Ferro behavior unchanged.

## 1. Expose semantic binding occurrences

- Add service tests for `.fer` declaration/use resolution, nested shadowing, UTF-16 offsets after Unicode, and ignoring text inside strings/comments.
- Run the focused Rust test and confirm the expected missing-function failure.
- Implement an internal `.fer` binding map from HIR `BindingId` to declaration and occurrence spans. Cover declarations, assignments, and expressions recursively through `if` blocks.
- Convert spans through `LineIndex`; reject errors and positions outside resolved identifier spans.
- Validate focused tests and relevant `ferro-service` suite.

## 2. Add `.fer` editor operations

- Expose `rename_target_ferro_rs`, `resolve_symbol_ferro_rs`, and `rename_symbol_ferro_rs` through the service, Tauri commands, and typed frontend API.
- Test that a rename updates only the selected semantic binding, rejects invalid identifiers and capture, and keeps `inicio` fixed.
- Reanalyze renamed source and compare the usage-to-declaration map before returning edits.
- Add frontend tests proving command availability is gated by document language and semantic commands remain disabled for Rust.

## 3. Wire existing UI paths

- Enable the current definition and rename actions for `.fer`, including context menu, outline click, and keyboard shortcuts.
- Route handlers to the matching IPC methods; retain stale request/source guards and error statuses.
- Keep Rust real-source commands disabled with their current explanations.
- Verify with UI tests and `npm run build`.

## 4. Docs and verification

- Update Ferro status in `docs/PARIDADE.md` and the integration gap report/traceability matrix.
- Run `cargo fmt --all -- --check`, focused tests, all available Rust/service tests, frontend tests, Clippy and build. Record actual outcomes and environmental limitations.
- Review diff for compatibility, source revision guards, accessibility and no textual rename shortcuts. Open a PR; merge only if all CI checks are green.
