# Comparar traduções Rust geradas — Implementation Plan

> **For agentic workers:** Use `superpowers:executing-plans` para executar os itens desta branch com TDD.

**Goal:** Permitir comparar a tradução Rust válida anterior com a atual sem modificar nenhum documento.

**Architecture:** O estado por aba preserva o artefato anterior apenas após uma nova geração válida e diferente. Um módulo puro calcula linhas de diff; a prévia existente alterna entre CodeMirror atual e um painel acessível somente leitura.

**Tech Stack:** TypeScript, DOM nativo, Node `node:test`, CodeMirror 6 existente; sem dependência nova.

**Spec:** `docs/superpowers/specs/2026-11-22-generated-rust-diff-design.md`

## Global Constraints

- Preservar origem e artefato atual; não aplicar mudanças do diff.
- Código gerado permanece somente leitura e identificado como gerado.
- Não apresentar comparação como atual enquanto houver pending/stale.
- Não adicionar dependência frontend.

## Review Focus

- Primeira geração, saída idêntica, erro e stale não devem inventar uma comparação válida.
- Inserções, remoções e substituições mantêm números de linha e rótulos legíveis sem depender de cor.
- Entradas extensas não podem causar alocação ilimitada no diff.
- Alternar de diff para código mantém a saída, source maps e seleção funcional do editor.
- Uma aba/workspace diferente não pode exibir a comparação da aba anterior.

### Task 1: Retenção do artefato anterior

**Files:** `apps/desktop/src/rust-preview-state.ts`, `apps/desktop/tests/rust-preview-state.test.mjs`.

- [x] Escrever testes RED para nova saída diferente, código idêntico, primeira geração, erro e stale.
- [x] Em `acceptRustPreview`, salvar `state.artifact` em `previousArtifact` apenas quando a nova geração for válida e o código diferir; preservar a transição sem diff para código idêntico.
- [x] Rodar testes focados e suíte frontend.

### Task 2: Diff de linhas puro e limitado

**Files:** `apps/desktop/src/generated-rust-diff.ts`, `apps/desktop/tests/generated-rust-diff.test.mjs`.

- [x] Escrever RED para contexto idêntico, inclusão, remoção, substituição, arquivo vazio e fallback acima do limite de células.
- [x] Exportar `diffGeneratedRust(previous, current)` com linhas `context | removed | added`, linha anterior/nova opcional e contagens; limitar LCS a no máximo 1.000.000 células e usar um bloco removido/adicionado no fallback.
- [x] Rodar testes focados; expectativas devem ser fixtures literais.

### Task 3: Alternância acessível na prévia

**Files:** `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/tests/source-map.test.mjs`.

- [x] Adicionar controle “Mostrar diferenças”, painel de linhas antes/depois, resumo textual e botão “Voltar ao código atual”.
- [x] Renderizar diff somente quando existirem duas versões diferentes e a geração atual não estiver pendente/stale; alternar sem editar o CodeMirror atual nem descartar source maps.
- [x] Testar contrato da UI e guarda de estado em testes Node existentes; registrar que diálogo/DOM Tauri não têm runner E2E.

### Task 4: Evidência e integração

**Files:** `docs/research/REPOSITORY_DISCOVERY_CURRENT.md`, spec e este plano.

- [x] Atualizar inventário UI-015/016 com comparação local e limites.
- [x] Rodar `npm test` (177 testes), `npm run build`, `cargo fmt --all --check` e `git diff --check`.
- [ ] Revisão independente, PR e merge somente após os checks ficarem verdes; remover branch remota e atualizar `main` local.

## Self-review

- Os requisitos de histórico temporário, diferença acessível, estado stale e proteção de alocação têm tarefas e testes correspondentes.
- Não muda contratos IPC nem exige migração persistente; rollback é remover o estado auxiliar, painel e renderer.
