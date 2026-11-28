# Playground: autocomplete, referência de sintaxe e terminal — design

## Intenção

Modernizar o Web Playground Ferro publicado em https://wendelmax.github.io/ferro-studio/ sem introduzir dependências nem mudar o runtime WASM. O pedido do usuário foi: (1) autocompletar com a sintaxe do Ferro, (2) criar um painel com a “API” da linguagem — esclarecido depois como **instruções e sintaxes do Ferro**, não o catálogo de bibliotecas padrão —, (3) reduzir a quantidade de botões — ações que hoje são botões podem virar item de combobox — e (4) fazer a área de execução parecer um terminal que mostra entrada e saída juntas.

## Objetivos

- Editor `.fer` com autocomplete de instruções, palavras-chave, tipos, funções embutidas de E/S, variantes e operadores da linguagem Ferro, navegável por teclado.
- Painel "Sintaxe e instruções Ferro" com busca, agrupado por categoria, mostrando forma canônica, descrição curta e exemplo; clicar em um item insere o símbolo no editor.
- Barra de controle com apenas um seletor de exemplo, um combobox de ações e um botão primário `Executar`.
- Painel de execução com cara de terminal: histórico com linhas de entrada prefixadas por `>`, linhas de saída, diagnósticos e um campo de entrada simples abaixo do histórico.

## Não-objetivos

- Referência das bibliotecas padrão (`UI.*`, `Matematica.*`, etc.). Ela continua existindo na IDE; o painel deste playground é sobre a **linguagem**.
- E/S interativa de verdade durante a execução (o programa pedir `leia()` e o usuário digitar enquanto a VM roda).
- Alterações em `crates/ferro-wasm`, no protocolo de memória linear ou em `pors_run(source, stdin)`.
- Introduzir CodeMirror, npm ou qualquer dependência JavaScript no playground.
- Refatorar a IDE desktop ou a arquitetura dos exemplos.

## Restrições

- `apps/web-playground` permanece estático e sem dependências externas; o bundle é copiado por `scripts/build-web-playground.sh`.
- Nenhum uso de `innerHTML` em arquivos do playground; toda DOM é criada com `createElement`/`textContent`, preservando a convenção existente.
- Textos da interface em pt-BR; temas claro/escuro e os breakpoints responsivos atuais permanecem.
- A referência e o autocomplete compartilham uma única lista em `apps/web-playground/ferro-language.mjs`; não há arquivo gerado nem etapa de sincronização de catálogo.
- Testes relevantes rodam com `node --test` a partir da raiz do repositório; nenhum `node_modules` é necessário para eles.

## Abordagens consideradas

1. **Autocomplete com CodeMirror no playground.** Daria recursos maduros de edição, mas quebra a restrição de zero dependências e exigiria build/bundling. Descartada.
2. **Autocomplete próprio sobre o `<textarea>` existente.** Mantém o realce atual e adiciona um `role="listbox"` posicionado abaixo do editor. Escolhida: compatível com o playground estático e suficiente para a sintaxe Ferro.
3. **Terminal interativo com streaming de stdin.** Exige mudar a fachada WASM e o protocolo `pors_run`. Descartada nesta fatia; o terminal será de histórico: o usuário preenche as entradas antes de executar e vê entrada + saída no mesmo painel.
4. **Painel gerado a partir de `pors-stdlib-catalog.ts`.** Descartada após o esclarecimento do usuário: a “API” pedida são as instruções e sintaxes da linguagem, não as funções das bibliotecas padrão.

## Decisões de implementação

- `apps/web-playground/ferro-language.mjs` é a fonte única da referência. Exporta `FERRO_SYNTAX_ENTRIES`, `wordRangeAt(source, caret)` e `completeFerro(source, caret)`.
- Cada entrada tem `{ id, label, kind, group, signature, description, example, insertText }`. Grupos: `Estrutura`, `Declarações`, `Tipos`, `Controle`, `Dados`, `Entrada e saída`, `Valores`, `Operadores`. `insertText` é o trecho inserido no editor; para palavras e variantes é igual a `label`.
- `wordRangeAt` retrocede sobre `[A-Za-z0-9_:]` para que `Resultado::O` seja um único token. `completeFerro` filtra por prefixo case-insensitive, limita a 12 resultados e retorna `[]` dentro de comentário `//` ou cadeia aberta; a varredura considera linhas anteriores porque literais Ferro podem conter `\n`.
- A referência e o autocomplete anunciam somente sintaxe aceita pelo frontend Ferro; formas como `senao_se`, `faca_enquanto`, `nulo`, `??`, `??=` e `?.` não aparecem no catálogo (ver RFC 008 e o lexer de `ferro-frontend`).
- O popup de autocomplete vive dentro de `.pg-editor-wrap`, é controlado por `ArrowDown`/`ArrowUp`/`Enter`/`Tab`/`Escape` e nunca intercepta `Ctrl+Enter`.
- O painel de referência é um card full-width abaixo da grade editor/terminal, com `<input type="search">`, lista agrupada e estado vazio; o clique insere `insertText` no caret do editor.
- Os botões `Validar`, `Formatar`, `Prévia UI` e `Ver Rust` viram opções de `<select id="pg-action">`. Escolher uma opção dispara a ação e o select volta ao placeholder; `Executar` permanece como único botão.
- O painel de resultado passa a se chamar Terminal. `#pg-terminal-history` recebe linhas `data-kind="input" | "output" | "error" | "empty"`; `#pg-stdin` vira o campo de entrada do terminal. `pors_run` continua recebendo o conteúdo completo de `#pg-stdin`.

## O que fica para quem implementar

- Detalhes visuais (espaçamento, cores e sombras) devem seguir o CSS existente; nenhuma decisão de layout além das estruturas acima é fixada.
- As descrições e exemplos curtos podem ser adaptados de `apps/desktop/src/portugol.ts`, `docs/PORTUGOL-RUST.md` e das RFCs em `docs/superpowers/specs/`; o texto exato não é parte do contrato, mas presença de `signature`, `description` e `example` é.
- O circuito de build WASM não precisa ser alterado; basta incluir `ferro-language.mjs` na cópia do bundle.
