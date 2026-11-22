# FT-20 — `pare` e `continue` em Portugol-RS

## Entrega

O subset `.pors` representa controle de laço na HIR comum, valida que `pare` e `continue` aparecem dentro de `enquanto` ou `para`, e gera controle de fluxo Rust compilável. Em `para`, o corpo fica em um bloco rotulado e o passo é emitido uma única vez após esse bloco: `continue` sai do corpo e deixa o passo normal executar, enquanto `pare` sai do laço contado. Isso preserva os bindings do cabeçalho mesmo quando o corpo sombreia seus nomes. Source maps apontam cada comando e o passo à sua span original. O editor PORS destaca e sugere os dois comandos. O Portugol clássico continua rejeitando as construções que não suportava.

## Validação executada

- TDD RED: parser falhou porque `LoopControl` ainda não existia; teste CodeMirror falhou porque `pare` e `continue` eram identificadores e ausentes do autocomplete.
- `CARGO_TARGET_DIR=.../target CARGO_BUILD_JOBS=1 cargo test` — aprovado, incluindo 9 testes do parser PORS, 12 de semântica, 34 do gerador Rust, 30 do serviço, 9 do runner e regressões do Portugol clássico.
- Codegen usa `rustc` real e confere stdout para `while`, `for` e laços aninhados, incluindo o passo em `continue` no `for`; spans dos dois controles têm source maps exatos.
- `node --experimental-strip-types apps/desktop/tests/pors-editor-loops.test.mjs` — 2/2.
- `npm --prefix apps/desktop test` — 108/108 testes.
- `npm --prefix apps/desktop run build` — aprovado; aviso do Vite para o bundle principal acima de 500 kB.
- `cargo test` — todos os default-members passaram, incluindo o corpus e a fixture HTTP local (`bibliotecas_internet_e_servicos_web`).
- `cargo clippy --workspace --exclude portugol-desktop --all-targets -- -D warnings` — aprovado em todos os crates sem o app desktop.
- `cargo test --workspace` — bloqueado no app Tauri porque o host não possui `javascriptcoregtk-4.1`/WebKitGTK 4.1 para `pkg-config`; o CI Linux compila e abre a janela nativa.
- `cargo fmt --all -- --check` e `git diff --check` — aprovados.
- Revisão independente detectou sombreamento incorreto quando `continue` copiava o passo para dentro do corpo. Teste RED reproduziu a saída `02` em vez de `0123`; com a atualização emitida após o bloco rotulado, dois casos com `i`/`passo` sombreados passam em `rustc` real. Os mapas agora verificam os intervalos completos do passo e dos comandos gerados.

## Limites conhecidos

O runner `.pors` ainda não tem entrada interativa, depuração, interrupção manual, funções ou execução de arquivos `.rs`; esses limites permanecem explicitados na matriz de paridade.
