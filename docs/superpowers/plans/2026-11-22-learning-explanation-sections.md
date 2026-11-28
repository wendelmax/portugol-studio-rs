# Learning Explanation Sections Implementation Plan

> **For agentic workers:** Implementar esta fatia na worktree atual com TDD, revisão de código e verificação antes do PR.

**Goal:** Completar os cartões UI-024 com mudança, relevância e prática apropriadas a cada conceito.

**Architecture:** Ampliar o catálogo editorial tipado já consumido pelo inspetor. Renderizar campos por `textContent`, sem backend, persistência ou engine.

**Tech Stack:** TypeScript, HTML/CSS existentes, Node `node:test`, Vite.

**Spec:** `docs/superpowers/specs/2026-11-22-learning-explanation-sections.md`

## Global Constraints

- Preservar catálogo existente, modos, exemplos e proveniência.
- Sem inferência pelo cursor, progresso por abertura ou alteração no runtime.
- Não adicionar dependências nem comandos Tauri.

## Review Focus

- Todos os conceitos de cada modo, inclusive Rust-only, devem ter os campos obrigatórios.
- Texto editorial não deve afirmar que snippets são gerados/compilados.
- Seleção manual e tópico fixado devem continuar renderizando o conceito e seções correspondentes.
- Texto não confiável é exibido como texto, nunca interpretado como HTML.
- Conteúdo permanece legível no painel estreito e com teclado/leitor de tela.

### Task 1: contrato do catálogo

**Files:** `apps/desktop/tests/learning-content.test.mjs`, `apps/desktop/src/learning-content.ts`.

- [x] Testar que cada conceito de beginner, bridge e rust tem `whatChanged`, `whyItMatters`, `practice` não vazios; observar falha antes da implementação.
- [x] Adicionar conteúdo editorial específico aos conceitos existentes.
- [x] Executar teste focado e conferir a distinção Ferro/Rust em mutabilidade, ownership e borrowing.

### Task 2: renderização acessível

**Files:** `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/tests/learning-content.test.mjs`.

- [x] Testar três regiões nomeadas no cartão e que `main.ts` preenche os conteúdos com `textContent`.
- [x] Renderizar seções para o conceito resolvido, incluindo contexto mapeado e pin válido sem mudar a proveniência existente.
- [x] Estilizar hierarquia/espacamento sem fixar altura nem esconder texto no painel estreito.
- [x] Rodar `npm test` (184/184), `npm run build`, `cargo test -p ferro-service --offline --locked` (57 testes), `cargo fmt --all --check` e `git diff --check`.

### Task 3: integração

- [x] Atualizar `docs/research/REPOSITORY_DISCOVERY_CURRENT.md` e `docs/reports/FEAT-LEARNING-EXPLANATION-SECTIONS.md`.
- [ ] Revisão independente, CI verde, PR e merge seguindo a autorização do usuário; apagar branch remota e atualizar a main.
