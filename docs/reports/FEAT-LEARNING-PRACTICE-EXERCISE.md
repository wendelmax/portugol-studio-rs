# FEAT — Abrir exercício relacionado no inspetor Aprender

- Requisito: UI-024, UC-13; oferecer uma ação para praticar um conceito com exercício do catálogo real.
- Mapeamentos editoriais inspecionados: variáveis → exercício 2 (leitura e escrita do nome), decisões → exercício 12 (par ou ímpar), repetições → exercício 24 (valores inteiros em intervalo crescente).
- Implementação: o cartão mostra “Abrir exercício relacionado” apenas para esses conceitos. O opener carrega/cacheia o catálogo, seleciona pelo ID, cria um `ExerciseDocument` real e chama a navegação de abertura existente.
- Estados: texto acessível informa abertura, falha de catálogo/ID ou ausência de mapeamento. Cliques duplicados durante carregamento são bloqueados.
- Compatibilidade: abrir somente cria/ativa uma aba de exercício; não executa, corrige, altera o arquivo original ou registra progresso. Conceitos Rust-only não apontam para exercício Portugol.

## Validação

- TDD focado: testes de opener, IDs de conceitos, presença condicional do CTA e estado acessível.
- `npm test --prefix apps/desktop` — PASS, 188/188.
- `npm run build --prefix apps/desktop` — PASS; aviso conhecido de chunk acima de 500 kB.
- `cargo test -p portugol-service --offline --locked` — PASS, 87 testes.
- `cargo fmt --all --check` e `git diff --check` — PASS.
- CI multiplataforma: pendente da PR.
- E2E visual/Tauri: NOT_RUN se nenhum runner estiver configurado.
