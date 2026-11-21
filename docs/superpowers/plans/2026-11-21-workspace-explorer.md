# Plano: explorador de workspace somente leitura

Design: `docs/superpowers/specs/2026-11-21-workspace-explorer.md`

1. Inventariar fluxo real de abas/arquivo e comandos Tauri na base atual; identificar módulos exatos e formato de erro existente.
2. Implementar enumeração/validação Rust da raiz e leitura de caminho relativo, com testes unitários para extensão, traversal, symlink e arquivo removido.
3. Expor comandos Tauri com permissões mínimas e testes de contrato disponíveis.
4. Implementar serviço frontend de workspace e árvore acessível com estados loading/empty/error.
5. Integrar abrir arquivo à gestão existente de abas e dirty state; adicionar testes Node de interação/estado.
6. Rodar testes Rust/frontend, build, lint e CI de desktop aplicáveis; corrigir falhas até verde.
7. Atualizar rastreabilidade e relatório de feature; abrir PR, aguardar checks verdes, mesclar e remover branch/worktree de feature conforme autorização existente.
