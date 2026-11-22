# FEAT — Explicação contextual por source map

- Requisito: UI-023/024, UC-13, FT-05; seleção no editor direciona uma explicação somente quando há evidência de um mapeamento exato para a tradução Rust atual.
- Implementação: `apps/desktop/src/learning-context.ts` resolve a seleção/caret sobre spans exatos da fonte e reconhece declarações `let`/`let mut` apenas no trecho Rust mapeado. O inspector exibe os trechos reais, proveniência e texto adequado a Portugol clássico ou Portugol-RS.
- Compatibilidade: a seleção manual continua disponível e tem precedência até a próxima mudança de seleção no editor. Nenhuma fonte, execução, IPC, serviço Rust ou progresso de exercício foi alterado.
- Recusa de inferência: contexto não aparece para traduções ausentes, antigas, de outra fonte, spans parciais fora do mapeamento, mapeamentos aproximados/gerados ou trechos sem declaração Rust reconhecida.

## Validação

- `node --experimental-strip-types --test tests/learning-context.test.mjs` — PASS; os casos cobrem mutabilidade, imutabilidade, limites de seleção, revisão, qualidade do mapa e sobreposição.
- `npm test` em `apps/desktop` — PASS, 23 arquivos de teste.
- `npx tsc --noEmit` — PASS.
- `npm run build` — PASS com Vite 8.3.5; permanece o aviso existente de chunk principal acima de 500 kB.
- `git diff --check` — PASS.
- E2E/visual Tauri: NOT_RUN nesta fatia; não foi possível validar a apresentação por screenshot nesta sessão.

## Limites e próximos passos

- A associação atual é deliberadamente limitada a mutabilidade e às declarações Rust cobertas pelo subset atual de source maps.
- Seleção manual e reatividade são cobertas no fluxo UI existente, sem harness de componente disponível; adicionar teste desktop/E2E dedicado quando o projeto configurar esse runner.
- Nenhum ownership/borrowing é inferido do texto-fonte.
