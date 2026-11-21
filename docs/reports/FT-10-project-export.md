# Relatório de implementação — FEAT-WORKSPACE-EXPORT-001

- Branch / commit: `feat/workspace-project-export` / pendente de commit.
- Escopo e IDs (UI/UC/FT/AC): UI-006, UI-008, UI-009, UC-14, FT-10, AC-16.
- Estado anterior (evidência path/linha): `apps/desktop/index.html` tinha ações Abrir e Novo no Explorador; `apps/desktop/src/api.ts` expunha abrir/ler/gravar/criar/renomear/excluir, sem exportação.
- Arquivos alterados: `apps/desktop/index.html`, `apps/desktop/src/api.ts`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/src/workspace-explorer.ts`, `apps/desktop/src-tauri/src/lib.rs`, `apps/desktop/src-tauri/src/workspace.rs`, `apps/desktop/tests/workspace-explorer.test.mjs`, `apps/desktop/tests/workspace-export.test.mjs`, `docs/PARIDADE.md`, `docs/reports/FT-10-project-export.md`, `docs/superpowers/plans/2026-11-21-workspace-project-export.md`, `docs/superpowers/specs/2026-11-21-workspace-project-export-design.md`.
- Alterações no contrato IPC/Rust/Tauri: novo comando `workspace_export`, com `workspaceId`, caminhos relativos selecionados e resultado anulável em cancelamento; seletor de pasta é apresentado pelo backend.
- Permissões/capabilities alteradas? Justificativa: nenhuma; reutiliza o plugin de diálogo existente e confina leitura ao workspace/cópia ao destino escolhido via `cap-std`.
- Compatibilidade preservada (fixtures): nenhum buffer aberto, arquivo de origem ou progresso é alterado; aceita `.por`, `.pors`, `.rs`, `.pex` já listados pelo Explorador.
- Estados UI implementados (loading/empty/error/success/cancelled): lista vazia; nenhuma seleção; seletor/cópia em andamento; cancelamento sem cópia; erro anunciado; sucesso após cópia concluída.
- Testes executados (comando, status PASS/FAIL/BLOCKED/NOT_RUN, log):
  - `node --experimental-strip-types --test tests/*.test.mjs` — PASS, 13 arquivos de teste, 0 falhas.
  - `npm run build` — PASS, TypeScript e Vite concluídos; Vite emitiu aviso preexistente de chunk IDE acima de 500 kB.
  - `cargo test -p portugol-desktop --lib workspace::tests::` — BLOCKED antes de compilar o crate: faltam as bibliotecas de sistema `webkit2gtk-4.1` e `javascriptcoregtk-4.1` no Linux; a CI precisa validar os testes Rust. Foram incluídos casos de sucesso, rejeição de entradas inválidas/symlink, colisão e rollback após falha parcial.
  - `cargo fmt --all -- --check` — PASS.
  - `git diff --check` — PASS.
- Screenshot/registro visual em viewport configurado: NOT_RUN.
- Problemas e limitações conhecidas: exporta os arquivos selecionados para uma subpasta com o nome do projeto, não gera arquivo `.zip`; colisão de pasta existente retorna erro e não sobrescreve.
- Rollback: remover o comando `workspace_export`, a seleção/ação do Explorador e os estilos/testes; nenhuma migração ou alteração da origem é necessária.
- Próxima fatia vertical: implementar runner de testes real quando forem definidos formato e descoberta, preservando o estado vazio até então.
