# FT-17 — Menu de novas linguagens na tela inicial

## Entrega

A ação principal **Novo Arquivo** continua abrindo o template Ferro e mantém o atalho Ctrl+N. Ferro (`.fer`) e Rust (`.rs`) ficam em um menu nativo e compacto no mesmo botão, reduzindo a quantidade de ações visíveis sem retirar os novos formatos. A seleção continua usando os templates e `newTab` existentes.

O menu usa `<details>/<summary>`, tem rótulos acessíveis, fecha após seleção e fecha com Escape devolvendo o foco ao acionador. As cores usam os tokens de tema existentes; o foco visível é explícito.

## Arquivos

- `apps/desktop/index.html` — marcação da ação e menu.
- `apps/desktop/src/home-new-file-menu.ts` — seleção de linguagem, fechamento e Escape.
- `apps/desktop/src/main.ts` — integração aos templates existentes.
- `apps/desktop/src/style.css` — estilo do menu e escopo dos botões grandes.
- `apps/desktop/tests/home-new-file-menu.test.mjs` e `workspace-explorer.test.mjs` — testes de interação e contrato.
- `docs/PARIDADE.md` — inventário atualizado.

## Validação

- `node --experimental-strip-types apps/desktop/tests/home-new-file-menu.test.mjs` — PASS, 4 casos: seleção FERRO/Rust, fechamento, Escape/foco e valor desconhecido.
- `node --experimental-strip-types tests/workspace-explorer.test.mjs` — PASS, 14 casos, incluindo markup principal, templates, CSS por tokens e foco visível.
- `npm --prefix apps/desktop test` — PASS, 16/16 arquivos de teste.
- `npm run build` — PASS (`tsc` + Vite); Vite emitiu aviso de bundle principal acima de 500 kB.
- `cargo fmt --all -- --check` e `git diff --check` — PASS.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo test --locked --workspace --exclude ferro-desktop` — PASS, incluindo corpus, runtime FERRO e testes existentes de internet com rede autorizada.
- `CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo clippy --locked --workspace --exclude ferro-desktop --all-targets -- -D warnings` — PASS.
- `PKG_CONFIG_PATH=/tmp/ft10-pkgconfig CARGO_TARGET_DIR=/home/wendel/Projetos/ferro-studio/target cargo check --locked --manifest-path apps/desktop/src-tauri/Cargo.toml --tests` — PASS.
- Playwright Firefox em Vite — PASS para renderização visual com fixture apenas de UI nos temas Dark e Ferro: 4 ações principais, catálogo de exemplos renderizado e menu com as duas escolhas. A inspeção também enviou Enter/Escape/Tab/Enter: abriu o disclosure, Escape fechou e devolveu foco, e as escolhas FERRO/Rust abriram suas abas. As fixtures só substituíram respostas IPC para compor a tela; não validam IPC nem inicialização Tauri nativa, e essa verificação não é um teste Playwright persistente no projeto.
- A comparação em tela inicial nativa com Ferro Studio Java continua pendente; a linha de paridade permanece parcial.
- O preflight do Superdesign ficou indisponível: o login de serviço externo foi rejeitado pela revisão automática por não haver autorização explícita para vincular conta. A implementação seguiu os contratos e assets existentes, sem contornar o bloqueio.
