# Design: evolução sintática inicial do FERRO

**Data:** 2026-11-26
**Status: aprovado**
**Relacionados:** [ADR 0001](../../adr/0001-webstudio-benchmark-e-ponte-progressiva-rust.md), [ADR 0002](../../adr/0002-hir-compartilhada-ferro.md), [ADR 0016](../../adr/0016-ferro-como-linguagem-autoral-e-retirada-do-por.md), [matriz de migração](../../research/ferro_MIGRATION_MATRIX.md)
**Revisado por:** [ADR 0020](../../adr/0020-ajustes-pedagogicos-da-sintaxe-ferro.md)

> **Revisão (ADR 0020):** no material didático, o exemplo motivador usa `retorne "Olá, {nome}!";`; o retorno implícito fica na trilha de ponte para Rust. A interpolação passa a aceitar `inteiro`, `real` e `logico`, e `leia()` converte para o tipo de um destino declarado.

> **Revisão de 2026-11-27:** o contrato original deste design congelava `leia()` como `cadeia` fixa. Ele foi superado pela revisão da ADR 0019 e pela RFC 002: `leia()` agora infere o tipo esperado pelo contexto e a conversão implícita vale também para a forma com variável-alvo. As seções históricas abaixo são mantidas como registro do planejamento, com os pontos superados sinalizados.

## Intenção

O aluno deve conseguir escrever o programa motivador com menos ruído operacional:

```ferro
funcao saudar(nome: cadeia) -> cadeia {
    "Olá, {nome}!"
}

funcao inicio() {
    seja nome = leia();
    escreval(saudar(nome));
}
```

e também a forma sem função auxiliar:

```ferro
funcao inicio() {
    seja nome = leia();
    escreval("Olá, {nome}!");
}
```

Hoje nenhuma das duas formas compila: `seja nome = leia();` exige `: tipo`, `leia()` só existe como comando com variável mutável, `escreval` não existe, `{nome}` é texto literal e a última expressão de um bloco não é retorno.

O objetivo é evoluir o núcleo da linguagem mantendo a ponte para Rust e a compatibilidade com o código FERRO já publicado. A semântica nova vive no frontend, na HIR e nos backends; a IDE e o playground entram no escopo apenas para expor a mesma sintaxe (RFC 006: realce, completions e ajuda), sem alterar suas arquiteturas.

## Por que agora

- É a menor fatia que entrega o exemplo discutido de ponta a ponta.
- As quatro lacunas são independentes da largura de inteiros e do modelo de memória; não exigem GC nem novos tipos numéricos.
- O projeto já tem HIR, bytecode, VM stack-based, I/O abstraído, dois backends (VM e Rust gerado) e WASM. O risco está em mudar contratos compartilhados sem especificação executável.
- Manter o exemplo atual (forma com variável-alvo de `leia`, `escreva`, concatenação com `+`) funcionando é obrigatório; ele está em documentação, testes e no Web Playground.

## Objetivos

1. Publicar um **ADR mínimo de contrato de tipos** que registre explicitamente o que fica congelado nesta fatia e o que fica adiado.
2. Escrever RFCs datadas em `docs/superpowers/specs/` para:
   - inferência de tipo em `seja nome = expressao;`;
   - `leia()` como expressão com conversão implícita ao tipo esperado (`cadeia` sem contexto);
   - `escreval` com quebra de linha implícita;
   - interpolação de cadeias com partes `Literal`/`Expressão` na HIR;
   - retorno implícito com expressão final de bloco;
   - contrato de realce de sintaxe para IDE e playground.
3. Definir um contrato único de realce de sintaxe para IDE e playground, cobrindo `escreval`, `leia()` e as partes de interpolação.
4. Criar testes de aceite que falhem antes da implementação e rodem o mesmo programa em:
   - frontend + HIR;
   - bytecode + VM (`compile_ferro`);
   - Rust gerado compilado com `rustc`;
   - fachada WASM quando a API pública participar da fatia;
   - editores (CodeMirror na IDE e camada de realce no playground).
