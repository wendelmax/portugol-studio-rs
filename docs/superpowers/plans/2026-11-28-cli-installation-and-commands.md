# CLI Installation and Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Install a standalone `ferro` executable and provide the explicit Portuguese CLI commands requested.

**Architecture:** Keep the existing Rust CLI crate and expose it through native packages/installers. Add dispatch aliases and tests at the CLI boundary; package scripts and release workflow build the binary for each target and place it on PATH using the platform installer mechanisms.

**Tech Stack:** Rust/Cargo, Tauri v2 bundle configuration, NSIS, WiX, Arch PKGBUILD, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-11-28-cli-user-experience-design.md`

## Global Constraints

- The installed command is `ferro` and must not require Cargo at runtime.
- `ferro executar` and existing CLI features remain supported.
- `ferro arquivo.fer` runs that file in the Ferro VM.
- Validation defaults to the current directory when no path is supplied.
- `ferro sintaxe` must describe only syntax accepted by the Ferro frontend.

## Review Focus

- File path beginning with a dash: command dispatch must not silently treat it as a subcommand.
- Empty current directory: validation must report zero supported files with a clear status.
- Unknown `novo --...` option: fail with usage instead of creating a filename.
- Windows per-user PATH: changes are scoped to the installing user and removed cleanly.
- Portable macOS/AppImage: explain the PATH registration action without claiming automatic installation.

---

### Task 1: CLI command dispatch and help

**Files:**
- Modify: `crates/ferro-cli/src/main.rs`
- Modify: `crates/ferro-cli/tests/cli.rs`

- [ ] Add tests for `valida <file>`, `valida` in current directory, `ajuda`, `sintaxe`, `ferro <file.fer>` execution, and `novo --jogo` behavior.
- [ ] Run `cargo test -p ferro-cli --test cli` and confirm new tests fail for missing behavior.
- [ ] Implement `valida`/`validar`, `ajuda`, `sintaxe`, direct file execution, current-directory validation, and precise unsupported project-template errors.
- [ ] Run `cargo test -p ferro-cli --test cli` and `cargo fmt --all --check`.

### Task 2: Package the CLI executable

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `packaging/arch/PKGBUILD`
- Modify: `packaging/arch/build-package.sh`
- Create: Windows NSIS hook / WiX fragment and portable installer helpers as needed.

- [ ] Build `ferro` for each release target and stage it for the platform bundler.
- [ ] Install `ferro` into standard PATH for Linux packages and Arch; add per-user PATH integration for Windows NSIS/MSI.
- [ ] Provide an explicit installer helper for portable macOS and AppImage distributions, with idempotent PATH setup.
- [ ] Add package validation that checks the installed `ferro` executable and location.

### Task 3: Document use and installation

**Files:**
- Modify: `docs/DISTRIBUICAO.md`
- Modify: `README.md`

- [ ] Document install paths, PATH behavior, portable installation, and the command examples in the spec.
- [ ] Run `git diff --check` and inspect every release asset path against workflow output.
