# Governança dos agentes

## Princípio
Modificar menos do que o necessário, com evidência máxima e comportamento real. Tauri/Rust existentes são a base; este pacote é proposta.

## Regras
1. Descobrir/referenciar arquivos reais antes de qualquer refactor.
2. Relacionar toda alteração aos IDs `UI-*`, `UC-*`, `FT-*` ou criar ID documentado.
3. Não instalar biblioteca, substituir framework, mudar versão Tauri ou criar engine paralela sem ADR e aprovação.
4. Não editar artifacts gerados, lockfiles, configs, capabilities ou permissões sem motivo registrado e testes.
5. Respeitar convenções de formatação/lint/build da base existente.
6. Mock só em teste, storybook ou demo **explicitamente rotulada**. Não mascarar ausência de backend.
7. Para mudanças da engine, preservar interpretação original via corpus de regressão.
8. Logs de testes devem indicar comandos executados e falhas; relatórios honestos sobre limites do ambiente.
9. Feature concluída exige critérios objetivos e evidências de interface e do serviço real.
10. Em conflito com o código atual, propor adapter ou migração incremental; nunca substituir tudo silenciosamente.

## Contratos entre agentes
- Agente de UI não assume que backend existe; consulta registry de capacidades.
- Agente de backend não assume nomes de payload sem verificar consumers existentes.
- Agente de testes deriva casos de uso, cobre happy path, erro, cancelamento, concorrência e regressão.
- Integração é responsabilidade do agente que finaliza o fluxo vertical, não uma tarefa implícita de outro agente.
