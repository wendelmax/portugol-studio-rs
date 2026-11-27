# RFC 005: retorno implícito

- **Data:** 2026-11-27
- **Status:** Aceita
- **Relacionados:** [ADR 0018](../../adr/0018-contrato-minimo-de-tipos-pors.md), [RFC 001](2026-11-27-rfc-001-inferencia-em-seja.md), [RFC 002](2026-11-27-rfc-002-leia-como-expressao.md), [RFC 003](2026-11-27-rfc-003-escreval.md), [RFC 004](2026-11-27-rfc-004-interpolacao-de-cadeias.md), [design de evolução sintática inicial do PORS](2026-11-26-pors-syntax-evolution-design.md)

## Contexto

Hoje o corpo de uma função com retorno declarado exige `retorne valor;` como última instrução: `guarantees_return()` em `crates/portugol-rs-frontend/src/parser/constants.rs:270-283` reconhece apenas `StmtKind::Return` e blocos aninhados com garantia, e sem isso o frontend emite `pors.missing-return`.

O design aprovado pede a última expressão do corpo como retorno implícito, com o exemplo motivador:

```pors
funcao saudar(nome: cadeia) -> cadeia {
    "Olá, {nome}!"
}
```

O ADR 0018 congela os pontos de tipo e spans; a RFC 004 formalizou a interpolação de cadeias; esta RFC fixa o contrato observável do retorno implícito no frontend, na HIR, no bytecode/VM, no Rust gerado, no formatador e nos editores. Não define o realce de sintaxe (RFC 006).

## Decisão

### Sintaxe e forma final

A **cauda** é a expressão final sem `;` do corpo de função.

1. A regra vale **somente para o corpo da função**: se a última posição do corpo de uma função com tipo de retorno declarado (`Function.return_type: Option<Type>`) for uma expressão sem `;`, essa expressão é a cauda e passa a ser retorno implícito. A única exceção são os ramos de valor de `tente` em expressão (RFC 007), cuja expressão final sem `;` é valor do ramo, não retorno da função.
2. O frontend guarda a expressão final em `Function.tail_expression: Option<Expr>`; a cauda não materializa como nó em `Function.body`; o backend converte a cauda em `StmtKind::Return { value }`, nomeando a conversão semântica nos backends.
3. Blocos aninhados (`se`, `senao`, `escolha`, laços) não ganham retorno implícito: neles `retorne valor;` continua obrigatório e a falta de `retorne` segue diagnosticada por `pors.missing-return`. A única exceção são os ramos de valor de `tente` em expressão (RFC 007), corpo protegido e `capture`: a expressão final sem `;` nesses ramos é o valor do ramo e não vira `retorne` da função.
4. `retorne valor;` continua válido em qualquer posição com retorno declarado; o `;` do `retorne` segue exigido pelo parser.
5. Funções sem retorno declarado (`return_type: None`) e blocos vazios não ganham retorno implícito: sem expressão final não há cauda, e a expressão final sem `;` em função sem retorno declarado gera `pors.implicit-return-without-type` no span da expressão.

### Decisão de escopo (chamada do autor)

A exceção para ramos de `tente` em expressão é deliberadamente estreita: ela não estende retorno implícito a `se`, `senao`, `escolha`, laços nem a qualquer outro bloco. Foi registrada como chamada do autor para manter os exemplos da RFC 007 válidos sem alterar a regra do corpo da função.

### HIR, spans e diagnósticos

1. `Function.tail_expression: Option<Expr>` é `Some` somente quando o corpo termina com expressão sem `;` e a função declara retorno; é `None` nos demais casos.
2. Quando a cauda existe em função com retorno declarado, ela satisfaz a garantição de retorno; o tipo da cauda deve coincidir com o retorno declarado, sob pena de `pors.return-type-mismatch` no span da cauda.
3. Quando a expressão final sem `;` aparece em função sem retorno declarado, o diagnóstico é `pors.implicit-return-without-type` no span da expressão.

### Lowering, bytecode, Rust gerado e source maps

1. Em `compile_function()`, com a cauda presente, emitir `self.expr(tail)?` seguido de `Op::Return` (sem `Op::PushVoid`); sem a cauda, manter `Op::PushVoid; Op::Return`. A cauda não é materializada como `StmtKind::Return` no corpo; `StmtKind::Return { value }` nomeia a conversão semântica no backend.
2. No codegen, após `render_statements(&function.body, ...)` e antes do `}` da função, emitir `return <expr>;` com `push_expr(..., owned = true, pors)` e mapeamento no span da cauda.

### Fronteiras e adiamentos

- Nenhum novo opcode, VM, crate, produto, API WASM ou pasta `specs/` paralela.
- `escreval` permanece açúcar de `Write` (RFC 003) e `leia()` tipo fixo `cadeia` (RFC 002); esta RFC não altera esses contratos.
- Retorno implícito é da RFC 005; realce de sintaxe é da RFC 006.

## Exemplos válidos

```pors
funcao somar(x: inteiro, y: inteiro) -> inteiro {
    x + y
}
```

A expressão final sem `;`; o retorno é implícito.

```pors
funcao saudar(nome: cadeia) -> cadeia {
    retorne "Olá, {nome}!";
}
```

`retorne ...;` continua válido.

## Exemplos inválidos

| Trecho (coluna 0) | Motivo | Diagnóstico |
| --- | --- | --- |
| `funcao inicio() { 1 }` | função sem retorno declarado com expressão final sem `;` | `pors.implicit-return-without-type` no span da expressão `[18, 19)` |
| `funcao f() -> cadeia { 1 + 1 }` | cauda com tipo incompatível com o retorno declarado | `pors.return-type-mismatch` no span da cauda `[23, 28)` |

## Consequências

- Os testes de aceite devem verificar o campo `Function.tail_expression`, o `StmtKind::Return { value }` do lowering, os backends, os spans UTF-8 em bytes e a ausência de novo opcode, VM ou API WASM.
- Qualquer mudança na regra de `;`, nos blocos aninhados, nos tipos ou nos diagnósticos acima exige revisão desta RFC e do ADR 0018 antes de entrar em código.
