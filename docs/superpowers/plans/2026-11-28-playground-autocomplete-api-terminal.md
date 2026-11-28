# Playground Autocomplete, Syntax Reference e Terminal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modernizar o Web Playground Ferro com autocomplete das instruções/sintaxes `.fer`, um painel de referência da linguagem, menos botões e um terminal que mostra entrada e saída juntas, sem dependências e sem mudar o WASM.

**Architecture:** Mantemos o playground estático atual (`index.html`, `app.mjs`, `style.css`, `playground-core.mjs`) e adicionamos um único módulo `.mjs` com a referência de sintaxe e funções puras de completions. A UI reutiliza o realce e o runtime existentes; o terminal permanece em modo histórico porque `pors_run(source, stdin)` já recebe toda a entrada de uma vez.

**Tech Stack:** JavaScript ESM sem dependências, DOM APIs (`createElement`/`textContent`), WebAssembly via `ferro-wasm`, Node `node:test` para testes.

**Spec:** `docs/superpowers/specs/2026-11-28-playground-autocomplete-api-terminal-design.md`

## Global Constraints

- `apps/web-playground` permanece sem dependências JavaScript externas e sem `innerHTML` em `index.html`, `app.mjs`, `playground-core.mjs` ou `ferro-language.mjs`.
- Textos visíveis em pt-BR; temas `data-theme="light|dark"` e os breakpoints 980 px / 720 px / 420 px continuam funcionando.
- `ferro_wasm` e o protocolo de memória linear não são alterados: `pors_run(source, stdin)` continua recebendo o conteúdo integral de `#pg-stdin`.
- `scripts/build-web-playground.sh` deve copiar `ferro-language.mjs` para `apps/web-playground/dist/`.
- Comandos de teste são executados a partir da raiz do repositório com `node --test`; não é necessário instalar `node_modules`.
- Nenhuma mensagem de commit contém atribuição a IA, Claude ou Anthropic.

## Review Focus

- Completions dentro de `// comentário` ou de uma cadeia aberta não devem aparecer — teste em Task 1.
- `Resultado::O` deve ser tratado como um único token e completar para `Resultado::Ok`, sem perder o `Resultado::` — teste em Task 1.
- Entrada com linha em branco no meio deve aparecer como linha de terminal; a linha vazia criada pelo `\n` final não deve virar linha extra — teste em Task 4.
- Saída com conteúdo parecido com HTML (`<script>`) deve aparecer como texto literal — teste em Task 4.
- Busca da referência sem resultados deve mostrar estado vazio sem lançar erro, e clique em item deve inserir o trecho no caret — teste em Task 3.

---

### Task 1: Referência de sintaxe e funções de completion

**Files:**
- Create: `apps/web-playground/ferro-language.mjs`
- Modify: `scripts/build-web-playground.sh:15-19`
- Test: `apps/desktop/tests/ferro-language.test.mjs`

**Interfaces:**
- Produces: `FERRO_SYNTAX_ENTRIES` (`[{ id, label, kind, group, signature, description, example, insertText }]`), `wordRangeAt(source, caret) -> { start, end, prefix }` e `completeFerro(source, caret) -> entries[]` com no máximo 12 itens. Nenhum outro módulo é consumido.

- [ ] **Step 1: Write the failing test**