5. Implementar a fatia sem alterar `inteiro`, overflow, heap/GC, nomes de tipos já publicados nem o dialeto legado.
6. Evoluir o debugger atual com mapeamento HIR → bytecode → Rust, sem criar outra VM.

## Não objetivos

- `leia<inteiro>()`, `leia<real>()`, `leia<logico>()` e conversão/validação tipada de entrada: **P1**.
- `inteiro8..128`, `natural*`, `real32`, `byte`, `vazio`, `nunca`, aliases e decisão de overflow: ADR posterior.
- `traco`, `implemente`, `self`, `assincrono`, `aguarde` como palavra reservada, mapas, conjuntos, filas, pilhas, intervalos e closures.
- Substituir `Rc<RefCell>`/orçamento atual por GC mark-and-sweep.
- Criar pacotes `ferro-*`, renomear produto/crates ou abrir uma pasta `specs/` paralela.
- Reabrir o PR #246 (redesign do Web Playground, já mesclado) ou misturar este trabalho com o PR #248 (correção do botão Prévia UI).

## Anti-objetivos

- Tratar `"Olá, {nome}"` como substituição textual antes de montar a HIR: perde spans, formatação, rename e diagnósticos.
- Adicionar `escreval` como novo opcode sem necessidade: isso duplicaria `Write` e enfraqueceria a equivalência VM/Rust.
- Quebrar a forma com variável-alvo de `leia`: a forma antiga continua válida e testada.
- Prometer tradução Rust “bonita” como se fosse o código compilável real. O painel educacional deve separar **Rust compilável** de **equivalência conceitual resumida**.

## Restrições verificadas

- `Type`, `Binding`, `StmtKind` e `ExprKind` vivem em `crates/ferro-hir/src/lib.rs`; spans são offsets UTF-8 semiabertos.
- `declaration()` exige `:` em `crates/ferro-rust-frontend/src/parser/stmt.rs`; `leia` é somente comando em `stmt.rs`.
- O lexer devolve `String(String)` já decodificada; `{`/`}` dentro de cadeia são literais.
- `Op::Read(ValueType)` já empilha valor na VM; `escreva` gera `Write` sem newline; `Op::Clear` existe mas o compilador FERRO não o emite.
- `inteiro` é `i32` em HIR/VM/Rust gerado, com `wrapping_add` e `__ferro_index(index: i32, ...)`.
- A VM já é stack-based e o debugger já expõe instruções, pilha e locais.
- Baseline do novo worktree: 75 testes passando nos crates `ferro-hir`, `ferro-bytecode` e `ferro-rust-frontend`.

## Abordagens

1. **Nova árvore `ferro/` com VM, parser, CLI e specs do zero.** Rejeitada: duplica HIR, bytecode, debugger, WASM e Rust gerado; contraria ADR 0002/0016 e reinicia a migração já concluída.
2. **ADR completo de tipos antes de qualquer sintaxe.** Rejeitada para esta fatia: largura/overflow/aliases são decisões ruins de serem tomadas sob pressão do exemplo motivador, que só usa `cadeia`. Fica como ADR separado e posterior.
3. **ADR mínimo + RFCs + testes executáveis sobre a arquitetura atual.** Escolhida: congela o contrato vigente, documenta o adiamento, especifica cada feature e implementa em PRs pequenos com testes nos dois backends.

## Decisões de design

### ADR mínimo de contrato de tipos

O primeiro artefato será um ADR curto registrando:

- `inteiro` continua `i32` com `wrapping` até decisão posterior;
- `leia()` infere o tipo do contexto e resulta em `cadeia` sem contexto tipado, sem `leia<T>()` (revisão de 2026-11-27);
- nomes atuais (`caracter`, `registro`, `enum`, `Opcao`, `Resultado`, `escolha`) permanecem;
- spans continuam UTF-8 em bytes;
- `vazio` existe apenas como tipo de retorno/função sem valor, sem valor `nunca`;
- aliases (`caractere`, `estrutura`, `opcional`, `resultado`) e novas larguras ficam explicitamente adiados.

