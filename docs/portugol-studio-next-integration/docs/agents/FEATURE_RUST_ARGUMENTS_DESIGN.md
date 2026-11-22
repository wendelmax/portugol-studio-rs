# Design — argumentos em programas Rust

## Intenção

Permitir que arquivos `.rs` recebam argumentos de linha de comando pelo campo já existente na IDE, para exercícios Rust usarem `std::env::args()`.

## Estado atual

- `apps/desktop/src/main.ts` já separa argumentos de Portugol, mas desabilita o campo para arquivos Rust.
- O comando Tauri `run_rust_source` recebe somente ID e fonte.
- `portugol-service::run_rust_source_with_cancel` compila com `rustc` fixo e executa o binário com `Command`, hoje sem `.args(...)`.
- A linha de execução já tem consentimento, cancelamento, limites de duração/saída e não usa shell.

## Abordagem

Usar o campo `Argumentos` existente com parser local de tokens. Espaços separam argumentos fora de aspas duplas; aspas agrupam espaços; `\"` e `\\` permitem incluir aspas e barras invertidas; aspas não fechadas geram mensagem clara antes de iniciar execução. O array é enviado via IPC tipado e aplicado somente a `Command::args` do executável Rust já compilado. O backend rejeita NUL, mais de 256 argumentos ou mais de 64 KiB UTF-8 acumulados (8 KiB no Windows para manter a linha de comando abaixo do limite do `CreateProcessW`).

Nenhuma string do aluno é interpretada como shell. A invocação do compilador permanece fixa e não recebe esses argumentos.

## Compatibilidade

- Portugol e Portugol-RS preservam a semântica atual; o parser de argumentos é compartilhado somente se os testes confirmarem equivalência nos exemplos atuais.
- Fonte `.rs`, consentimento, limites, cancelamento e diagnósticos mantêm comportamento.
- Argumentos são opcionais; array vazio reproduz o fluxo anterior.
- Não muda extensão, framework, Tauri, capabilities ou toolchain exigida.

## Critérios de aceite

1. `std::env::args()` recebe cada argumento intacto, incluindo espaços, acentos, aspas literais e argumentos vazios.
2. Tokens não são concatenados nem executados via shell.
3. Aspas malformadas e limite excedido geram erro claro antes de executar.
4. A UI habilita o campo para `.rs` e documenta a sintaxe de aspas.
5. Cancelamento, consentimento e limites permanecem preservados.

## Risco e rollback

O risco é uma interpretação diferente do texto digitado ou argumentos excessivos. O parser será puro e testado; o backend valida limites independentemente da UI. Rollback remove `args` do contrato e volta a desabilitar o campo em `.rs`; execução sem argumentos continua preservada.
