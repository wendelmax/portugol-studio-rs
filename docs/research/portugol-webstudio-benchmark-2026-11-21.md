# Pesquisa de produto: Portugol Webstudio

**Data da captura:** 21/11/2026
**Método:** Playwright com Google Chrome, viewport 1440 × 1000, navegando a página pública e abrindo os fluxos de Sobre, novo arquivo, configurações, exercícios e criação de exercício. Não houve erros JavaScript de página nos fluxos visitados.
**Fontes:** [portugol.dev](https://portugol.dev/), [repositório Portugol-Webstudio](https://github.com/dgadelha/Portugol-Webstudio), [README e créditos](https://github.com/dgadelha/Portugol-Webstudio/blob/main/README.md), [histórico de mudanças](https://github.com/dgadelha/Portugol-Webstudio/blob/main/CHANGELOG.md).

## O que foi observado

### Entrada e navegação

- A tela inicial oferece Novo arquivo, Abrir arquivo, Abrir exemplo, Exercícios, Ajuda, Configurações e Sobre. Os links de Sugestões e Dúvidas levam às discussões do GitHub.
- Uma seção de novidades documenta o que mudou, explica atalhos e chama para a versão beta. Isso torna as evoluções visíveis sem exigir que a pessoa leia o histórico de commits.
- As abas anunciam atalhos de teclado; Delete fecha a aba e F2 renomeia.

### Editor e configurações

- O editor tem barra lateral para executar, parar, depurar, abrir, salvar, compartilhar, ajuda, extensões e sobre; o painel inferior agrupa Saída, Problemas e Variáveis.
- A barra de status apresenta posição do cursor, indentação, linguagem e estado de erros.
- Configurações ficam em uma janela organizada por Aparência, Editor, Saída, Execução e Abas. O tema segue o sistema, com opções clara e escura, prévia visual e gravação automática.
- O editor expõe sugestões contextuais com variáveis do escopo, funções de bibliotecas, documentação de parâmetros e trechos de código. Hover explica declarações, F12 navega à declaração e Ctrl/Cmd+clique oferece o mesmo caminho. Os menus do editor têm suporte de leitor de tela em português.

### Exercícios e autoria para professores

- O catálogo apresenta **75 exercícios** agrupados numa sequência de assuntos, com busca sem acentos, dificuldade e prévia do enunciado. O teclado permite selecionar e abrir.
- O exercício abre em aba própria, com enunciado e código lado a lado, exemplos de entrada e saída e botão Verificar. A correção distingue exemplos visíveis de testes escondidos, mostra o resultado de cada caso e dá pistas sem revelar a resposta secreta.
- A correção aceita comparação de texto, valores tipados ou retorno de função; interrompe casos que excedem o tempo e os que pedem/leem entradas incorretamente.
- O código e o progresso ficam salvos no navegador. Para um exercício enviado por arquivo ou link, a IDE mantém o contexto do enunciado, verifica o código e separa a cópia do aluno do progresso pessoal.
- O criador de exercícios é um assistente com cinco etapas: Exercício, Enunciado, Código, Casos e Compartilhar. O professor escreve uma solução privada; o app executa essa solução para gerar as respostas esperadas, testa cada caso e não inclui a solução no link.
- Há exemplos e testes secretos, entradas, descrições por caso, saída de texto ou comparação por valores/funções, dificuldade e tópico. O rascunho completo pode ser baixado como `.exercicio.json`; o link compartilhado expira em 180 dias e alerta que o exercício será publicado sem a solução.

### Ajuda, Sobre e créditos

- A página inicial mantém as novidades em destaque. O changelog documenta recursos e correções em linguagem voltada a quem aprende.
- A janela Sobre apresenta descrição, versão e data, estado de disponibilidade offline, busca de atualizações, pesquisa, estrelas, código-fonte, relatório de problema e histórico de atualizações.
- A janela Sobre ao vivo não identifica os autores no corpo visível. O README identifica Douglas Gadêlha, Danilo Gadêlha e o LITE/UNIVALI como contribuidores; o projeto está sob GPL-3.0.

## Arquitetura vista no repositório

O README descreve Angular, RxJS e `antlr4ng`, divididos em pacotes de gramática/lexer/parser, interface, recursos, runner, runtime/transpilador Portugol→JavaScript e worker. A execução no Webstudio acontece em Web Workers. É uma arquitetura diferente do nosso núcleo Rust: lexer, parser recursivo, AST com spans, análise semântica, compilador de bytecode, VM e depurador próprio.

O [README do projeto](https://github.com/dgadelha/Portugol-Webstudio#caracter%C3%ADsticas) e o [guia de autoria](https://github.com/dgadelha/Portugol-Webstudio/blob/main/packages/resources/ajuda/topicos/portugol_webstudio/criando_exercicios.md) são referências de comportamento e desenho de produto. O código do Webstudio usa GPL-3.0; não transportar código ou recursos dele para a base LGPL-3.0-only sem uma revisão de compatibilidade de licença. A implementação da ponte Portugol–Rust deve usar os nossos crates.

## Oportunidades para este projeto

| Prioridade | Observação | Situação nesta IDE Rust | Evolução recomendada |
| --- | --- | --- | --- |
| P0 | Um Sobre maduro explica autoria, versão, licença e origem. | A janela atual informa base UNIVALI, licença, versão, atualização e bugs. | Completar créditos dos autores e fontes de assets/documentação; incluir links para código, releases, licença e histórico. |
| P1 | Há 75 exercícios e criação de novos enunciados e casos. | Existem 54 exercícios originais, avaliação dinâmica/estática e importação `.pex`. | Investigar os 21 casos de diferença no catálogo e planejar autorias reutilizando o formato `.pex` ou uma extensão compatível. |
| P1 | Testes secretos e geração de resposta pela solução do professor viabilizam tarefas corrigíveis. | O verificador da VM já aceita entradas, saídas e chamadas de funções. | Especificar um criador offline `.pex`, com solução privada, entradas de exemplo/teste e geração verificável de resultados. |
| P1 | O link para exercício tem prazo e não expõe solução; progresso é separado do código entregue. | A IDE é local, pode abrir `.pex`/`.por` e corrigir exercícios, sem serviço de links. | Priorizar pacote offline e arquivos compartilháveis. Só adicionar publicação por link após decidir hospedagem, retenção e aviso de privacidade. |
| P1 | Hover e F12 tornam código existente explorável. | Símbolos semânticos, escopos, navegação estrutural e renomeação por símbolo já existem. | Reaproveitar esses dados para tooltip de declaração/documentação e navegar ao `Span` da declaração pelo F12 e Ctrl/Cmd+clique. |
| P2 | O changelog comunica as mudanças para alunos e docentes. | Há matriz de paridade e releases, sem um feed de novidades pedagógico. | Exibir novidades da release e links de feedback no Sobre, sem telemetria obrigatória. |
| P2 | Acessibilidade aparece no autocomplete, execução, diagnósticos e painel de correção. | Há atalhos, rótulos e mensagens, com verificações pontuais. | Adotar uma revisão com leitor de tela e teclado para menus, sugestões, debug, exercícios e diálogos antes de afirmar paridade. |
| P2 | O Webstudio executa em navegadores e torna explícitos exemplos e depuração. | O produto-alvo é uma IDE desktop, com VM nativa e depuração. | Manter foco em uso desktop; tratar versão web como uma distribuição futura que reutiliza os crates Rust, não como um segundo interpretador. |

## Decisões de produto a manter explícitas

- O progresso e os exercícios privados devem continuar locais por padrão.
- Um recurso de compartilhamento precisa declarar o que sai do computador, quem pode ler, quanto tempo fica disponível e como revogar/remover.
- Links compartilhados de exercício nunca devem incluir a solução do docente.
- Usar nomes de autores e laboratórios com fontes e licença atribuídas; inspiração de fluxo não significa reutilização de código ou imagens.
- O Portugol clássico continua compatível. Ferro será uma opção pedagógica, não uma mudança silenciosa de sintaxe.
