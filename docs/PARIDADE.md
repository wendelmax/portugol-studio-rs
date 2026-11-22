# Inventário histórico do Portugol Studio clássico

> **Direção atual:** conforme o [ADR 0016](adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md), este inventário documenta capacidades do produto Java que podem ser portadas para PORS/Rust. Ele não define que `.por` será mantido no produto final, nem representa paridade já alcançada por PORS. A fonte de estado e dos critérios de migração é [`PORS_MIGRATION_MATRIX.md`](research/PORS_MIGRATION_MATRIX.md).

Inventário do que o Portugol Studio original oferece e o estado no port.
Atualize esta tabela junto com cada recurso entregue.

Legenda: ✅ feito · 🟡 parcial · ⬜ falta · ➖ não se aplica

O protocolo de quadros e o contrato `Host` estão em
`crates/portugol-vm/src/graphics.rs` e `host.rs`; a suíte de regressão usa um
host headless que captura os quadros, então os exemplos gráficos do corpus já
são executados e validados sem tela.

## Linguagem e execução

| Recurso | Estado | Observações |
|---|---|---|
| Sintaxe completa do Portugol 2.0 | ✅ | 194/196 exemplos; os 2 restantes têm erro no próprio arquivo |
| Análise semântica | ✅ | casos do `AnalisadorSemanticoTest.java` portados |
| Execução (tipos, operadores, vetores, matrizes, funções, referência) | ✅ | semântica do Java gerado pelo original |
| Execução do subset Portugol-RS (`.pors`) | 🟡 | Execução normal compila o Rust gerado via `rustc`; `Depurar` agora tem backend inicial HIR→bytecode e reutiliza a VM, com breakpoints, snapshots, entrada tipada e saída. Diferencial verificado para declaração, incremento, `para`/`enquanto`, `continue`, curto-circuito, overflow, concatenação e linha de divisão por zero; snapshots respeitam escopos e shadowing. Sem toolchain é possível iniciar a sessão de debug. A VM aplica 3 s de runtime ativo/1 MiB de saída/16 MiB cumulativos de cadeias; pausas e espera por entrada são excluídas do limite temporal. Teste dedicado confirma que uma cadeia lida acima de 16 MiB encerra com limite de memória tipado. Permanecem maior cobertura diferencial e smoke de execução nativa KDE. Execução normal suporta loops, controle e `leia` para tipos primitivos, apresenta saída antes da entrada, valida o valor em português, limita execução e mapeia diagnósticos; veja `docs/reports/FEAT-PORTUGOL-RS-DEBUGGER.md`. Arquivos Rust standalone (`.rs`) têm análise `rustc` de buffer isolado (Rust 2021, 5 s/1 MiB, diagnósticos no linter), execução/cancelamento, argumentos `argv` e formatação com `rustfmt`. Arquivos `.rs` abertos no workspace com `Cargo.toml` ancestral usam `cargo build` para executar o alvo padrão ou o binário selecionado por pacote; a descoberta usa metadata offline sem scripts e mostra seletor acessível quando não há padrão inequívoco. Também oferecem `cargo check` explícito pela paleta, com confirmação de confiança, cancelamento, limite de 20 s e diagnósticos do arquivo aberto no painel Mensagens/linter. O campo Parâmetros de execução passa `argv` ao binário Cargo depois de `--`, sem shell e sob os mesmos limites por plataforma do Rust standalone. Build scripts/dependências podem acessar a rede e executar com permissões locais, sem sandbox. O seletor cobre alvos `bin`; lint contínuo e debugger Rust seguem pendentes. Evidências: `crates/portugol-service/tests/portugol_rs_debugger.rs`, `cargo_project.rs`, `rust_source_runner.rs`, `rust_source_check.rs` e `rust_format.rs`. |
| `escreva`, `leia`, `limpa`, `sorteia` | ✅ | |
| Parâmetros de execução (`inicio(cadeia params[])`) | ✅ | |
| Mensagens de erro de execução | ✅ | mesmas mensagens do original |
| Depurador: pontos de parada, passo a passo | ✅ | pausa antes de cada instrução; continue, step into/over/out e pausa manual |
| Inspetor de variáveis, vetores e matrizes | ✅ | globais, locais, pilha de chamadas e elementos expandidos |

## Bibliotecas

