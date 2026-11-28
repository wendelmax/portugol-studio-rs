# Estratégia de qualidade — do design até o Rust backend

## Testes necessários

1. **Unitários Rust**: parser, AST, tipos, emissão Rust, sanitização de caminhos, state machines.
2. **Contratos IPC Tauri**: payload/serde, códigos de erro, eventos de solicitação, revisão, permissões e concorrência (testes Rust + frontend adapter conforme stack).
3. **Unitários UI**: estados de componentes, atalho, tratamento de erros e foco (runner detectado no repo).
4. **Integração real**: salvar/ler com diretório temporário, interpreter real, rustc/Cargo real quando disponíveis.
5. **BDD feature tests**: arquivos `.feature` fornecidos são critérios; ligar steps ao runner existente ou escolher runner mediante ADR.
6. **E2E desktop Tauri**: usar infraestrutura do projeto; testar WebView/plataformas suportadas. Testes browser-only não substituem IPC real.
7. **Visual regression**: screenshots por viewport e tema, com baseline revisado; nunca aceitar diffs automaticamente.
8. **Compatibilidade**: corpus de códigos do Ferro antigo e regressões de formatos.
9. **Segurança**: path traversal, cancelamento, timeout, flooding stdout, build com permissões mínimas.
10. **Acessibilidade**: teclado, foco, nomeação e contraste verificáveis.

## Gates por PR/feature
- G0: descoberta, análise de impacto, ID de requisito e teste.
- G1: lint, formatter, build nas ferramentas atuais e unitários.
- G2: contratos Rust↔frontend e integração testados.
- G3: BDD com steps reais, E2E UI no ambiente suportado, teste visual quando tela alterada.
- G4: segurança/compatibilidade, evidence report e revisão de regressões.
- G5: manual de rollback para mudanças com migração/alteração de runtime.

Regra: não usar status PASS se runner não existe ou teste não rodou. Use `NOT_RUN`, `BLOCKED`, `FAIL`, `PASS` com comando e evidência.

## Matriz de execução (preencher)

| Nível | Comando REAL | Plataforma | Resultado | Evidência |
|---|---|---|---|---|
| Cargo | UNKNOWN | UNKNOWN | NOT_RUN | — |
| Frontend | UNKNOWN | UNKNOWN | NOT_RUN | — |
| E2E Tauri | UNKNOWN | UNKNOWN | NOT_RUN | — |
| BDD | UNKNOWN | UNKNOWN | NOT_RUN | — |