```js
// apps/desktop/tests/ferro-language.test.mjs
import assert from "node:assert/strict";
import test from "node:test";
import { FERRO_SYNTAX_ENTRIES, completeFerro, wordRangeAt } from "../../web-playground/ferro-language.mjs";

const labels = (source) => completeFerro(source, source.length).map((entry) => entry.label);

test("complete Ferro keywords, types and builtins by prefix", () => {
  assert.ok(labels("escr").includes("escreval"));
  assert.ok(labels("inte").includes("inteiro"));
  assert.ok(completeFerro("escr", 4).every((entry) => entry.insertText === entry.label));
});

test("wordRangeAt keeps Resultado:: as part of the same token", () => {
  const source = "Resultado::O";
  assert.deepEqual(wordRangeAt(source, source.length), {
    start: 0,
    end: 12,
    prefix: "Resultado::O",
  });
  assert.ok(labels(source).includes("Resultado::Ok"));
});

test("completions are suppressed inside comments and strings", () => {
  assert.deepEqual(labels("// escr"), []);
  const quoted = 'escreval("escr";';
  assert.deepEqual(completeFerro(quoted, quoted.indexOf('";')), []);
});

test("syntax entries cover every reference group and carry examples", () => {
  const groups = new Set(FERRO_SYNTAX_ENTRIES.map((entry) => entry.group));
  for (const group of [
    "Estrutura",
    "Declarações",
    "Tipos",
    "Controle",
    "Dados",
    "Entrada e saída",
    "Valores",
    "Operadores",
  ]) {
    assert.ok(groups.has(group), `grupo ${group} ausente`);
  }
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Estrutura" && e.label === "funcao"));
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Controle" && e.label === "enquanto"));
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Entrada e saída" && e.label === "escreval"));
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Operadores" && e.label === "?"));
  for (const entry of FERRO_SYNTAX_ENTRIES) {
    assert.ok(entry.signature.length > 0, entry.label);
    assert.ok(entry.description.length > 0, entry.label);
    assert.ok(entry.example.length > 0, entry.label);
    assert.equal(entry.insertText, entry.label);
  }
  assert.ok(completeFerro("T", 1).length <= 12);
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test apps/desktop/tests/ferro-language.test.mjs`
Expected: FAIL porque `ferro-language.mjs` ainda não existe.

- [ ] **Step 3: Implement `ferro-language.mjs`**

Defina `FERRO_SYNTAX_ENTRIES` com entradas únicas nos grupos abaixo, na ordem em que devem aparecer na referência e no autocomplete. Use `insertText === label` para todas. Textos de `description`/`example` podem ser curtos, mas devem ser válidos em pt-BR e derivados da sintaxe já documentada em `apps/desktop/src/portugol.ts`, `docs/PORTUGOL-RUST.md` e RFCs de `docs/superpowers/specs/`:

- `Estrutura`: `funcao`, `inicio`, `retorne`.
- `Declarações`: `seja`, `mutavel`.
- `Tipos`: `inteiro`, `real`, `logico`, `cadeia`, `caracter`, `vetor`, `Opcao`, `Resultado`.
- `Controle`: `se`, `senao`, `senao_se`, `enquanto`, `faca_enquanto`, `para`, `pare`, `continue`, `escolha`, `caso`.
- `Dados`: `registro`, `enum`, `adicionar`, `remover`, `obter`, `tamanho`.
- `Entrada e saída`: `escreva`, `escreval`, `leia`, nesta ordem.
- `Valores`: `verdadeiro`, `falso`, `nulo`, `Resultado::Ok`, `Resultado::Erro`, `Opcao::Algum`, `Opcao::Nenhum`.
- `Operadores`: `?`, `??`, `??=`, `?.`, `+=`, `&&`, `||`.

Implemente `wordRangeAt(source, caret)`: retroceda sobre `[A-Za-z0-9_:]` a partir de `caret` e devolva `{ start, end: caret, prefix: source.slice(start, caret) }`. Implemente `completeFerro(source, caret)`: retorne `[]` quando a linha até o caret contiver `//` fora de cadeia ou quando houver quantidade ímpar de aspas não escapadas antes do caret; caso contrário, use `wordRangeAt` e filtre `FERRO_SYNTAX_ENTRIES` por `label.toLowerCase().startsWith(prefix.toLowerCase())`, preservando a ordem da lista e limitando a 12.

- [ ] **Step 4: Add the module to the bundle script**

Em `scripts/build-web-playground.sh`, inclua `apps/web-playground/ferro-language.mjs` na lista de `cp` junto dos quatro arquivos atuais.

- [ ] **Step 5: Run the test to verify it passes**

Run: `node --test apps/desktop/tests/ferro-language.test.mjs`
Expected: PASS com 4 testes.

- [ ] **Step 6: Commit**

```bash
git add apps/web-playground/ferro-language.mjs scripts/build-web-playground.sh apps/desktop/tests/ferro-language.test.mjs
git commit -m "feat: adiciona referência e completions da sintaxe Ferro"
```

### Task 2: Popup de autocomplete no editor

**Files:**
- Modify: `apps/web-playground/index.html:70-78`
- Modify: `apps/web-playground/style.css:394-470`
- Modify: `apps/web-playground/app.mjs:1-30,128-137,316-328`
- Modify: `apps/desktop/tests/web-playground.test.mjs:192-224`
- Test: `apps/desktop/tests/web-playground.test.mjs`

