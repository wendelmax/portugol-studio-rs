# Histórico de versões

## 0.3.0 (2026-11-24)

### Novidades

- **pors:** add consented cancellable Internet library across VM, WASM, Rust and IDE (#127)
- **ide:** default new files to .pors, refine Tests panel and add Json/Arquivos examples and exercises (#125)

### Manutenção

- remove LICENSE.md duplicado (#128)

Comparação completa: https://github.com/wendelmax/portugol-studio-rs/compare/v0.2.0...v0.3.0

## 0.2.0 (2026-11-24)

### Novidades

- **pors:** add bitwise/shift operators and wildcard arm in escolha (#124)
- **pors:** add zero-dependency Json library across VM, WASM, Rust and IDE (#122)
- **pors:** expand examples, exercises and IDE help
- **pors:** add workspace-confined Arquivos library across VM, Rust and IDE
- **pors:** add Util and Calendario libraries with deterministic UTC clock and IDE catalog sync (#118)
- **pors:** add dynamic vectors ([], adicionar, remover, obter) and Texto.dividir (#117)
- **ide:** add PORS contextual autocomplete, snippets and hover tooltips (#116)
- **pors:** add core Matematica, Texto and Tipos libraries across VM, WASM and Rust (#115)
- **pors:** add built-in Opcao<T>, Resultado<T, E> and ? propagation (#114)
- **ide:** integrate .pors examples and versioned exercises into desktop app (#113)
- **pors:** add tested .pors examples corpus and versioned exercise evaluator
- **rust:** export visual PORS MVU apps to standalone Rust and Cargo projects
- **ide:** add PORS visual MVU preview panel, Tauri backend and examples
- **ui:** add TypeScript WebAssembly host bridge for PORS MVU preview (#109)
- **pors:** add typed UI.* declarative calls and MVU entry support (#108)
- **pors:** add records and exhaustive enums
- **cli:** support creating, validating, formatting, exporting and running .pors (#105)
- **vm:** add declarative UI contract and event-driven MVU runtime
- **pors:** add canonical formatter and editor integration
- **vm:** gate stdlib for wasm32 PORS core
- **pors:** add typed vector support
- **pors:** add user-defined functions (#97)
- depurador VM para Portugol-RS (#90)
- encaminha argumentos para projetos Cargo (#89)
- open related learning exercises (#86)
- enrich learning inspector explanations (#85)
- fixar tópico no inspetor de aprendizagem (#84)
- comparar versões de Rust gerado (#83)
- confirmar desvinculação de Rust gerado (#82)
- render generated Rust in read-only editor (#81)
- persist pane layout per workspace (#80)
- persist learning mode per workspace (#79)
- selecionar binários Cargo no editor (#78)
- analisar projetos Cargo no editor (#77)
- executar projetos Cargo no workspace Rust (#76)
- adiciona diagnósticos Rust no editor (#74)
- map learning concepts to generated Rust
- add rustfmt support for standalone Rust files
- pass arguments to standalone Rust programs
- habilitar modo de aprendizagem Rust
- adicionar entrada interativa ao Portugol-RS
- cancel Portugol-RS executions
- cancel standalone Rust executions
- execute standalone Rust sources in desktop IDE (#66)
- **ui:** contextual learning from Rust source maps
- **ui:** compose Portugol Studio Next desktop shell
- support Portugol-RS loop control (#61)
- run exercise cases in tests panel
- highlight supported PORS loops
- group home page language actions
- support counted loops in Portugol-RS
- add Portugol Rust source mapping gutter
- execute supported Portugol-RS programs
- **portugol-rs:** add semantic symbol navigation and rename (#52)
- **workspace:** exportar arquivos selecionados do projeto (#51)
- add honest project tests panel
- add accessible command palette
- persist accessible resizable pane layout
- detach generated Rust into editable documents
- delete workspace files from explorer
- rename workspace files from explorer
- restore last workspace on startup (#44)
- create Portugol files in workspace (#43)
- add scoped workspace explorer (#42)
- integrate .pors document flow in desktop IDE
- add initial Portugol-RS frontend
- add shared typed HIR foundation
- **editor:** navegação semântica até declaração (#38)
- **desktop:** laboratório visual de ownership (#37)
- register verified local exercise progress
- add progressive learning concept cards
- add progressive learning mode selector
- preview and export a supported Portugol subset to Rust (#31)
- add popular offline color themes

### Correções

- **cargo:** separate target discovery from program arguments
- **pors:** complete editor support for helper functions
- reject Cargo manifest symlinks in service (#88)
- improve secondary text contrast in themes (#87)
- **ui:** preserve stale Rust preview per tab
- **desktop:** allow Vite to serve repository license (#62)
- fit resizable panes to narrow windows
- disable Portugol symbol navigation in Rust tabs
- canonicalize workspace path before delete
- limit workspace delete wrapper to tests
- confine workspace file deletion to root handle

### Documentação

- reescreve README para o público do projeto (#123)
- define visual UI state and WASM runtime ADR
- define PORS-first product migration (#96)
- reprioritize progressive learning product (#92)
- atualiza inventário e lacunas do Portugol Studio Next (#75)
- record FT-06 verification evidence
- link workspace delete report to pull request
- record workspace delete implementation commit
- clarify pending home screen parity evidence (#36)
- record green UI learning CI
- record Portugol Webstudio product research

### Manutenção

- filtro por caminhos e fluxo de release (#126)
- Merge pull request #106 from wendelmax/feat/pors-wasm-facade
- Merge pull request #104 from wendelmax/feat/gemini-ui-renderer
- verify PORS diagnostic spans (#95)
- expand Portugol-RS differential operator coverage (#94)
- guard Portugol-RS debugger against stale run events
- cover Portugol-RS chain input memory limit (#91)
- Adiciona loops seguros ao Portugol-RS (#55)
- Mapear Rust gerado e diagnósticos para a origem Portugol
- canonicalize workspace rename assertion
- generate Rust from shared HIR
- Abre programas e exercícios enviados pelo sistema
- Completa a API de teclado e o autocompletar de bibliotecas
- Valida o aplicativo desktop também em Windows e macOS
- Adiciona regras estáticas ao corretor de exercícios
- Ajusta filtros e centralização do código-fonte
- Permite importar e corrigir exercícios .pex
- Aproxima o diálogo de localizar e substituir
- Carrega os alertas públicos do projeto
- Aproxima a tela inicial e o editor do Portugol Studio
- Adiciona verificação de atualizações da IDE
- Adiciona exercícios originais e correção dinâmica
- Adiciona editor de temas com perfis e prévia
- Adiciona renomeio semântico de símbolos
- Adiciona os diálogos de alertas e relato de bug
- Adiciona formatação de código-fonte à IDE
- Adiciona dobras de código no editor
- Adiciona as 18 dicas de interface
- Adiciona tabela e faixa de diagnósticos à IDE
- Porta a biblioteca Sons com saída nativa e regressões do corpus
- Usa a seleção UTF-8 no teste gráfico em X11
- Executa o teste da IDE com sessão D-Bus no Ubuntu 24.04
- Disponibiliza os 119 exemplos originais com seus recursos
- Localiza a IDE por geometria e limita o tempo do teste gráfico
- Reproduz a moldura e os comandos visuais do Portugol Studio
- Verifica a janela gráfica por pixels no Xvfb sem o inspetor
- Corrige o IPC gráfico empacotado e testa a janela nativa na CI
- Adiciona árvore estrutural e inspetor ao Painel Utilitários
- Conecta Graficos, Teclado e Mouse à janela da IDE
- Integra a ajuda original de linguagem e bibliotecas
- Aproxima a IDE do Portugol Studio e adiciona início da depuração

Comparação completa: https://github.com/wendelmax/portugol-studio-rs/compare/v0.1.0...v0.2.0
