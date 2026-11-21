# Ir à declaração — Design

## Intenção

Deixar quem lê um programa navegar de qualquer uso de variável, parâmetro, constante ou função até a declaração que o analisador já associou àquele uso. Isso reduz atrito em programas maiores e reforça o modelo real de escopos do Portugol.

## Estado atual

- `portugol-service::rename::bindings` usa `symbol_bindings` para mapear índices de tokens de uso para os tokens de declaração, respeitando resolução semântica e escopos.
- `rename_target` e `rename_symbol` usam esse mapa, mas não expõem a declaração resolvida.
- `SymbolInfo.position` alimenta a árvore estrutural, cujo clique navega apenas para declarações já listadas.
- O editor usa CodeMirror 6. F12 abre Sobre por um listener global; há menu editorial para renomear símbolo. Não existe navegação de uma referência até sua declaração.

## Objetivos

- Expor a localização da declaração semanticamente resolvida sob o cursor.
- Ir para a declaração com F12 quando o editor está focado e com Ctrl/Cmd+clique no identificador.
- Selecionar a declaração, centralizá-la no editor e preservar o texto e o histórico de edição.
- Manter o F12 global para Sobre quando o foco não está no editor.
- Rejeitar sintaxe/semântica inválida com mensagem existente do serviço; ignorar resposta se aba/documento mudarem durante a chamada.
- Testar referências locais, nomes sombreados em escopos distintos, funções e offsets UTF-16.

## Fora de escopo

- Abrir arquivos distintos, navegar para bibliotecas externas, referências cruzadas entre módulos ou busca de usos.
- Alterar o parser, o compilador, a VM, o conteúdo do buffer ou atalhos globais do sistema.
- Substituir a árvore estrutural ou o fluxo existente de renomeação.

## Abordagens consideradas

1. Procurar o mesmo texto no frontend e escolher a ocorrência mais próxima. É simples, mas falha com shadowing, parâmetros e escopos; rejeitada.
2. Expor do serviço `portugol-service` o token de declaração já encontrado por `symbol_bindings`. Recomendada: reaproveita resolução testada pela renomeação, preserva arquitetura e fornece offset Unicode correto para o CodeMirror.
3. Adicionar um resolvedor de navegação independente à análise semântica. Não é necessário: duplicaria regras e aumentaria o risco de divergir da renomeação.

## Comportamento proposto

- `resolve_symbol(source, offset)` retorna início/fim (offset UTF-16) e linha/coluna da declaração para o identificador associado ao cursor.
- Sem símbolo sob o cursor ou com erro semântico, o serviço retorna erro pt-BR; o editor apresenta o texto na barra de status.
- F12 usa o serviço somente quando a view do CodeMirror tem foco. O F12 atual de Sobre permanece válido fora do editor.
- Ctrl/Cmd+clique navega apenas quando o ponto contém um identificador resolvido; se não houver resultado, mantém a seleção/clique normal sem alterar conteúdo.
- Após a resposta, aplicar seleção e `scrollIntoView`; descartar resultado se a aba ou o estado do documento não forem mais os mesmos que iniciaram a consulta.

## Arquivos previstos

- `crates/portugol-service/src/rename.rs`: tipo e resolução de localização usando os bindings atuais; testes unitários.
- `crates/portugol-service/src/lib.rs`: exportar tipo e função.
- `apps/desktop/src-tauri/src/lib.rs`: comando Tauri tipado e registro.
- `apps/desktop/src/api.ts`: contrato IPC.
- `apps/desktop/src/main.ts`: navegação do editor, F12 contextual, Ctrl/Cmd+clique, atualização da lista de atalhos.
- `apps/desktop/tests/definition-navigation.test.mjs`: contrato de interação/atalho não DOM, se necessário; testes de serviço são a autoridade para escopos.
- `docs/PARIDADE.md`, plano e relatório da feature.

## Critérios de aceite

- Uso de identificador vai à declaração correta, inclusive com símbolos de mesmo nome em escopos distintos.
- Declaração destino fica selecionada/visível e o texto do programa não muda.
- F12 no editor navega; F12 fora dele ainda abre Sobre.
- Ctrl/Cmd+clique navega apenas para um símbolo real e não quebra seleção normal nos demais pontos.
- Resposta atrasada não navega uma aba/documento diferente.
- `cargo test -p portugol-service`, frontend tests/build e quality gates disponíveis passam.
- Nenhuma mudança em semântica de programas, execução, arquivos, IPC não relacionado ou permissões.

## Rollback

Remover o comando/resolvedor, ação de editor e testes desta feature; árvore estrutural e renomeação atuais permanecem funcionais.
