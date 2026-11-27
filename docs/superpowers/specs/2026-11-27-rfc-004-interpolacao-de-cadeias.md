# RFC 004: interpolação de cadeias

- **Data:** 2026-11-27
- **Status:** Aceita
- **Relacionados:** [ADR 0018](../../adr/0018-contrato-minimo-de-tipos-pors.md), [RFC 002](2026-11-27-rfc-002-leia-como-expressao.md), [RFC 003](2026-11-27-rfc-003-escreval.md), [design de evolução sintática inicial do PORS](2026-11-26-pors-syntax-evolution-design.md)

## Contexto

O design aprovado pede `"Olá, {nome}!"` como cadeia com partes na HIR, e classifica como anti-objetivo substituir o texto antes de montar a HIR: isso perderia spans, formatação, rename e diagnósticos. Hoje o lexer devolve `String(String)` já com os escapes decodificados e trata `{`/`}` dentro de cadeia como literais, então `"{nome}"` imprime `{nome}`.

O ADR 0018 congela `cadeia` e `caracter` como os únicos tipos que participam da concatenação de texto e mantém spans como offsets UTF-8 semiabertos em bytes. A regra vigente em `crates/portugol-rs-frontend/src/parser/constants.rs:318-323` aceita `Add` entre `String`/`String` e `Character`/`Character` com resultado `String`; o bytecode reusa `Op::Add` e o Rust gerado reusa `__pors_concat`, sem opcode novo. A RFC 002 dá tipo `cadeia` fixo a `leia()`, e a RFC 003 define `escreval` como açúcar de `escreva`, sem alterar a cadeia em si.

Esta RFC fixa o contrato observável da interpolação no frontend, na HIR, no bytecode/VM, no Rust gerado, no formatador e nos editores. Não define retorno implícito (RFC 005) nem o contrato completo de realce (RFC 006).

## Decisão

### Gramática exata