### Inferência

Tornar `:` opcional em declarações com inicializador. O tipo vem do literal, chamada, construtor ou operação binária; sem anotação, o contexto não altera o tipo inferido. `[]` sem contexto continua erro. Tipos explícitos continuam aceitos e são a forma canônica em documentação. `seja nome = leia();` infere `cadeia` porque não há tipo esperado; `seja n: inteiro = leia();` mantém o binding `inteiro` e a leitura converte implicitamente ao tipo anotado.

### `leia()` como expressão

Adicionar nó de expressão de leitura ao HIR, mantendo `StmtKind::Read` como açúcar para compatibilidade. **A leitura sem argumento infere o tipo esperado pelo contexto**; sem contexto, resulta em `cadeia`. Declaração tipada, parâmetro, retorno, atribuição e campo fornecem o tipo; entrada inválida gera erro de execução, não erro de tipo estático. A forma com variável-alvo continua com o tipo da variável e não é convertida em expressão nesta fatia. A varredura de `has_read` no exportador Rust e o protocolo `FERRO_INPUT:{expected}` devem passar a enxergar leituras dentro de expressões. `leia<T>()` e validação tipada explícita de entrada ficam em P1.

### `escreval`

Implementar como sugar sintático:

```ferro
escreval(a, b);
```

equivale a escrever `a`, `b` e uma quebra de linha final com `escreva`. A quebra é sintética: o fonte do exemplo não precisa conter `"\n"` explícito.

Isso preserva `StmtKind::Write`, `Op::Write`, o `print!` do Rust gerado e os limites de saída existentes.

### Interpolação

Adicionar ao HIR uma expressão de cadeia com partes:

- `Literal(String, Span)`;
- `Expressão(Expr, Span)`.

**Gramática exata nesta fatia:**

- `\{` e `\}` produzem chaves literais e viram parte `Literal`.
- Um `{` não escapado inicia interpolação somente quando existe um `}` correspondente na mesma cadeia e o conteúdo entre eles não é vazio.
- O conteúdo é uma expressão FERRO completa; o primeiro `}` não escapado fecha a parte. `{}` e `{` sem fechamento permanecem texto literal, preservando cadeias já publicadas que usam chaves.
- Conteúdo que não parseia (por exemplo `{1 +}`) gera diagnóstico com o span exato do segmento entre chaves.
- Não há especificador de formato, largura ou alinhamento nesta fatia.

A conversão para valores não cadeia segue a regra atual de concatenação: somente `cadeia`/`caracter` são convertidos diretamente; números exigem decisão futura e devem gerar diagnóstico nesta fatia. O lowering inicial reusa `__ferro_concat` e `Op::Add` com `Str`, mantendo source maps por parte.

O scanner de realce dos dois editores marca todo `{...}` não escapado e não vazio como `interpolation`; a validade da expressão é decidida pelo parser, não pelo realce. O formatador preserva o conteúdo da interpolação verbatim. Rename e autocompletar **dentro** da expressão interpolada ficam em P1 — nesta fatia os editores apenas mantêm o texto bruto intacto e documentam `escreval`/`leia()`. O debugger lê as partes e os source maps, não o texto bruto.

### Retorno implícito

