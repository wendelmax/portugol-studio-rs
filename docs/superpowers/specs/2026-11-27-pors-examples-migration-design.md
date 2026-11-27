# Design: migração total dos exemplos PORS para a sintaxe evoluída

- **Data:** 2026-11-27
- **Status:** Proposta (aguardando aprovação)
- **Relacionados:** [ADR 0019](../../adr/0019-contrato-minimo-de-tipos-pors.md), [RFC 001](2026-11-27-rfc-001-inferencia-em-seja.md), [RFC 002](2026-11-27-rfc-002-leia-como-expressao.md), [RFC 003](2026-11-27-rfc-003-escreval.md), [RFC 004](2026-11-27-rfc-004-interpolacao-de-cadeias.md), [RFC 005](2026-11-27-rfc-005-retorno-implicito.md), [RFC 007](2026-11-27-rfc-007-tratamento-de-erros.md), [RFC 008](2026-11-27-rfc-008-coalescencia-e-navegacao-segura.md), [matriz de migração](../../research/PORS_MIGRATION_MATRIX.md)

## Intenção

O PR #254 implementou a evolução sintática no frontend, realce e debugger, mas deixou os exemplos públicos praticamente intactos: no `main` há 211 arquivos `.pors` e apenas um usa a forma nova. Este trabalho:

1. migra **todos** os exemplos versionados e embutidos para a sintaxe simplificada;
2. ajusta `leia()` para converter implicitamente ao tipo esperado pelo contexto, porque a alternativa (conversão explícita com `Resultado` em toda leitura numérica) torna os exemplos introdutórios mais longos e desloca o foco de aprendizado;
3. corrige as contradições documentais encontradas na auditoria, incluindo RFCs e ADRs.

Público: quem aprende PORS pela IDE, pelo playground, pelos exercícios, pela ajuda integrada, pelos documentos normativos e pelo README. Hoje essas superfícies ensinam predominantemente `escreva`, `leia(variavel)`, tipos explícitos e concatenação manual.

## Análise: ganho de aprendizado na conversão de entrada

| Critério | `leia()` + `Tipos.para_*` + `escolha`/`Resultado` | `leia()` com conversão implícita |
| --- | --- | --- |
| Exemplo introdutório | 6–10 linhas para uma leitura de inteiro | `seja n: inteiro = leia();` |
| Foco do aluno | validação e `Resultado` antes de laços/condicionais | conceito que o exemplo quer demonstrar |
| Entrada inválida | erro tratado no programa | erro de execução claro |
| Coerência com PR #254 | quebra a promessa de sintaxe simplificada | reforça a simplificação |
| Tratamento robusto | sempre disponível | disponível via `Tipos.para_*` em exemplos avançados |

**Decisão:** conversão implícita, com erro de execução em entrada inválida. O caminho explícito com `Resultado` permanece documentado e é usado nos exemplos que ensinam tratamento de erro — ele deixa de ser obrigatório em leituras comuns.

## Objetivos

1. Migrar os 211 arquivos `.pors` versionados para a nova sintaxe sempre que a conversão preserve o significado.
2. Migrar exemplos embutidos em TypeScript, JavaScript, JSON, HTML e Markdown em **todas** as superfícies: IDE, playground, ajuda, snippets/autocomplete, README, RFCs, ADRs e planos.
3. Simplificar saída com `escreval`, declarações com inferência e texto com interpolação onde os tipos permitem.
4. Garantir conversão implícita nas duas formas de `leia`: `leia(variavel)` converte ao tipo declarado da variável (comportamento atual) e `leia()` converte ao tipo esperado pelo contexto (`inteiro`, `real`, `caracter`, `logico`, `cadeia`), falhando em execução com mensagem clara quando a entrada não converte.
5. Corrigir o plano do núcleo que afirma que `escreval` é reservado.
6. Corrigir os exemplos da RFC 007 e da RFC 008 que usam interpolação de tipos inválidos segundo a RFC 004, mantendo aviso explícito de que `tente`, `capture`, `lance` e os exemplos de coalescência/navegação segura ainda não executam no `main`.
7. Atualizar RFC 002, ADR 0019 e a matriz de migração para refletir a nova semântica de `leia()`.
8. Preservar a saída esperada de exemplos usados por testes e exercícios.

## Não-objetivos

- Não introduzir sintaxe nova além da semântica de conversão implícita de `leia()`.
- Não implementar RFC 007 nem RFC 008.
- Não alterar o contrato de compatibilidade: `escreva`, `retorne`, tipos explícitos, `leia(variavel)` e concatenação com `+` continuam válidos; a migração é uma escolha de apresentação dos exemplos.
- Não tocar em `apps/desktop/package-lock.json`; ele é uma alteração preexistente do usuário no checkout principal.
- Não alterar frontend, HIR, bytecode, VM, Rust gerado, WASM, IDE ou playground em comportamento **além** do necessário para a conversão implícita de `leia()`.

