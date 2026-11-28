# Layout de painéis por workspace — Implementation Plan

> **For agentic workers:** Execute este plano nativamente na worktree atual com TDD; revisão independente antes do merge.

**Goal:** Persistir separadamente as proporções do explorador, painel inferior e utilitários para cada workspace.

**Architecture:** Reutilizar a chave de preferência por workspace já fornecida pelo backend. `pane-layout.ts` mantém normalização e chaves no armazenamento local; `main.ts` carrega a preferência da aba ativa e grava resize no escopo correto.

**Tech Stack:** TypeScript, Node `node:test`, Tauri existente.

**Spec:** `docs/superpowers/specs/2026-11-22-workspace-pane-layout-design.md`

## Global Constraints

- Sem novo framework, serviço Tauri, dependência ou mudança de CSS.
- Manter compatibilidade com `ferro.pane-layout.v1` e limites em `pane-layout.ts`.
- A preferência deve seguir a aba ativa, sem alterar editor, execução ou modo de aprendizagem.

## Review Focus

- Workspace sem layout próprio deve carregar o global legado e só gravar sua própria chave ao redimensionar.
- Aba standalone usa global mesmo depois de trocar para e de um workspace.
- Layout malformado ou storage indisponível respeita fallback e não quebra a IDE.
- Redimensionar ou trocar abas em viewports distintos mantém dimensões dentro dos limites atuais.
- A restauração deve atualizar as divisórias visuais e seus atributos `aria-valuenow`.

### Task 1: Storage por workspace

**Files:** `apps/desktop/src/pane-layout.ts`, `apps/desktop/tests/pane-layout.test.mjs`.

- [ ] Test-first: provar persistência independente, fallback global, chave legado standalone, valor inválido e erro de storage.
- [ ] Adicionar parâmetro opcional `preferenceKey` a `loadPaneLayout` e `savePaneLayout`, com chave `ferro.pane-layout.v1.workspace.${encodeURIComponent(preferenceKey)}`.
- [ ] Rodar `node --experimental-strip-types --test tests/pane-layout.test.mjs` e confirmar sucesso.

### Task 2: Integração com abas e resize

**Files:** `apps/desktop/src/main.ts`, `apps/desktop/tests/pane-layout.test.mjs`.

- [x] Test-first: os testes da Task 1 cobrem a chave workspace em `loadPaneLayout`/`savePaneLayout`, persistência independente, fallback global e aba standalone.
- [x] Carregar `paneLayout` no começo da ativação e reaplicar os três tamanhos/atributos ARIA já existentes; persistir resize pela preferência ativa.
- [x] Rodar testes focalizados, suíte `npm test` (31 arquivos) e `npm run build`.

### Task 3: Documentação e CI

**Files:** ADR 0010, design/plan, `docs/research/REPOSITORY_DISCOVERY_CURRENT.md`.

- [x] Atualizar inventário para declarar layout por workspace.
- [x] Rodar `cargo fmt --all --check` e `git diff --check`.
- [ ] Abrir PR, aguardar revisão e CI multiplataforma verde antes do merge.

## Self-review

- Todos os requisitos de preferência por workspace, fallback e limites existentes estão cobertos nos testes das Tasks 1–2.
- Sem mudança de interface visual; foco é restaurar o layout real associado a cada projeto.