1. A interpolação existe somente dentro de literais de cadeia entre aspas duplas. O frontend particiona o texto bruto entre as aspas — não a `String` já decodificada — para preservar escapes e offsets originais.
2. `\{` e `\}` produzem chaves literais e viram parte `Literal`. Um `{` ou `}` é considerado escapado quando precedido por número ímpar de `\` no texto bruto; `\\{` é barra literal seguida de `{` não escapado.
3. Um `{` não escapado inicia interpolação somente quando existe um `}` não escapado correspondente na mesma cadeia e o conteúdo entre eles não é vazio. O `}` correspondente é o primeiro `}` não escapado depois do `{`.
4. O conteúdo entre as chaves é uma expressão PORS completa, parseada pelo parser normal nos offsets de bytes originais do fonte. Não há reconhecimento de strings aninhadas para adiantar o fechamento: o primeiro `}` não escapado fecha a parte.
5. `{}` e um `{` sem fechamento não escapado na mesma cadeia permanecem texto literal; o `{` é emitido como literal e a varredura continua. Quando há mais de um `{` antes do primeiro `}`, somente o primeiro par forma a parte: `"{nome} e { aberto"` interpola `{nome}` e deixa o último `{` literal.
6. Espaço em branco conta como conteúdo: `{ }` não é vazio e tenta parsear como expressão, com o diagnóstico da regra de conteúdo inválido.
7. Não há especificador de formato, largura, alinhamento, precisão, conversão especial ou sintaxe reservada para eles nesta fatia. `{nome:>10}` é conteúdo inválido, não formatação.
8. Cadeias sem `{` não escapado, sem `\{`, `\}` e sem interpolação continuam `ExprKind::String` como hoje; nenhuma cadeia existente muda de forma só por esta RFC.

### HIR e spans

1. A cadeia interpolada vira `ExprKind::InterpolatedString { parts }`, com `parts: Vec<StringPart>` e `StringPart::Literal(String, Span)` ou `StringPart::Expression(Expr, Span)`. O nó `Expr` resultante tem `ty = cadeia`, independentemente do número de partes ou do tipo das expressões internas.
2. `StringPart::Literal(String, Span)` guarda o texto decodificado; seu `Span` cobre os bytes correspondentes no fonte, sem as aspas. Em `\{` e `\}`, o span cobre os dois bytes do escape, embora o valor decodificado tenha um único caractere.
3. `StringPart::Expression(Expr, Span)` guarda a expressão parseada; `Expr.span` cobre exatamente o conteúdo entre chaves, sem as chaves, e o `Span` da parte cobre `{...}` incluindo as duas chaves. Texto literal contíguo, inclusive escapes, forma uma única parte `Literal`.
4. A ordem das partes é a ordem dos bytes no fonte. O `Expr` de `InterpolatedString` cobre a cadeia inteira, incluindo as aspas, como já acontece com `ExprKind::String`.
5. Spans são offsets UTF-8 semiabertos `[início, fim)`, medidos a partir da coluna 0, inclusive com acentos; não há índice por caractere ou ponto de código.

| Trecho (coluna 0) | Span da cadeia | Partes |
| --- | --- | --- |
| `escreval("Olá, {nome}!");` | `[9, 24)` | `Literal("Olá, ", [10, 16))`, `Expression(nome, [16, 22))` com `Expr.span = [17, 21)`, `Literal("!", [22, 23))` |
| `escreva("\{x\}");` | `[8, 15)` | uma única `Literal("{x}", [9, 14))`; `\{` e `\}` ocupam 2 bytes cada no fonte |
| `escreva("{}");` | `[8, 12)` | sem interpolação; cadeia literal comum |

### Tipos e diagnósticos

1. Somente expressões do tipo `cadeia` ou `caracter` são convertidas diretamente para texto. `caracter` entra na concatenação como cadeia de um caractere, pela conversão existente; `cadeia` entra como ela mesma. `leia()` satisfaz esta regra com o tipo `cadeia` fixo da RFC 002.
2. Conteúdo que não parseia como expressão PORS completa gera `pors.interpolation-invalid` no span exato do conteúdo entre chaves, sem as chaves. Exemplo: `escreva("{1 +}");` aponta `[10, 13)`.
3. Conteúdo que parseia, mas cujo tipo não é `cadeia` nem `caracter`, gera `pors.interpolation-non-string` no span da expressão interna, sem as chaves. Números inteiros e reais, lógicos, vetores, registros, enums e `vazio` usam todos esse código; não há `to_string` implícito nem conversão numérica nesta fatia. Exemplo: `escreva("{42}");` aponta `[10, 12)`.
4. Um diagnóstico não transforma a parte em literal: a cadeia interpolada continua com `ty = cadeia`, e o erro é localizado na parte ofensiva.

| Trecho (coluna 0) | Diagnóstico |
| --- | --- |
| `escreva("{1 +}");` | `pors.interpolation-invalid` em `[10, 13)` |
| `escreva("{ }");` | `pors.interpolation-invalid` em `[10, 11)` |
| `escreva("{42}");` | `pors.interpolation-non-string` em `[10, 12)` |
| `escreva("{2.5}");` | `pors.interpolation-non-string` em `[10, 13)` |
| `escreva("{nome:>10}");` | `pors.interpolation-invalid` em `[10, 18)`; não há especificador de formato |

### Lowering, bytecode, Rust gerado e source maps

1. O lowering do `ExprKind::InterpolatedString` monta o texto das partes na ordem do fonte, reusando `Op::Add` com resultado `cadeia`; a VM mantém a concatenação atual de `Op::Add` (inclusive a conversão de `caracter` para texto), e o Rust gerado mantém `__pors_concat`. Não há opcode, helper, VM, crate ou tipo novo.
2. Quando há uma única parte literal, o resultado é o próprio valor da cadeia; não é introduzida operação de concatenação vazia. Quando há mais partes, a dobra é associativa à esquerda, na ordem do fonte.
3. Cada operação de concatenação, incluindo a conversão de um `caracter`, recebe source map apontando para o `Span` da parte correspondente; a expressão interna mantém seu próprio span. Debugger, bytecode/VM e Rust gerado leem as partes e esses mapas, não o texto bruto da cadeia.
4. VM e Rust gerado produzem stdout idêntico, incluindo para escapes e cadeias com chaves literais. Não há campo JSON, fachada ou API WASM nova.

### Formatação, editores e debugger

1. O formatador preserva o conteúdo da interpolação **verbatim**: espaços, quebras, escapes e a expressão entre chaves não são reformatados nem normalizados. Fora das chaves, as regras atuais de formatação continuam valendo.
2. O scanner de realce da IDE e do playground marca cada trecho `{...}` não escapado e não vazio como `interpolation`, incluindo as chaves; texto literal fora dele continua `string`. A validade da expressão é decidida pelo parser, não pelo realce, e o realce nunca altera o documento. A taxonomia completa e os tokens de cada editor são da RFC 006.
3. Rename e autocompletar **dentro** da expressão interpolada ficam em **P1**: nesta fatia os editores apenas mantêm o texto bruto intacto e documentam `escreval` e `leia()` como nas RFCs 002 e 003.
4. O debugger mostra o mapeamento HIR → bytecode → Rust para as partes e seus source maps, sem nova superfície de API.

### Fronteiras e adiamentos

- Especificadores de formato não são aceitos nesta fatia e nenhuma sintaxe é reservada para eles; aceitar algum formato exige revisar esta RFC.
- A decisão sobre conversão de números para texto também fica adiada: até lá, números em interpolação geram `pors.interpolation-non-string`.
- `leia<T>()`, conversão e validação tipada de entrada continuam em **P1**, conforme RFC 002 e ADR 0018.
- Retorno implícito é da RFC 005 e realce de sintaxe é da RFC 006; esta RFC não altera nenhum dos dois contratos.
- Permanecem adiados pelo ADR 0018: novas larguras de inteiro e a decisão sobre overflow, aliases, `nunca`, Unicode além dos offsets de bytes dos spans e mudanças em heap/GC.
- Não há nova crate, renomeação de produto, pasta `specs/` paralela ou API WASM. RFCs e design docs continuam em `docs/superpowers/specs/` e ADRs em `docs/adr/`.

## Exemplos válidos

```pors
funcao saudar(nome: cadeia) -> cadeia {
    retorne "Olá, {nome}!";
}

funcao inicio() {
    seja nome = leia();
    escreval(saudar(nome));
    escreval("Olá, {nome}!");
}
```

`escreval("Olá, {nome}!");` imprime `Olá, <valor de nome>` seguido de uma quebra de linha, sem bytes extras. A interpolação também é válida em qualquer posição de expressão, inclusive como argumento de `escreva`:

```pors
funcao inicio() {
    seja letra: caracter = 'a';
    escreva("letra: {letra}\n");
    escreva("\{nome\} e {} ficam literais; { aberto também");
    escreval("Olá, {leia()}!");
}
```

O segundo `escreva` imprime `{nome} e {} ficam literais; { aberto também` — `\{`, `\}` e `{` sem fechamento são literais. A quebra em `escreva("letra: {letra}\n");` vem do `\n` escrito pelo programa; `escreval` acrescenta apenas a sua quebra, conforme RFC 003.

## Exemplos inválidos

| Trecho (coluna 0) | Motivo |
| --- | --- |
| `escreva("{1 +}");` | conteúdo não parseia: `pors.interpolation-invalid` em `[10, 13)` |
| `escreva("{42}");` | número não é `cadeia` nem `caracter`: `pors.interpolation-non-string` em `[10, 12)` |
| `escreva("{nome:>10}");` | especificador de formato inexistente: `pors.interpolation-invalid` em `[10, 18)` |

## Consequências

- Os testes de aceite devem verificar a estrutura `StringPart`, `ty = cadeia`, os spans UTF-8 em bytes, os dois códigos de diagnóstico e a saída observável igual entre bytecode/VM e Rust gerado.
- O programador motivador do design passa a imprimir a saudação interpolada sem concatenação manual, preservando `leia(variavel)`, `escreva` e a concatenação com `+` já publicados.
- Qualquer mudança na gramática das chaves, nas partes da HIR, nos códigos de diagnóstico, na conversão permitida ou na preservação verbatim do conteúdo exige revisão desta RFC e do ADR 0018 antes de entrar em código.
