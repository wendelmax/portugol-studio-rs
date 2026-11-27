# RFC 007: tratamento estruturado de erros no PORS

- **Data:** 2026-11-27
- **Status:** Proposta
- **Relacionados:** [ADR 0018](../../adr/0018-contrato-minimo-de-tipos-pors.md), [RFC 005](2026-11-27-rfc-005-retorno-implicito.md), [RFC 006](2026-11-27-rfc-006-realce-de-sintaxe.md), [RFC 008](2026-11-27-rfc-008-coalescencia-e-navegacao-segura.md), [design de evolução sintática inicial do PORS](2026-11-26-pors-syntax-evolution-design.md)

## Contexto

O PORS já tem erros como valor: `Resultado<T, E>` com as variantes `Ok` e `Erro`, construtores tipados, `escolha` exaustivo e o operador `?`. Em `crates/portugol-rs-frontend/src/parser/expr.rs:60-106`, `?` sobre `Resultado<T, E>` extrai `T` quando a função retorna `Resultado<_, E>` com o mesmo tipo de erro; fora disso, o diagnóstico é `pors.try-return-mismatch` ou `pors.try-invalid-type`. Não existe `tente`, `capture`, `finalmente` nem `lance`, e a VM não tem pilha de tratadores.

A proposta é dar ao estudante a forma familiar do Java e do C# sem abandonar o modelo explícito e tipado do Rust. A distinção arquitetural é preservada: `Resultado::Erro` é um valor quando retornado, e só se torna transferência de controle quando um `?`, um `lance` ou uma captura o transforma em fluxo.

O ADR 0018 congela `enum`, `Resultado` e `escolha`, mantém `vazio` apenas como tipo de retorno e adia aliases. A RFC 005 define a expressão final sem `;` como retorno implícito; a RFC 006 fixa a taxonomia de realce. Esta RFC adiciona construtos sobre esse contrato, sem renomear nada e sem criar opcode, VM, crate ou API WASM nova.

## Decisão

### Nomes e valores preservados

1. O tipo é `Resultado<T, E>` e as variantes são `Resultado::Ok(valor)` e `Resultado::Erro(erro)`, cada uma com exatamente um valor. Não há alias `OK`, nem `Resultado::Erro` sem argumento, nem valor unitário `()`.
2. Tipos de erro são qualquer tipo PORS, inclusive `enum` definido pelo usuário: `enum ErroMatematico { DivisaoPorZero, Estouro, ValorInvalido(cadeia) }`. Não se introduce `enumeracao` como sinônimo de `enum`.
3. Pattern matching continua sendo `escolha`. Não se introduce `corresponda`.
4. `registro` permanece o nome do tipo composto; `estrutura` não vira sinônimo.
5. `leia()` continua `cadeia` fixa, sem `leia<T>()`. `vazio` continua restrito a tipo de retorno e não aparece como argumento de `Resultado`.

### `lance`

1. Sintaxe: `lance expressao;`. A expressão é o **payload do erro**, não um `Resultado`.
2. Requer `expressao` do tipo `E` e um destino para o erro: função com retorno declarado `Resultado<T, E>` ou `tente` envolvente com tratador compatível. Sem destino, `pors.throw-return-mismatch`, com span na expressão.
3. Fora de `tente`, `lance e;` equivalente a `retorne Resultado::Erro(e);`. Dentro de `tente`, transfere o controle para o primeiro `capture` compatível da região mais interna.
4. `lance Resultado::Erro("x");` é erro de tipo: o payload já é o erro, não um resultado. O diagnóstico é `pors.throw-return-mismatch`, com span no argumento.

### `tente` como comando

1. Gramática:

   ```text
   tente bloco
   { capture [padrao] bloco }+
   [ finalmente bloco ]
   ```

   A região protegida é o bloco imediatamente após `tente`. Um `capture` sem padrão introduce um nome que recebe o valor do erro (`capture erro { ... }`). Um `capture` com padrão casa uma variante do tipo de erro (`capture ErroMatematico::DivisaoPorZero { ... }`, `capture ErroMatematico::ValorInvalido(motivo) { ... }`). `capture _ { ... }` é cura-tudo e deve ser o último.
