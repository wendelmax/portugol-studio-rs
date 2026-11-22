# Editor Rust gerado — Implementation Plan

> **For agentic workers:** Execute neste worktree com TDD; revisão independente completa antes do merge.

**Goal:** Exibir a saída Rust em CodeMirror somente leitura com sintaxe, navegação source map e ações de texto puro.

**Architecture:** Um editor secundário existe dentro do `rust-preview` atual. O renderer troca o documento e decorações baseado no artefato aceito; eventos convertem posição gerada em offset da fonte e preservam estado stale.

**Tech Stack:** CodeMirror 6 já instalado, TypeScript, Node `node:test`, Tauri atual.

**Spec:** `docs/superpowers/specs/2026-11-22-generated-rust-editor-design.md`

## Global Constraints

- Sem framework/dependência nova e sem edição automática do documento original.
- Respeitar estados pending/stale/error do `rust-preview-state.ts` e mapeamentos existentes.
- Copiar/salvar/desvincular exporta Rust puro, sem gutter, decorations ou conteúdo de UI.
- Tema, foco e acessibilidade continuam consistentes com o editor atual.

## Review Focus

- Artefato vazio/ausente limpa o editor sem manter mapeamento anterior.
- Artefato stale continua visível e não permite ações, sem navegar por mapeamentos antigos.
- Alternar abas com mesmo texto Rust mas source maps distintos atualiza a origem da navegação.
- Offset mapeado inválido ou posição sem mapa não navega nem lança.
- Mensagem com vários números de linha não contamina copiar/salvar/desvincular.

### Task 1: Componente/estado de Rust gerado

**Files:** criar `apps/desktop/src/generated-rust-editor.ts`, testar em `apps/desktop/tests/generated-rust-editor.test.mjs`.

- [x] RED: testar `EditorState.readOnly`, `EditorView.editable`, Rust document, ranges de decoração e busca do segmento clicado para um offset literal.
- [x] Implementar factory de estado/extension e helper de origem mapeada com posicionamento dentro dos ranges (`from <= pos < to`).
- [x] Verificar testes focados em RED→GREEN.

### Task 2: Integrar no preview existente

**Files:** `apps/desktop/index.html`, `apps/desktop/src/main.ts`, `apps/desktop/src/style.css`, `apps/desktop/tests/source-map.test.mjs`.

- [x] Trocar `<pre id="rust-output">` por mount point acessível e montar `EditorView` readonly, linha numerada e rust parser.
- [x] Sincronizar conteúdo/decorations em toda renderização ativa, inclusive em troca de abas, e trocar `copy/save/detach` para `generatedRustEditor.state.doc.toString()`.
- [x] Ligar clique de segmento ao offset fonte e `focusRustMapping` à linha CodeMirror; atualizar o segundo editor em `applyTheme`.
- [x] Reaproveitar status/stale existente e garantir ações desabilitadas conforme o renderer atual.
- [x] Rodar teste focado de estado/source map e manter texto puro/segmentos reatribuídos por render da aba ativa.

### Task 3: Validação da fatia

**Files:** ADR 0011, design/plan, inventário de repositório.

- [x] Atualizar inventário de UI-013/014/015 para distinguir EditorView Rust gerado do preview `<pre>` anterior.
- [x] Rodar `npm test` (32 arquivos, 162 testes), `npm run build`, `cargo fmt --all --check` e `git diff --check`.
- [ ] Revisão independente, PR, todos os checks CI verdes, merge e excluir branch remota.

## Self-review

- O design cobre edição read-only, sintaxe, tema, source maps, stale e texto exportado sem HTML.
- Testes unitários verificam estado e helpers; integração visual Tauri/E2E continua explicitamente pendente.
