# FEAT-WORKSPACE-001 — Explorador de projeto

- Escopo / rastreabilidade: UI-001, UI-002, UI-008; UC-01, UC-03; FT-01. Primeira fatia do explorador no Portugol Studio Next, somente leitura da árvore; arquivos `.por`/`.pors` podem continuar sendo editados e salvos pelo fluxo existente, `.pex` abre como exercício.
- Arquitetura preservada: frontend TypeScript/Vite + CodeMirror 6, Tauri v2, `tauri-plugin-dialog` e serviços Rust atuais. Nenhuma dependência de runtime foi adicionada.
- Interface: ação Abrir Pasta na tela inicial, botão de explorador na toolbar, árvore recolhível, extensões visíveis, seleção de arquivo, estados de vazio/carregamento/erro, foco e navegação por teclado (setas, Home/End, Enter/Space nativo), nomes e níveis acessíveis. Atalho novo Ctrl+Shift+E.
- Serviços/IPC: `workspace_open` abre seletor nativo Rust e enumera arquivos; `workspace_read_file` lê por ID do workspace e caminho relativo; `workspace_write_file` salva apenas fontes existentes `.por`/`.pors`. IDs mantêm abas ligadas à pasta original quando outra pasta é aberta. Sem operações de criar, renomear ou excluir.
- Segurança: raiz canonicalizada após seleção nativa; apenas arquivos suportados entram na árvore; symlinks não são seguidos; leitura e escrita revalidam componentes, tipo, extensão e canonicalização contra a raiz registrada. Caminhos de traversal/fora da raiz são rejeitados. Arquivos `.pex` são somente leitura pelo explorador.
- Compatibilidade: os fluxos atuais de abrir/salvar arquivos individuais e recents permanecem. Arquivos do explorador usam caminho absoluto já selecionado para preservar a resolução de imports e a execução relativa; IPC de leitura/escrita continua restrito ao ID + caminho relativo.
- Arquivos alterados: `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/src/api.ts`, `apps/desktop/src/workspace-explorer.ts`, `apps/desktop/tests/workspace-explorer.test.mjs`, `apps/desktop/src-tauri/src/lib.rs`, novo `apps/desktop/src-tauri/src/workspace.rs`, `docs/PARIDADE.md`, esta especificação/plano e este relatório.

## Validação executada

- `cd apps/desktop && npm test` — **PASS**, 7 arquivos de teste / 37 testes.
- `cd apps/desktop && npm run build` — **PASS**, TypeScript e Vite. O Vite mantém o aviso existente sobre o bundle principal acima de 500 kB.
- `cargo test -p portugol-service --offline` — **PASS**, 17 testes unitários + 10 testes de integração do serviço.
- `cargo clippy -p portugol-service --offline -- -D warnings` — **PASS**.
- `cargo fmt --all -- --check` — **PASS**.
- `git diff --check` — **PASS** antes das últimas alterações de documentação; repetir antes do commit.
- `cargo test -p portugol-desktop --offline workspace::tests --lib` — **BLOCKED neste Linux local**, antes de compilar o crate: faltam `webkit2gtk-4.1.pc` e `javascriptcoregtk-4.1.pc`. Os testes unitários do novo módulo cobrem enumeração recursiva/extensões, leitura relativa, traversal, arquivos não suportados, symlink externo e escrita limitada a fontes; precisam ser compilados/executados pelo CI desktop.
- E2E e inspeção visual com WebKitGTK — **NOT RUN** localmente pela mesma dependência de sistema ausente. Testes de markup/domínio não substituem o smoke da janela nativa.

## Limitações e próximos passos

- A seleção atual de workspace não persiste entre reinicializações; o estado permanece na sessão Tauri.
- A árvore não permite criar, renomear, excluir, filtrar ou observar mudanças externas. Essas ações precisam de uma próxima fatia com tratamento de buffers dirty e segurança de caminhos.
- Pasta vazia ou sem arquivos suportados mostra estado explicativo. Diretórios vazios não são incluídos na árvore.
- Próximo passo: rodar testes `portugol-desktop` e smoke Tauri em CI, corrigir falhas, abrir PR e mesclar somente quando verde; depois continuar as lacunas da interface Next e da issue #2.
- Rollback: remover comandos/módulo de workspace e controles/árvore; o fluxo pré-existente de arquivos individuais não depende deles.