**Interfaces:**
- Consumes: `completeFerro(source, caret)` e `wordRangeAt(source, caret)` de `apps/web-playground/ferro-language.mjs` (Task 1).
- Produces: `completionState` interno em `app.mjs`; nenhum módulo externo depende dele.

- [ ] **Step 1: Write the failing test**

Adicione em `apps/desktop/tests/web-playground.test.mjs` um teste `web playground autocomplete suggests Ferro words and supports keyboard selection` com um fake DOM próprio (elementos com `listeners` como arrays e método `fire(type, event)`), fornecendo todos os ids já usados por `app.mjs` mais `pg-completions`. Importe `../../web-playground/app.mjs?autocomplete` com `globalThis.document` trocado. As asserções são:

```js
const editor = elements.get("pg-editor");
const list = elements.get("pg-completions");
editor.value = "escr";
editor.selectionStart = editor.selectionEnd = 4;
editor.fire("input");
assert.equal(list.hidden, false);
assert.ok(list.children.some((child) => child.textContent === "escreval"));

editor.fire("keydown", { key: "ArrowDown", ctrlKey: false, metaKey: false, preventDefault() {} });
editor.fire("keydown", { key: "Enter", ctrlKey: false, metaKey: false, preventDefault() {} });
assert.equal(editor.value, "escreval");
assert.equal(list.hidden, true);

editor.value = "// escr";
editor.selectionStart = editor.selectionEnd = 7;
editor.fire("input");
assert.equal(list.hidden, true);
```

No teste existente `Prévia UI loads a visual example...`, acrescente `"pg-completions"` ao array `ids` para que o `getElementById` não devolva `null`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test apps/desktop/tests/web-playground.test.mjs`
Expected: FAIL porque `#pg-completions` e o comportamento de teclado ainda não existem.

- [ ] **Step 3: Add the popup markup and styles**

Em `index.html`, dentro de `.pg-editor-wrap` após o `<textarea id="pg-editor">`, adicione `<ul id="pg-completions" class="pg-completions" role="listbox" aria-label="Sugestões de sintaxe Ferro" hidden></ul>` e inclua `aria-autocomplete="list" aria-controls="pg-completions"` no textarea. Em `style.css`, estilize `.pg-completions` como painel absoluto dentro do wrapper, com `z-index` acima do editor, `max-height`, `overflow: auto`, `.pg-completion-item` com `cursor: pointer`, item ativo via `[aria-selected="true"]`, e regra `@media (forced-colors: active)` mantendo o popup legível.

- [ ] **Step 4: Implement the popup behavior in `app.mjs`**

Importe `completeFerro` e `wordRangeAt`. Adicione `const completionsEl = byId("pg-completions")` e `let completionState = null`. Implemente `closeCompletions()` (limpa filhos, marca `hidden`, zera estado), `renderCompletions(entries, activeIndex, range)` (cria `<li role="option">` com `textContent` do `label`, define `aria-selected`, guarda estado), `applyCompletion(index)` (substitui `source.slice(range.start, range.end)` por `entry.insertText`, atualiza `selectionStart/selectionEnd`, chama `setEditorValue`, fecha popup e mantém foco) e `updateCompletions()` (usa `editorEl.selectionStart`, chama `completeFerro` e abre/fecha o popup). No handler de `input` do editor, chame `renderHighlight` e `updateCompletions`. No `keydown`, trate `Escape` (fechar), `ArrowDown`/`ArrowUp` (mover índice com `preventDefault`), `Enter`/`Tab` quando o popup estiver aberto (aplicar com `preventDefault`); só depois verifique `Ctrl/Cmd+Enter`. Feche o popup em `loadExample`, `runFormat` e `blur` do editor.

- [ ] **Step 5: Run the test to verify it passes**

Run: `node --test apps/desktop/tests/web-playground.test.mjs`
Expected: PASS com 9 testes, incluindo o novo teste de autocomplete.

- [ ] **Step 6: Commit**

```bash
git add apps/web-playground/index.html apps/web-playground/style.css apps/web-playground/app.mjs apps/desktop/tests/web-playground.test.mjs
git commit -m "feat: adiciona popup de autocomplete ao playground"
```

