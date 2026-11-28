# FT-11 — Navegação e renomeação semânticas em Ferro

- **Estado:** implementado na branch `feat/ferro-semantic-navigation`; PR aguardando validação.
- **Lacuna:** arquivos `.fer` tinham análise semântica e símbolos no inspetor, mas as ações Ir à declaração e Renomear símbolo ficavam desabilitadas.
- **Implementação:** serviços Rust derivam declarações e usos da HIR tipada, agrupam ocorrências por `BindingId`, convertem spans UTF-8 para offsets UTF-16 e validam renomes reanalisando o source para evitar captura ou mudança de referência.
- **Integração:** três comandos Tauri e wrappers TypeScript; os caminhos existentes do menu, atalhos e árvore agora funcionam com `.fer`. Execução/depuração de `.fer` e navegação/rename de `.rs` continuam desabilitadas.
- **Compatibilidade:** runtime clássico, parser, comandos e editor Ferro não foram alterados; sem dependências ou permissões novas.
- **Segurança semântica:** strings e comentários não entram no mapa de usos; `inicio` permanece o ponto de entrada fixo; bindings sombreados mantêm identidades distintas; source revisado durante uma chamada assíncrona continua protegido pelos guards existentes do editor.
- **Testes executados:**
  - TDD: testes novos primeiro falharam porque os serviços `.fer` não existiam; após implementação, testes focados passaram.
  - `cargo fmt --all -- --check` — PASS.
  - `cargo test -p ferro-service` — PASS, 20 testes unitários e 10 de integração.
  - `cd apps/desktop && npm test` — PASS, 79 testes.
  - `cd apps/desktop && npm run build` — PASS (TypeScript + Vite). Aviso já conhecido: bundle IDE maior que 500 kB.
- `cargo clippy -p ferro-service --all-targets -- -D warnings` — PASS.
- `PKG_CONFIG_PATH=/tmp/ft10-pkgconfig cargo check -p ferro-desktop --tests` — PASS para compilação Rust dos comandos Tauri. Os arquivos pkg-config temporários permitem checagem sem a instalação local de WebKitGTK.
- `PKG_CONFIG_PATH=/tmp/ft10-pkgconfig cargo clippy -p ferro-desktop --all-targets -- -D warnings` — PASS.
- **Pendente:** testes nativos do desktop e suíte completa do workspace/CI multiplataforma; o host local não tem WebKitGTK para linkar/executar o binário. Screenshot/E2E DOM dedicado não está configurado.
- **Rollback:** remover os serviços e comandos `.fer`, restaurar a desativação desses botões para `.fer` e reverter a linha de paridade. pseudocódigo clássico e a exportação Rust não dependem desta funcionalidade.