## Anti-objetivos

- Transformar exemplos didáticos em código mais longo e menos legível.
- Trocar uma leitura numérica por uma leitura de `cadeia` e perder a demonstração original.
- Apagar cobertura de sintaxe legada dos testes só porque os exemplos foram reescritos — a compatibilidade ainda é testada em casos dedicados.

## Restrições

1. Ambas as formas de `leia` convertem implicitamente ao destino: `leia(variavel)` ao tipo declarado da variável e `leia()` ao tipo esperado pelo contexto (inicialização de `seja` com tipo anotado, argumento de função tipado, atribuição, retorno e campos). `leia()` sem contexto tipado continua retornando `cadeia`. Nos exemplos migrados, prefere-se `leia()` — `seja x: tipo = leia();` ou `x = leia();`. `leia(variavel)` permanece válido e é usado apenas em testes de compatibilidade e em laços com leitura repetida.
2. A conversão implícita usa as mesmas regras dos helpers `Tipos.para_*`: `caracter` exige exatamente um caractere e `logico` aceita espaços ao redor. Exemplos que dependiam de `leia(caracter)` pegando o primeiro caractere da linha ou de `leia(logico)` sem `trim` são ajustados e a diferença é registrada na matriz de migração.
3. Entrada que não converte produz erro de execução com mensagem identificando o tipo esperado; não há valor padrão silencioso.
4. O caminho explícito permanece: `Tipos.para_inteiro(...)`, `Tipos.para_real(...)` etc. retornam `Resultado`; `?` exige função que retorne `Resultado<_, E>`/`Opcao<_>`, então não serve em `inicio`. Exemplos com campo de texto (`UI.entrada`) e validação de seu valor bruto **devem** usar esse caminho explícito com `escolha`/`Resultado`. Leituras `leia(...)` não devem ser reescritas como `leia(cadeia)` seguido de `Tipos.para_*`: a conversão é implícita na própria leitura.
5. A RFC 004 aceita somente `cadeia` e `caracter` dentro de `{...}`. Números, lógicos, vetores, registros, enums e `vazio` continuam gerando `pors.interpolation-non-string`.
6. `escreva`/`escreval` rejeitam valores de registro e enum diretamente; para esses casos, use campos, variantes ou mensagens fixas.
7. Inferência se aplica a `seja nome = valor;`. `seja mutavel`, inicialização vazia, parâmetros, retornos e tipos de coleção continuam explícitos quando necessário.
8. Retorno implícito só vale para a expressão final do corpo de uma função com tipo de retorno declarado.
9. O operador `escreval` é builtin não reservado, conforme RFC 003, RFC 006 e o teste `escreval_can_still_be_used_as_an_identifier`.

## Abordagens

### A — alta visibilidade, linguagem inalterada

Migrar README, playground, template/dicas da IDE, ajuda, exercícios e alguns `.pors` em destaque; manter `leia(variavel)` nos exemplos de entrada tipada e o catálogo clássico intacto.

**Trade-off:** diff menor, mas a afirmação “todos os exemplos” continuaria falsa e a contradição documental permaneceria.

### B — migração total (escolhida)

Migrar todos os 211 `.pors` e todos os exemplos embutidos, inclusive entrada tipada, catálogo clássico, RFCs e ADRs, com conversão implícita de `leia()` no tipo esperado.

**Trade-off:** diff grande e mudança semântica localizada no compilador; em troca, nenhuma superfície pública ensina a forma antiga e os exemplos introdutórios permanecem curtos.

### C — somente documentação normativa

Corrigir plano e RFCs sem tocar nos exemplos.

**Trade-off:** barato, mas não resolve a queixa original.

### Variante considerada e rejeitada

Migração total mantendo `leia()` como `cadeia` e usando `Tipos.para_*` + `Resultado` em toda leitura numérica. Rejeitada pelo ganho de aprendizado menor, conforme a tabela acima.

## Decisões tomadas

