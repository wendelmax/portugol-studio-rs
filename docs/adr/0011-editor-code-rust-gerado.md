# ADR 0011 — Exibir Rust gerado em um editor CodeMirror somente leitura

- **Status:** aceita
- **Data:** 2026-11-22

## Contexto

A Ponte já gera Rust, mantém estado stale por revisão, apresenta source maps e oferece ações de copiar/salvar/desvincular. A interface usa um `<pre>` com segmentos DOM clicáveis; não oferece realce Rust nem comportamento de editor para a saída.

## Decisão

Montar um segundo `EditorView` CodeMirror na prévia existente, com linguagem Rust, números de linha, tema compartilhado e edição desabilitada. Decorar os intervalos do source map e encaminhar clique ou F12 na seleção ativa para os offsets correspondentes na fonte. O atalho F12 é anunciado por `aria-keyshortcuts`. Ações de exportação leem o documento puro do editor gerado.

## Consequências

- A comparação Ferro–Rust usa a linguagem Rust existente e mantém navegação semântica por source map.
- O editor gerado não cria um arquivo autoral até que o aluno escolha desvincular.
- A prévia mantém o layout e os controles atuais; não há nova dependência nem novo backend.
