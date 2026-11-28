# Relatório de implementação — Contexto de aprendizagem mapeado

- Branch/base: `feat/contextual-learning-mappings` / `ca509ed`.
- Objetivo: ampliar a associação pedagógica da aba Aprender para decisões e repetições selecionadas no editor.
- Implementação: `learning-context.ts` classifica trechos Rust gerados como mutabilidade (`let`), decisão (`if`) e repetição (`while` ou label interno do `para`) apenas em mapas exatos e dentro da revisão atual. A UI apresenta o conceito correspondente, o trecho original e Rust mapeado. Contexto aproximado, gerado-only, obsoleto ou sem padrão reconhecido não produz alegação contextual.
- Compatibilidade: catálogo editorial, seleção manual, modos, compilador e exportador permanecem iguais. Progresso de exercícios continua local e vinculado a avaliação real e revisão SHA-256.

## Evidências

- `node --experimental-strip-types --test apps/desktop/tests/learning-context.test.mjs` — PASS, 9 testes.
- `npm test --prefix apps/desktop` — PASS, 147 testes.
- `npm run build --prefix apps/desktop` — PASS; Vite mantém aviso de bundle principal acima de 500 kB.
- `cargo test -p ferro-rust --lib --offline --locked -- --test-threads=1` — PASS, 34 testes; inclui source maps exatos para `if`, `enquanto` e `para`.
- `git diff --check` — PASS.
- E2E visual da janela — NOT_RUN; smoke gráfico nativo não navega pela aba Aprender nem seleciona código.

## Limites

- O classificador falha fechado e reconhece somente os padrões de Rust do gerador atual: `let`, `if`, `while` e o label interno usado pelo `para` Ferro.
- O vínculo depende de o preview estar atualizado e o source map declarar qualidade `exact`.
- Não há currículo versionado, associação automática de exercícios a conceitos ou progresso por abertura de conceito.
