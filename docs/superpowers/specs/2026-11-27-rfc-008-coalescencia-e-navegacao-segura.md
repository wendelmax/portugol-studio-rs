# RFC 008: coalescência nula e navegação segura no PORS

- **Data:** 2026-11-27
- **Status:** Proposta
- **Relacionados:** [ADR 0018](../../adr/0018-contrato-minimo-de-tipos-pors.md), [RFC 004](2026-11-27-rfc-004-interpolacao-de-cadeias.md), [RFC 006](2026-11-27-rfc-006-realce-de-sintaxe.md), [RFC 007](2026-11-27-rfc-007-tratamento-de-erros.md), [design de evolução sintática inicial do PORS](2026-11-26-pors-syntax-evolution-design.md)

## Contexto

O PORS já é *null-safe* por construção: não há `null`, ponteiro nulo, nem valor `nenhum` na linguagem. A ausência é `Opcao<T>` com `Opcao::Algum(valor)` e `Opcao::Nenhum`, representada no bytecode como arranjo com discriminante (`Value::Array` + `PorsDebugShape::Enum`) e gerada em Rust como `Option<T>`. O operador `?` já existe e propaga `Opcao<T>` ou `Resultado<T, E>` para a função chamadora, com diagnóstico `pors.try-return-mismatch` quando a assinatura não é compatível (`crates/portugol-rs-frontend/src/parser/expr.rs:60-106`).

A proposta recebida pede `??`, `?.` e `??=`, tipos compactos como `cadeia?`, o valor `nenhum` e um `OptionalValue` com opcodes novos na VM. Esta RFC registra a versão compatível: os três operadores entram, os aliases e a sobrecarga de `?` como sufixo de tipo ficam adiados, e a VM reusa a representação existente. A RFC 007 mantém a fronteira entre ausência (`Opcao`) e erro (`Resultado`); `??` não trata erro.

## Decisão

### Nomes e valores preservados

1. O tipo opcional é `Opcao<T>`; os valores são `Opcao::Algum(valor)` e `Opcao::Nenhum`. Não há alias `Opcional`, nem palavra `nenhum`, nem sufixo `cadeia?` nesta RFC — todos exigiriam nova decisão sobre aliases, explicitamente adiada pelo ADR 0018.
2. `registro` continua o nome do tipo composto; `estrutura` não vira sinônimo.
3. `leia()` continua `cadeia` fixa, sem `leia<T>()`. A conversão de entrada continua explícita.
4. `?` continua sendo **somente** propagação de `Opcao` ou `Resultado`. Em posição de tipo, `?` não é aceito; `Opcao<T>` é a grafia canônica.
5. Variáveis mutáveis usam `seja mutavel nome: Opcao<T> = ...;`, conforme o parser atual.

### Coalescência `??`

1. Sintaxe: `esquerda ?? direita`. O operando esquerdo deve ter tipo `Opcao<T>`; o direito deve ter tipo `T`; o resultado tem tipo `T`.
2. `Opcao::Algum(valor) ?? alternativa` produz `valor`; `Opcao::Nenhum ?? alternativa` produz `alternativa`. A alternativa é avaliada somente quando a esquerda é `Opcao::Nenhum`.
3. `??` é associativo à direita: `a ?? b ?? c` equivale a `a ?? (b ?? c)`. Isso preserva a avaliação da esquerda para a direita e a preguiça: em `nome ?? buscar_nome()`, `buscar_nome()` só é chamado se `nome` for `Opcao::Nenhum`.
4. Zero, `falso` e cadeia vazia são valores presentes. `??` reage apenas a `Opcao::Nenhum`.
5. Esquerda que não é `Opcao<T>` gera `pors.coalesce-invalid-type`, com span no operador `??`. Direita de tipo diferente de `T` gera `pors.coalesce-type-mismatch`, com span na direita.
6. `??` não aceita `Resultado<T, E>`. Nesse caso, `pors.coalesce-invalid-type` com span no operador e mensagem orientando a usar `escolha` ou `tente/capture` da RFC 007. O método `ok()`, que descartaria o erro explicitamente, fica adiado.

