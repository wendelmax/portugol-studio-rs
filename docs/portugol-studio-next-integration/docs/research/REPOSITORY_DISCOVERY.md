# Inventário do repositório

- Commit/branch analisados: `28b686e` / `feat/next-learning-modes` (worktree isolada).
- Tauri: v2; dependência `tauri = "2"` em `apps/desktop/src-tauri/Cargo.toml:17`; resolução observada no Cargo.lock: 2.12.5. Plugin direto: `tauri-plugin-dialog = "2"` (`:18`; lock 2.8.4). Não há plugin shell/processo.
- Rust workspace/crates e MSRV: workspace edition 2024, oito membros definidos em `Cargo.toml:1-23`; MSRV (`rust-version`) não declarada. `mise.toml` fixa toolchain local Rust 1.99.1 e Node 22.23.5.
- Frontend/framework/linguagem/build: HTML/CSS + TypeScript sem framework SPA; Vite (`apps/desktop/package.json:7-11,25-29`, Vite resolvido 8.3.5, TypeScript 7.0.2).
- Editor/highlight: CodeMirror 6 e extensões próprias Portugol em `apps/desktop/src/main.ts:13-25,213-244`; `src/portugol.ts`.
- Parser/AST/interpreter: `crates/portugol-syntax` (lexer Logos/parser/AST), `portugol-semantics`, `portugol-bytecode`, `portugol-vm`, `portugol-service`; CLI em `crates/portugol-cli`. Debugger e runtime reais na VM.
- Formatos: `.por` e `.pex`; leitura de UTF-8/Latin-1 e gravação UTF-8; `.rs` é exportação gerada pela subset suportada. Sem modelo de workspace/projeto persistente.
- Contratos frontend: `src/api.ts:102-132` define `inspect(source)`, `exportRust(source)`, `run(source,args,path,breakpoints,pauseOnStart) -> number`, controles de debug/input e IO de arquivo. `src-tauri/src/lib.rs:154,255,303-356` implementa exportação, execução, controle, leitura/gravação; o run é associado a ID numérico.
- Eventos/ciclo: `lib.rs:93-133` emite `run:output`, `run:clear`, `run:input`, `run:paused`, `run:finished`; serviço da VM executa em thread e envia stdout em lotes. Cliente filtra pelos IDs da execução. Não há `requestId`, `fileId`, revisão ou sequência no contrato de eventos.
- Permissões: `apps/desktop/src-tauri/capabilities/default.json:8-18` concede core default, operações de janela e diálogo open/save/ask/message. Arquivos são acessados por comandos próprios com caminho selecionado; não existe capability de shell/processo. CSP e janela customizada em `tauri.conf.json`.
- Persistência: preferências (tema, fonte, centralização, dicas e temas customizados) em `localStorage`; buffers/dirty tracking e abas são em memória. Não existe `WorkspaceStore`/ID de workspace.
- Testes/scripts: `npm test` executa Node test runner nos testes `.mjs` (atualmente `tests/theme-presets.test.mjs`); `npm run build` roda `tsc && vite build`. Cargo workspace possui testes unitários, integração e regressão; `cargo test --workspace --offline --locked` passou com sysroot WebKitGTK local. `graphics-smoke.mjs` existe para smoke nativo. Gherkin no pacote é especificação sem runner configurado.
- Plataformas/CI: workflows `.github/workflows/ci.yml` e `release.yml`; CI testa core multiplataforma e build/test Tauri em Linux/Windows/macOS conforme jobs. Node 22 na CI.

## Tabela de lacunas

| ID | Requisito | Situação | Evidência | Risco | Próxima ação |
|---|---|---|---|---|---|
| UC-04/06 | editar, executar e apresentar resultado real | EXISTS | `apps/desktop/src/main.ts`, `src/api.ts`, `src-tauri/src/lib.rs` | baixo | preservar; manter compatibilidade |
| UI-015 / UC-07/08 | prévia/exportação Portugol→Rust | PARTIAL | `apps/desktop/index.html:91-102`, `src/main.ts:1047-1078`, `src/api.ts:104-109` | tradução cobre subset, não executa Rust | apresentar como saída gerada; melhorar subconjunto incrementalmente |
| UI-028 / UC-05 | seletor de níveis beginner/bridge/rust | MISSING | HTML tem apenas `btn-rust` (`index.html:75`); nenhum `LearningMode` encontrado | modo Rust poderia sugerir capacidade inexistente | adicionar selector acessível; rust indisponível com explicação; bridge reaproveita exportação |
| UI-023..027 / UC-13/15 | explicações, progresso, ownership visual | MISSING/PARTIAL | exercícios e debugger existentes; sem serviço de lições/snapshot de memória | inventar progresso ou proveniência seria enganoso | fatiar após definir conteúdo e modelo semântico rastreável |
| UC-01 / UI-001..002 | workspaces/projetos com persistência | MISSING | sem store/ID de workspace; abas voláteis | requisito por workspace conflita com estado atual | usar preferência global somente como solução temporária explícita; migrar quando houver workspace real |

## Decisões existentes a preservar

- Não substituir o parser/runtime; reutilizar crates e comandos atuais.
- Não trocar framework, editor, Tauri, build ou permissões para introduzir o seletor.
- Execução Portugol e depuração usam a VM real; exportação Rust é geração limitada, não compilação.
- Nenhum comando shell arbitrário é exposto ao frontend.

## Incompatibilidades e perguntas bloqueantes

- A especificação pede preferência por workspace, mas o projeto ainda não possui identidade/store de workspace. Esta fatia usa `localStorage` global e documenta migração como dívida; não altera execução nem arquivos.
- O nível `rust` depende de editor e toolchain/compiler ausentes; deve permanecer desabilitado, com motivo acessível.
- Eventos atuais não carregam revisão de documento; tarefas futuras de tradução/execução precisarão tratar stale results sem presumir campos inexistentes.

## Alterações propostas (fatia UI-028)

- Adicionar seletor de modo em opções existentes do editor; persistir somente valores válidos.
- `beginner`: preservar experiência atual e ocultar painel Rust; `bridge`: abrir painel de exportação real; `rust`: disabled com explicação.
- Reutilizar `export_rust`, sem comandos/permissões/dependências Rust novos.
- Cobrir persistência e fallback por testes Node; registrar teste Gherkin como especificação não executada.
