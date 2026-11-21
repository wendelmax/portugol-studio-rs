# Gap analysis — Portugol Studio Next

Inventário completo: [`../research/REPOSITORY_DISCOVERY.md`](../research/REPOSITORY_DISCOVERY.md). A implementação mantém a arquitetura e contratos atuais.

| UI → UC | Contrato/serviço atual | Estado observado | Incremento seguro | Testes/aceitação |
|---|---|---|---|---|
| UI-003/005/033 → UC-06/09 | `run`/`stop`/`debug_*`, VM real, eventos `run:*` | Existe | manter; cobrir fluxos restantes separadamente | Rust tests e smoke nativo |
| UI-011/022 → UC-04/12 | CodeMirror + `inspect(source)` → diagnósticos/símbolos | Existe | manter; sem trocar editor | TS build e testes do serviço |
| UI-015/016 → UC-07/08 | `btn-rust` + `export_rust(source)` | Parcial: subset exportado, sem edição/compilação Rust | reaproveitar como preview somente; status e erros atuais | serviço `portugol-rust`, teste de UI futura |
| **UI-028 → UC-05** | preferência de aprendizagem e visibilidade do preview | **Ausente** | seletor `beginner/bridge/rust`; beginner conserva UX; bridge revela export; rust desabilitado por ausência de compiler/editor | AC-02: trocar modo não altera source/engine; FT-02 especificado; testes Node para persistência e markup |
| UI-023/024 → UC-13 | `learning-content.ts` + aba Aprender, seleção explícita | Parcial: catálogo local e cartões Beginner/Bridge; sem resolver AST ou currículo versionado | manter proveniência manual, exemplos Rust editoriais, expandir conteúdo com validação docente | testes Node do catálogo passam; FT-05/E2E ainda sem runner |
| UI-025 → UC-13 | progresso via resultados verificados de exercícios | Ausente | só modelar progresso após identidade/revisão e avaliações reais; não contar abertura de cartão | FT-05 e integração de resultados pendentes |
| UI-026/027/032 → UC-09/15 | debugger VM fornece variáveis e stepping; ownership trace ausente | Parcial | preservar debugger; não simular ownership | FT-07 e contrato de proveniência pendentes |
| UI-001/002/008/009 → UC-01/03 | diálogo de arquivo e comandos de IO pontuais | Parcial; sem workspace/tree mutations | desenhar workspace/file service antes de UI dependente | FT-01/08 pendentes |

## Plano incremental

1. Fatia atual, FEAT-LEARN-001: seletor UI-028, preferência global temporária em localStorage, estados honestos; sem alteração Rust/Tauri/IPC. Risco baixo. Compatibilidade preservada: source e escolha do motor não mudam. Rollback: remover seletor, módulo e testes; preview antigo continua disponível pelo botão Rust.
2. Conteúdo explicativo UI-023/024 com catálogo versionado e associação a conceitos, sem progresso até existir validador de exercícios.
3. Workspace real e DocumentStore antes de mover preferências para escopo de projeto ou adicionar árvore de arquivos.
4. Tradução progressiva UI-014/016 com resultado vinculado à revisão; ampliar subset no núcleo Rust e testar equivalência semântica.
5. Laboratório de memória somente com snapshots da VM ou simulação explicitamente rotulada e baseada em modelo verificável.
6. Rust editável/compilação apenas após escolha de runtime seguro e contratos/capabilities aprovados; sem shell arbitrário.

## Estados e limitações da fatia atual

- Beginner é sucesso funcional sem operação assíncrona; Bridge usa estados de carregamento/diagnóstico/resultado já existentes no exportador.
- Rust é desabilitado, com explicação persistente no controle; cancelamento de exportação não existe hoje e não será alegado.
- Preferência global é provisória, pois não existe workspace persistente. Não modifica conteúdo nem execução.
- Gherkin/FT-02 permanece `NOT_RUN`: nenhum runner BDD configurado. Testes automatizados desta feature serão Node `node:test` no padrão existente.
- Sem screenshot/E2E do seletor disponível no setup atual; registrar como `NOT_RUN`.