2. O tipo de erro `E` da região é unificado a partir dos operandos de `?` e dos payloads de `lance` no corpo protegido, e confirmado pelos padrões de `capture` e pelo retorno `Resultado<_, E>` da função quando presente. Fontes de erro com tipos diferentes no mesmo corpo geram `pors.catch-error-type-mismatch`, com span na segunda fonte. Sem fonte no corpo, `E` vem do padrão de `capture` ou do retorno da função.
3. É obrigatório haver ao menos um `capture` ou um `finalmente`. Um `tente` só com `finalmente` executa a finalização e propaga o erro conforme a assinatura da função. Um erro sem `capture` compatível continua para o `tente` externo mais próximo; o `finalmente` do nível interno executa antes dessa entrega. O retorno `Resultado<_, E>` da função é usado apenas como último destino.
4. Os blocos de `capture` e `finalmente` não pertencem à região protegida para efeito de `?` ou `lance`: um erro originado neles mira o `tente` externo mais próximo, ou o retorno da função, nunca o próprio tratador.
5. Os `capture` são testados na ordem do fonte; o primeiro compatível vence. Um `capture` depois de `capture _` gera `pors.catch-unreachable`, com span na palavra `capture`.
6. Padrões de variante só são válidos quando `E` é `enum` com essa variante; caso contrário, `pors.catch-pattern-invalid`, com span no padrão.
7. Cobertura: se `E` é `enum` e nenhum cura-tudo existe, todos os casos devem ser cobertos **salvo** quando a função retorna `Resultado<_, E>`; nesse caso, variantes não capturadas são propagadas e não há `pors.catch-incomplete`. Sem retorno compatível, a cobertura incompleta gera `pors.catch-incomplete`, com span na palavra `tente`. Para `E` que não é `enum`, a ausência de cura-tudo exige retorno `Resultado<_, E>` compatível, sob pena de `pors.try-return-mismatch` no `tente`.
8. A entrada `inicio` continua sem retorno e não passa a aceitar `Resultado<...>`. Um erro que escapa de toda a cadeia de `tente` e não encontra retorno compatível termina a execução com diagnóstico de execução em português, sem pânico, sem código de saída numérico próprio e sem mudança em `compile_program` ou `run()`. Erros fatais da VM, como encerramento forçado, ficam fora dessa garantia.

### `?` dentro de `tente`

1. Sobre `Resultado<T, E>`: quando o `?` está lexicamente dentro de `tente` cujo tipo de erro é `E`, `Ok(valor)` extrai `valor` e `Erro(erro)` transfere para o primeiro `capture` compatível. A exigência de que a função retorne `Resultado<_, E>` fica satisfeita pelo tratador, e não pela assinatura.
2. Se não há tratador compatível no `tente` atual, o erro tenta o `tente` externo mais próximo; só se nenhum tratador externo existir vale a regra atual, exigindo que a função retorne `Resultado<_, E>`. Sem tratador e sem retorno compatível, `pors.try-return-mismatch` no `?`.
3. Sobre `Opcao<T>`, `?` continua exigindo retorno `Opcao<_>`. `tente` captura erro, não ausência; `Opcao::Nenhum` não é redirecionado para `capture`. A RFC 008 define `??` para ausência.

### `finalmente`

1. O bloco `finalmente` executa após o sucesso do corpo, após qualquer `capture` e antes de um `retorne`, `lance`, `pare`, `continue` ou erro propagado que tenha saído da região protegida. O fluxo pendente é retomado ao final.
2. `finalmente` não produz valor e não altera o valor do `capture` ou da expressão `tente`.
3. `retorne`, `lance`, `pare` e `continue` são proibidos dentro de `finalmente`: `pors.finally-control-flow`, com span na palavra-chave do comando. Isso evita perda silenciosa do erro pendente.
4. `?` dentro de `finalmente` é permitido, mas mira tratador externo ou o retorno da função, nunca o `tente` ao qual aquele `finalmente` pertence.
5. `finalmente` não é garantido contra falha fatal da VM.

