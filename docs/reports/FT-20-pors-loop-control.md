# FT-20 — `pare` e `continue` em Portugol-RS

## Entrega

O subset `.pors` representa controle de laço na HIR comum, valida que `pare` e `continue` aparecem dentro de `enquanto` ou `para`, e gera `break`/`continue` em Rust compilável. `continue` dentro de `para` executa primeiro o passo composto do laço. Source maps apontam cada comando à sua span original. O editor PORS destaca e sugere os dois comandos. O Portugol clássico continua rejeitando as construções que não suportava.

## Validação executada

- TDD RED: parser falhou porque `LoopControl` ainda não existia; teste CodeMirror falhou porque `pare` e `continue` eram identificadores e ausentes do autocomplete.
- `CARGO_TARGET_DIR=.../target CARGO_BUILD_JOBS=1 cargo test -p portugol-rs-frontend -p portugol-rust -p portugol-service` — aprovado: 9 parser, 12 semântica, 33 testes do codegen, 30 testes unitários do serviço, 9 testes do runner e 10 testes de serviço.
- Codegen usa `rustc` real e confere stdout para `while`, `for` e laços aninhados, incluindo o passo em `continue` no `for`; spans dos dois controles têm source maps exatos.
- `node --experimental-strip-types apps/desktop/tests/pors-editor-loops.test.mjs` — 2/2.
- `npm --prefix apps/desktop test` — 19/19 arquivos.
- `npm --prefix apps/desktop run build` — aprovado; aviso do Vite para o bundle principal acima de 500 kB.
- `cargo test` — todos os default-members passaram, incluindo o corpus e a fixture HTTP local (`bibliotecas_internet_e_servicos_web`).
- `cargo clippy --workspace --exclude portugol-desktop --all-targets -- -D warnings` — aprovado em todos os crates sem o app desktop.
- `cargo test --workspace` — bloqueado no app Tauri porque o host não possui `javascriptcoregtk-4.1`/WebKitGTK 4.1 para `pkg-config`; o CI Linux compila e abre a janela nativa.
- `cargo fmt --all -- --check` e `git diff --check` — aprovados.

## Limites conhecidos

O runner `.pors` ainda não tem entrada interativa, depuração, interrupção manual, funções ou execução de arquivos `.rs`; esses limites permanecem explicitados na matriz de paridade.
