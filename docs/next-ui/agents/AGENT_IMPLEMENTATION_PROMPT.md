# Prompt para Codex / Claude Code

Você está dentro de um repositório **existente** de uma IDE em **Rust + Tauri**.

Leia `START_HERE_FOR_AGENT.md` e `README.md` nesta pasta. Execute obrigatoriamente o inventário somente leitura com `docs/research/REPOSITORY_DISCOVERY.template.md`. Não crie outra aplicação. Compare as capacidades existentes às propostas em `docs/product/UI_FUNCTIONAL_CONTRACT.md`, `docs/architecture/TAURI_INTEGRATION_CONTRACT.md` e `docs/design/DESIGN_AS_CODE.md`. Mostre as lacunas com evidência de caminhos do repositório. Proponha uma pequena fatia vertical preservando a implementação atual, especificando arquivos que seriam alterados, contratos Rust↔WebView, permissão Tauri mínima, testes e rollback. Depois implemente a primeira fatia priorizada de acordo com a autonomia autorizada no projeto, sem refactors abrangentes, sem mocks permanentes e sem falsos resultados. Entregue o relatório `templates/FEATURE_REPORT.template.md`.
