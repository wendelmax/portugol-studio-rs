# Learning Practice Exercise Implementation Plan

> **For agentic workers:** Implementar na worktree com TDD, revisão independente e CI antes do merge.

**Goal:** Ligar alguns cartões a exercícios reais do catálogo Portugol para que o aluno possa iniciar uma atividade relacionada.

**Architecture:** Mapear IDs explicitamente no conteúdo de aprendizagem; encapsular carregamento/seleção de exercício no frontend existente e reutilizar `openExercise`/`openSource` sem novo IPC.

**Tech Stack:** TypeScript, DOM nativo, Node `node:test`, serviço de catálogo Tauri existente.

**Spec:** `docs/superpowers/specs/2026-11-22-learning-practice-exercise.md`

## Global Constraints

- Somente conceitos explicitamente associados a exercícios comprovadamente existentes.
- Abrir não executa, corrige ou registra progresso.
- Não criar exercícios de Rust falsos nem alterar engine/backend/IPC.
- O botão mostra loading/erro/sucesso real e não permite chamadas duplicadas.

## Review Focus

- Exercício ausente ou catálogo indisponível mantém mensagem de erro e não abre conteúdo falso.
- Cliques repetidos enquanto carrega não criam abas duplicadas.
- Conceitos sem mapa não mostram ação.
- O documento aberto continua associado ao `ExerciseDocument` real para execução/correção posterior.
- Mudar conceito/aba durante a chamada não anuncia o ID de outra ação.

### Task 1: navegação tipada a exercício

**Files:** `apps/desktop/src/exercise-opening.ts`, `apps/desktop/tests/exercise-opening.test.mjs`, `apps/desktop/src/exercises.ts`.

- [x] Testar abertura por ID existente e falha para ID ausente/listagem rejeitada.
- [x] Implementar opener puro com dependências list/open injetadas; integrar o loader/cache já utilizado pelo modal.
- [x] Rodar o teste focado (3/3).

### Task 2: ações de prática em cartões

**Files:** `apps/desktop/src/learning-content.ts`, `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/tests/learning-content.test.mjs`, `apps/desktop/src/style.css`.

- [x] Testar os mapas 2/12/24 e ausência da ação nos conceitos Rust-only.
- [x] Renderizar botão condicional e estado acessível de loading, sucesso ou falha.
- [x] Testar que abertura não chama avaliador nem persistência de progresso; opener só cria e entrega `ExerciseDocument` à navegação.
- [x] Rodar `npm test` (188/188), `npm run build`, `cargo test -p portugol-service --offline --locked` (87 testes), `cargo fmt --all --check`, `git diff --check`.

### Task 3: integração

- [x] Atualizar inventário e relatório de feature.
- [ ] Revisão, CI verde, PR, merge squash e remoção da branch remota.