### Task 3: Painel de referência de sintaxe e instruções

**Files:**
- Modify: `apps/web-playground/index.html:119-121`
- Modify: `apps/web-playground/style.css:640-660`
- Modify: `apps/web-playground/app.mjs:1-30,282-289`
- Modify: `apps/desktop/tests/web-playground.test.mjs:15-18,192-224,389-460`
- Test: `apps/desktop/tests/web-playground.test.mjs`

**Interfaces:**
- Consumes: `FERRO_SYNTAX_ENTRIES` de `apps/web-playground/ferro-language.mjs` (Task 1) e `wordRangeAt` (Task 1).
- Produces: funções internas `renderSyntaxPanel(filter)` e `insertSyntaxEntry(entry)` em `app.mjs`.

- [ ] **Step 1: Write the failing tests**

Em `apps/desktop/tests/web-playground.test.mjs`:

```js
test("web playground ships a searchable Ferro syntax reference panel", () => {
  const html = readFileSync(new URL("../../web-playground/index.html", import.meta.url), "utf8");
  assert.match(html, /id="pg-syntax-panel"/);
  assert.match(html, /id="pg-syntax-search"[^>]*type="search"/);
  assert.match(html, /id="pg-syntax-list"/);
  assert.match(html, /id="pg-syntax-empty"/);
});
```

Acrescente um teste com fake DOM importando `../../web-playground/app.mjs?syntax-panel` que: (a) espera `pg-syntax-list.children.length > 0` após o carregamento; (b) preenche `pg-syntax-search.value = "inexistente"`, dispara `input` e espera `children.length === 0` com `pg-syntax-empty.hidden === false`; (c) preenche `"escreval"`, dispara `input`, localiza o item cujo `textContent` contém `escreval`, dispara `click` e espera que `pg-editor.value` contenha `escreval`. Atualize os arrays `ids` dos testes com fake DOM para incluir `pg-syntax-search`, `pg-syntax-list` e `pg-syntax-empty`. Na lista de arquivos do teste `without innerHTML`, inclua `ferro-language.mjs`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test apps/desktop/tests/web-playground.test.mjs`
Expected: FAIL porque o painel de sintaxe não existe.

- [ ] **Step 3: Add markup and styles for the panel**

Após `.pg-panels` em `index.html`, adicione:

```html
<section id="pg-syntax-panel" class="pg-card pg-syntax-card" aria-labelledby="pg-syntax-heading">
  <header class="pg-card-header">
    <div>
      <p class="pg-eyebrow">Referência</p>
      <h2 id="pg-syntax-heading">Sintaxe e instruções Ferro</h2>
    </div>
    <span class="pg-badge">.fer</span>
  </header>
  <label class="pg-syntax-search" for="pg-syntax-search">
    <span>Buscar instrução</span>
    <input id="pg-syntax-search" type="search" placeholder="enquanto, Resultado::Ok, ??…" />
  </label>
  <div id="pg-syntax-list" class="pg-syntax-list"></div>
  <p id="pg-syntax-empty" class="pg-empty" hidden>Nenhuma instrução encontrada.</p>
