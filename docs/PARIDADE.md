# Paridade com o Portugol Studio (Java)

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
| Depuração na IDE (`AbaDepuracao`, `Depurador`) | ✅ | gutter de breakpoints, início pausado com Shift+F5, pilha e variáveis |
| Laboratório pedagógico de ownership | 🟡 | quatro cenários guiados de `String` para move, `&T`, `&mut T` e escopo; fixtures verificadas com `rustc`; simulação editorial, não é trace Rust nem interpreta o buffer do aluno |
| Autocompletar bibliotecas | ✅ | opções, assinaturas e descrições da mesma referência usada pela ajuda |
| Localizar e substituir (`PSFindReplace`) | ✅ | diálogo Ctrl+F com direção, palavra inteira, maiúsculas, expressões regulares e substituição em lote |
| Atalhos do original (`TelaAtalhos`) | ✅ | tela F11, atalhos de edição, depuração, arquivos, abas, filtros e centralização |
| Temas claro/escuro (`TelaEditarTemas`) | ✅ | Portugol, Dark, Sistema, seis presets inspirados em temas populares do VS Code e perfis personalizados editáveis com prévia; troca funciona offline. Referências: [ranking do Marketplace](https://www.vsxatlas.com/rankings?category=Themes&registry=marketplace) e [GitHub Theme oficial](https://marketplace.visualstudio.com/items?itemName=GitHub.github-vscode-theme) |
| Moldura e barra de comandos | ✅ | moldura de 5px, controles nativos, área de arraste, abas com ícone e barra lateral com ícones da versão 2.7.5 |
| Expandir editor (Shift+Esc) | ✅ | barra horizontal compacta e botão de restauração |
| Explorador de pasta para projetos (`.por`, `.pors`, `.pex`) | 🟡 | árvore navegável com teclado, abertura e salvamento escopado; cria arquivos `.por` e `.pors` na raiz com validação e sem sobrescrever; renomear, excluir e restaurar workspace ainda pendentes |
| Tela inicial com exemplos (`AbaInicial`, `PainelExemplos`) | 🟡 | Abrir/Novo, até seis recentes, 119 exemplos com busca, descrição, imagens e cópia de trabalho com recursos, acesso às dicas e links do rodapé |
| Ajuda da linguagem e das bibliotecas (`abaAjuda`, `assets/ajuda`) | ✅ | aba F1, busca, tópicos e diagramas da versão 2.7.5, abertura de exemplos e referências das bibliotecas |
| Árvore estrutural (variáveis e funções do programa) | ✅ | painel redimensionável com busca, engrenagem de filtros, navegação, inspetor e inclusão de bibliotecas e modelos |
| Ir à declaração (`resolve_symbol`) | ✅ | F12 no editor, Ctrl/Cmd+clique e ação no menu; resolução semântica respeita escopos, preserva o buffer e descarta respostas obsoletas |
| Renomear símbolo (`TelaRenomearSimbolo`) | ✅ | Ctrl+R, menu do editor e duplo clique na árvore; análise de escopo e verificação de conflitos |
| Dobras de código | ✅ | blocos aninhados e comentários de várias linhas, com marcadores no cabeçalho |
| Formatador de código (`editor/formatador`) | ✅ | opções do editor e Ctrl+Alt+F, com alterações de espaço e desfazer em uma etapa |
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

Capturas em Linux sob Xvfb. As comparações do editor usam janelas de 1200 ×
800 pixels, com a versão Java 2.7.5 à esquerda e o port em Rust à direita; a
aba de código mostra o programa inicial de cada versão.

| Tela | Dark | Portugol |
|---|---|---|
| Inicial | ⬜ Comparação pendente | ⬜ Comparação pendente |
| Editor | [Comparar editor em Dark](imagens/paridade/editor-dark.webp) | [Comparar editor em Portugol](imagens/paridade/editor-light.webp) |

As capturas `home-dark.webp` e `home-light.webp` são composições de telas
diferentes: mostram o editor Java à esquerda e a tela inicial do port à
direita. Elas não permitem avaliar a paridade da tela inicial e, por isso,
não são apresentadas como comparação. Devem ser substituídas por capturas da
tela inicial nas duas aplicações, com o mesmo tema e dimensões, antes de
marcar essa linha como validada.

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
