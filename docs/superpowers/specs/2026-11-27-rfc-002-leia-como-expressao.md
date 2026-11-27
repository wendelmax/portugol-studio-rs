# RFC 002: `leia()` como expressão

- **Data:** 2026-11-27
- **Status:** Aceita
- **Relacionados:** [ADR 0018](../../adr/0018-contrato-minimo-de-tipos-pors.md), [RFC 001](2026-11-27-rfc-001-inferencia-em-seja.md), [design de evolução sintática inicial do PORS](2026-11-26-pors-syntax-evolution-design.md)

## Contexto

Hoje `leia` só é aceito como comando: `leia(variavel);` gera `StmtKind::Read` e usa o valor lido para atualizar a variável. O bytecode já tem `Op::Read(ValueType)` em `crates/portugol-bytecode/src/op.rs:166-167`, que empilha o valor lido, e o exportador Rust já emite `__portugol_read_line` em `crates/portugol-rust/src/codegen/mod.rs:50`, que anuncia `PORTUGOL_INPUT:{expected}` quando o protocolo de entrada está ativo. Falta expor a leitura como expressão sem quebrar o comando legado.

O ADR 0018 congela o contrato: `leia()` tem tipo `cadeia` fixo, contexto nunca altera esse tipo, `leia(variavel)` mantém o tipo da variável, `inteiro` continua `i32` com `wrapping` e spans são offsets UTF-8 em bytes. A RFC 001 depende desta decisão ao afirmar que `seja nome = leia();` infere `cadeia` pela expressão, não pelo `seja`.

## Decisão

### Sintaxe e forma legada

1. `leia()` é uma chamada sem argumentos, aceita em qualquer posição de expressão: inicializador de `seja`, argumento de chamada, expressão de retorno, operando e interpolação (esta última definida na RFC 004). Entre parênteses só são permitidos espaços em branco.
2. O span da chamada cobre `leia` até o `)` final, incluindo os parênteses; é esse span que os diagnósticos abaixo apontam.
3. A forma legada `leia(variavel);` permanece inalterada como comando, com `StmtKind::Read` mantido como açúcar de compatibilidade. Ela não é convertida em `ExprKind::Read` nesta fatia, continua com o tipo da variável e não gera diagnóstico novo.
4. `leia(argumento)` em posição de expressão não é reinterpretado como comando nem como leitura tipada: gera `pors.read-arguments-unsupported` no span da chamada.

### Tipos

1. A leitura sem argumento vira `ExprKind::Read` na HIR, com tipo fixo `Type::String` (`cadeia`). Não há parâmetro de tipo, conversão implícita ou inferência a partir do contexto.
2. Declaração, parâmetro e retorno nunca mudam esse tipo. `seja nome = leia();` infere `cadeia`, como especificado na RFC 001; `seja mensagem: cadeia = leia();`, `saudar(leia())` com parâmetro `cadeia` e `retorne leia();` em função `-> cadeia` são válidos.
3. Quando o contexto exige outro tipo — por exemplo `seja n: inteiro = leia();`, argumento de parâmetro `inteiro` ou `retorne leia();` em função `-> inteiro` — o diagnóstico é `pors.type-mismatch` no span da chamada. Não há parse automático de texto para `inteiro`: `inteiro` permanece `i32` com `wrapping`, conforme ADR 0018.

### Bytecode, VM e Rust gerado

1. O backend reusa `Op::Read(ValueType)` com o tipo de cadeia; não há opcode, VM, crate, produto ou API WASM nova, nem pasta `specs/` paralela. Os source maps passam a registrar o span da chamada também quando a leitura está dentro de uma expressão.
2. A varredura de `has_read` no exportador Rust passa a enxergar `ExprKind::Read` dentro de expressões, inclusive aninhadas, para que o helper `__portugol_read_line` seja emitido mesmo quando `leia()` não aparece como comando.
3. O helper continua chamado com `"cadeia"` e, quando `PORTUGOL_STUDIO_INPUT_PROTOCOL` está ativo, anuncia `PORTUGOL_INPUT:cadeia` antes de ler a linha; o valor devolvido é a própria cadeia, sem parse tipado. Não há campo JSON ou nova API na fachada WASM.

### Spans e diagnósticos

Spans são offsets UTF-8 semiabertos `[início, fim)` em bytes, medidos a partir da coluna 0, inclusive com acentos. O span da chamada é o mesmo em HIR, bytecode/VM e Rust gerado.

Exemplos válidos, com o span da chamada `leia()`:

| Trecho | Span |
| --- | --- |
| `seja nome = leia();` | `[12, 18)` |
| `seja mensagem: cadeia = leia();` | `[24, 30)` |
| `escreva(leia());` | `[8, 14)` |
| `escreva(saudar(leia()));` | `[15, 21)` |
| `retorne leia();` (em função `-> cadeia`) | `[8, 14)` |
| `leia(nome);` (comando legado) | `[0, 10)` |

```pors
funcao saudar(nome: cadeia) -> cadeia {
    retorne nome;
}

funcao inicio() {
    seja nome = leia();
    escreva(saudar(leia()));
    escreva(nome);
    leia(nome);
}
```

Exemplos inválidos:

| Trecho (coluna 0) | Diagnóstico |
| --- | --- |
| `seja n: inteiro = leia();` | `pors.type-mismatch` em `[18, 24)` |
| `seja café: inteiro = leia();` | `pors.type-mismatch` em `[22, 28)`; `é` ocupa 2 bytes |
| `escreva(saudar(leia()));` com `saudar(nome: inteiro)` | `pors.type-mismatch` em `[15, 21)` |
| `retorne leia();` em função `-> inteiro` | `pors.type-mismatch` em `[8, 14)` |
| `seja nome = leia(nome);` | `pors.read-arguments-unsupported` em `[12, 22)` |

### Escopo adiado e fronteiras

- `leia<T>()` e conversão ou validação tipada da entrada ficam em **P1**, sem sintaxe reservada nesta fatia.
- Interpolação de cadeias é decidida na RFC 004 e retorno implícito na RFC 005; esta RFC não altera essas regras. `escreval(a, b);` é apenas açúcar para `escreva(a, b, "\n")`, conforme ADR 0018 e RFC 003, sem tipo, opcode ou tratamento de leitura novo.
- Permanecem adiados pelo ADR 0018: novas larguras de inteiro e a decisão sobre overflow, aliases, `nunca`, Unicode além dos offsets de bytes dos spans e mudanças em heap/GC.

## Consequências

- Os testes de aceite devem executar os exemplos em HIR, bytecode/VM e Rust gerado, comparando saída observável e spans UTF-8, sem novo contrato de WASM.
- `leia(variavel);` continua válido e testado; o protocolo de entrada da IDE deve continuar funcionando quando a leitura acontece dentro de uma expressão.
- Qualquer mudança no tipo fixo `cadeia`, no comando legado ou nos diagnósticos acima exige revisão desta RFC e do ADR 0018 antes de entrar em código.