</section>
```

Em `style.css`, estilize `.pg-syntax-card` como card full-width, `.pg-syntax-list` como grade responsiva, `.pg-syntax-group` com cabeçalho, `.pg-syntax-item` como botão/linha selecionável, `.pg-syntax-signature` em fonte monoespaçada e `.pg-syntax-description`/`.pg-syntax-example` com cor `--pg-muted`. Reuse `--pg-radius`, `--pg-border` e variáveis de tema existentes.

- [ ] **Step 4: Render and wire the panel in `app.mjs`**

Importe `FERRO_SYNTAX_ENTRIES`. Adicione os elementos `pg-syntax-search`, `pg-syntax-list`, `pg-syntax-empty`. Implemente `renderSyntaxPanel(filter = "")` agrupando as entradas por `group` na ordem em que aparecem em `FERRO_SYNTAX_ENTRIES`; cada item é um `<button type="button" class="pg-syntax-item">` com `textContent` igual a `label` e com `signature`, `description` e `example` em elementos filhos. O filtro compara `label`, `group`, `signature` e `description` em minúsculas. Implemente `insertSyntaxEntry(entry)` usando `wordRangeAt(editorEl.value, editorEl.selectionStart)`, substituindo o trecho por `entry.insertText`, chamando `setEditorValue`, `closeCompletions` e `editorEl.focus()`. Registre o listener de `input` da busca e chame `renderSyntaxPanel()` na inicialização.

- [ ] **Step 5: Run the test to verify it passes**

Run: `node --test apps/desktop/tests/web-playground.test.mjs`
Expected: PASS com 11 testes, incluindo o painel de sintaxe.

- [ ] **Step 6: Commit**

```bash
git add apps/web-playground/index.html apps/web-playground/style.css apps/web-playground/app.mjs apps/desktop/tests/web-playground.test.mjs
git commit -m "feat: adiciona painel de sintaxe e instruções Ferro"
```

### Task 4: Combobox de ações e terminal de entrada/saída

**Files:**
- Modify: `apps/web-playground/index.html:33-118`
- Modify: `apps/web-playground/style.css:228-533`
- Modify: `apps/web-playground/app.mjs:9-30,139-230,301-305`
- Modify: `apps/desktop/tests/web-playground.test.mjs:21-32,192-321`
- Test: `apps/desktop/tests/web-playground.test.mjs`

**Interfaces:**
- Consumes: funções existentes `runValidate`, `runFormat`, `runConsole`, `runUiSession`, `runExportRust`.
- Produces: `renderTerminal(stdin, stdout, diagnostics)` em `app.mjs`; `#pg-action` dispara as ações não padrão e `#pg-run` continua disparando `runConsole`.

- [ ] **Step 1: Write the failing tests**

Em `apps/desktop/tests/web-playground.test.mjs`:

```js
test("web playground replaces action buttons with a single command combobox and a terminal", () => {
  const html = readFileSync(new URL("../../web-playground/index.html", import.meta.url), "utf8");
  for (const id of ["pg-validate", "pg-format", "pg-run-ui", "pg-export-rust", "pg-output"]) {
    assert.doesNotMatch(html, new RegExp(`id="${id}"`));
  }
  assert.match(html, /id="pg-action"/);
  assert.match(html, /<option value="validate">Validar<\/option>/);
  assert.match(html, /id="pg-terminal-history"/);
  assert.match(html, /id="pg-stdin"/);
});
```

Acrescente um teste de DOM com runtime falso (o mesmo padrão do teste de prévia UI) cobrindo: (a) `pg-stdin.value = "Ada\n\n"`, `pors_run` devolve `stdout: "Olá\n<script>x</script>\n"`; depois de disparar `pg-run`, `pg-terminal-history.children` contém uma linha `data-kind="input"` com texto `Ada`, uma segunda linha de entrada vazia, duas linhas `data-kind="output"` e nenhuma terceira linha de entrada; o texto `<script>x</script>` permanece literal; (b) trocar `pg-action.value = "validate"` e disparar `change` faz `pg-status-text` conter "válido" sem clicar em botão. Atualize o teste `Prévia UI...` para trocar `elements.get("pg-run-ui").fire("click")` por `const action = elements.get("pg-action"); action.value = "ui"; action.fire("change")`, e remova dos arrays `ids` os elementos extintos, acrescentando `pg-action` e `pg-terminal-history`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test apps/desktop/tests/web-playground.test.mjs`
Expected: FAIL porque `#pg-action` e `#pg-terminal-history` ainda não existem.

- [ ] **Step 3: Replace the controls and output markup**

Em `index.html`, substitua `.pg-actions` por:

```html
<label class="pg-select-wrap pg-action-wrap" for="pg-action">
  <span>Ação</span>
  <select id="pg-action" aria-label="Outras ações do playground">
    <option value="">Outras ações…</option>
    <option value="validate">Validar</option>
    <option value="format">Formatar</option>
    <option value="ui">Prévia UI</option>
    <option value="rust">Ver Rust</option>
  </select>
</label>
<button id="pg-run" type="button" class="pg-primary">Executar <span class="pg-shortcut" aria-hidden="true">Ctrl+Enter</span></button>
```

No card de resultado, troque o título para `Terminal`, o badge para `id="pg-terminal-badge"`, remova `pg-diagnostics`, `pg-output` e `pg-output-empty`, e use:

