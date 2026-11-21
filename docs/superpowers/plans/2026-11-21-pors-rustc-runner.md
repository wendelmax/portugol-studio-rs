# Plan: execute Portugol-RS through the Rust compiler

> Work test-first; do not change the classic VM or enable `.rs` execution.

## 1. Rust compiler service

- Add tests for toolchain detection, successful compile/run stdout and exit code, syntax/semantic rejection before spawn, compiler errors, program nonzero exit, and cleanup of temporary artifacts.
- Run the focused tests RED before implementing.
- Add an RAII temp directory and fixed `rustc`/binary invocations in `portugol-service`; never call a shell or Cargo.
- Keep compiler diagnostics honest and include available stderr; cap accepted source size if needed for resource control.
- Verify focused service tests, full service tests, fmt, and Clippy.

## 2. Tauri capability and execution commands

- Add `rust_toolchain_available` and async `run_portugol_rs` commands with `spawn_blocking`.
- Verify command returns are typed/serializable, compiler absence is explicit, and generated `.pors` diagnostics precede process execution.
- Compile/check the desktop target.

## 3. IDE flow

- Query compiler availability at startup; keep `.pors` Execute disabled until the result is known, then explain the missing toolchain or enable execution.
- Route `.pors` Execute through the new command; show actual stdout/stderr/exit status in Console.
- Keep debug/step/stop, input, and `.rs` execution unavailable; discard stale results and restore controls after errors.
- Add helper tests for capability gating and language routing; run complete frontend tests and build.

## 4. Documentation and delivery

- Update `docs/PARIDADE.md`, the Portugol-RS report, and this feature report with supported syntax/runtime limits.
- Run all locally available checks, then open a PR. Correct CI failures and merge only when every required check is green; delete the merged branch/worktree.
