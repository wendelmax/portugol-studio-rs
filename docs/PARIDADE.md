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
| Graficos | 56 | 🟡 | na VM: formas, texto, imagens, gradiente, rotação e quadros; falta a janela na IDE |
| Teclado | 5 | 🟡 | na VM via `InputState`; eventos dependem da janela |
| Mouse | 8 | 🟡 | na VM via `InputState`; eventos dependem da janela |
| Sons | 17 | ⬜ | |

## IDE

| Recurso (classe no original) | Estado | Observações |
|---|---|---|
| Editor com realce de sintaxe (`PSTextArea`) | ✅ | CodeMirror 6 |
| Autocompletar (`PSAutoCompletion`) | ✅ | palavras, snippets, funções de bibliotecas |
| Erros ao digitar e painel de mensagens (`AbaMensagemCompilador`) | ✅ | |
| Abas de arquivos, abrir, salvar, salvar como | ✅ | |
| Console com entrada (`AbaConsole`) | ✅ | |
| Depuração na IDE (`AbaDepuracao`, `Depurador`) | ✅ | gutter de breakpoints, barra de comandos, pilha e variáveis |
| Localizar e substituir (`PSFindReplace`) | ✅ | Ctrl+F do CodeMirror |
| Atalhos do original (`TelaAtalhos`) | ✅ | |
| Temas claro/escuro (`TelaEditarTemas`) | 🟡 | claro, escuro e do sistema; sem editor de temas |
| Expandir editor (Shift+Esc) | ✅ | |
| Tela inicial com exemplos (`AbaInicial`, `PainelExemplos`) | ⬜ | |
| Ajuda da linguagem e das bibliotecas (`abaAjuda`, `assets/ajuda`) | ⬜ | |
| Árvore estrutural (variáveis e funções do programa) | ⬜ | |
| Renomear símbolo (`TelaRenomearSimbolo`) | ⬜ | |
| Formatador de código (`editor/formatador`) | ⬜ | |
| Exercícios com correção automática (`assets/exercicios`, módulo `checker`) | ⬜ | |
| Tamanho da fonte do editor | ⬜ | |
| Dicas de interface (`TelaDicas`) | ⬜ | |
| Sobre / licenças (`Sobre`, `TelaLicencas`) | ⬜ | |
| Verificação de atualizações (`TelaAtualizacoes`) | ⬜ | updater do Tauri |
| Relatar bug (`TelaRelatarBug`) | ⬜ | link para as issues do GitHub |
| Plugins (`plugins`, `PainelPlugins`) | ➖ | plugins do original são classes Java; exigiria outro modelo de extensão |
| Carrossel de cursos, coletor de interações | ➖ | dependem de serviços da UNIVALI |

## Distribuição

| Recurso | Estado | Observações |
|---|---|---|
| Instalador Windows | ✅ | `.exe` (NSIS) e `.msi`, assinatura configurável (docs/DISTRIBUICAO.md) |
| Instalador Linux | ✅ | `.deb`, `.rpm`, `.AppImage` |
| Instalador macOS | ✅ | `.dmg`, assinatura e notarização configuráveis |
| Associação de arquivos `.por` | ✅ | |
| Versão de console (módulo `console`) | ✅ | `portugol executar` |
