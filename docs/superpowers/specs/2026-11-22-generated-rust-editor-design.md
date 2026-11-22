# Editor Rust gerado — design

## Objetivo

Transformar a prévia Rust da Ponte em um segundo editor CodeMirror real, com sintaxe Rust, linhas numeradas e leitura/seleção acessíveis. A fonte permanece editável; o texto gerado continua somente leitura e vinculado à tradução atual.

## Abordagens

1. **Manter o `<pre>` e apenas aprimorar CSS:** barato, mas não entrega highlight Rust, semântica de editor nem navegação/foco robustos.
2. **Editor CodeMirror readonly montado diretamente em `main.ts` (escolhida):** reutiliza dependências, temas, linguagem e layout existentes; mudanças pequenas sobre a prévia atual.
3. **Reestruturar todos os editores num novo framework de UI:** troca ampla e risco desnecessário para um incremento vertical; contraria a arquitetura preservada.

## Comportamento

- Na Ponte, mostrar a fonte Portugol e o editor CodeMirror Rust gerado lado a lado; o layout responsivo já existente mantém a prévia como painel à direita.
- O editor gerado não aceita edição (CodeMirror `EditorState.readOnly` e `EditorView.editable = false`) e expõe rótulo acessível que diz que é código gerado/somente leitura.
- O editor mostra números de linha, highlight Rust e o tema atual. A troca de tema atualiza os dois editores.
- Segmentos de source map ficam marcados visualmente; clique em segmento mapeado navega até o offset correspondente na fonte. Links do gutter focam a linha gerada no CodeMirror.
- Atualização, estado stale, mensagens, copiar, salvar `.rs` e desvincular continuam vinculados ao artefato/revisão já aceitos. A fonte original nunca é alterada pela tradução.
- Texto consumido por copiar/salvar/desvincular vem de `generatedRustEditor.state.doc`, sem números de linha ou marcação HTML.

## Não objetivos

- Não editar Rust gerado inline, adicionar diff editor, debounce novo, sincronização automática de scroll ou alterar a semântica do gerador/source maps.
- Não separar comandos de runtime nem mudar extensões de arquivo.

## Restrições e validação

- Manter Rust gerado como subset educacional semanticamente mapeado.
- Testar estado read-only, documento, segmentos, navegação de offset e cópia do texto puro; rodar suíte Node, build e CI Tauri.
- E2E real com DOM/Tauri continua fora do escopo porque o repositório não possui runner de UI.