### Atribuição por coalescência `??=`

1. Sintaxe: `alvo ??= expressao;`, onde `alvo` é o nome de uma variável **mutável** de tipo `Opcao<T>` e `expressao` tem tipo `T`. Campos de registro e elementos de vetor não são alvos nesta fatia, embora sejam alvos de `=`; nesses casos, `pors.coalesce-assign-invalid-type` com span no alvo.
2. Se `alvo` é `Opcao::Nenhum`, recebe `Opcao::Algum(expressao)`; se já é `Opcao::Algum(_)`, nada acontece e `expressao` não é avaliada.
3. `??=` é comando nesta RFC: não produz valor e não pode ser encadeado. A forma de expressão fica adiada.
4. Alvo imutável, inexistente ou de tipo diferente de `Opcao<T>` gera `pors.coalesce-assign-invalid-type`, com span no alvo. Expressão de tipo diferente de `T` gera `pors.coalesce-type-mismatch`, com span na expressão.

### Navegação segura `?.`

1. Sintaxe: `base?.campo`, onde `base` tem tipo `Opcao<T>` e `T` é `registro` com o campo `campo`.
2. Se `base` é `Opcao::Algum(registro)`, o resultado é o valor do campo; se é `Opcao::Nenhum`, o resultado é `Opcao::Nenhum`.
3. O resultado é sempre `Opcao<Tcampo>`, inclusive quando `campo` já é `Opcao<Tcampo>`: `?.` achata exatamente um nível, nunca produz `Opcao<Opcao<_>>`. É o equivalente a `and_then`, não a `map` seguido de `map`.
4. Encadeamento é permitido: `cliente?.endereco?.cidade` aplica um `?.` por vez, achatando um nível em cada passo. O encadeamento é associativo à esquerda.
5. `?.` seguido de `(` não é suportado: o PORS não tem sintaxe de chamada de método em instância. Nesse caso, `pors.safe-call-unsupported`, com span no `?.`. Chamadas seguras ficam adiadas.
6. `base` que não é `Opcao<registro>`, ou campo inexistente, gera `pors.safe-access-invalid-type`, com span no `?.`.
7. `?.` não captura erro: sobre `Resultado`, o diagnóstico é `pors.safe-access-invalid-type`, com a mesma mensagem orientando a RFC 007.

### Conversão implícita para `Opcao<T>`

1. Quando o tipo esperado é conhecido e é exatamente `Opcao<T>`, um valor do tipo `T` é aceito e embrulhado em `Opcao::Algum`. A conversão se aplica somente a inicializador de `seja`, argumento de chamada e valor de retorno: `seja nome: Opcao<cadeia> = "Maria";`, parâmetro `Opcao<inteiro>` chamado com `18`, `retorne 42;` em função `-> Opcao<inteiro>`.
2. Não se aplica a campos de registro, payload de enum, elementos de vetor, alvo de atribuição, operando de `??`, base de `?.` nem ramo de `tente`. Assim, `seja x: Opcao<inteiro> = 1 ?? 2;` continua inválido pelo operando esquerdo, e a atribuição `x = 1;` continua exigindo `Opcao::Algum(1)`.
3. A conversão é de um único nível. Não converte `T` em `Opcao<Opcao<T>>`, nem `Opcao<T>` em `Opcao<Opcao<T>>`, e nunca altera o tipo de uma expressão sem tipo esperado.
4. `Opcao::Algum(x)` e `Opcao::Nenhum` continuam exigindo contexto quando o tipo não pode ser inferido; a conversão implícita não remove essa exigência. Em particular, o lado direito de `??` **não** fornece contexto para inferir `Opcao::Nenhum`: `seja x = Opcao::Nenhum ?? 0;` gera `pors.option-type-inference` no `Opcao::Nenhum`. A forma válida é `seja x: Opcao<inteiro> = Opcao::Nenhum; escreval(x ?? 0);`.
5. A conversão não se aplica a `Resultado`, nem a `vazio`, mantendo o ADR 0018.

