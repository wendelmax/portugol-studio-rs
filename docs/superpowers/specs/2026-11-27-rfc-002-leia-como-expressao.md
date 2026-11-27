# RFC 002: `leia()` como expressão

- **Data:** 2026-11-27
- **Status:** Aceita
- **Relacionados:** [ADR 0019](../../adr/0019-contrato-minimo-de-tipos-pors.md), [RFC 001](2026-11-27-rfc-001-inferencia-em-seja.md), [design de evolução sintática inicial do PORS](2026-11-26-pors-syntax-evolution-design.md)
- **Revisada por:** [ADR 0020](../../adr/0020-ajustes-pedagogicos-da-sintaxe-pors.md)

> **Revisão (ADR 0020):** Revisada pela decisão 3 do ADR 0020. A regra "tipo fixo `cadeia`" vale só sem destino tipado; com destino declarado (inicializador anotado, atribuição, argumento de parâmetro e `retorne`), a entrada é convertida com as regras da forma com variável-alvo. Os exemplos inválidos com destino `inteiro` desta RFC passam a ser válidos.

## Contexto

Hoje `leia` só é aceito como comando: a forma com variável-alvo gera
`StmtKind::Read` e usa o valor lido para atualizar a variável. O bytecode já tem
`Op::Read(ValueType)` em `crates/portugol-bytecode/src/op.rs:166-167`, que
empilha o valor lido, e o exportador Rust já emite `__portugol_read_line` em
`crates/portugol-rust/src/codegen/mod.rs:50`, que anuncia
`PORTUGOL_INPUT:{expected}` quando o protocolo de entrada está ativo. Falta
expor a leitura como expressão sem quebrar o comando legado.

O ADR 0019 registra o contrato revisado: `leia()` infere o tipo do contexto e
resulta em `cadeia` sem contexto tipado; a forma com variável-alvo converte
implicitamente ao tipo da variável; entrada inválida gera erro de execução;
`inteiro` continua `i32` com `wrapping` e spans são offsets UTF-8 em bytes. A
RFC 001 depende desta decisão: `seja nome = leia();` infere `cadeia` porque não
há tipo esperado, não por causa do `seja`.

## Decisão

### Sintaxe e forma legada

1. `leia()` é uma chamada sem argumentos, aceita em qualquer posição de
   expressão: inicializador de `seja`, argumento de chamada, expressão de
   retorno, operando e interpolação (esta última definida na RFC 004). Entre
   parênteses só são permitidos espaços em branco.
2. O span da chamada cobre `leia` até o `)` final, incluindo os parênteses; é
   esse span que os diagnósticos abaixo apontam.
3. A forma legada com variável-alvo permanece inalterada como comando, com
   `StmtKind::Read` mantido como açúcar de compatibilidade. Ela não é convertida
   em `ExprKind::Read` nesta fatia, continua com o tipo da variável e não gera
   diagnóstico novo. Nos exemplos migrados ela fica reservada à compatibilidade
   e a laços com leitura repetida.
4. `leia` com argumento em posição de expressão não é reinterpretado como
   comando nem como leitura tipada: gera `pors.read-arguments-unsupported` no
   span da chamada. A conversão de tipo é implícita na própria leitura, não
   escrita como argumento.

### Tipos

1. A leitura sem argumento vira `ExprKind::Read` na HIR, com tipo inferido do
   contexto esperado; sem contexto, `Type::String` (`cadeia`). Não há parâmetro
   de tipo; a conversão implícita segue as regras dos helpers `Tipos.para_*`.
   Escalares passam por `trim` antes da conversão; `cadeia` permanece bruta.
2. Declaração tipada, parâmetro, retorno, atribuição e campo fornecem o tipo
   esperado: `seja n: inteiro = leia();` lê e converte para `inteiro`, enquanto
   `seja nome = leia();` infere `cadeia`, como na RFC 001. `caracter` exige
   exatamente um caractere e `logico` aceita espaços ao redor.
3. Entrada que não converte ao tipo esperado gera erro de execução com mensagem
   sobre o tipo, sem valor padrão silencioso. Não há erro de tipo estático por
   usar `leia()` em contexto `inteiro`. `leia<T>()`, validação tipada explícita
   adiada para P1, não é necessária em leituras comuns; `inteiro` permanece
   `i32` com `wrapping`, conforme ADR 0019.

