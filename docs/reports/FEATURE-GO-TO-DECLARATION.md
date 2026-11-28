# Relatório de implementação — FEAT-GO-TO-DECLARATION-001

- Branch / commit: `feat/go-to-declaration` / `b2c855972d75b7d49d62f5360fbfb650a5721013`.
- Rastreabilidade local: UI-036, UC-21, AC-14, FT-08; relacionada a UI-011/UC-04.
- Estado anterior: a árvore estrutural navegava para offsets de declaração; referências no editor não tinham navegação. O serviço já usava `symbol_bindings` para renomeação com escopos, mas não expunha a declaração associada.
- Implementação: `ferro-service::resolve_symbol` localiza a declaração via os mesmos bindings de `rename_symbol`, entrega início/fim e linha/coluna em unidades UTF-16; comando Tauri e wrapper TypeScript conectam ao CodeMirror.
- Interface: F12 com o editor focado navega à declaração; F12 fora do editor continua abrindo Sobre. Ctrl/Cmd+clique e ação no menu do editor também navegam. A declaração fica selecionada e centralizada. A navegação não altera texto nem histórico.
- Concorrência: a resposta só é aplicada se aba e `EditorState` ainda forem os mesmos e a requisição for a mais recente; erros não exibem mensagens obsoletas.
- Segurança/compatibilidade: nenhum shell, processo, capability ou permissão nova. Não altera runtime, VM, semântica, arquivo ou progresso de aprendizagem.
- TDD: os testes de serviço foram executados primeiro e falharam pela ausência de `resolve_symbol`. Durante a implementação, o teste do símbolo `inicio` revelou que a restrição específica de rename também bloqueava navegação; a validação de renomeabilidade foi separada da resolução e os testes passaram.
- Validação executada:
  - `CARGO_TARGET_DIR=/tmp/ferro-target cargo test --offline --locked -p ferro-service` — **PASS**, 14 testes unitários, 10 testes de integração e doctests.
  - `CARGO_TARGET_DIR=/tmp/ferro-target cargo clippy --offline --locked -p ferro-service --all-targets -- -D warnings` — **PASS**.
  - `cargo fmt --all --check` — **PASS**.
  - `cd apps/desktop && npm test` — **PASS**, 5 suítes sem falhas.
  - `cd apps/desktop && npm run build -- --outDir /tmp/ferro-definition-dist --emptyOutDir` — **PASS**, TypeScript e Vite; Vite mantém o aviso existente de chunk acima de 500 kB.
  - `git diff --check` — **PASS**.
  - `CARGO_TARGET_DIR=/tmp/ferro-target cargo check --offline --locked -p ferro-desktop` — **BLOCKED**, ambiente não possui `webkit2gtk-4.1` / `javascriptcoregtk-4.1` de desenvolvimento. CI remoto ainda deve validar os comandos Tauri em Linux, Windows e macOS.
- Limites: E2E de mouse/teclado e screenshot do WebView **NOT_RUN** localmente; não há Playwright/Chromium nem bibliotecas GTK/WebKit nesta máquina. Os testes semânticos cobrem usos locais, função auxiliar, a função `inicio`, shadowing, offsets UTF-16 após emoji e posição sem símbolo.
- Rollback: remover `resolve_symbol` do serviço/comando/API, action de editor e atalho contextual; renomeação e árvore estrutural permanecem inalteradas.
