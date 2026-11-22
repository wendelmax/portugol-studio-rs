# Standalone Rust Diagnostics

## Goal

Show `rustc` errors and warnings for `.rs` buffers in CodeMirror without executing the student's program. Keep the check scoped to one source buffer and use the existing Tauri/service process supervision.

## Design

- Add `check_rust_source(source)` in `portugol-service` using a fixed `rustc` invocation with Rust 2021, JSON diagnostics, `--crate-type=lib`, and metadata emission into a private temporary directory.
- Never invoke Cargo, a shell, a user-selected path, or the resulting binary.
- Enforce the existing 1 MiB source limit, a five-second compile limit, and the existing 1 MiB combined output cap.
- Reuse `RustSourceCompileDiagnostic` and its UTF-8-byte to UTF-16 position conversion.
- Expose an async Tauri command and a typed frontend wrapper.
- Run the Rust lint only when rustc is available. Serialize checks to one process at a time, keep only the newest pending source, and discard results for replaced tabs/documents.
- Preserve classic Portugol/Portugol-RS lint behavior and runtime diagnostics.

## Acceptance

- Valid Rust returns compiler warnings and no errors; invalid Rust returns positioned diagnostics without running code.
- Diagnostics use UTF-16 offsets compatible with CodeMirror, including a non-BMP prefix.
- Oversized input and process limits produce a visible tool warning and never lock the editor.
- Overlapping edits do not spawn parallel rustc processes or apply stale results.
- `.por` and `.pors` lint paths remain unchanged.

## Limits

- This checks a single `.rs` buffer as a Rust library. Cargo projects, external dependencies, and cross-file module resolution are out of scope.
- No Cargo build scripts or student code are executed.
