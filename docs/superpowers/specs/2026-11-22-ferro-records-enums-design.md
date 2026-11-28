# Registros, enums e escolha exaustivo em FERRO

## Intenção

Fechar o bloqueador B1 da auditoria visual: dar a programas FERRO tipos de estado
e eventos que possam ser analisados, executados no debugger/VM e exportados para
Rust com a mesma semântica. Isso prepara a transição funcional do ADR 0017,
`atualizar(estado, evento) -> Estado`, sem criar uma segunda linguagem para o
designer visual.

## Abordagens

1. **HIR nominal compartilhada (escolhida):** o frontend produz tipos e
   construções nominais; o mesmo HIR alimenta bytecode/VM e Rust. Preserva
   diagnósticos e equivalência observável, com mais trabalho nas duas saídas.
2. **Gerar diretamente Rust para tipos compostos:** mais simples inicialmente,
   mas o debugger não executaria os mesmos programas e a UI dependeria de
   `rustc`; contraria o ADR 0017.
3. **Objetos dinâmicos/mapas:** facilita campos arbitrários, mas enfraquece a
   checagem estática, torna `escolha` não exaustivo e afasta o aprendizado de
   tipos Rust; rejeitada.

## Escopo

- `registro Nome { campo: Tipo, ... }` define record types nominais com campos
  nomeados, tipados e obrigatórios.
- `Nome { campo: valor, ... }` constrói um registro. Cada campo deve aparecer
  exatamente uma vez e o valor precisa ter o tipo declarado.
- `enum Nome { Variante, Variante(Tipo, ...) }` define soma nominal com payloads
  posicionais tipados.
- `Nome::Variante` e `Nome::Variante(valor, ...)` constroem variantes.
- `registro.campo` lê campos. `registro.campo = valor` e atribuições compostas
  escrevem somente quando o binding do registro foi declarado `mutavel`.
- `escolha expressao { caso Nome::Variante => { ... } ... }` é uma instrução;
  toda variante do enum precisa aparecer exatamente uma vez. `caso
  Nome::Variante(a, b)` vincula cópias imutáveis dos payloads no escopo do braço.
- Tipos compostos podem conter primitivos, vetores existentes, registros e
  enums já definidos. Referências cíclicas entre registros ficam fora deste
  recorte; a ordem de declaração deve permitir resolver os tipos sem `Box`.
- Registros, enums e vetores desses tipos têm semântica de valor: atribuição,
  parâmetros por valor, retornos e payloads são cópias independentes.
- O debugger expande registros em campos e mostra a variante/payload do enum.
- Rust gerado usa `struct`, `enum`, `match` e `Vec<T>` idiomáticos. Os dois
  backends devem concordar em resultados, falhas de tipo e limites de strings.

## Fora de escopo

Métodos, generics, traits, referências/borrowing, `Box`, recursão de tipos,
pattern matching como expressão, guards, wildcard, enum com campos nomeados,
struct update syntax, mutabilidade individual de campos e vetores aninhados.
`escolha` não substitui `se/senao` para condições booleanas.

## Semântica e erros

- Nomes de tipos/variantes são case-sensitive; nomes duplicados e campos ou
  variantes inexistentes são erros localizados.
- A construção incompleta, repetida ou incompatível aponta ao campo/valor
  responsável.
- `escolha` duplicando ou omitindo variantes é erro estático; não há braço
  implícito.
- Atribuir um campo de binding imutável é rejeitado antes de gerar bytecode ou
  Rust.
- Cópias profundas de valores que contêm `cadeia` contam cada alocação para o
  limite cumulativo já existente em cada backend.
- A sintaxe da primeira fatia é:

  ```ferro
  registro Estado { contador: inteiro, historico: vetor<inteiro> }
  enum Evento { Incrementar, Definir(inteiro) }

  funcao atualizar(estado: Estado, evento: Evento) -> Estado {
      escolha evento {
          caso Evento::Incrementar => {
              seja mutavel proximo: Estado = estado;
              proximo.contador += 1;
              retorne proximo;
          }
          caso Evento::Definir(valor) => {
              seja mutavel proximo: Estado = estado;
              proximo.contador = valor;
              retorne proximo;
          }
      }
  }
  ```

- `TypeId(u32)` identifica nominalmente as definições na HIR. `Program` guarda
  `RecordDef` e `EnumDef`; tipos e variantes são resolvidos antes da análise de
  funções. `Type` adiciona `Record(TypeId)`, `Enum(TypeId)`,
  `VectorRecord(TypeId)` e `VectorEnum(TypeId)`, mantendo os vetores escalares
  existentes.
- HIR usa `RecordLiteral { type_id, fields: Vec<Expr> }` com valores na ordem
  declarada, `Field { value, field_index }`, `EnumConstructor { type_id,
  variant_index, payload }`, `FieldAssign { binding, field_index, op, value }`
  e `Choose { value, arms }`. Cada braço guarda o índice da variante, bindings
  de payload e seu bloco.
- `escolha` opera sobre um binding/expressão enum imutável; cada `caso` é um
  bloco e nomes de payload são bindings locais imutáveis. Não há fallthrough.
- Campos são lidos com encadeamento (`estado.usuario.nome`); escrita no primeiro
  recorte é direta em binding (`estado.contador = ...`). Vetores de registros e
  enums são aceitos; atribuição em caminho indexado/nested, como
  `lista[0].nome = ...`, fica para uma fatia posterior.
- Referências de tipos só apontam para tipos definidos antes; referências
  diretas e indiretas cíclicas são rejeitadas, inclusive através de vetores.
- Spans HIR continuam em bytes UTF-8; serviços convertem para UTF-16 na borda do
  editor.

## Critérios de aceite

1. `ferro-hir`, frontend, bytecode, VM, debugger, gerador Rust e serviço
   representam registros/enums e acessos com tipos e spans.
2. Testes verificam construção e leitura/escrita de campos, vetores de tipos
   compostos, payloads, exaustividade, mutabilidade, cópias aninhadas, shadowing
   e texto Unicode.
3. Testes diferenciais executam exemplos com estado/eventos na VM e em Rust
   gerado e comparam saída, falha de bounds e diagnóstico.
4. A matriz FERRO marca o suporte como parcial e lista exatamente as exclusões.
5. `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D
   warnings`, `cargo fmt --all -- --check`, frontend e CI multiplataforma ficam
   verdes.

## Restrições

Preservar Rust + Tauri, nome do produto, extensão `.fer`, suporte legado `.por`
e `.pex`, ferramentas atuais e contratos de execução. Nenhuma dependência nova.
