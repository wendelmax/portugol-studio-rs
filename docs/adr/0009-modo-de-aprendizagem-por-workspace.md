# ADR 0009 — Persistir o modo de aprendizagem por workspace

- **Status:** aceita
- **Data:** 2026-11-22

## Contexto

O seletor Beginner/Bridge/Rust salva uma única preferência global no `localStorage`. O design do produto pede que cada workspace lembre o modo do próprio projeto; um exemplo de Rust e uma aula de Ferro podem coexistir em abas sem trocar a fonte.

## Decisão proposta

Associar cada listagem de workspace a uma chave estável, calculada no backend como SHA-256 do caminho canônico, sem retornar o caminho absoluto à interface. Cada aba de arquivo de workspace carrega essa chave. O frontend persiste e restaura o modo sob uma chave `localStorage` por workspace; abas sem workspace continuam usando a chave global existente. Se ainda não houver preferência por workspace, o modo global existente funciona como valor inicial/migração.

## Consequências

- Alternar abas de workspaces diferentes restaura a preferência correspondente sem alterar o documento.
- Novos projetos preservam o comportamento global existente até que uma escolha específica seja salva.
- Renomear arquivos não afeta a preferência. Mover a pasta gera uma chave nova e usa o valor global como padrão.
- A chave derivada não é autorização nem identidade de segurança; comandos Tauri continuam validando o `workspaceId` de sessão.
