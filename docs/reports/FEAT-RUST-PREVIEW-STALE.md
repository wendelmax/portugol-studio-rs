# Rust preview stale state

## What changed

- Rust export artifacts are cached on their source editor tab, with the source snapshot and source map that produced them.
- Editing keeps the previous generated Rust visible and labels it as outdated until the current translation succeeds.
- Stale source mappings cannot navigate, and copy/save/detach stay disabled while translation is pending or stale. Stale Rust remains readable and keyboard-scrollable as plain text.
- Failed exports retain the prior artifact with an error and stale label. Responses for another tab or source revision are discarded.
- Editing in beginner mode also marks a cached Bridge artifact stale without starting a hidden translation.
- Switching tabs uses a shared-DOM render key, avoiding cross-file previews and avoiding full preview rebuilds on every edit.
- Saving under another language invalidates in-flight results and stale-marks the prior artifact; Bridge schedules a translation for the new dialect.

## Validation

- `npm test`: 124 tests passed.
- `npm run build`: TypeScript and Vite build passed. Vite reported the existing 500 kB chunk-size advisory.
- `cargo test --workspace`: passed, including doctests.
- `git diff --check`: passed.
- Native smoke: Tauri launched successfully from this worktree using Vite port 5174, leaving the user's existing instance on 5173 untouched. GTK emitted the known `gtk_window_set_titlebar() called on a realized window` warning. No screenshot/visual inspection was captured.
- Independent review findings around stale navigation, render caching, partial results, diagnostics and async language changes were fixed; final review is pending.

## Pending

- CI checks and review on the pull request.
