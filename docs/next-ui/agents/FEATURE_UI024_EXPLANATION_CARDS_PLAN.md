# Plano: cartões de explicação UI-023/024

## Arquivos e passos

1. `apps/desktop/tests/learning-content.test.mjs`: testes primeiro para catálogo beginner/bridge, campos seguros, ownership e proveniência manual.
2. `apps/desktop/src/learning-content.ts`: tipos estáveis e catálogo editorial local; funções para listar conceitos por modo e resolver ID.
3. `apps/desktop/index.html`: adicionar tab `Aprender` e painel sem alterar as tabs atuais de saída.
4. `apps/desktop/src/main.ts`: generalizar `showPanel` para aceitar `learning`, renderizar com `textContent`, ligar modo já existente ao catálogo e manter seleção válida.
5. `apps/desktop/src/style.css`: layout acessível usando tokens atuais, com scroll para conteúdo.
6. `apps/desktop/package.json`: o script atual já encontra `tests/*.test.mjs`; conferir sem dependência nova.
7. `docs/.../TRACEABILITY_MATRIX.yaml`: registrar FEAT-LEARN-002, UI-023/024, UC-13 e testes executáveis.
8. `docs/.../FEATURE_REPORT_UI024.md`: comandos/resultados após validação real.

## Testes e revisão de risco

- Node test runner falha antes do catálogo; após implementação valida lista por modo e conteúdo explicativo.
- TypeScript/Vite garante ligação de interface e tipos.
- O exemplo Rust é editorial, não invoca `export_rust` nem `rustc`; verificar rótulos para não ser confundido com saída desta aba.
- Renderização usa `textContent`, não `innerHTML`, para evitar interpretação de conteúdo como HTML.
- Clicar na aba/cartões não modifica arquivo nem cria progresso.
- `cargo fmt`, `cargo test --workspace --offline --locked`, Clippy do núcleo/app; sem runner de UI, reportar E2E como NOT_RUN.

## Rollback

Reverter somente os arquivos listados nesta fatia; UI-028 e o preview de exportação existentes permanecem intactos.
