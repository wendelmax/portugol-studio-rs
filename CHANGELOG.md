# Histórico de versões

## 0.7.0 (2026-11-27)

### Novidades

- **ide:** exportar projeto Cargo e suportar subpastas e atualização no Explorador (#250)
- **ide:** trilha interativa, paleta expandida e preferências de editor em Ajustes (#249)
- **help:** nova Ajuda interativa FERRO + Rust com árvore, histórico, F1 contextual e 100% da stdlib (#247)
- **web:** redesinar playground com temas e editor destacado (#246)
- **wasm:** expor exportação Rust no ferro-wasm e no Web Playground (#242)

### Correções

- **web:** carregar exemplo visual ao abrir prévia UI (#248)
- **ui:** corrigir navegação da ajuda, atalhos, contraste WCAG, CSS e textos legados (#245)
- **web:** corrigir painéis ocultos e status da troca de exemplo (#244)
- **core:** corrigir rename em vetores, stack trace e controles de execução (#243)

### Manutenção

- **arch:** adicionar empacotamento PKGBUILD e job no CI/Release (#251)
- publicar Web Playground WASM no GitHub Pages e atualizar actions para v7 (#241)

Comparação completa: https://github.com/wendelmax/ferro-studio/compare/v0.6.0...v0.7.0

## 0.6.0 (2026-11-26)

### Novidades

- verificação real com rustc no Laboratório de Ownership e Web Playground WASM (#239)
- exercícios e LearningInspector 100% FERRO, docs atualizadas e ícones proprietários do bundle Tauri (#233)
- **desktop:** adapta Bibliotecas e Modelos do Inspector para FERRO e integra E2E visual no CI (#232)
- **desktop:** conclui shell Next (TitleBar, Configurações, reset de layout), LearningInspector FERRO→Rust, polimento visual e E2E 1536x960/1280x800 (#231)
- **desktop:** define Sistema (VS Code Modern) como tema padrão e remove preset de cor Ferro Studio Next (#230)
- **desktop:** refaz menus da ajuda para FERRO, organiza docs/next-ui e padroniza temas IntelliJ Rust e VS Code (#229)
- **ide:** adota interface proprietária com barra superior, ícones SVG, tela inicial e dicas próprias (#228)

### Documentação

- **release:** atualizar notas de distribuição e aviso de retirada do .por/.pex (#240)

### Manutenção

- **desktop:** aplicar code-splitting no Vite, atualizar testes FERRO e indexar docs/plans (#238)
- **vm:** remover módulos clássicos mortos da stdlib e dependências ociosas (#237)
- **core:** extrair diagnósticos para ferro-hir e remover crates e corpus legados (#236)
- **core:** remover caminhos legados .por/.pex de cli, service e rust (#235)
- **desktop:** remover suporte legado a .por e .pex da IDE e Tauri (#234)
- Migrar todos os exemplos visíveis de .por para FERRO (#226)
- endurece backend, alinha docs e modulariza núcleo FERRO (#227)
- Unificar a linguagem visual do shell do editor (#224)

Comparação completa: https://github.com/wendelmax/ferro-studio/compare/v0.5.0...v0.6.0

## 0.5.0 (2026-11-25)

### Manutenção

- Barra de status com linguagem, modo e contagem de diagnósticos (#222)
- Comparação lado a lado entre código e Rust gerado (#221)
- Comandos de depuração indisponíveis para .fer na paleta e no botão Pausar (#220)
- Trilha de aprendizado visível no painel Aprender (#219)

Comparação completa: https://github.com/wendelmax/ferro-studio/compare/v0.4.0...v0.5.0

## 0.4.0 (2026-11-25)

### Novidades

- **ide:** amplia acessibilidade de temas e controles
- **ferro:** recupera erros e acumula diagnósticos
- **ferro:** add typed multimedia bridge for Graficos, Teclado, Mouse, and Sons (#131)
- **ide:** executar .fer na VM sem exigir Rust instalado (#142)
- **ide:** unify cargo test execution and results in Tests panel (#130)
- **ferro:** add faca-enquanto loop and interactive CLI debugger (#129)

### Documentação

- fluxo de reserva de issues para trabalho em paralelo (#147)

### Manutenção

- **files:** extrai composição de arquivos
- **run:** extrai composição da execução
- **editor:** extrai composição do editor
- **shell:** extrai shell de toolchain
- **arquivos:** extrai shell de arquivos do sistema
- **execucao:** extrai shell de eventos
- **comandos:** extrai shell de atalhos
- **aprendizado:** extrai shell do modo
- **learning:** extrai shell de exercícios
- **run:** extrai shell dos controladores
- **run:** extrai shell de apresentação
- **editor:** extrai shell de ações
- **ferro:** extrai shell da prévia visual
- **state:** extrai composição de ativação de abas
- **run:** extrai estado de execução
- **editor:** compõe ações de símbolos
- Refatoração: modularizar main.ts por responsabilidade (#202)
- **dep:** extrai composição do debugger
- **ide:** extrai contexto da paleta de comandos (parte de #139) (#200)
- **files:** extrai eventos de arquivos do sistema (parte de #139) (#199)
- **ide:** extrai controle de temas do editor (parte de #139) (#198)
- **ide:** extrai setup e navegação do editor (parte de #139) (#197)
- **ide:** extrai montagem compartilhada do editor (parte de #139) (#196)
- Refatoração: extrair visualização dos recentes (#195)
- Refatoração: extrair composição das abas (#194)
- Refatoração: extrair despacho de atalhos (#193)
- Refatoração: extrair barra de atividades (#192)
- Refatoração: extrair controles da barra de execução (#191)
- Refatoração: modularizar main.ts por responsabilidade (#190)
- Refatoração: modularizar main.ts por responsabilidade (#189)
- Refatoração: modularizar main.ts por responsabilidade (#188)
- Refatoração: modularizar main.ts por responsabilidade (#187)
- Refatoração: modularizar main.ts por responsabilidade (#186)
- Refatoração: modularizar main.ts por responsabilidade (#185)
- Refatoração: modularizar main.ts por responsabilidade (#184)
- Refatoração: modularizar main.ts por responsabilidade (#183)
- Refatoração: modularizar main.ts por responsabilidade (#182)
- Refatoração: modularizar main.ts por responsabilidade (#181)
- Refatoração: modularizar main.ts por responsabilidade (#180)
- Refatoração: modularizar main.ts por responsabilidade (#179)
- Refatoração: modularizar main.ts por responsabilidade (#178)
- Refactor editor update lifecycle (#177)
- Refactor IDE shortcut catalog (#176)
- Refactor editor formatting workflow (#175)
- Refactor editor symbol navigation (#174)
- Refactor system file intake (#173)
- Refactor IDE tab activation lifecycle (#172)
- **ide:** extrai capacidades por linguagem (#171)
- **ide:** extrai fechamento das abas (#170)
- **ide:** extrai factory e metadados das abas (#169)
- **ide:** extrai renderizacao das abas (#168)
- **ide:** extrai execução do pseudocódigo clássico (#167)
- **ide:** extrai eventos de execução (#166)
- **ide:** extrai execução do Ferro (#165)
- **ide:** extrai execução de projetos Rust (#164)
- **ide:** extrai comandos do depurador (#163)
- **ide:** extrai controles de execução (#162)
- **ide:** extrai orquestração Cargo (#161)
- **ide:** extrai painel de testes de exercícios (#160)
- **ide:** extrai controlador de aprendizagem (#159)
- **ide:** extrai leitura e escrita de arquivos (#158)
- **ide:** separa style.css em módulos semânticos e tokens (#150)
- Consolidar propostas de UX em docs/UX.md (#155)
- FERRO: aceitar senao se encadeado (#149)
- **ide:** extrai janela, ajuda e diálogos do projeto do main.ts (#154)
- **ide:** extrai paleta de comandos e atalhos do main.ts (#152)
- **ide:** extrai painel Aprender e laboratório de ownership do main.ts (#151)
- **ide:** extrai prévia do Rust gerado do main.ts (#148)
- **ide:** extrai explorador do workspace e tipo Tab do main.ts (#146)
- **ide:** extrai console, status e visão do depurador do main.ts (#145)
- **ide:** extrai tema e decorações do editor do main.ts (#144)
- **ide:** extrai layout de painéis e arquivos recentes do main.ts (#143)

Comparação completa: https://github.com/wendelmax/ferro-studio/compare/v0.3.0...v0.4.0

## 0.3.0 (2026-11-24)

### Novidades

- **ferro:** add consented cancellable Internet library across VM, WASM, Rust and IDE (#127)
- **ide:** default new files to .fer, refine Tests panel and add Json/Arquivos examples and exercises (#125)

### Manutenção

- remove LICENSE.md duplicado (#128)

Comparação completa: https://github.com/wendelmax/ferro-studio/compare/v0.2.0...v0.3.0

## 0.2.0 (2026-11-24)

### Novidades

- **ferro:** add bitwise/shift operators and wildcard arm in escolha (#124)
- **ferro:** add zero-dependency Json library across VM, WASM, Rust and IDE (#122)
- **ferro:** expand examples, exercises and IDE help
- **ferro:** add workspace-confined Arquivos library across VM, Rust and IDE
- **ferro:** add Util and Calendario libraries with deterministic UTC clock and IDE catalog sync (#118)
- **ferro:** add dynamic vectors ([], adicionar, remover, obter) and Texto.dividir (#117)
- **ide:** add FERRO contextual autocomplete, snippets and hover tooltips (#116)
- **ferro:** add core Matematica, Texto and Tipos libraries across VM, WASM and Rust (#115)
- **ferro:** add built-in Opcao<T>, Resultado<T, E> and ? propagation (#114)
- **ide:** integrate .fer examples and versioned exercises into desktop app (#113)
- **ferro:** add tested .fer examples corpus and versioned exercise evaluator
- **rust:** export visual FERRO MVU apps to standalone Rust and Cargo projects
- **ide:** add FERRO visual MVU preview panel, Tauri backend and examples
- **ui:** add TypeScript WebAssembly host bridge for FERRO MVU preview (#109)
- **ferro:** add typed UI.* declarative calls and MVU entry support (#108)
- **ferro:** add records and exhaustive enums
- **cli:** support creating, validating, formatting, exporting and running .fer (#105)
- **vm:** add declarative UI contract and event-driven MVU runtime
- **ferro:** add canonical formatter and editor integration
- **vm:** gate stdlib for wasm32 FERRO core
- **ferro:** add typed vector support
- **ferro:** add user-defined functions (#97)
- depurador VM para Ferro (#90)
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
- adicionar entrada interativa ao Ferro
- cancel Ferro executions
- cancel standalone Rust executions
- execute standalone Rust sources in desktop IDE (#66)
- **ui:** contextual learning from Rust source maps
- **ui:** compose Ferro Studio Next desktop shell
- support Ferro loop control (#61)
- run exercise cases in tests panel
- highlight supported FERRO loops
- group home page language actions
- support counted loops in Ferro
- add Ferro Rust source mapping gutter
- execute supported Ferro programs
- **ferro-rust:** add semantic symbol navigation and rename (#52)
- **workspace:** exportar arquivos selecionados do projeto (#51)
- add honest project tests panel
- add accessible command palette
- persist accessible resizable pane layout
- detach generated Rust into editable documents
- delete workspace files from explorer
- rename workspace files from explorer
- restore last workspace on startup (#44)
- create Ferro files in workspace (#43)
- add scoped workspace explorer (#42)
- integrate .fer document flow in desktop IDE
- add initial Ferro frontend
- add shared typed HIR foundation
- **editor:** navegação semântica até declaração (#38)
- **desktop:** laboratório visual de ownership (#37)
- register verified local exercise progress
- add progressive learning concept cards
- add progressive learning mode selector
- preview and export a supported Ferro subset to Rust (#31)
- add popular offline color themes

### Correções

- **cargo:** separate target discovery from program arguments
- **ferro:** complete editor support for helper functions
- reject Cargo manifest symlinks in service (#88)
- improve secondary text contrast in themes (#87)
- **ui:** preserve stale Rust preview per tab
- **desktop:** allow Vite to serve repository license (#62)
- fit resizable panes to narrow windows
- disable Ferro symbol navigation in Rust tabs
- canonicalize workspace path before delete
- limit workspace delete wrapper to tests
- confine workspace file deletion to root handle

### Documentação

- reescreve README para o público do projeto (#123)
- define visual UI state and WASM runtime ADR
- define FERRO-first product migration (#96)
- reprioritize progressive learning product (#92)
- atualiza inventário e lacunas do Ferro Studio Next (#75)
- record FT-06 verification evidence
- link workspace delete report to pull request
- record workspace delete implementation commit
- clarify pending home screen parity evidence (#36)
- record green UI learning CI
- record Ferro Webstudio product research

### Manutenção

- filtro por caminhos e fluxo de release (#126)
- Merge pull request #106 from wendelmax/feat/ferro-wasm-facade
- Merge pull request #104 from wendelmax/feat/gemini-ui-renderer
- verify FERRO diagnostic spans (#95)
- expand Ferro differential operator coverage (#94)
- guard Ferro debugger against stale run events
- cover Ferro chain input memory limit (#91)
- Adiciona loops seguros ao Ferro (#55)
- Mapear Rust gerado e diagnósticos para a origem Ferro
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
- Aproxima a tela inicial e o editor do Ferro Studio
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
- Reproduz a moldura e os comandos visuais do Ferro Studio
- Verifica a janela gráfica por pixels no Xvfb sem o inspetor
- Corrige o IPC gráfico empacotado e testa a janela nativa na CI
- Adiciona árvore estrutural e inspetor ao Painel Utilitários
- Conecta Graficos, Teclado e Mouse à janela da IDE
- Integra a ajuda original de linguagem e bibliotecas
- Aproxima a IDE do Ferro Studio e adiciona início da depuração

Comparação completa: https://github.com/wendelmax/ferro-studio/compare/v0.1.0...v0.2.0
