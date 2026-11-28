# INSTRUÇÃO DE ENTRADA PARA AGENTES — LEIA ANTES DE CODIFICAR

Você está integrando a evolução **Ferro Studio Next** em um **projeto em andamento em Rust + Tauri**. Sua tarefa é preservar o projeto e introduzir os novos comportamentos de maneira incremental.

## Guardrails inegociáveis

- NÃO criar novo app, novo workspace Rust, novo frontend ou substituir arquivos raiz por padrão.
- NÃO mudar framework, arquitetura, versão Tauri, gerenciamento de dependências, build, permissões ou editor sem justificativa aprovada.
- NÃO remover funcionalidades clássicas do Ferro para implantar as novas.
- NÃO introduzir mocks permanentes, resultados artificiais de execução/compilação, traduções falsas ou progresso de lição calculado por cliques.
- NÃO permitir comandos de shell arbitrários vindos do frontend; limitar operações Tauri a capacidades/escopos verificáveis.
- NÃO sobrescrever buffers com alterações não salvas nem código Rust gerado editado pelo aluno.
- NÃO implementar 'tradução' como simples troca de keywords sem testes semânticos.

## Etapa A — Inventário SOMENTE LEITURA

Inspecione os arquivos reais (`Cargo.toml`, `src-tauri/`, `tauri.conf.*`, `capabilities/`, `permissions/`, `package.json`, estrutura de `src/`, testes, CI, README e docs) onde existirem. Identifique:
- versão de Tauri, plugins instalados, features Rust e ambiente de execução;
- framework e build tool do frontend; linguagem TypeScript/JS ou outra;
- engine/interpreter existente, parser, AST, debugger, tradutor, formato de arquivos;
- fluxo atual editor → comando → Rust → resultado, contratos IPC, estado e persistência;
- testes disponíveis e sua execução; matriz de plataforma; controles de segurança;
- o que já existe, é parcial, está ausente ou é incompatível (com caminhos/evidências).

Produza um diagnóstico segundo `docs/research/REPOSITORY_DISCOVERY.template.md`. Se algo for desconhecido, marque **UNKNOWN**, não invente.

## Etapa B — Gap analysis e plano

Mapeie componentes `UI-*` → `UC-*` → contrato Tauri → testes. Escreva lista incremental de mudanças por arquivo, com risco, compatibilidade, rollback e teste. Comece por uma fatia vertical verificável: **abrir/editar/salvar/executar/apresentar resultado real** se ainda não existir; caso exista, escolha a menor lacuna real.

Não execute refactor estrutural antes de demonstrar que é necessário. Solicite revisão de mudanças disruptivas antes de aplicá-las.

## Etapa C — Implementar com testes

- Use padrões atuais do repositório para camadas Rust/UI/IPC.
- Execute testes unitários, integração, frontend, E2E e compatibilidade disponíveis.
- Para novas features: inclua IDs de rastreabilidade, erros tipados, loading/error/empty/success, cancelamento e stale-response.
- Gherkin é **especificação**, não teste em execução até existir runner com steps reais.
- Documente features bloqueadas por capacidade ausente; desabilite controles com explicação.

## Etapa D — Evidências

Entregue relatório no modelo `templates/FEATURE_REPORT.template.md` contendo arquivos alterados, comandos de testes realmente executados, resultados e bloqueios. Nunca declarar aprovação de teste sem execução real.

## Ordem de leitura
1. Este arquivo e `README.md`.
2. `product/UI_FUNCTIONAL_CONTRACT.md`.
3. `architecture/TAURI_INTEGRATION_CONTRACT.md`.
4. `architecture/STATE_AND_EVENT_CONTRACT.md`.
5. `product/LANGUAGE_SEMANTICS_CONTRACT.md`.
6. `design/DESIGN_AS_CODE.md` e `design/ACCESSIBILITY_AND_UX_CONTRACT.md`.
7. `quality/TEST_STRATEGY_AND_QUALITY_GATES.md`.
8. Demais docs sob demanda.