Adicionar `tail_expression` ao bloco/HIR. A regra vale **somente para o corpo da função**: se a última posição do corpo de uma função com tipo de retorno declarado for uma expressão sem `;`, ela vira retorno implícito. Blocos aninhados (`se`, `senao`, `escolha`, laços) não ganham retorno implícito — neles, `retorne valor;` continua obrigatório. A única exceção são os ramos de valor de `tente` em expressão (RFC 007), nos quais a expressão final sem `;` produz o valor do ramo e não retorno implícito da função. `retorne valor;` continua válido em qualquer posição. Blocos vazios e funções `vazio`/sem retorno declarado não ganham retorno implícito. Uma expressão sem `;` em corpo de função sem retorno declarado é erro localizado. A regra de `;` é documentada com exemplos e erros localizados.

### Realce de sintaxe: IDE e playground

Definir uma taxonomia compartilhada de tokens, derivada do mesmo contrato lexical usado pelo frontend:

- `keyword`, `type`, `builtin`, `string`, `interpolation`, `number`, `comment`, `operator`.
- `escreval` é `builtin`, não `keyword`, porque é sugar de `escreva`; `leia` continua `builtin` também quando usado como expressão.
- Dentro de uma cadeia, o texto literal continua `string` e cada trecho `{...}` não escapado e não vazio vira `interpolation`. O realce nunca altera o documento; é apenas apresentação.
- O contrato compartilhado vai até o par `string`/`interpolation`. Tokens internos ao trecho interpolado são decisão de cada editor, não parte do contrato.

**IDE.** Em `apps/desktop/src/ferro.ts`, `escreval` entra em `FERRO_BUILTINS` e o casamento atual de cadeia em `token()` é substituído por um pequeno scanner que emite `string`, `interpolation` e, quando aplicável, `variableName` para o identificador dentro de `{...}`, preservando escapes e spans. Esse token interno é exclusivo da IDE; o playground não o reproduz. `tokenTable` mapeia `interpolation` para uma tag dedicada derivada de `tags.special(tags.string)`. Em `editor/theme.ts`, `themes.ts` e `theme-presets.ts`, adicionar `--syntax-interpolation` com fallback às cores de tipo/função para temas existentes. Completions e hover passam a documentar `escreval` e `leia()`; `leia<T>()` não é sugerido nesta fatia.

**Playground.** Em `apps/web-playground/app.mjs`, estender `TOKEN_NAMES` com `interpolation`, incluir `escreval` no grupo `builtin` e substituir o grupo `string` da regex por um scanner de partes que produza `pg-tok-string` e `pg-tok-interpolation`, sempre com `createTextNode`/`span` e sem `innerHTML`. O trecho `{...}` inteiro recebe `pg-tok-interpolation`; não há span separado para o identificador interno. Em `style.css`, adicionar `.pg-tok-interpolation` usando `--pg-interpolation` nos temas claro/escuro; o fallback de `forced-colors` continua escondendo a camada de realce. O texto renderizado no `<pre id="pg-highlight">` deve ser **exatamente igual** ao valor do `<textarea>`: o `\n` extra atual, usado para alinhamento, é removido e o alinhamento passa a ser resolvido por CSS.

### Debugger e painel Rust

O debugger atual ganha exibição do mapeamento HIR → bytecode → Rust para as novas construções. Nenhuma VM nova é criada. A visualização Rust compilável continua distinta da equivalência pedagógica resumida.

A superfície é concreta e limitada:

- **HIR e bytecode:** source maps internos já existentes ganham spans por parte de interpolação e pelo nó de leitura; offsets e nomes de binding continuam nos eventos atuais.
- **IDE:** o painel do debugger mostra, para a instrução selecionada, o nó HIR, o offset do bytecode e a linha correspondente no Rust gerado. É a única superfície nova voltada ao usuário.
- **Rust gerado:** o preview existente recebe o mesmo source map; nenhuma formatação “bonita” é introduzida.
- **WASM:** nenhuma nova API ou campo JSON. A fachada continua expondo apenas o que já expõe; os testes de WASM verificam o mesmo programa, não um mapa serializado.

Isso mantém o mapeamento testável nos crates e evita ampliar o contrato público da fachada.

### Decisões de contorno (revisão do design)

Registro único para quem for planejar a implementação:

