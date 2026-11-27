# ADR 0018: Contrato mínimo de tipos do PORS

- **Status:** Aceita
- **Data:** 2026-11-27
- **Responsável:** mantenedor do produto
- **Relacionados:** [ADR 0002](0002-hir-compartilhada-portugol-rs.md), [ADR 0016](0016-pors-como-linguagem-autoral-e-retirada-do-por.md), [design de evolução sintática inicial do PORS](../superpowers/specs/2026-11-26-pors-syntax-evolution-design.md)

## Contexto

A evolução sintática inicial do PORS precisa de um contrato mínimo de tipos
publicado antes das RFCs de inferência, `leia()`, `escreval`, interpolação,
retorno implícito e realce de sintaxe. O exemplo motivador usa apenas
`cadeia`, e a mudança acontece sobre a HIR, o bytecode/VM e o Rust gerado já
existentes, sem nova VM, nova API WASM ou renomeação de crates e produto.

O estado vigente, verificado no design doc aprovado, é: `inteiro` é `i32` com
`wrapping` em HIR/VM/Rust gerado; `leia(variavel)` é comando e toda leitura sem
argumento terá tipo `cadeia` fixa nesta fatia; os nomes `caracter`, `registro`,
`enum`, `Opcao`, `Resultado` e `escolha` já estão publicados; spans são UTF-8 em
bytes; `vazio` aparece apenas como tipo de retorno. `escreval(a, b);` é somente
açúcar para `escreva(a, b, "\n")`, sem tipo ou opcode novo.

Larguras adicionais, regras de overflow, aliases, suporte a Unicode e modelo de
memória são decisões maiores que não devem ser tomadas sob pressão do exemplo
motivador.

## Decisão

1. `inteiro` continua `i32` com `wrapping` em HIR, bytecode/VM e Rust gerado
   até decisão posterior. Nenhuma largura nova é introduzida nesta fatia.
2. `leia()` tem tipo `cadeia` fixa, sem `leia<T>()`: contexto de declaração,
   parâmetro ou retorno nunca altera esse tipo, e usar `leia()` onde se espera
   `inteiro` é erro de tipo com span na chamada. A forma legada
   `leia(variavel)` mantém o tipo da variável.
3. Os nomes atuais `caracter`, `registro`, `enum`, `Opcao`, `Resultado` e
   `escolha` permanecem. Nenhum deles é renomeado nesta fatia.
4. Spans continuam sendo offsets UTF-8 em bytes, semiabertos. Índices por
   caractere ou por ponto de código não entram neste contrato.
5. `vazio` existe apenas como tipo de retorno de função sem valor. Não há valor
   `nunca` nesta fatia.
6. Ficam explicitamente adiados: novas larguras de inteiro e a decisão sobre
   overflow; aliases `caractere`, `estrutura`, `opcional` e `resultado`;
   `nunca`; Unicode além dos offsets de bytes dos spans; e heap/GC, mantido o
   modelo atual de `Rc<RefCell>` com orçamento.

## Alternativas consideradas

### ADR completo de tipos antes de qualquer sintaxe

Rejeitada. Largura, overflow, aliases e heap/GC exigem decisões que não podem
ser bem tomadas sob pressão do exemplo motivador, que só usa `cadeia`. O
contrato mínimo congela o que já existe e explicita o adiamento.

### Tipo de `leia()` dependente do contexto

Rejeitada. Tornar `leia()` polimórfico por inferência tornaria ambíguo um
`seja nome = leia();` sem contexto e exigiria `leia<T>()` ou conversão tipada
de entrada, ambos adiados para P1.

### Renomear nomes publicados ou usar spans por caractere

Rejeitada. Trocar `caracter`, `registro`, `enum`, `Opcao`, `Resultado` ou
`escolha`, ou medir spans em caracteres em vez de UTF-8 em bytes, quebraria
documentação, códigos já publicados e diagnósticos existentes sem benefício
para esta fatia.

### Novo tipo ou opcode para `escreval`

Rejeitada. `escreval(a, b);` equivale a `escreva(a, b, "\n")`, preservando
`StmtKind::Write`, `Op::Write` e a equivalência observável entre VM e Rust
gerado.

## Consequências

- As RFCs 001–006 referenciam este ADR por
  `../adr/0018-contrato-minimo-de-tipos-pors.md`.
- Qualquer mudança nos pontos congelados acima — inclusive larguras, overflow,
  aliases, `nunca`, Unicode e heap/GC — exige novo ADR antes de entrada em
  código.