1. **Abordagem B.**
2. **Conversão implícita nas duas formas de `leia`.** `leia(variavel)` converte ao tipo da variável; `leia()` converte ao tipo esperado pelo contexto; falha é erro de execução. Sem tipo esperado, `leia()` resulta em `cadeia`.
3. **Caminho explícito preservado para `UI.entrada`.** `Tipos.para_*` com `Resultado` continua nos exemplos de campo de texto, onde ensinar tratamento do valor bruto é o objetivo. Nas duas formas de `leia`, a conversão é implícita; os exemplos migrados preferem `leia()`, e `leia(variavel)` fica reservado à compatibilidade e a laços com leitura repetida.
4. **Escopo documental ampliado.** Todos os Markdown com snippets PORS (RFCs 001–008, ADR 0019, planos, README), `apps/desktop/src/portugol.ts`, ajuda HTML, playground e exercises são migrados. Passagens que explicitamente documentam compatibilidade com a sintaxe legada podem ser mantidas e marcadas como tal.
5. **RFC 007 e RFC 008 corrigidas e sinalizadas.** Interpolações de números/enums são substituídas por argumentos de `escreval` compatíveis com a RFC 004; ambas recebem aviso explícito de que seus exemplos não executam no `main`.
6. **Contradição do plano corrigida.** O plano do núcleo passa a descrever `escreval` como builtin não reservado, alinhado à implementação.
7. **Exceções** à conversão literal são registradas neste design e em `docs/research/PORS_MIGRATION_MATRIX.md`; não é criada nova RFC.
8. **Validação forte.** Teste de aceite compila os 172 programas `.pors` completos, varre padrões legados nos 211 arquivos e em todos os fontes embutidos, e exige allowlist explícita para qualquer exceção.

## Exceções registradas

Estas exceções cumprem a Decisão 7 e são espelhadas em
`docs/research/PORS_MIGRATION_MATRIX.md`. Nenhuma delas cria nova RFC nem
enfraquece a allowlist existente.

| Item | Registro |
|---|---|
| Conversão de leitura | `leia(variavel)` converte ao tipo declarado da variável e `leia()` ao tipo esperado pelo contexto; sem contexto, `leia()` é `cadeia`. Entrada inválida é erro de execução, sem valor padrão silencioso. |
| Preferência | Exemplos migrados preferem `leia()`. `leia(variavel)` fica restrito à compatibilidade e a laços com leitura repetida. |
| `caracter`/`logico` | `caracter` exige exatamente um caractere; `logico` aceita espaços ao redor. A diferença em relação às antigas leituras tipadas é registrada na matriz. |
| `UI.entrada` | Campo de texto bruto mantém `Tipos.para_*` + `Resultado` com `escolha` quando a validação explícita é o objetivo da lição. |
| Rótulo divergente | `crates/portugol-rust/src/codegen/stmt.rs:216` usa `lógico` para leitura com variável, enquanto `crates/portugol-rust/src/codegen/types.rs:100` usa `logico` para leitura como expressão. Divergência pré-existente: `stmt.rs` não é alterado nesta fatia. |
| Fronteira `1 + leia()` | O tipo esperado não é propagado ao operando direito de uma operação binária: ele é analisado com `target=None` (`cadeia`). `target=Some(Integer)` aparece em `seja n: inteiro = leia();`, argumento tipado, atribuição, retorno e campo; a soma com `leia()` não é usada nos exemplos. |
| Cobertura do gate | 211 arquivos `.pors` são varridos por marcadores; 172 programas completos são compilados. Os outros 39 são fragmentos `data-type="sintaxe"` sem `funcao inicio()` e não compilam isoladamente. |
| Quebra explícita | O marcador `escreva-com-quebra-explicita` só sinaliza `escreva(...)` com `\n` em cadeia; `escreva` sem quebra continua válido. A allowlist preserva 15 arquivos do catálogo e 1 exemplo de ajuda (16 entradas, 32 chamadas), porque `escreval` acrescentaria quebra final e mudaria o stdout. |
| Laços e compatibilidade HTML | Quatro laços do catálogo e dois da ajuda com `leia(variavel)` e duas entradas HTML de compatibilidade (`entrada_saida/index.html` e `entrada_saida/leia.html`) permanecem na allowlist, sem alteração das entradas existentes. |

## Plano de implementação esperado

1. Implementar a conversão implícita de `leia()` no frontend/HIR/bytecode/VM, com testes para cada tipo e para erro em entrada inválida; ajustar o teste que hoje exige erro de tipo em `seja idade: inteiro = leia();` e o teste de argumentos de `leia()`.
2. Adicionar teste de aceite que percorre os 211 `.pors` e os fontes embutidos, falhando quando restarem padrões legados fora da allowlist; `leia(variavel)` só é permitido fora de testes de compatibilidade mediante allowlist explícita para laços com leitura repetida.
3. Migrar os exemplos em lotes: catálogo de serviços, ajuda da IDE, fontes embutidos, exercícios, README/RFCs/ADRs/planos e matriz de migração.
4. Validar que todos os `.pors` continuam compilando e que os exemplos com saída esperada mantêm stdout idêntico.
5. Rodar a suíte completa e abrir um único PR contra `main`, sem `apps/desktop/package-lock.json`.

## O que fica para o implementador

- A ordem exata dos lotes e o nome do teste de aceite.
- A redação final das mensagens de erro em exemplos convertidos, desde que preservem o significado.
- A codificação interna da conversão (novo tipo de expressão HIR ou reuso de helper), desde que a semântica observacional seja a descrita aqui.
- A decisão de usar `escolha` em exemplos avançados, desde que não introduza nova API pública.