### Precedência

Da mais alta para a mais baixa, acrescentando ao quadro vigente de `crates/portugol-rs-frontend/src/parser/constants.rs:285-306`:

| Nível | Operadores | Associatividade |
| --- | --- | --- |
| primário | literal, identificador, `(...)`, `.campo`, `[indice]`, `(...)` de chamada | — |
| pós-fixo | `?.`, `?` de propagação | esquerda |
| unário | `!`, `-`, `+` | direita |
| multiplicativo | `*`, `/`, `%` | esquerda |
| aditivo | `+`, `-` | esquerda |
| deslocamento | `<<`, `>>` | esquerda |
| relacional | `<`, `<=`, `>`, `>=` | esquerda |
| igualdade | `==`, `!=` | esquerda |
| bit a bit e lógico | `&`, `^`, `|`, `&&`, `\|\|` | esquerda |
| coalescência | `??` | direita |
| atribuição | `=`, `??=` | direita |

1. `??` fica abaixo de `||`, então `a ?? b || c` é `a ?? (b || c)`.
2. `?.` fica acima de `?` de propagação, então `p?.campo?` é `(p?.campo)?` e a propagação continua exigindo `Opcao` no retorno da função.
3. `??=` tem a mesma precedência de `=` e só aparece como comando.
4. O analisador continua usando precedência numérica; a tabela acima é o contrato observável, não uma exigência de renomear os níveis internos.

### Bytecode, VM e Rust gerado

1. Não há `OptionalValue`, opcode novo ou valor nulo nesta RFC. `??` é rebaixado para teste do discriminante da enum existente com `Op::Jump`/`Op::JumpIfFalse`, e a alternativa é emitida em rótulo separado para preservar a avaliação preguiçosa.
2. `?.` é rebaixado como encadeamento de testes de variante com achatamento de um nível, reusando o caminho de acesso a campos de `registro` e as verificações de erro existentes.
3. `??=` é rebaixado como teste de variante seguido de atribuição condicional ao slot da variável; a expressão só é avaliada no ramo `Opcao::Nenhum`.
4. Rust gerado usa `unwrap_or_else`, `and_then` e `get_or_insert_with` sobre `Option<T>`, com os mesmos spans UTF-8 em bytes e source maps internos. Nenhuma API WASM nova.

### Realce, formatador e editores

1. `??`, `??=` e `?.` entram na classe `operator` da RFC 006 na IDE e no playground, sem variável CSS nova.
2. O lexer passa a emitir três tokens distintos em vez de dois `?` consecutivos; o formatador usa um espaço de cada lado de `??` e `??=`, e nenhum espaço em `?.`.
3. Completions e hover documentam os três operadores e o tipo `Opcao<T>`; `Opcional`, `nenhum` e `cadeia?` não são sugeridos nesta fatia.

## Exemplos válidos

```pors
registro Pessoa {
    nome: cadeia,
    telefone: Opcao<cadeia>
}

funcao inicio() {
    seja nome: Opcao<cadeia> = "Maria";
    seja usuario = nome ?? "Visitante";
    escreval("Olá, {usuario}!");

    seja pessoa: Opcao<Pessoa> = Opcao::Nenhum;
    escreval(pessoa?.nome ?? "Desconhecido");
    escreval(pessoa?.telefone ?? "Não informado");
}
```

```pors
funcao inicio() {
    seja celular: Opcao<cadeia> = Opcao::Nenhum;
    seja telefone: Opcao<cadeia> = Opcao::Algum("1234");
    seja email: Opcao<cadeia> = Opcao::Nenhum;
    seja contato = celular ?? telefone ?? email ?? "Sem contato";
    escreval(contato);

    seja mutavel apelido: Opcao<cadeia> = Opcao::Nenhum;
    apelido ??= "Visitante";
}
```

```pors
funcao inicio() {
    seja idade: Opcao<inteiro> = 18;
    seja valor = idade ?? 0;
    escreval(valor);
}
```

## Exemplos inválidos

