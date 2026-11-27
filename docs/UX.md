# Experiência de uso da IDE

Este documento define os princípios de interação, a organização atual da IDE e
os critérios para evoluir suas telas. É uma especificação do produto; planos de
implementação, auditorias e relatórios de agentes não são contratos de UX.

## Direção do produto

O Portugol Studio é uma aplicação desktop Rust + Tauri para aprender lógica,
Portugol-RS e Rust real. Portugol-RS (`.pors`) é a linguagem autoral e o caminho
principal de aprendizagem; arquivos Rust (`.rs`) permitem avançar para a
linguagem e a toolchain reais. A interface deve preservar familiaridade útil
com uma IDE educacional e evoluir em função desses fluxos, sem exigir paridade
pixel a pixel com o Portugol Studio clássico.

O suporte a Portugol clássico (`.por`/`.pex`) foi retirado na Fase 6 do
[ADR 0016](adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md). Quando o
usuário tenta abrir um arquivo `.por` ou `.pex`, a interface e a CLI exibem uma
orientação clara em português indicando o Portugol Studio original — sem
converter nem modificar o arquivo do usuário.

## Princípios de experiência

1. **Execução verdadeira.** A IDE só apresenta como saída de execução o
   resultado devolvido pelo runtime para a revisão atual do arquivo. Exemplos,
   resultados ilustrativos e simulações são identificados como tais.
2. **Origem sempre visível.** O aluno consegue distinguir código escrito por
   ele, Rust gerado a partir da revisão atual, exemplos editoriais e estados
   simulados. Uma tradução pendente ou desatualizada não é apresentada como
   correspondência atual.
3. **O modo de aprendizagem não muda o programa.** Iniciante, ponte e Rust
   alteram explicações e conceitos exibidos. Não trocam o dialeto do arquivo,
   não reescrevem seu conteúdo e não convertem `.pors`, `.rs` ou `.por`.
   Portugol-RS continua sendo a linguagem de entrada do percurso inicial; o
   modo Rust orienta a abrir ou criar um arquivo `.rs` sem modificar o arquivo
   ativo.
4. **Progresso exige evidência.** Abrir conteúdo, ler uma explicação ou iniciar
   um exercício não conclui uma etapa. O progresso só muda quando a solução
   revisada passa em todos os testes definidos para o exercício.
5. **Simulações são didáticas, não telemetria.** O laboratório de ownership
   explica cenários pré-definidos. Deve informar que não executa o código do
   aluno, não inspeciona memória real e não produz um trace Rust.
6. **A ação seguinte é clara e reversível.** Salvar, executar, depurar,
   interromper e exportar têm estados e resultados explícitos. Comparar ou
   visualizar código gerado não altera o fonte do aluno.
7. **A linguagem da interface é direta.** Diagnósticos em português informam o
   problema, sua localização e, quando disponível, uma ação para corrigi-lo.
   Recursos ainda não suportados são descritos sem sugerir compatibilidade que
   não existe.
8. **Acessibilidade faz parte do comportamento.** Funcionalidades essenciais
   são operáveis por teclado, têm foco visível, nomes acessíveis e estados
   anunciados; cor nunca é o único indicador de erro, sucesso ou seleção.

## Organização atual da IDE

### Início

- Criar Portugol-RS (`.pors`) é a ação principal; Rust (`.rs`) também pode ser
  criado pelo menu de linguagens.
- Abrir arquivo ou pasta inicia trabalho existente em `.pors`, `.rs` ou
  `.pors.json`; arquivos recentes e o catálogo 100% `.pors` de exemplos ajudam
  a retomar ou explorar programas.
- Exercícios `.pors.json`, Ajuda PORS, Dicas de interface, Atalhos, Alertas e
  Sobre ficam acessíveis a partir da tela inicial.

### Espaço de trabalho

