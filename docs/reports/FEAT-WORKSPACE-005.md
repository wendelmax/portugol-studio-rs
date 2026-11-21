# Relatório de implementação — FEAT-WORKSPACE-005

- Branch / commit: `feat/workspace-file-delete` / `6a2d6bb` (PR pendente).
- Escopo e IDs (UI/UC/FT/AC): UI-009 `ExplorerFileActions`, UC-03, AC-14; exclusão scoped de `.por`, `.pors` e `.pex`.
- Estado anterior (evidência path/linha): a árvore tinha somente criação e renomeação (`apps/desktop/src/main.ts`, renderizador `renderWorkspaceExplorer`); serviço Rust oferecia `checked_file` para leitura/gravação, sem exclusão (`apps/desktop/src-tauri/src/workspace.rs`).
- Arquivos alterados: `apps/desktop/index.html`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src-tauri/src/workspace.rs`, `apps/desktop/src/api.ts`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/src/workspace-file-actions.ts`, `apps/desktop/tests/workspace-explorer.test.mjs`, `apps/desktop/tests/workspace-file-actions.test.mjs`, spec e plano da feature.
- Alterações no contrato IPC/Rust/Tauri: comando `workspace_delete_file(workspaceId, relativePath)` retorna `WorkspaceDeleteResult` com listing atualizado e caminho removido; `checked_file` valida workspace, caminho, tipo e extensão. Nenhuma permissão/capability nova.
- Permissões/capabilities alteradas? Justificativa: não.
- Compatibilidade preservada (fixtures): `.por`, `.pors` e `.pex`; exclusão limitada a arquivo regular dentro do workspace, sem seguir symlinks.
- Estados UI implementados (loading/empty/error/success/cancelled): confirmação, dirty/save, save-error, erro de backend, cancelamento, sucesso. “Salvar alterações e manter arquivo” salva e encerra a solicitação; excluir depois requer confirmação nova. “Excluir e descartar” só fecha abas após sucesso do backend.
- Testes executados (comando, status PASS/FAIL/BLOCKED/NOT_RUN, log): `npm ci --offline` PASS; `npm test` PASS (57/57); `npm run build` PASS (aviso Vite para chunk existente de 522 kB); `cargo fmt --all -- --check` PASS; `git diff --check` PASS; `cargo test -p portugol-desktop --lib --features tauri/custom-protocol` BLOCKED localmente, faltam `javascriptcoregtk-4.1` e `webkit2gtk-4.1`.
- Screenshot/registro visual em viewport configurado: NOT_RUN; a UI mudou, mas este worktree não possui runner de screenshot/E2E desktop exercitado.
- Problemas e limitações conhecidas: validação Rust depende de CI multiplataforma; exclusão não usa lixeira e não é restaurável.
- Rollback: reverter o PR remove o comando, o diálogo e a ação; sem migração de dados.
- Próxima fatia vertical: fechar outras lacunas verificadas de paridade no issue #2, mantendo os contratos clássicos e Portugol-RS.
