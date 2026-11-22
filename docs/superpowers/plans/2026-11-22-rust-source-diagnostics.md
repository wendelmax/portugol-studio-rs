# Standalone Rust Diagnostics — Plan

1. [x] Add failing service integration tests for valid source, compiler error/UTF-16 span, warning preservation, and the 1 MiB boundary.
2. [x] Implement bounded `rustc --emit=metadata --crate-type=lib` checking and export the service API.
3. [x] Add the async Tauri command and frontend IPC types.
4. [x] Add a serialized latest-only checker and CodeMirror Rust linter, preserving stale tab/document guards and actionable tool-limit diagnostics.
5. [x] Test serialization/stale responses and run Rust, frontend, build, Clippy, and format checks.
6. [ ] Update gap/traceability docs, request independent review, open PR, and merge only when cross-platform CI is green.