- **Barra superior (`TitleBar`):** identidade proprietária `RS`, seletor do projeto aberto, seletor rápido de modo de aprendizagem (`Iniciante`, `Ponte para Rust`, `Rust`), ações primárias (`Executar`, `Depurar`, `Parar`, `Rust`, `UI`) e controles da janela.
- **Barra de atividades (`ActivityRail`):** Aprender, Exemplos, Depurar e Ajustes (abre o diálogo dedicado de `Configurações`, `Ctrl+,`, com Tema, Fonte, Modo, Parâmetros e `Restaurar layout padrão`), além de atalhos rápidos de Explorador, Salvar, Abrir e Ajuda.
- **Explorador e abas:** navegação pela pasta aberta e pelos documentos em
  edição. O conteúdo e a extensão da aba ativa determinam a linguagem do
  documento.
- **Editor e comandos:** edição central, salvar, executar, depurar, interromper,
  abrir ajuda, inspecionar a tradução Rust e abrir a prévia visual PORS quando
  aplicável.
- **Painel inferior:** Console, Testes, Mensagens e Depuração. Diagnósticos
  apontam para o código; o depurador apresenta estado real da VM para
  linguagens suportadas.
- **Inspector (`Aprender` e utilitários PORS):** explicações contextuais mapeadas por source map (mutabilidade, decisões, laços, funções, `Opcao`/`Resultado`, vetores, registros/enums), prática vinculada diretamente a exercícios `.pors`, 13 módulos PORS e 11 modelos `.pors`, estrutura do programa e inspetor de variáveis.
- **Prévia Rust:** mostra Rust gerado, estado da tradução, mapeamentos de
  origem e diferenças. O usuário pode copiar, salvar ou desvincular para editar
  um arquivo `.rs` independente.
- **Prévia Visual PORS:** executa uma prévia interativa dos programas visuais
  suportados; seus estados, limitações e ações de cancelar/reiniciar são
  explícitos.
- **Paletas de cores padronizadas:** `Sistema` como padrão (alternando entre VS Code Light Modern e VS Code Dark Modern em `tokens.css`), `IntelliJ Rust Dark/Light`, temas populares do VS Code e `Alto contraste`, todos verificados em WCAG AA (`>= 4,5:1`) — ver [`docs/next-ui/design/COLOR_THEMES.md`](next-ui/design/COLOR_THEMES.md).

### Arquivos e execução

| Documento | Comportamento apresentado ao usuário |
| --- | --- |
| `.pors` | Linguagem principal; análise, execução e depuração na VM PORS, além de prévia/exportação Rust. Executar o fonte não exige `rustc`. |
| `.rs` | Código Rust real. Análise, formatação, compilação e execução dependem das ferramentas Rust correspondentes instaladas; a IDE informa quando uma capacidade não está disponível. |
| `.pors.json` | Exercício estruturado Portugol-RS avaliado no serviço local com casos de teste reais. |
| `.por` / `.pex` | Não suportados (Fase 6 / ADR 0016). A IDE e a CLI rejeitam a abertura com aviso claro orientando o usuário ao Portugol Studio original, sem alterar nem converter o arquivo. |

O Rust gerado é um artefato de tradução do Portugol-RS, não uma promessa de
compatibilidade com toda a linguagem Rust. A prévia deve corresponder à revisão
atual e indicar quando está sendo recalculada ou ficou desatualizada. Uma
execução real e a simulação didática de ownership nunca compartilham o mesmo
rótulo de proveniência.

## Critérios de aceite para telas e fluxos

Uma tela ou mudança de interação está pronta para uso quando:

- explica o que o usuário está vendo e qual ação pode tomar, incluindo estados
  vazio, carregando, sucesso, erro, indisponível e desatualizado quando se
  aplicarem;
- não inventa saída, execução, análise, progresso ou correspondência entre
  códigos;
- distingue fonte do usuário, conteúdo editorial, Rust gerado e simulação;
- preserva o arquivo e a revisão do usuário em mudanças de modo, comparação,
  prévia, exemplos e painéis;
