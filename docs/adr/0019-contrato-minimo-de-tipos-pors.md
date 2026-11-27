# ADR 0019: Contrato mínimo de tipos do PORS

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

O estado vigente, com a revisão registrada na Decisão 7, é: `inteiro` é `i32` com
`wrapping` em HIR/VM/Rust gerado; a leitura com variável-alvo é comando e a forma
`leia()` infere o tipo esperado pelo contexto, resultando em `cadeia` quando não
há contexto tipado; entrada inválida gera erro de execução, sem valor padrão
silencioso. Os nomes `caracter`, `registro`, `enum`, `Opcao`, `Resultado` e
`escolha` já estão publicados; spans são UTF-8 em bytes; `vazio` aparece apenas
como tipo de retorno. `escreval(a, b);` acrescenta uma quebra final à escrita de
`escreva(a, b)`, sem tipo ou opcode novo.

Larguras adicionais, regras de overflow, aliases, suporte a Unicode e modelo de
memória são decisões maiores que não devem ser tomadas sob pressão do exemplo
motivador.

## Decisão

1. `inteiro` continua `i32` com `wrapping` em HIR, bytecode/VM e Rust gerado
   até decisão posterior. Nenhuma largura nova é introduzida nesta fatia.
2. `leia()` infere o tipo do contexto — inicialização de `seja` com tipo
   anotado, parâmetro, retorno, atribuição ou campo — e resulta em `cadeia`
   sem contexto tipado; não há `leia<T>()`. A forma com variável-alvo converte
   implicitamente ao tipo declarado da variável. Entrada que não converte gera
   erro de execução com mensagem sobre o tipo esperado, sem valor padrão
   silencioso.
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
7. **Tipo de `leia()` dependente do contexto — aceita nesta fatia.** A opção
   antes rejeitada foi adotada: `leia()` converte implicitamente ao tipo
   esperado pelo contexto e a forma com variável-alvo converte ao tipo da
   variável. O contrato mínimo anterior, que congelava `leia()` como `cadeia`
   fixa, foi superado; sem contexto tipado a leitura continua resultando em
   `cadeia`. Entrada inválida é erro de execução, não valor padrão nem erro
   estático de tipo.

## Alternativas consideradas

### ADR completo de tipos antes de qualquer sintaxe

Rejeitada. Largura, overflow, aliases e heap/GC exigem decisões que não podem
ser bem tomadas sob pressão do exemplo motivador, que só usa `cadeia`. O
contrato mínimo congela o que já existe e explicita o adiamento.

### Tipo de `leia()` dependente do contexto

Não é mais rejeitada: foi aceita nesta fatia e registrada como Decisão 7. A
justificativa anterior — ambiguidade em `seja nome = leia();` e necessidade de
`leia<T>()` ou conversão tipada explícita — deixa de valer porque a leitura sem
contexto continua `cadeia` e a conversão implícita resolve o contexto tipado.

### Renomear nomes publicados ou usar spans por caractere

Rejeitada. Trocar `caracter`, `registro`, `enum`, `Opcao`, `Resultado` ou
`escolha`, ou medir spans em caracteres em vez de UTF-8 em bytes, quebraria
documentação, códigos já publicados e diagnósticos existentes sem benefício
para esta fatia.

### Novo tipo ou opcode para `escreval`

Rejeitada. `escreval(a, b);` escreve `a`, `b` e uma quebra final como `escreva`,
preservando `StmtKind::Write`, `Op::Write` e a equivalência observável entre VM
e Rust gerado.

## Consequências

- As RFCs 001–006 referenciam este ADR por
  `../adr/0019-contrato-minimo-de-tipos-pors.md`.
- Qualquer mudança nos pontos congelados acima — inclusive larguras, overflow,
  aliases, `nunca`, Unicode e heap/GC — exige novo ADR antes de entrada em
  código.
- A revisão do tipo de `leia()` é registrada também na RFC 002 e na
  [matriz de migração](../research/PORS_MIGRATION_MATRIX.md), que documenta as
  diferenças observadas em `caracter` e `logico` e as exceções da allowlist.
