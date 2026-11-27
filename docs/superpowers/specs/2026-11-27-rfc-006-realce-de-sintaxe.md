# RFC 006: contrato de realce compartilhado

- **Data:** 2026-11-27
- **Status:** Aceita
- **Relacionados:** [ADR 0019](../../adr/0019-contrato-minimo-de-tipos-pors.md), [RFC 002](2026-11-27-rfc-002-leia-como-expressao.md), [RFC 003](2026-11-27-rfc-003-escreval.md), [RFC 004](2026-11-27-rfc-004-interpolacao-de-cadeias.md), [RFC 005](2026-11-27-rfc-005-retorno-implicito.md), [design de evolução sintática inicial do PORS](2026-11-26-pors-syntax-evolution-design.md)

## Contexto

O design de evolução sintática inicial do PORS (documento de 2026-11-26) declarou, na seção
"Realce de sintaxe: IDE e playground", a necessidade de uma **taxonomia compartilhada de tokens**
para o realce, além da herança do contrato lexical já usado pelo frontend.

A RFC 004 já fixou o contrato observável da interpolação no frontend, na HIR, no bytecode, no
formatador e nos editores, incluindo a regra de que o scanner de realce dos dois editores marca
todo `{...}` não escapado e não vazio como `interpolation`, com a validade da expressão decidida
pelo parser e não pelo realce.

A evolução sintática em curso (RFCs 002 a 005) deixou o próximo passo declarado no design: definir o
**contrato de realce compartilhado** entre a IDE e o playground, de modo que os dois editores
apresentem os mesmos tokens para as mesmas regras sem divergir.

## Decisão

### Taxonomia compartilhada de tokens

1. A taxonomia compartilhada de realce é: `keyword`, `type`, `builtin`, `string`, `interpolation`,
   `number`, `comment`, `operator`.
2. A taxonomia é derivada do mesmo contrato lexical usado pelo frontend, que já classifica esses
   rótulos para fins de realce e parsing; o contrato de realce não cria rótulos novos.
3. O contrato compartilhado vai até o par `string`/`interpolation`. Tokens internos ao trecho
   interpolado são decisão de cada editor, não fazem parte do contrato.

### `escreval` e `leia` como builtins

1. `escreval` é `builtin`, nunca `keyword`: é sugar de `escreva` com quebra de linha (RFC 003) e
   entra no mesmo grupo de `escreva` e `leia` da taxonomia.
2. `leia` continua `builtin`, inclusive quando usado como expressão (`leia()`, RFC 002) e dentro de
   interpolação (RFC 004); também não é palavra reservada.
3. `leia<T>()` não é sugerido nem realçado como genérico nesta fatia.

### Regras de `{...}` no realce

1. Dentro de uma cadeia, o texto literal continua `string` e cada trecho `{...}` não escapado e não
   vazio é `interpolation`; o par `string`/`interpolation` fecha o contrato de realce.
