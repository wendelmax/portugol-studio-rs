# ADR 0020: Ajustes pedagógicos da evolução sintática do FERRO

- **Status:** Aceita
- **Data:** 2026-11-27
- **Responsável:** mantenedor do produto
- **Revisa:** [ADR 0019](0019-contrato-minimo-de-tipos-ferro.md), item 2
- **Relacionados:** [ADR 0016](0016-ferro-como-linguagem-autoral-e-retirada-do-por.md), [ADR 0018](0018-projetos-desktop-ferro-com-designer-visual.md), [design de evolução sintática](../superpowers/specs/2026-11-26-ferro-syntax-evolution-design.md), RFCs [002](../superpowers/specs/2026-11-27-rfc-002-leia-como-expressao.md), [004](../superpowers/specs/2026-11-27-rfc-004-interpolacao-de-cadeias.md), [005](../superpowers/specs/2026-11-27-rfc-005-retorno-implicito.md), [007](../superpowers/specs/2026-11-27-rfc-007-tratamento-de-erros.md) e [008](../superpowers/specs/2026-11-27-rfc-008-coalescencia-e-navegacao-segura.md)

## Contexto

A primeira fatia da evolução sintática (PR #254) entregou `escreval`,
inferência em `seja`, `leia()` como expressão, interpolação de cadeias e
retorno implícito. A direção é boa: os programas iniciais ficam mais curtos e
mais parecidos com o Ferro que o aluno já conhece.

Uma revisão do ponto de vista de quem está aprendendo, feita com programas
reais executados na `main` (`a733dc6`), encontrou quatro problemas na fatia
entregue e excesso de escopo nas duas RFCs ainda propostas:

1. **Interpolação recusa números.** `escreval("Você tem {idade} anos");` com
   `idade: inteiro` gera `ferro.interpolation-non-string`. É o uso mais comum
   de interpolação no ensino. Os próprios exemplos das RFCs 007 e 008 fazem
   isso e não compilam.
2. **Erro de digitação na interpolação passa em silêncio.**
   `escreval("Olá, {nmoe}!");` imprime `Olá, {nmoe}!` sem aviso, porque a
   regra 7 da RFC 004 mantém como literal o conteúdo com nome não resolvido.
3. **As duas formas de `leia` divergem.** `leia(idade);` converte a entrada
   para `inteiro`, mas `seja idade: inteiro = leia();` é erro de tipo, porque
   o ADR 0019 fixa `leia()` em `cadeia`. O aluno não consegue explicar a
   diferença. A branch `ferro-examples-migration` já implementa a conversão
   pelo contexto (commit `de13f04`), contrariando o ADR 0019.
4. **Retorno implícito tem armadilhas para iniciantes.** `x * 2;` no fim de
   uma função `-> inteiro` gera `Esperada declaração, atribuição, escreva ou
   se`, sem mencionar `retorne` nem o `;`. Uma expressão final dentro de
   `se`/`senao` também é erro, embora funcione no corpo da função.
5. **RFC 007** soma exceções no estilo Java a `Resultado` com `?`, inferência
   do tipo de erro, regras de cobertura com exceções, `finalmente` duplicado
   em cada saída e `tente` como expressão com uma exceção própria à RFC 005.
   É mais superfície do que uma linguagem educacional precisa na primeira
   entrega.
6. **RFC 008** propõe `??`, `?.`, `??=` e conversão implícita de `T` para
   `Opcao<T>` em três contextos. `??` resolve um problema real; os demais
   rendem pouco numa linguagem sem métodos e criam regras com exceções
   (`seja x: Opcao<inteiro> = 1` valeria, `x = 1` não).

## Decisão

### 1. Interpolação aceita escalares

Dentro de `{...}`, são aceitas expressões dos tipos `cadeia`, `caracter`,
`inteiro`, `real` e `logico`.

- O texto de cada parte é o mesmo que `escreva` exibiria para o valor:
  `inteiro` em decimal, `real` no formato atual do `escreva` (por exemplo
  `7.75`), `logico` como `verdadeiro`/`falso`, `caracter` como cadeia de um
  caractere.
- VM e Rust gerado usam a mesma conversão do `escreva`, e os testes de
  paridade cobrem os cinco tipos.
- `vetor`, `matriz`, `registro`, `enum`, `Opcao`, `Resultado` e `vazio`
  continuam gerando `ferro.interpolation-non-string`, com dica de interpolar um
  campo ou um valor escalar.
- Especificadores de formato continuam fora (`{nome:>10}` segue inválido).
- A concatenação com `+` não muda nesta decisão. Ela mantém a compatibilidade
  com o Ferro original, em que `logico` vira `true`/`false`. A diferença
  fica documentada na ajuda; alinhar `+` ao `escreva` exige decisão própria.

### 2. Aviso para nome desconhecido em interpolação

Quando o conteúdo entre chaves é um identificador ou um acesso a campo
(`{nome}`, `{pessoa.nome}`) e o nome não é resolvido, o frontend emite o
**aviso** `ferro.interpolation-unknown-name`, no span do conteúdo, com sugestão
do nome mais próximo em escopo quando houver ("Você quis dizer `nome`?").

- A cadeia continua literal, como hoje. Programas publicados que usam chaves
  em texto não deixam de compilar.
- A dica do aviso ensina a escrever chaves literais com `\{` e `\}`.
- Conteúdo com erro léxico (`{@}`) e chaves sem fechamento continuam literais
  sem aviso.

### 3. `leia()` converte para o tipo do destino (revisa o ADR 0019, item 2)

O item 2 do ADR 0019 passa a ser:

> `leia()` sem destino tipado tem tipo `cadeia`. Quando `leia()` é o valor
> inteiro de um destino com tipo declarado, a entrada é convertida para esse
> tipo, com as mesmas regras de `leia(variavel)`. Os destinos são:
> inicializador de `seja` com anotação de tipo, lado direito de atribuição a
> variável tipada, argumento de parâmetro tipado e valor de `retorne` em
> função com retorno declarado.

- Os tipos convertidos são `inteiro`, `real`, `caracter`, `logico` e
  `cadeia`. Outros destinos (`vetor`, `registro`, `Opcao`, ...) são
  `ferro.type-mismatch` no span da chamada.
- As regras de conversão são **exatamente** as de `leia(variavel)` (as do
  console do Ferro Studio, em `parse_input` da VM), para que as duas formas
  se comportem igual. Uma entrada que não converte termina a execução com a
  mensagem já existente, que cita o tipo esperado.
- `leia()` como operando (`leia() + 1`), dentro de interpolação ou como
  argumento de `escreva` continua `cadeia`. O contexto não se propaga através
  de operadores.
- `seja idade = leia();` continua inferindo `cadeia`: sem anotação não há
  destino tipado.
- O backend continua usando `Op::Read(ValueType)` com o tipo do destino, sem
  opcode novo. No Rust gerado, o helper de leitura recebe o tipo do destino,
  e o protocolo `FERRO_INPUT:<tipo>` anuncia esse tipo.
- `leia<T>()` não é criado. A validação sem encerrar o programa continua pelo
  caminho explícito (`Tipos.para_*` + `Resultado`), que é o ensinado em
  formulários (`UI.entrada` e controles do ADR 0018).

### 4. Retorno implícito fica fora do material inicial

A regra da RFC 005 não muda: a expressão final sem `;` do corpo de uma função
com retorno declarado é o retorno. Mudam o ensino e os diagnósticos:

- `retorne valor;` é a forma canônica em ajuda, exemplos iniciais,
  exercícios, snippets e no exemplo motivador do design. O retorno implícito
  aparece na trilha de ponte para Rust, apresentado como "como o Rust faz".
- Em função com retorno declarado cujo último comando é uma expressão seguida
  de `;` (`x * 2;`), o diagnóstico passa a ser `ferro.missing-return` no span
  da expressão, com a dica: "Para devolver este valor, escreva
  `retorne x * 2;` ou remova o `;`."
- Uma expressão final sem `;` dentro de `se`, `senao`, `escolha` ou laço passa
  a explicar a regra: "Dentro de `se`, use `retorne` para devolver o valor;
  só a última linha da função pode omitir `retorne`."

### 5. RFC 007 entra em fatia reduzida

A primeira entrega de tratamento de erros contém somente:

- `tente { ... } capture erro { ... }` como **comando**, com um único
  `capture` que dá nome ao erro;
- `?` dentro de `tente` desviando para o `capture`;
- `lance expressao;`, com a semântica da RFC 007 (dentro de `tente` desvia
  para o `capture`; fora dele equivale a `retorne Resultado::Erro(expressao);`).

Ficam adiados: `finalmente`, `capture` por variante e `capture _`, vários
`capture` no mesmo `tente`, `tente` como expressão e as regras de cobertura.
Sem `tente` como expressão, a exceção que a RFC 007 abria na RFC 005 deixa de
existir. Cada item adiado volta por emenda à RFC 007 depois que a fatia
reduzida estiver em uso.

### 6. RFC 008 entra só com `??`

A primeira entrega contém somente `esquerda ?? direita` sobre `Opcao<T>`, com
as regras de tipo, preguiça e precedência da RFC 008.

Ficam adiados: `?.`, `??=` e a conversão implícita de `T` para `Opcao<T>`.
Sem a conversão, os exemplos usam a forma explícita
`Opcao::Algum("Maria")`.

### 7. Operadores em palavras ficam para RFC própria

`e`, `ou` e `nao` como sinônimos de `&&`, `||` e `!` são uma direção
desejada, porque o pseudocódigo clássico os usa e reduzem a carga de símbolos.
Não entram agora: `e` é nome comum de variável, parâmetro e padrão (nos
exemplos publicados, aparece 17 vezes em 7 arquivos, por exemplo em
`Resultado::Erro(e)`), e tornar a palavra reservada exige estudo de impacto e
migração. O assunto fica para
uma RFC própria, e nenhuma das três palavras é reservada por este ADR.

## Alternativas consideradas

### Manter `leia()` sempre `cadeia` e ensinar `Tipos.para_*`

Rejeitada. Obriga `escolha` sobre `Resultado` em todo exercício de entrada
numérica, antes de o aluno ver tipos compostos, e mantém duas formas de
leitura com comportamentos diferentes.

### `leia<inteiro>()` ou `leia_inteiro()`

Rejeitada. Acrescenta sintaxe genérica ou funções duplicadas, quando o tipo já
está escrito no destino, como em `leia(variavel)`.

### Tornar erro o nome desconhecido em interpolação

Rejeitada por compatibilidade: cadeias publicadas com chaves em texto
deixariam de compilar. O aviso aparece no editor e no CLI.

### Remover o retorno implícito

Rejeitada. Ele é útil na ponte para Rust e já está implementado e testado.
O problema está em ensiná-lo cedo e nos diagnósticos, e não na regra.

### Implementar as RFCs 007 e 008 inteiras

Rejeitada. Cada regra adicional vira um diagnóstico que o aluno encontra. A
entrega reduzida cobre os casos de ensino; o restante volta com evidência de
uso.

## Consequências

- **RFC 002:** revisada pela decisão 3. Os exemplos inválidos com destino
  `inteiro` passam a ser válidos; continua inválido `leia(argumento)` em
  expressão.
- **RFC 004:** revisada pelas decisões 1 e 2. `escreva("{42}")` passa a ser
  válido; `ferro.interpolation-non-string` fica restrito aos tipos não
  escalares.
- **RFC 005:** regra mantida; diagnósticos e uso em material didático
  revistos pela decisão 4.
- **RFC 007** e **RFC 008:** continuam propostas, com o escopo da primeira
  entrega limitado pelas decisões 5 e 6. Os exemplos que interpolam números
  passam a ser válidos pela decisão 1; os que usam conversão implícita para
  `Opcao` precisam da forma explícita.
- **Branch `ferro-examples-migration`:** a conversão de `leia()` que ela
  implementa fica coberta por este ADR, desde que siga as regras da
  decisão 3 (destinos listados, mesmas regras de `leia(variavel)`). A
  regra "caracter exige exatamente um caractere" do design dessa branch
  diverge de `leia(variavel)`, que usa o primeiro caractere da linha, e deve
  ser alinhada antes da mesclagem.
- **ADR 0018:** o code-behind pode interpolar números em rótulos
  (`"Resultado: {a + b}"`), sem `Tipos.inteiro_para_cadeia`.
- Testes de aceite necessários: interpolação dos cinco tipos escalares na VM e
  no Rust gerado; aviso de nome desconhecido com sugestão; `leia()` nos quatro
  destinos e o erro em destino não escalar; equivalência entre `leia()` e
  `leia(variavel)` para as mesmas entradas, incluindo entradas inválidas; os
  dois novos diagnósticos de retorno.