- erros e avisos apontam para uma posição navegável no documento sempre que
  houver localização disponível, e são explicados em texto;
- em larguras compactas, painéis podem ser recolhidos ou alternados sem perder
  acesso ao fluxo principal; comparações lado a lado permanecem legíveis ou
  oferecem uma alternativa por abas;
- estados de carregamento e operações longas permitem entender o andamento e,
  quando canceláveis, interromper a operação.

## Critérios de acessibilidade

- Todo fluxo principal funciona por teclado, com ordem de foco previsível,
  foco visível, atalhos descobríveis e nenhuma armadilha de foco.
- Controles têm nome, papel e estado acessíveis; diálogos têm título e o foco
  retorna ao controle de origem ao fechar.
- Diagnósticos, resultados de testes, execução e mudanças de estado são
  anunciados sem exigir que o usuário perceba apenas uma mudança visual.
- Contraste atende WCAG AA: pelo menos 4,5:1 para texto comum e 3:1 para texto
  grande e elementos visuais necessários para identificar controles e estados.
- Erro, sucesso, estado ativo e seleção não dependem apenas de cor; há rótulo,
  ícone, texto ou forma complementar.
- A interface respeita `prefers-reduced-motion` e continua utilizável em
  `forced-colors`/alto contraste.
- Temas claros, escuros e de alto contraste preservam a legibilidade de código,
  diagnósticos, foco e seleções.

Esses são critérios de projeto, não uma declaração de conformidade já medida.
Testes automatizados de contraste e uma verificação manual por teclado devem
acompanhar a implementação dos tokens e temas.

## Evolução rastreável

As issues abaixo registram entregas e próximas mudanças alinhadas a esta
experiência:

| Issue | Escopo | Estado |
| --- | --- | --- |
| [#132](https://github.com/wendelmax/portugol-studio-rs/issues/132) | Executar `.pors` na VM sem exigir Rust instalado | Entregue |
| [#133](https://github.com/wendelmax/portugol-studio-rs/issues/133) | Barra de status com linguagem, modo e diagnósticos | Aberta |
| [#134](https://github.com/wendelmax/portugol-studio-rs/issues/134) | Condicionais encadeadas `senao se` | Entregue |
| [#135](https://github.com/wendelmax/portugol-studio-rs/issues/135) | Recuperação de erros no parser | Aberta |
| [#136](https://github.com/wendelmax/portugol-studio-rs/issues/136) | Tokens, contraste, movimento reduzido e alto contraste | Aberta; depende de #140 |
| [#137](https://github.com/wendelmax/portugol-studio-rs/issues/137) | Trilha visível de aprendizagem | Aberta |
| [#138](https://github.com/wendelmax/portugol-studio-rs/issues/138) | Comparação lado a lado com Rust gerado | Aberta; depende de #139 |
| [#139](https://github.com/wendelmax/portugol-studio-rs/issues/139) | Modularização do shell da IDE | Aberta/em andamento |
| [#140](https://github.com/wendelmax/portugol-studio-rs/issues/140) | Separação de estilos e tokens | Aberta/em andamento |

As decisões de produto relacionadas são os ADRs
[0015](adr/0015-evolucao-visual-orientada-a-aprendizagem.md),
[0016](adr/0016-pors-como-linguagem-autoral-e-retirada-do-por.md),
[0017](adr/0017-modelo-execucao-e-estado-ui-visual-pors.md),
[0018](adr/0018-projetos-desktop-pors-com-designer-visual.md),
[0019](adr/0019-contrato-minimo-de-tipos-pors.md) e
[0020](adr/0020-ajustes-pedagogicos-da-sintaxe-pors.md). Eles definem,
respectivamente, a direção visual, a evolução de `.por` para PORS, o modelo
para programas visuais PORS, os projetos desktop com designer visual, o
contrato mínimo de tipos e os ajustes pedagógicos da sintaxe.
