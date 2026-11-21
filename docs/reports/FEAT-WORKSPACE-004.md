# FEAT-WORKSPACE-004 — Rename files in the workspace Explorer

## Delivered

- Accessible rename action for `.por`, `.pors`, and `.pex` entries.
- The action remains visible on touch devices without hover.
- The original extension is preserved; invalid names, path traversal, symlinks, and destination collisions are rejected by the Rust workspace service.
- Rename is exclusive and preserves file contents. The refreshed tree and canonical file path are returned to the frontend.
- Open source tabs are kept attached to the renamed file. Dirty source tabs must be saved first; canceling or failing to save keeps the original path. Exercise tabs remain attached to `.pex` without treating their generated answer buffer as the exercise file.
- Recent files and selection follow the new path after the backend operation succeeds.

## Validation

- `npm test`: 50 tests passed, including confirmation cancellation, save failure, ordered saves, `.pex` buffers, and path updates after success.
- `npm run build`: passed; Vite reports the existing 518 kB IDE chunk warning.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- `cargo test --locked --all-targets`: blocked locally because Linux `javascriptcoregtk-4.1` and `webkit2gtk-4.1` development packages are not installed. CI must validate the Rust tests and platform builds.

## Remaining workspace actions

Deletion, moving files, renaming folders, creating subfolders, and watching external filesystem changes remain outside this feature slice.