| Biblioteca | Funções | Estado | Observações |
|---|---|---|---|
| Matematica | 11 | ✅ | |
| Texto | 8 | ✅ | |
| Tipos | 23 | ✅ | |
| Util | 7 | ✅ | |
| Calendario | 11 + 19 constantes | ✅ | |
| Arquivos | 13 | ✅ | `selecionar_arquivo` usa o diálogo da IDE; no terminal gera erro |
| Objetos | 22 + 7 constantes | ✅ | JSON no formato do Jackson; XML como o `XmlMapper` |
| Internet | 4 | ✅ | |
| ServicosWeb | 8 + 4 constantes | ✅ | |
| Graficos | 56 | 🟡 | janela Tauri e Canvas com formas, fontes, imagens, gradientes, rotação e captura RGBA; desenho e captura RGBA verificados no Linux; falta teste interativo em Windows e macOS |
| Teclado | 4 + 135 constantes | 🟡 | códigos públicos do Java e eventos em `InputState`; verificado no Linux |
| Mouse | 7 + 3 constantes | 🟡 | eventos da janela alimentam `InputState`; verificado no Linux |
| Sons | 12 | ✅ | MP3 e WAV, reprodução simultânea de clipes, pausa, posição e volume no host da IDE |

## IDE

| Recurso (classe no original) | Estado | Observações |
|---|---|---|
| Editor com realce de sintaxe (`PSTextArea`) | ✅ | CodeMirror 6 |
| Autocompletar (`PSAutoCompletion`) | ✅ | palavras, snippets, funções de bibliotecas |
| Erros ao digitar e painel de mensagens (`AbaMensagemCompilador`) | ✅ | tabela com ícones, linha e mensagem. Faixa lateral de diagnósticos clicável |
| Abas de arquivos, abrir, salvar, salvar como | ✅ | |
| Console com entrada (`AbaConsole`) | ✅ | entrada e engrenagem para limpar, copiar e ajustar fonte |
| Depuração na IDE (`AbaDepuracao`, `Depurador`) | 🟡 | gutter, início pausado, pilha e snapshots reais para Portugol clássico e primeira fatia Portugol-RS via VM; permanece pendente ampliar a cobertura e validar a execução nativa no KDE |
| Laboratório pedagógico de ownership | 🟡 | quatro cenários guiados de `String` para move, `&T`, `&mut T` e escopo; fixtures verificadas com `rustc`; simulação editorial, não é trace Rust nem interpreta o buffer do aluno |
| Painel Testes (UI-020 / FT-06) | 🟡 | aba inferior executa os casos reais de exercícios `.pex` na revisão atual e mostra aprovação/falhas; progresso só é concluído quando todos passam. Arquivos sem exercício continuam sem runner genérico configurável. Evidência: FT-19 |
| Níveis de aprendizagem (`beginner`, `bridge`, `rust`) | ✅ | Iniciante e Ponte preservam o fluxo Portugol; Rust usa CodeMirror Rust e o executor `.rs` com `rustc` já existente, habilitado apenas quando a toolchain é detectada. O modo não converte nem modifica o arquivo ativo. O laboratório de ownership continua rotulado como simulação didática. |
| Autocompletar bibliotecas | ✅ | opções, assinaturas e descrições da mesma referência usada pela ajuda |
| Prévia Portugol ↔ Rust com navegação de origem | 🟡 | exportação `.por`/`.pors` inclui source maps; o gutter bridge apresenta correspondências reais da linha selecionada e navega ao trecho Rust, que continua podendo voltar à origem. Localizações aproximadas são identificadas; trechos gerados sem origem permanecem sem navegação. |
| Localizar e substituir (`PSFindReplace`) | ✅ | diálogo Ctrl+F com direção, palavra inteira, maiúsculas, expressões regulares e substituição em lote |
| Atalhos do original (`TelaAtalhos`) | ✅ | tela F11, atalhos de edição, depuração, arquivos, abas, filtros e centralização |
| Temas claro/escuro (`TelaEditarTemas`) | ✅ | Portugol, Dark, Sistema, seis presets inspirados em temas populares do VS Code e perfis personalizados editáveis com prévia; troca funciona offline. Referências: [ranking do Marketplace](https://www.vsxatlas.com/rankings?category=Themes&registry=marketplace) e [GitHub Theme oficial](https://marketplace.visualstudio.com/items?itemName=GitHub.github-vscode-theme) |
| Moldura e barra de comandos | ✅ | moldura de 5px, controles nativos, área de arraste, abas com ícone e barra lateral com ícones da versão 2.7.5 |
| Expandir editor (Shift+Esc) | ✅ | barra horizontal compacta e botão de restauração |
| Explorador de pasta e exportação local (`.por`, `.pors`, `.rs`, `.pex`) | ✅ | árvore navegável com teclado, abertura/salvamento escopados, criação, renomeação e exclusão; permite selecionar arquivos e copiar para uma nova pasta local sem alterar a origem |
| Tela inicial com exemplos (`AbaInicial`, `PainelExemplos`) | ✅ | Abrir/Novo Portugol, menu compacto para criar `.pors`/`.rs`, até seis recentes, 119 exemplos com busca, descrição, imagens e cópia de trabalho com recursos, acesso às dicas e links do rodapé. A paridade visual atual com o Java foi aceita; novos fluxos podem evoluir a partir da experiência de aprendizagem. |
| Ajuda da linguagem e das bibliotecas (`abaAjuda`, `assets/ajuda`) | ✅ | aba F1, busca, tópicos e diagramas da versão 2.7.5, abertura de exemplos e referências das bibliotecas |
| Árvore estrutural (variáveis e funções do programa) | ✅ | painel redimensionável com busca, engrenagem de filtros, navegação, inspetor e inclusão de bibliotecas e modelos |
| Ir à declaração (`resolve_symbol`) | ✅ | F12 no editor, Ctrl/Cmd+clique e ação no menu; Portugol clássico e `.pors` resolvem bindings por escopo, preservam o buffer e descartam respostas obsoletas |
| Renomear símbolo (`TelaRenomearSimbolo`) | ✅ | Ctrl+R, menu do editor e duplo clique na árvore; Portugol clássico e `.pors` verificam conflitos e preservam referências sombreadas |
| Dobras de código | ✅ | blocos aninhados e comentários de várias linhas, com marcadores no cabeçalho |
| Formatador de código (`editor/formatador`) | ✅ | opções do editor e Ctrl+Alt+F; Portugol usa o formatador existente e `.rs` usa rustfmt instalado (Rust 2021, limite de 5 s/1 MiB), com uma única etapa de desfazer. Portugol-RS permanece sem formatador próprio. |
| Exercícios com correção automática e progresso validado (`assets/exercicios`, módulo `checker`) | ✅ | 54 exercícios originais, arquivos `.pex` externos e seis regras estáticas opcionais. Conclusões locais só são registradas quando todos os casos reais passam e ficam vinculadas ao hash da revisão avaliada; abrir/executar não registra progresso. |
| Tamanho da fonte do editor | ✅ | ajuste de 10 a 50, salvo localmente |
| Dicas de interface (`TelaDicas`) | ✅ | 18 dicas do original, F3, setas, acesso ao editor de temas e opção de mostrar ao iniciar |
| Sobre / licenças (`Sobre`, `TelaLicencas`) | ✅ | diálogo F12, versão, origem e texto da licença |
| Verificação de atualizações (`TelaAtualizacoes`) | ✅ | consulta versões estáveis em Sobre e abre as versões oficiais para baixar o instalador |
| Relatar bug (`TelaRelatarBug`) | ✅ | diálogo em Sobre, com abertura das issues do port no navegador |
| Mostrar alertas (`TelaAlertas`) | ✅ | consulta issues públicas abertas com o rótulo `alerta` no repositório do port |
| Plugins (`plugins`, `PainelPlugins`) | ➖ | plugins do original são classes Java; exigiria outro modelo de extensão |
| Carrossel de cursos, coletor de interações | ➖ | dependem de serviços da UNIVALI |

## Comparação visual

**Critério atualizado (ADR 0015):** a paridade visual atual com o Portugol
Studio clássico está aceita. Novas mudanças de interface devem servir aos
fluxos progressivos Portugol → Portugol-RS → Rust e não precisam reproduzir a
IDE Java pixel a pixel. As capturas desta seção ficam como referência histórica
e regressão; a comparação pendente da tela inicial não bloqueia releases.

Capturas em Linux sob Xvfb. As comparações do editor usam janelas de 1200 ×
800 pixels, com a versão Java 2.7.5 à esquerda e o port em Rust à direita; a
aba de código mostra o programa inicial de cada versão.

| Tela | Dark | Portugol |
|---|---|---|
| Inicial | Fora do gate visual | Fora do gate visual |
| Editor | [Comparar editor em Dark](imagens/paridade/editor-dark.webp) | [Comparar editor em Portugol](imagens/paridade/editor-light.webp) |

As capturas `home-dark.webp` e `home-light.webp` são composições de telas
diferentes: mostram o editor Java à esquerda e a tela inicial do port à
direita. Elas não permitem avaliar a tela inicial e permanecem apenas como
artefatos históricos, sem substituir a comparação válida do editor.

Nas capturas válidas do editor, o rodapé de status, a guia Depuração e o acesso
aos Exercícios aparecem no port. A IDE Java também apresenta o acesso a Plugins
e reserva espaço aos cursos recomendados.

## Distribuição

O CI compila e executa os testes do aplicativo Tauri em Linux, Windows e
macOS. No Linux, a suíte também abre o aplicativo e verifica diretamente a
janela gráfica, o teclado e o mouse em Xvfb.

| Recurso | Estado | Observações |
|---|---|---|
| Instalador Windows | ✅ | `.exe` (NSIS) e `.msi`, assinatura configurável (docs/DISTRIBUICAO.md) |
| Instalador Linux | ✅ | `.deb`, `.rpm`, `.AppImage` |
| Instalador macOS | ✅ | `.dmg`, assinatura e notarização configuráveis |
| Associação de arquivos `.por` e `.pex` | ✅ | abertura via sistema e arraste na IDE |
| Versão de console (módulo `console`) | ✅ | `portugol executar` |
