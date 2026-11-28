# Plano — Gutter de mapeamento Ferro ↔ Rust

> Plano incremental da ponte Ferro, baseado em source maps já existentes. Executar cada fatia com teste primeiro.

**Meta:** navegar dos trechos da origem ao Rust correspondente usando apenas source maps atuais, mantendo a navegação de retorno.

**Restrições:** sem alteração Rust/Tauri/IPC, sem heurísticas, sem dependências, sem editar a origem. UI somente no modo bridge; considerar abas, documento e geração para descartar estado obsoleto.

## Tarefa 1 — Derivar links de linha com evidência

Arquivos: `apps/desktop/src/source-map-ui.ts`, `apps/desktop/tests/source-map.test.mjs`.

- [x] Testar pares extraídos de `SourceMapInfo` com origem e destino válidos.
- [x] Ver o teste falhar antes da função.
- [x] Deduplicar pares linha origem/Rust, priorizar `exact` e ordenar deterministicamente.
- [x] Excluir source maps sem origem, offsets inválidos e `generated_only`.
- [x] Executar teste focal de source maps.

## Tarefa 2 — Renderizar gutter bidirecional e acessível

Arquivos: `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`.

- [x] Escrever contratos UI que falhem sem `nav`, nomes de acessibilidade e foco ao Rust.
- [x] Adicionar gutter visível apenas junto da prévia no modo bridge.
- [x] Ligar a linha atual à lista real de links e atualizar quando muda a seleção ou a geração.
- [x] Um link foca e rola ao segmento Rust ligado; manter o clique no trecho Rust voltando à origem.
- [x] Mostrar mensagem acessível sem mapeamento e indicar qualidade aproximada.
- [x] Testar troca de abas/documentos, geração nova, fechamento do painel e modos iniciante/Rust via contratos e guardas existentes.
- [x] Executar `npm test` e TypeScript; executar build frontend no CI se pacote local ausente.

## Tarefa 3 — Registrar e integrar

Arquivos: `docs/PARIDADE.md`, `docs/reports/FT-15-ferro-mapping-gutter.md`.

- [x] Documentar comportamento, contratos preservados e limitações.
- [x] Rodar `cargo fmt --all -- --check`, testes Rust workspace suportados, check Tauri, Clippy sem desktop e TypeScript; `git diff --check` segue como gate antes do commit.
- [x] Revisão independente; filtrar `generated_only` também nos segmentos de preview, comprovado por regressão RED→GREEN (classificado Minor pela baixa chance de ocorrer com o DTO atual).
- [ ] Abrir PR, aguardar CI verde, squash-merge e apagar branch/worktree; sincronizar main.