### `tente` como expressão

1. `tente { ... } capture ... [finalmente { ... }]` também pode aparecer onde expressão é esperada: inicializador de `seja`, argumento de chamada ou posição de retorno/expressão final conforme a RFC 005.
2. Na forma de comando, os blocos são listas de comandos e não têm expressão final. Na forma de expressão, um ramo pode divergir com `retorne` ou `lance` sem produzir valor; os ramos que produzem valor devem concordar. Divergência gera `pors.catch-type-mismatch`, com span na expressão do ramo divergente.
3. `finalmente` não participa do tipo do resultado.
4. Para desambiguar as duas formas: na forma de comando, `tente ...` não é seguido de `;`; na forma de expressão usada como comando, o `;` final é obrigatório e o valor é descartado.
5. `tente` em expressão não é aceito dentro de interpolação de cadeia nesta RFC: o scanner textual da RFC 004 fecha no primeiro `}` não escapado. Esse uso gera `pors.interpolation-invalid` no conteúdo da interpolação.

### Bytecode, VM e Rust gerado

1. Não há opcode novo, `Handler Stack` nem enum `ControlFlow` nesta RFC. `tente` é rebaixado para desvios e rótulos já existentes: teste de variante seguido de `Op::Jump`/`Op::JumpIfFalse`, com os ramos de `capture` emitidos como blocos do mesmo quadro.
2. `?` dentro de `tente` é rebaixado como teste de `Resultado::Ok`/`Resultado::Erro` com desvio ao rótulo do tratador, em vez de `Op::Return`. `lance` fora de `tente` usa `Op::Return` com o `Resultado::Erro` construído.
3. `finalmente` é duplicado pelo compilador em cada saída normal, de `capture` ou propagada, incluindo antes de `Op::Return`. A duplicação é tempo de compilação; em tempo de execução cada saída passa uma única vez pelo bloco.
4. A representação de `Resultado` continua sendo a já existente (`Value::Array` com discriminante e `PorsDebugShape::Enum`); spans e source maps permanecem UTF-8 em bytes. Rust gerado usa `match` sobre `Result<T, E>` e o mesmo texto de diagnóstico da VM. Nenhuma API WASM nova.

### Realce, formatador e editores

1. `tente`, `capture`, `finalmente` e `lance` entram na classe `keyword` da RFC 006 na IDE e no playground, sem variável CSS nova.
2. O lexer passa a reconhecer as quatro palavras como reservadas; o formatador usa o mesmo espaçamento de outras palavras-chave seguidas de bloco; completions e hover documentam os quatro construtos.
3. Nenhum opcode, helper de Rust gerado, crate, VM ou API WASM é adicionado.

## Exemplos válidos

```pors
funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se b == 0 {
        lance "Divisão por zero";
    }

    Resultado::Ok(a / b)
}

funcao inicio() {
    tente {
        seja valor = dividir(10, 0)?;
        escreval("Resultado: {valor}");
    } capture erro {
        escreval("Falha: {erro}");
    } finalmente {
        escreval("Operação encerrada.");
    }
}
```

```pors
enum ErroMatematico {
    DivisaoPorZero,
    Estouro,
    ValorInvalido(cadeia)
}

funcao dividir(a: inteiro, b: inteiro)
    -> Resultado<inteiro, ErroMatematico> {
    se b == 0 {
        lance ErroMatematico::DivisaoPorZero;
    }

    Resultado::Ok(a / b)
}

funcao inicio() {
    tente {
        seja valor = dividir(10, 0)?;
        escreval(valor);
    } capture ErroMatematico::DivisaoPorZero {
        escreval("Não é possível dividir por zero.");
    } capture ErroMatematico::Estouro {
        escreval("Limite numérico excedido.");
    } capture erro {
        escreval("Erro inesperado: {erro}");
    } finalmente {
        escreval("Execução finalizada.");
    }
}
```

```pors
funcao inicio() {
    seja numero = tente {
        dividir(10, 2)?
    } capture erro {
        0
    } finalmente {
        escreval("Leitura encerrada.");
    };

    escreval("Número: {numero}");
}
```

