# Progresso de exercícios validado

## Intenção

Dar ao aluno uma indicação local de exercícios tentados e concluídos sem
transformar a abertura de cartões ou a execução comum em progresso educacional.
Uma conclusão deve apontar para a revisão exata do código que passou pelo
corretor real.

## Objetivos e limites

- Registrar somente resultados retornados por `checkExercise`.
- Concluir somente quando `total > 0` e todos os casos foram aprovados.
- Vincular cada tentativa a um identificador de exercício e ao SHA-256 da fonte.
- Persistir apenas metadados locais no `localStorage`; não armazenar fonte,
  dados pessoais ou sincronizar com serviços externos.
- Mostrar conclusão histórica no catálogo sem afirmar que uma edição posterior
  da solução também passou.
- Falha do corretor/infraestrutura não gera tentativa nem conclusão.

Fora do escopo: conta de aluno, sincronização, currículo remoto, nota, ranking,
progresso por abertura do cartão e alterações ao contrato do corretor Rust.

## Abordagens consideradas

1. Marcar exercício concluído ao abrir ou executar: descartada por não provar
   aprendizagem nem correção.
2. Armazenar apenas um booleano por ID: descartada porque não distingue uma
   solução corrigida de uma revisão que ainda não foi avaliada.
3. Guardar resultados locais por identidade do exercício e hash da revisão:
   escolhida porque preserva proveniência com o avaliador existente e não exige
   backend ou identidade de aluno.

## Decisões

- O catálogo interno usa `builtin:<id>`; arquivos `.pex` importados usam o hash
  do conteúdo do exercício para evitar colisões de IDs.
- Cada registro guarda o hash SHA-256 da fonte, estado `passed`/`failed`,
  quantidades avaliadas/aprovadas e instante da avaliação.
- Só resultados com casos reais (`total > 0`) contam como avaliação. Conteúdo
  avaliado apenas por análise estática continua sem conclusão.
- O catálogo sinaliza que existe uma conclusão validada para aquele exercício.
  A tela de resultado identifica explicitamente a revisão avaliada.
- Se WebCrypto ou `localStorage` estiver indisponível, corrigir o exercício
  continua funcionando e o progresso fica indisponível sem erro falso.

## Arquitetura afetada

Novo módulo puro `exercise-progress.ts`; integração em `exercises.ts` e na
renderização do seletor de exercícios em `index.html`/`main.ts`. O corretor,
serviço Rust, comandos Tauri e formato `.pex` permanecem inalterados.

## Validação

Testes Node verificam identidade, hash de revisão, aprovação parcial/total,
versão de armazenamento e falhas de persistência. Testes de integração garantem
que abrir o cartão não grava progresso e que erro do avaliador não registra uma
tentativa. Validar `npm test`, `npm run build`, `cargo test --workspace`,
`cargo clippy --workspace --all-targets -- -D warnings` e CI do PR.
