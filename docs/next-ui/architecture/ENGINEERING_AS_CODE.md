# Engineering as Code — Rust + Tauri (baseline proposto)

Este documento não declara a estrutura física atual do repositório. O agente deve mapear diretórios reais antes de criar módulos.

## Responsabilidades lógicas
- **Frontend WebView**: componentes, editor, stores, status visual, input e acessibilidade. Não executa shell ou parse do compilador de maneira autoritativa.
- **Application commands**: casos de uso, orquestração, validação de contexto/revisão/cancelamento.
- **Tauri IPC adapter**: transporte tipado, validação, publicação de eventos e integração com lifecycle.
- **Rust domain/services**: workspace, parser, interpreter, translator, debugger, exercise grading.
- **Infrastructure**: filesystem autorizado, subprocessos Rust, logs, cache, persistência.

## Fluxo de dependências
Frontend UI → frontend adapters/stores → IPC → Rust application → domain ports → infrastructure adapters. Preferir inversão de dependência onde arquitetura existente permitir.

## Restrições
- Não presumir Tauri v2, estado global específico ou biblioteca CSS/editor.
- Não criar HTTP API/servidor local para contornar IPC sem necessidade comprovada.
- Serialização padronizada conforme contrato existente para evitar divergência camelCase/snake_case.
- Comandos executados fora da thread principal, UI fluida, cancelamento e correlação explícitos.
- Serviços idempotentes quando aplicável e sem race de revisão.

## Decisions pending
1. Frontend framework real e estratégia de estado.
2. Interpreter e AST atuais.
3. Shell/process API e capability model conforme Tauri existente.
4. Estratégia de tradução e target Rust edition.
5. Teste E2E suportado em Windows/Linux/macOS conforme CI real.
