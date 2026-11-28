# Gap analysis — Ferro Studio Next

Inventário completo: [`../research/REPOSITORY_DISCOVERY.md`](../research/REPOSITORY_DISCOVERY.md). A implementação mantém a arquitetura e contratos atuais.

| UI → UC | Contrato/serviço atual | Estado observado | Incremento seguro | Testes/aceitação |
|---|---|---|---|---|
| UI-003/005/033 → UC-06/09 | `run`/`stop`/`debug_*`, VM real, eventos `run:*` | Existe | manter; cobrir fluxos restantes separadamente | Rust tests e smoke nativo |
| UI-011/022 → UC-04/12 | CodeMirror + `inspect(source)` → diagnósticos/símbolos | Existe | manter; sem trocar editor | TS build e testes do serviço |
| UI-015/016 → UC-07/08 | preview `export_rust(source)` e `.rs` editável com lint `rustc`, execução, `argv` e `rustfmt` | Parcial: preview gerado tem subset; arquivo Rust separado recebe diagnóstico de buffer, execução e formatação. Sem Cargo, dependências externas, módulos do workspace ou debugger | preservar distinção preview versus arquivo Rust escrito pelo aluno; evoluir Cargo/debugger como capacidades explícitas | `rust_source_check.rs`, `rust_source_runner.rs`, `rust_format.rs`, capability/stale checks e source maps; E2E permanece pendente |
| **UI-028 → UC-05** | preferência de aprendizagem e visibilidade do preview | **Implementado** | `learning-mode.ts`, `main.ts` e `index.html`; três modos persistem localmente; Rust habilita quando `rust_toolchain_available` confirma `rustc`; nível Rust apresenta conceitos didáticos sem converter o documento | testes Node de persistência, catálogo, disponibilidade e preservação do documento; E2E visual ainda pendente |
| UI-023/024 → UC-13 | `learning-content.ts` + aba Aprender; `learning-context.ts` associa contexto por source map exato | Implementado local: catálogo editorial local + seleções manuais + mapeamento exato em Bridge para mutabilidade, decisões, repetições, funções tipadas, `Opcao`/`Resultado`/`escolha`, `vetor<T>` (`Vec<T>`) e `registro`/`enum` (`struct`/`enum`) | manter proveniência manual/mapeada e evoluir o catálogo com validação docente | testes `learning-content.test.mjs` e `learning-context.test.mjs` |
| UI-002/031/035 → UC-01/18/20 | `TitleBar` (`#titlebar-workspace`, `#titlebar-learning-mode`), `#settings-dialog` e `pane-layout.ts` (`resetPaneLayout`) | Implementado local: barra superior mostra projeto e modo, `Ajustes` abre diálogo dedicado de Configurações com tema (`Sistema` em VS Code Modern + IntelliJ Rust + VS Code populares), fonte, modo, parâmetros e **Restaurar layout padrão** | manter consistência responsiva em `1536×960` e `1280×800` | `design-as-code-shell.test.mjs`, `pane-layout.test.mjs`, `theme-presets.test.mjs` |
| UI-025 → UC-13 | `exercise-progress.ts` + `createValidatedExerciseGrader` | Implementado local: só resultados de casos reais registram revisão SHA-256; conclusão exige todos os casos aprovados; cartões não contam como progresso. Sem conta/sync entre dispositivos | preservar avaliação por revisão e sem coleta pessoal; considerar sincronização somente após política de identidade/privacidade | `exercise-progress.test.mjs` cobre aprovação parcial, revisão, dados inválidos, avaliação obsoleta e storage indisponível; E2E visual pendente |
| UI-026/027/032 → UC-09/15 | debugger VM fornece variáveis e stepping; ownership trace ausente | Parcial | preservar debugger; não simular ownership | FT-07 e contrato de proveniência pendentes |
| UI-001/002/008/009 → UC-01/03 | `workspace_open/restore/read/write/create/rename/delete/export` + explorer | Implementado local: workspace restaura a última pasta e suporta arquivos `.por`, `.fer`, `.rs` e exercícios, ações de árvore, guarda de dirty em exclusão e exportação seletiva. | Não reimplementar o workspace; cobrir fluxos ponta a ponta em Tauri e distinguir workspace simples de projeto Cargo. | Rust tests em `src-tauri/src/workspace.rs`; testes `workspace-explorer`, `workspace-file-actions`, `workspace-export`; E2E visual permanece pendente. |

## Plano incremental

1. FEAT-LEARN-001 e FEAT-LEARN-003: seletor UI-028 agora inclui Rust condicionado à toolchain detectada; preferência segue global em localStorage e ainda não é associada ao workspace. Source e engine não mudam ao trocar modo.
2. Conteúdo UI-023/024 existe como catálogo local de seleção explícita. Próximo incremento: ampliar conteúdo com revisão pedagógica e associação semântica verificável, sem progresso por abertura de cartão.
3. Workspace simples já existe em `src-tauri/src/workspace.rs`; atualizar relatórios antigos. Preferências continuam globais/local, então definir ciclo de vida/migração antes de torná-las por workspace.
4. Tradução progressiva UI-014/016 com resultado vinculado à revisão; ampliar subset no núcleo Rust e testar equivalência semântica.
5. Laboratório de memória permanece simulação didática baseada em fixtures verificadas; trace de runtime Rust continua ausente.
6. Rust editável, lint `rustc`, execução `.rs`, argumentos `argv` e formatação por `rustfmt` já existem para buffers standalone. Permanecem pendentes projetos Cargo, debugger Rust/Ferro e integração E2E visual; nenhum fluxo deve executar comandos de shell arbitrários.

## Estados e limitações da fatia atual

- Beginner é sucesso funcional sem operação assíncrona; Bridge usa estados de carregamento/diagnóstico/resultado já existentes no exportador.
- O modo Rust requer toolchain detectada; a execução `.rs` mantém consentimento, limites e ausência de sandbox claramente apresentados.
- Preferência global é provisória, pois não existe workspace persistente. Não modifica conteúdo nem execução.
- Verificação E2E e visual automatizada em Firefox headless (`apps/desktop/tests/e2e-visual.mjs`, `npm run test:e2e`) executada nas resoluções de referência `1536×960` e `1280×800` (60/60 verificações passando: tela inicial clara/escura, criação de `.fer`, TitleBar sem sobreposição, modo `bridge` em split e compacto, diálogo de Configurações, troca de temas IntelliJ Rust / VS Code Modern, reset de layout, LearningInspector + Ownership Lab e menus da Ajuda FERRO).
- Cenários Gherkin declarativos em `quality/features/` seguem documentados sem runner Cucumber dedicado; a cobertura executável está em `node:test` (447 testes unitários/contrato/componente) + `tests/e2e-visual.mjs` + `tests/graphics-smoke.mjs` (Tauri nativo no CI).