| Trecho (coluna 0) | Diagnóstico | Motivo |
| --- | --- | --- |
| `funcao inicio() { seja x = 1 ?? 2; }` | `pors.coalesce-invalid-type` em `[29, 31)` | `1` não é `Opcao<T>` |
| `funcao inicio() { seja x: Opcao<inteiro> = Opcao::Algum(1); escreval(x ?? "b"); }` | `pors.coalesce-type-mismatch` em `[74, 77)` | alternativa `cadeia` não é `inteiro` |
| `funcao inicio() { seja p: Pessoa = Pessoa { nome: "Ana", telefone: Opcao::Nenhum }; escreval(p?.nome); }` | `pors.safe-access-invalid-type` em `[94, 96)` | `p` não é `Opcao<registro>` |
| `funcao inicio() { seja p: Opcao<Pessoa> = Opcao::Nenhum; escreval(p?.nome()); }` | `pors.safe-call-unsupported` em `[67, 69)` | chamada segura não existe nesta fatia |
| `funcao inicio() { seja r: Resultado<inteiro, cadeia> = Resultado::Erro("x"); escreval(r ?? 0); }` | `pors.coalesce-invalid-type` no `??` | ausência e erro são domínios distintos |

## Decisões de escopo (chamadas do autor)

- A conversão implícita `T -> Opcao<T>` fica restrita a inicializador de `seja`, argumento de chamada e retorno; não alcança campos, vetores, atribuição, operandos de `??`, base de `?.` nem ramos de `tente`.
- O lado direito de `??` não infere `T` para `Opcao::Nenhum`; a anotação continua obrigatória.
- `??=` aceita somente nome de variável mutável; campos e índices ficam adiados.
- `?.` cobre apenas acesso a campo de registro; chamada segura fica adiada porque o PORS não tem chamada de método em instância.
- `?.` achata exatamente um nível por operador, inclusive em encadeamento.
- Nenhum operador novo entra em interpolação de cadeia além do que a RFC 004 já permite.

## Consequências

- Três tokens novos (`??`, `??=`, `?.`), nenhum alias de tipo, nenhum valor `nenhum`, nenhuma sobrecarga de `?` em posição de tipo e nenhum opcode novo.
- `Opcao<T>` continua sendo o único tipo de ausência; `Resultado<T, E>` continua sendo o único tipo de erro recuperável. A conversão explícita de erro em ausência fica adiada com `ok()`.
- A RFC 007 define o comportamento de `?` sobre `Resultado` dentro de `tente`; esta RFC define `??` sobre `Opcao`. As duas regras não se sobrepõem.
- A precedência de `??` é mais baixa que a de `ou`/`||` e mais alta que a de atribuição; `?.` é pós-fixo. Isso fecha a ressalva levantada na proposta original sobre a distinção entre os usos de `?`.

## Abordagens consideradas

1. **Compatibilidade sobre o contrato publicado (escolhida).** Manter `Opcao`/`Algum`/`Nenhum`, `registro` e `leia()`, adicionando `??`, `??=` e `?.` com rebaixamento sobre a representação de enum já existente. Trade-off: não adota `Opcional`, `nenhum`, `cadeia?` nem `OptionalValue`, mas evita revogar o ADR 0018 e mantém um único caminho de ausência em HIR, bytecode, VM, Rust gerado e WASM.
2. **Proposta literal, com aliases, sobrecarga de `?` e opcodes novos.** Adotar `Opcional<T>`, `nenhum`, `cadeia?`, `OptionalValue`, `MAKE_SOME`/`MAKE_NONE`/`JUMP_IF_NONE` e `ok()`. Trade-off: mais próximo do texto motivador, mas exige ADR novo, dois nomes para o mesmo tipo, uma tabela de precedência maior e muito mais superfície de teste em todos os backends e editores.
3. **Somente `??`, adiando `?.` e `??=`.** Trade-off: documentação menor e mais barata, mas deixa o encadeamento seguro sem contrato e cria incentivo para encadeamento manual de `escolha`, que é exatamente o caso que a proposta quer simplificar.
