# Workspace File Delete Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Excluir arquivos `.por`, `.pors` e `.pex` do Explorer sem sair do workspace nem perder buffers sem escolha explícita.

**Architecture:** Reutilizar `WorkspaceSession` e validação scoped do serviço Rust. Retornar árvore e caminho apagado; frontend usa confirmação acessível, protege abas sujas e só altera árvore/abas/recents depois do sucesso backend.

**Tech Stack:** Rust, Tauri 2, TypeScript, Node test runner.

**Spec:** `docs/superpowers/specs/2026-11-21-workspace-delete-file-design.md`

## Global Constraints

- Operações de filesystem limitadas pelo ID de workspace e caminho relativo validado no Rust.
- Não excluir diretórios, seguir symlinks, aceitar traversal ou ampliar permissões do Tauri.
- Cancelar e erro preservam documento e árvore; só a escolha explícita de descarte fecha buffers.
- Salvar alterações encerra o pedido sem excluir o arquivo; exclusão requer confirmação após salvar.

## Review Focus

- `../fora.por` ou separador Windows: backend rejeita e preserva ambos os arquivos (testado na tarefa 1).
- Origem por symlink externo: rejeita sem remover alvo ou link (testado na tarefa 1).
- `.txt`, diretório ou arquivo ausente: erro seguro e estado intacto (testado na tarefa 1).
- Abas dirty: cancelar/salvar sem exclusão e descartar só remove abas após resultado backend positivo (testado na tarefa 2).
- `.pex` aberto: salvar usa fluxo Save As; falha ou cancelamento não exclui exercício nem fecha resposta (testado na tarefa 2).

---

### Task 1: Serviço Rust e comando Tauri scoped

**Files:** `apps/desktop/src-tauri/src/workspace.rs`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src/api.ts`

- [ ] Escrever testes Rust para apagar arquivos suportados, rejeitar traversal, symlink, diretório/ausente/unsupported e preservar arquivos vizinhos.
- [ ] Rodar os testes e observar as falhas esperadas antes da implementação.
- [ ] Implementar `delete_relative_file(root: &Path, relative: &Path) -> Result<PathBuf, String>` validando com `checked_file(..., true)` e removendo somente arquivo regular autorizado.
- [x] Expor `workspace_delete_file(workspace_id, relative_path) -> WorkspaceDeleteResult` com lista atualizada e caminho removido absoluto/relativo; registrar comando e tipo TS.
- [x] Rodar `cargo fmt --all -- --check`; CI executará testes nativos multiplataforma.

### Task 2: Confirmação, documentos e sincronização

**Files:** `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/src/api.ts`, `apps/desktop/src/workspace-file-actions.ts`, `apps/desktop/tests/workspace-file-actions.test.mjs`, `apps/desktop/tests/workspace-explorer.test.mjs`

- [x] Escrever testes comportamentais para cancelamento, salvar e manter arquivo, falha de save, descarte adiado até sucesso backend e sincronização das abas/recents.
- [x] Implementar ação acessível “Excluir” na árvore e diálogo com estado limpo/sujo, botões sem ambiguidade e cancelamento bloqueado durante IPC.
- [x] Dirty source/exercise: salvar percorre abas sequencialmente e não chama exclusão; descartar chama backend primeiro e fecha abas somente no sucesso.
- [x] Erro backend mantém abas, árvore, seleção e recentes; sucesso atualiza listing, remove recentes e abas associadas.
- [x] `@media (hover: none), (pointer: coarse)` mantém ação visível.
- [x] Rodar testes completos, build e `git diff --check`.

### Task 3: Relatório e entrega

**Files:** `docs/reports/FEAT-WORKSPACE-005.md`, plano e spec acima

- [x] Documentar validações reais e limitação local de WebKitGTK/JavascriptCoreGTK.
- [ ] Pedir revisão do diff, corrigir achados, abrir PR, aguardar CI verde, mesclar e excluir branch.