2. As regras de `{...}` são idênticas às da RFC 004; a RFC 004 é normativa e esta RFC apenas consolida
   o resumo do contrato de realce:
   - `\{` e `\}` são literais; um `{` ou `}` é escapado quando precedido por número ímpar de `\` no
     texto bruto, de modo que `\\{` é barra literal seguida de `{` não escapado.
   - O primeiro `}` não escapado fecha a parte; quando houver mais de um `{` antes do primeiro `}`, o
     primeiro par forma a parte e os demais permanecem literais.
   - `{}` e um `{` sem fechamento permanecem `string`; o conteúdo precisa ser não vazio; o conteúdo pode
     conter espaços, que contam como conteúdo.
3. O realce **não valida** a expressão: a validade é decidida pelo parser, com os diagnósticos
   `pors.interpolation-invalid` e `pors.interpolation-non-string` (RFC 004). Conteúdo com erro
   léxico ou nomes não resolvidos permanece literal, sem diagnóstico de interpolação. O scanner apenas
   emite o token `interpolation`.

### IDE

1. Na IDE, em `apps/desktop/src/portugol.ts`, o scanner de cadeia do token emitido passa a
   produzir `string`, `interpolation` e, quando aplicável, o nó interno `variableName` para o
   identificador dentro de `{...}`, preservando escapes e spans.
2. O nó interno `variableName` é exclusivo da IDE e usado apenas por ela; o playground não o
   reproduz.
3. O `tokenTable` da IDE mapeia `interpolation` para uma tag dedicada derivada de
   `tags.special(tags.string)`.
4. Em `editor/theme.ts`, `themes.ts` e `theme-presets.ts`, a variável `--syntax-interpolation` é
   adicionada com fallback às cores de tipo/função já existentes nos temas.
5. Completions e hover continuam documentando `escreval` e `leia()` como builtins.

### Playground

1. No playground, em `apps/web-playground/app.mjs`, o `TOKEN_NAMES` ganha `interpolation`, o grupo
   `string` da regex é substituído por um scanner de partes que produz `pg-tok-string` e
   `pg-tok-interpolation`.
2. O trecho inteiro `{...}` recebe `pg-tok-interpolation`; não há span separado para o identificador
   interno, pois o contrato do playground para com o segmento inteiro é o token `pg-tok-interpolation`.
3. Em `style.css`, a classe `.pg-tok-interpolation` usa a variável `--pg-interpolation` nos temas
   claro e escuro; o fallback de `forced-colors` continua escondendo a camada de realce.
4. O texto renderizado no `<pre id="pg-highlight">` deve ser **exatamente igual** ao valor do
   `<textarea id="pg-editor">`; o `\n` extra usado antes para alinhamento é removido e o alinhamento
   passa a ser resolvido por CSS.

### Proibições

1. `innerHTML` é proibido no playground: apenas `createTextNode`/`span` e manipulação via DOM
   API.

### Fronteiras

- A implementação dos passos acima está planejada em
  `docs/superpowers/plans/2026-11-27-pors-syntax-highlighting.md` (Tasks 1 a 4); esta RFC não
  repete as tarefas do plano, apenas fixa o contrato que eles produzem.
- Não há novo opcode, VM, crate, produto, API WASM ou pasta `specs/` paralela.
- O debugger, o painel Rust e o formato continuam fora do escopo deste contrato de realce.

## Exemplos válidos

### Realce de cadeias com interpolação

| Código | Realce |
| --- | --- |
| `escreval("Olá, {nome}!")` | `string` para `"Olá, "`, `interpolation` para `{nome}`, `string` para `"!"` |
| `escreval("\{nome}");` | `string` único, sem `interpolation` |
| `escreval("{}");` | `string` único, sem `interpolation` |
| `escreval("abri {");` | `string` único, sem `interpolation` |

### IDE

| Token emitido | Conteúdo |
| --- | --- |
| `string` | texto literal da cadeia, incluindo escapes e literais de chave |
| `interpolation` | par `{...}` não escapado e não vazio, incluindo as chaves |
| `variableName` | identificador dentro do `{...}`; nó interno exclusivo da IDE |

### Playground

| Token | Conteúdo |
| --- | --- |
| `pg-tok-string` | texto literal da cadeia, incluindo escapes e literais de chave |
| `pg-tok-interpolation` | segmento inteiro `{...}` não escapado e não vazio |

## Exemplos inválidos

| Código | Motivo |
| --- | --- |
| `escreval("{1 +}");` | o realce emite `interpolation`; a expressão inválida é diagnosticada pelo parser como `pors.interpolation-invalid` no span do conteúdo |
| `escreva("{42}");` | o realce emite `interpolation`; o tipo não é `cadeia` nem `caracter`, com `pors.interpolation-non-string` no parser |

## Consequências

- Os testes de aceite (fase seguinte) consomem esta taxonomia: `keyword`, `type`, `builtin`,
  `string`, `interpolation`, `number`, `comment`, `operator`.
- A implementação planejada em `docs/superpowers/plans/2026-11-27-pors-syntax-highlighting.md` (Tasks
  1 a 4) mantém a mesma taxonomia nos dois editores, sem tokens internos no playground.
- Qualquer mudança na gramática de `{...}`, nas regras de escapes ou na lista de tokens exige
  revisão desta RFC e do ADR 0019 antes de entrar em código.
- Não há novo opcode, VM, crate, produto, API WASM ou pasta `specs/` paralela; o contrato de realce
  reusa os tokens existentes do frontend.
