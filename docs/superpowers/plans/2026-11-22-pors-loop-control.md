# PORS Loop Control Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Adicionar `pare;` e `continue;` ao subset Portugol-RS com semântica de laço verificada e geração Rust.

**Architecture:** Acrescentar um comando folha à HIR compartilhada. O parser PORS valida que o comando está dentro de `enquanto`/`para`; o gerador emite Rust. Preservar o comportamento do Portugol clássico.

**Tech Stack:** Rust, `portugol-hir`, parser PORS existente, `rustc` para equivalência observável.

**Spec:** `docs/superpowers/specs/2026-11-22-pors-loop-control-design.md`

## Global Constraints

- Não alterar a sintaxe ou execução clássica.
- Nenhum resultado artificial; validar Rust gerado com compilador real.
- Manter diagnósticos com spans UTF-8 em bytes.
- Nenhuma dependência nova ou alteração IPC.

## Review Focus

- `pare`/`continue` em bloco condicional dentro de laço devem atingir o laço mais interno.
- Uso fora de laço, incluindo `se` no topo do programa, deve produzir diagnóstico localizado.
- `continue` em `para` deve ainda executar a atualização do controle.
- Nome sombreado e source maps não devem ser afetados.
- O parser e o exportador clássicos devem manter sua aceitação atual.

---

### Task 1: Parse typed loop-control statements

**Files:** `crates/portugol-hir/src/lib.rs`, `crates/portugol-rs-frontend/src/parser.rs`, `crates/portugol-rs-frontend/tests/parser.rs`

- [x] Add parser tests for `pare` and `continue` inside while/for, nested blocks, and outside-loop diagnostic span.
- [x] Run the focused parser tests and confirm the new cases fail.
- [x] Add HIR variants and loop-depth validation to the PORS parser.
- [x] Run `cargo test -p portugol-rs-frontend`.

### Task 2: Emit and verify Rust control flow

**Files:** `crates/portugol-rust/src/codegen.rs`, `crates/portugol-rust/src/hir.rs`, `crates/portugol-service/src/rename.rs`, `crates/portugol-rust/src/lib.rs`

- [x] Add real-rustc execution tests for break/continue in while and for loops, including continue updating the for control.
- [x] Implement Rust emission and leaf traversal; keep classic lowering rejecting unsupported constructs.
- [x] Run `cargo test -p portugol-rust -p portugol-service` and verify source spans/mappings.

### Task 3: Record and verify the feature

**Files:** `apps/desktop/src/portugol.ts`, `apps/desktop/tests/pors-editor-loops.test.mjs`, `docs/PARIDADE.md`, `docs/reports/FT-20-pors-loop-control.md`

- [x] Add `pare` and `continue` to PORS completion and highlighting only after compiler support is implemented; test through real CodeMirror support.
- [x] Update parity and write FT-20 with executed commands and platform limits.
- [x] Run core workspace tests, clippy (excluding desktop), frontend tests/build, and `git diff --check`; `cargo test --workspace` was also attempted and is blocked by missing WebKitGTK 4.1 development packages on this host.
- [ ] Commit, request independent review, open PR, fix CI until green, squash-merge, and delete branch/worktree.
