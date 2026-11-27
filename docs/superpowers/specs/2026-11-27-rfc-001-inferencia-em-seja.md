# RFC 001: Inferência de tipo em seja

- **Data:** 2026-11-27
- **Status:** Aceita
- **Relacionados:** [ADR 0018](../../adr/0018-contrato-minimo-de-tipos-pors.md), RFC 002 (`leia()` como expressão)

## Contexto

`declaration()` em `crates/portugol-rs-frontend/src/parser/stmt.rs:284-346` exige hoje `:` e `=` antes do inicializador e compara o tipo anotado com o da expressão. Em `crates/portugol-rs-frontend/src/parser/expr.rs:843-852`, `[]` só recebe tipo quando existe um tipo vetorial esperado. O design aprovado de evolução sintática pede a forma do programa motivador, e o ADR 0018 congela `inteiro` como `i32` com `wrapping` e spans UTF-8 em bytes.

## Decisão

A sintaxe com inferência é `seja nome = expressao;`: `seja`, `mutavel` opcional, identificador, `=`, expressão inicial e `;`.

1. `:` é opcional; `=` e a expressão inicial são obrigatórios, sem declaração que fique sem valor (regra 1).
2. Sem anotação, o tipo do binding é o tipo da expressão inicial, obtido de literal, chamada, construtor ou operação binária. O contexto — declaração, parâmetro ou retorno — nunca altera o tipo inferido. `seja nome = leia();` infere `cadeia` porque `leia()` tem tipo fixo `cadeia` (RFC 002), não por causa do `seja`. `inteiro` permanece `i32` com `wrapping`, conforme ADR 0018.
3. O tipo explícito continua aceito e é a forma canônica da documentação; incompatibilidade com o inicializador gera `pors.type-mismatch` com span na expressão inicial (regra 3).
4. Vetor vazio não muda: sem tipo esperado, `[]` gera `pors.empty-vector-type-inference` com span no par de colchetes (regra 4).
5. Spans são offsets UTF-8 semiabertos `[início, fim)` em bytes, inclusive quando há acentos; índices por caractere ou ponto de código não entram nesta RFC.

### Fontes do tipo inferido

| Origem | Exemplo | Tipo inferido |
| --- | --- | --- |
| literal | `"Olá"` | `cadeia` |
| chamada | `saudar(nome)` | tipo de retorno de `saudar` |
| construtor de registro | `Ponto { x: 1, y: 2 }` | registro `Ponto` |
| construtor de enum | `Opcao::Algum(1)` | `Opcao` |
| operação binária | `1 + 2` | `inteiro` |
| vetor | `[1, 2, 3]` | `vetor<inteiro>` |
| leitura | `leia()` | `cadeia`, por RFC 002 |

## Exemplos válidos

```pors
seja nome = leia();
seja saida = saudar(nome);
seja total = 1 + 2;
seja pontos = Ponto { x: 1, y: 2 };
seja opcao = Opcao::Algum(1);
seja numeros = [1, 2, 3];
seja rotulo: cadeia = "Olá";
```

Spans dos inicializadores, com cada trecho medido a partir da coluna 0:

| Trecho | Span |
| --- | --- |
| `seja nome = leia();` | `[12, 18)` |
| `seja saida = saudar(nome);` | `[13, 25)` |
| `seja total = 1 + 2;` | `[13, 18)` |
| `seja pontos = Ponto { x: 1, y: 2 };` | `[14, 34)` |
| `seja opcao = Opcao::Algum(1);` | `[13, 28)` |
| `seja numeros = [1, 2, 3];` | `[15, 24)` |
| `seja rotulo: cadeia = "Olá";` | `[22, 28)`; `á` ocupa 2 bytes |

## Exemplos inválidos

| Trecho (coluna 0) | Diagnóstico |
| --- | --- |
| `seja n: inteiro = "Olá";` | erro da regra 3 em `[18, 24)`; `á` ocupa 2 bytes |
| `seja v = [];` | erro da regra 4 em `[9, 11)` |
| `seja n;` | erro da regra 1 no `;` de `[6, 7)` |
| `seja n = 1 + ;` | erro de expressão no `;` de `[13, 14)` |

## Consequências

- A mudança de parser aceita `:` ausente quando há inicializador; não há opcode, VM, API WASM, crate ou nome de produto novo.
- A forma com tipo explícito segue válida e permanece como forma canônica em documentação e exemplos.
- RFCs 002 a 006 cobrem `leia()`, `escreval`, interpolação, retorno implícito e realce; esta RFC não decide essas sintaxes.
- Os testes de aceite devem executar os exemplos acima em HIR, bytecode/VM e Rust gerado, comparando os spans UTF-8 e a saída observável.