```html
<div class="pg-terminal">
  <div id="pg-terminal-history" class="pg-terminal-history" role="log" aria-live="polite" aria-label="Histórico do terminal"></div>
  <label class="pg-terminal-input" for="pg-stdin">
    <span aria-hidden="true">&gt;</span>
    <textarea id="pg-stdin" class="pg-stdin" rows="2" spellcheck="false" aria-label="Entrada padrão do programa" placeholder="Uma linha por leitura (leia). Pressione Executar para rodar."></textarea>
  </label>
</div>
```

Mantenha `#pg-ui-section` e `#pg-rust-section` intactos. Em `style.css`, remova as regras `.pg-output`/`.pg-output-wrap`/`.pg-diagnostics` que não são mais usadas e adicione `.pg-terminal`, `.pg-terminal-history` (altura fixa, `overflow: auto`, fonte monoespaçada), `.pg-terminal-line[data-kind="input"]` com `::before { content: "> "; }`, `.pg-terminal-line[data-kind="error"]` com cor de erro e `.pg-terminal-input` com o prompt `>` alinhado ao textarea. Ajuste os media queries para o novo combobox.

- [ ] **Step 4: Rewire `app.mjs`**

Remova as constantes e listeners de `btnValidate`, `btnFormat`, `btnUi`, `btnExportRust`, `outputEl`, `outputEmptyEl` e `diagnosticsEl`; adicione `actionSelect` e `terminalHistoryEl`. Substitua `syncOutput` e `renderDiagnostics` por `renderTerminal(stdin, stdout, diagnostics)`: limpe o histórico; para cada linha de `stdin.split("\n")`, descarte apenas o último elemento vazio quando a entrada termina com `\n`; para `stdout` faça o mesmo; crie `<div class="pg-terminal-line" data-kind="input|output|error">` com `textContent`; quando não houver linhas, crie uma linha `data-kind="empty"` com texto `Execute para ver entrada e saída aqui.` e atualize `pg-terminal-badge` com a quantidade de diagnósticos. Chame `renderTerminal` em `loadExample`, `runValidate`, `runFormat`, `runConsole`, `applyUiResult` e `runExportRust`. Em `setControlsEnabled`, habilite/desabilite apenas `btnRun` e `actionSelect`. Registre `actionSelect.addEventListener("change", ...)` mapeando `validate|format|ui|rust` para as funções existentes e resetando `actionSelect.value = ""` depois de cada ação. `btnRun` continua chamando `runConsole`, e o atalho `Ctrl/Cmd+Enter` permanece como está.

- [ ] **Step 5: Run the test to verify it passes**

Run: `node --test apps/desktop/tests/web-playground.test.mjs`
Expected: PASS com 13 testes, incluindo o terminal e o combobox de ações.

- [ ] **Step 6: Commit**

```bash
git add apps/web-playground/index.html apps/web-playground/style.css apps/web-playground/app.mjs apps/desktop/tests/web-playground.test.mjs
git commit -m "refactor: unifica terminal e ações do playground"
```

### Task 5: Documentação e verificação final

**Files:**
- Modify: `apps/web-playground/README.md:10-19`
- Test: comandos de verificação abaixo.

**Interfaces:**
- Consumes: todos os módulos e testes das Tasks 1–4.
- Produces: bundle estático atualizado e documentação coerente com a UI.

- [ ] **Step 1: Update the README feature list**

Substitua a lista de recursos por uma que inclua: autocomplete das instruções/sintaxes `.fer`, painel pesquisável de sintaxe com exemplos, combobox de ações com `Executar` como botão primário e terminal com entrada/saída no mesmo histórico. Mantenha o texto sobre validação, formatação, execução WASM, prévia MVU e temas.

- [ ] **Step 2: Run the full focused test suite**

Run: `node --test apps/desktop/tests/ferro-language.test.mjs apps/desktop/tests/web-playground.test.mjs`
Expected: PASS com 17 testes, 0 falhas.

- [ ] **Step 3: Verify the diff and the bundle script**

Run: `git diff --check && bash -n scripts/build-web-playground.sh && grep -n "ferro-language.mjs" scripts/build-web-playground.sh`
Expected: exit 0, nenhum aviso de whitespace e uma linha de `cp` contendo `ferro-language.mjs`.

- [ ] **Step 4: Commit**

```bash
git add apps/web-playground/README.md
git commit -m "docs: descreve playground com autocomplete e terminal"
```
