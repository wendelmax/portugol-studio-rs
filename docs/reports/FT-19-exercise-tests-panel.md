# FT-19 — casos de exercício no painel Testes

## Entrega

O painel **Testes** agora executa os casos de um exercício `.pex` associado à aba atual usando o comando Tauri `check_exercise` e o validador de progresso existente. Mostra carregamento, quantidade de casos aprovados, falhas e erros do serviço. A conclusão continua condicionada à aprovação de todos os casos na revisão exata do código.

O resultado é descartado quando a aba é fechada, a revisão muda ou uma execução mais nova substitui a solicitação. O painel não abre o diálogo modal de correção. Para arquivos sem exercício, mantém o estado vazio e não inventa runner ou resultados.

## Validação executada

- `node --experimental-strip-types apps/desktop/tests/tests-panel.test.mjs` — 2 testes aprovados.
- `node --experimental-strip-types apps/desktop/tests/exercise-test-panel.test.mjs` — 5 testes aprovados para estados, revisão exata e descarte de respostas obsoletas.
- `npm --prefix apps/desktop test` — 18 arquivos de teste aprovados.
- `npm --prefix apps/desktop run build` — TypeScript e bundle de produção aprovados. Vite reportou o aviso já conhecido de chunk JS principal acima de 500 kB.
- `git diff --check` — sem erros de whitespace.

## Limites conhecidos

- O painel executa casos de exercícios, não uma suíte configurável para qualquer projeto.
- O laboratório de ownership e os programas Rust não ganham um runner Rust com esta entrega.