### Bytecode, VM e Rust gerado

1. O backend reusa `Op::Read(ValueType)` com o tipo inferido; não há opcode,
   VM, crate, produto ou API WASM nova, nem pasta `specs/` paralela. Os source
   maps passam a registrar o span da chamada também quando a leitura está
   dentro de uma expressão.
2. A varredura de `has_read` no exportador Rust passa a enxergar
   `ExprKind::Read` dentro de expressões, inclusive aninhadas, para que o
   helper `__portugol_read_line` seja emitido mesmo quando `leia()` não aparece
   como comando.
3. O helper continua anunciando `PORTUGOL_INPUT:{expected}` antes de ler a
   linha quando `PORTUGOL_STUDIO_INPUT_PROTOCOL` está ativo; `{expected}` é o
   tipo inferido (`cadeia` na ausência de contexto). A conversão observável é a
   do tipo esperado; não há campo JSON ou nova API na fachada WASM.

### Spans e diagnósticos

Spans são offsets UTF-8 semiabertos `[início, fim)` em bytes, medidos a partir
da coluna 0, inclusive com acentos. O span da chamada é o mesmo em HIR,
bytecode/VM e Rust gerado.

Exemplos válidos, com o span da chamada `leia()`:

| Trecho | Span |
| --- | --- |
| `seja nome = leia();` | `[12, 18)` |
| `seja n: inteiro = leia();` | `[18, 24)` |
| `seja mensagem: cadeia = leia();` | `[24, 30)` |
| `escreval(leia());` | `[9, 15)` |
| `escreval(saudar(leia()));` | `[16, 22)` |
| `retorne leia();` (em função `-> cadeia` ou `-> inteiro`) | `[8, 14)` |

O bloco abaixo usa apenas `leia()`. A forma legada com variável-alvo continua
válida, mas é descrita em prosa para não reintroduzir a grafia antiga nos
exemplos publicados; ela fica reservada à compatibilidade e a laços com leitura
repetida, conforme o design de migração.

```pors
funcao saudar(nome: cadeia) -> cadeia {
    retorne nome;
}

funcao inicio() {
    seja nome = leia();
    escreval(saudar(leia()));
    escreval(nome);
}
```

Exemplos inválidos:

| Trecho (coluna 0) | Diagnóstico |
| --- | --- |
| `seja café: inteiro = leia();` com entrada não numérica | erro de execução, não diagnóstico estático; `é` ocupa 2 bytes no span |
| `leia` com argumento em inicializador sem anotação | `pors.read-arguments-unsupported` no span da chamada |
| `leia` com argumento textual dentro de `escreval` | `pors.read-arguments-unsupported` no span da chamada |

### Escopo adiado e fronteiras

- `leia<T>()` e validação tipada explícita da entrada ficam em **P1**, sem
  sintaxe reservada nesta fatia. O caminho explícito com `Tipos.para_*` e
  `Resultado` continua disponível, em especial para `UI.entrada`.
- Interpolação de cadeias é decidida na RFC 004 e retorno implícito na RFC 005;
  esta RFC não altera essas regras. `escreval(a, b);` acrescenta apenas uma
  quebra final à escrita de `escreva(a, b)`, conforme ADR 0019 e RFC 003, sem
  tipo, opcode ou tratamento de leitura novo.
- Permanecem adiados pelo ADR 0019: novas larguras de inteiro e a decisão sobre
  overflow, aliases, `nunca`, Unicode além dos offsets de bytes dos spans e
  mudanças em heap/GC.

## Consequências

- Os testes de aceite devem executar os exemplos em HIR, bytecode/VM e Rust
  gerado, comparando saída observável e spans UTF-8, sem novo contrato de WASM.
- A forma com variável-alvo continua válida e testada; o protocolo de entrada
  da IDE deve continuar funcionando quando a leitura acontece dentro de uma
  expressão.
- Qualquer mudança no tipo inferido, na conversão implícita, no comando legado
  ou nos diagnósticos acima exige revisão desta RFC e do ADR 0019 antes de
  entrar em código.
