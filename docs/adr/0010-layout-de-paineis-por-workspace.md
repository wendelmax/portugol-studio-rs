# ADR 0010 — Persistir o layout de painéis por workspace

- **Status:** aceita
- **Data:** 2026-11-22

## Contexto

O design do Ferro Studio Next pede que as proporções de painéis sejam persistidas por workspace. A implementação atual salva as proporções das divisórias explorador, painel inferior e utilitários sob uma única chave global `ferro.pane-layout.v1`.

## Decisão

Manter o `localStorage` e adicionar uma chave namespaced por workspace, usando `WorkspaceListing.preferenceKey`, que é estável durante o ciclo de vida da pasta e não expõe seu caminho absoluto. Abas standalone mantêm a chave global; workspaces sem preferência própria herdam o layout global.

## Consequências

- Alternar abas de workspaces distintos restaura as proporções daquele projeto.
- Preferências existentes são preservadas como fallback e não exigem migração destrutiva.
- Mover um workspace cria uma identidade nova e usa o layout global até a pessoa redimensionar seus painéis.
- As funções de normalização e limites de resize permanecem a fonte única de validação.