## Exemplos inválidos

| Trecho (coluna 0) | Diagnóstico | Motivo |
| --- | --- | --- |
| `funcao inicio() { lance "falha"; }` | `pors.throw-return-mismatch` em `[24, 31)` | `inicio` não retorna `Resultado<_, cadeia>` |
| `funcao f() -> cadeia { seja y = tente { 1 } capture erro { "x" }; retorne y; }` | `pors.catch-type-mismatch` em `[59, 62)` | ramo de captura produz `cadeia`, corpo produz `inteiro` |
| `funcao inicio() { tente { 1 } finalmente { retorne; } }` | `pors.finally-control-flow` em `[43, 50)` | `retorne` sai de `finalmente` |
| `funcao inicio() { seja x = Opcao::Algum(1)?; }` | `pors.try-return-mismatch` no `?` | `tente` não captura ausência e `inicio` não retorna `Opcao<_>` |
| `funcao inicio() { tente { 1 } capture _ { 2 } capture erro { 3 }; }` | `pors.catch-unreachable` no segundo `capture` | cura-tudo já cobriu o erro |

## Decisões de escopo (chamadas do autor)

- `E` é inferido das fontes de erro do corpo e confirmado pelos padrões e pelo retorno da função; não há anotação de tipo em `tente`.
- Blocos de `capture` e `finalmente` não são região protegida para efeito de `?` ou `lance`, evitando reentrada no próprio tratador.
- Um erro não tratado internamente procura o `tente` externo antes de cair no retorno da função; cada `finalmente` interno roda antes dessa entrega.
- `inicio` permanece sem retorno; erro escapado é diagnóstico de execução da VM, não mudança de assinatura.
- `retorne`, `lance`, `pare` e `continue` continuam proibidos dentro de `finalmente`; `?` é permitido, mas mira para fora.
- Blocos da forma de comando não têm expressão final; ramos da forma de expressão podem divergir sem valor.
- `tente` não entra em interpolação de cadeia nesta fatia, para não exigir scanner com reconhecimento de chaves aninhadas.

## Consequências

- As RFCs 001–006 não mudam. `?` sobre `Resultado` ganha uma segunda forma de satisfazer a exigência de retorno, apenas dentro de `tente`.
- `lance` é açúcar de `retorne Resultado::Erro(...)` fora de `tente`, e um desvio ao tratador dentro dele. Não há valor, opcode ou tipo novo.
- A identidade híbrida fica explícita: Java/C# fornecem a forma `tente/capture/finalmente`/`lance`; Rust fornece `Resultado`, `?`, `escolha` e erro tipado. Nenhuma das duas formas esconde o tipo do erro.
- Os casos ausência (`Opcao`) e erro (`Resultado`) continuam separados. A RFC 008 cobre a primeira família.

## Abordagens consideradas

1. **Compatibilidade sobre o contrato publicado (escolhida).** Manter `Resultado`/`Ok`/`Erro`, `enum`, `escolha`, `registro` e `leia()`, adicionando `tente/capture/finalmente`, `lance` e captura tipada. O rebaixamento usa desvios e duplicação de `finalmente`, sem opcode novo. Trade-off: a sintaxe não usa `OK`, `enumeracao` ou `corresponda` como no texto original, mas exige zero migração de código publicado e nenhum ADR de revogação.
2. **Proposta literal, com aliases e instruction set novo.** Adotar `Resultado::OK`, `enumeracao`, `corresponda`, `TRY_BEGIN`/`THROW`/`Handler Stack` e `vazio` como argumento de tipo. Trade-off: mais próximo do texto motivador, mas revoga partes do ADR 0018, exige valor unitário e multiplica a superfície de testes em frontend, bytecode, VM, Rust gerado, WASM, IDE e playground.
3. **Duas RFCs separadas, uma para `tente` e outra para `lance`/erro tipado.** Trade-off: documentos menores, mas o contrato de `?` e a precedência entre tratadores ficariam divididos, com risco de divergência entre as duas peças.