1. `leia()` infere o tipo do contexto; sem contexto é `cadeia`. A forma legada com variável-alvo mantém o tipo da variável e também converte implicitamente.
2. Retorno implícito vale somente para o corpo da função; blocos aninhados exigem `retorne`.
3. Interpolação: `\{`/`\}` são literais; `{...}` não vazio e fechado é expressão; `{}` e `{` aberto permanecem literais; conteúdo inválido gera diagnóstico com span.
4. Superfície do mapa HIR → bytecode → Rust: source maps internos + painel do debugger na IDE + preview Rust existente; nenhuma API WASM nova.
5. O escopo de IDE e playground está explicitamente incluído via RFC 006; a frase inicial de "núcleo da linguagem" não exclui esses arquivos.
6. `variableName` dentro de `{...}` é exclusivo da IDE; o playground usa apenas `pg-tok-interpolation` para o segmento inteiro.
7. O `<pre id="pg-highlight">` deve ser exatamente igual ao `<textarea>`; o `\n` extra de alinhamento é removido e o alinhamento fica no CSS.
8. Formatter preserva o conteúdo interpolado verbatim; rename e completion dentro da expressão interpolada ficam em P1.

## Sequência de entrega

1. ADR mínimo de contrato de tipos.
2. RFC 001 — inferência; RFC 002 — `leia()`; RFC 003 — `escreval`; RFC 004 — interpolação; RFC 005 — retorno implícito; RFC 006 — realce de sintaxe na IDE e no playground.
3. Testes de aceite falhando para as cinco sintaxes e para o contrato de realce, com execução nos dois backends, WASM aplicável e nos dois editores.
4. Implementação em PRs pequenos: `escreval` → inferência → interpolação → `leia()` → retorno implícito → realce final alinhado nos dois editores.
5. Atualização de formatter, snippets, ajuda, exemplos e matriz de migração conforme cada feature fechar.
6. ADR posterior de tipos numéricos e decisão sobre heap/GC.

## Verificação

- `cargo fmt --check`.
- `cargo test -p ferro-hir -p ferro-rust-frontend -p ferro-bytecode -p ferro-vm -p ferro-rust`.
- Testes de runner FERRO e fachada WASM cobrindo o mesmo programa das RFCs.
- Comparação observável de stdout entre VM e Rust gerado para cada exemplo válido.
- Diagnósticos localizados para cada sintaxe inválida, com spans UTF-8 corretos.
- Testes CodeMirror (`syntaxTree`) confirmando `escreval` como `variableName.standard`, `leia` como builtin em expressão e os nós `string`/`interpolation` com spans exatos em `"Olá, {nome}!"`.
- Testes do playground confirmando `pg-tok-interpolation`, `escreval` no grupo `builtin`, texto do `<pre id="pg-highlight">` idêntico ao `<textarea>` e zero uso de `innerHTML`.
- Presets de tema cobrindo a nova variável `--syntax-interpolation`.
- Nenhuma regressão nos testes atuais de `leia`, `escreva`, concatenação, formatação e retorno explícito.

## Riscos

- Inferência mal definida pode tornar `leia()` ambíguo em contextos sem tipo; o resultado `cadeia` sem contexto e o erro para `[]` mitigam.
- Interpolação pode perder spans se for implementada no lexer; por isso as partes vivem na HIR com spans próprios.
- `leia()` como expressão pode quebrar o protocolo de entrada da IDE se `has_read` não for atualizado; testes de runner cobrem isso.
- `escreval` pode duplicar newline se o programa já usar `"\n"`; o sugar é explícito e não altera `escreva`.
- A fatia pode ser contaminada pela decisão numérica; o ADR mínimo congela o estado atual e adia o debate.
- IDE e playground podem divergir se o contrato de tokens for implementado duas vezes sem teste compartilhado; a taxonomia comum e os testes nos dois editores reduzem esse risco.
