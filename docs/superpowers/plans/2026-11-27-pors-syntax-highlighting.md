# PORS Syntax Highlighting (IDE e Playground) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dar a `escreval`, `leia()` e aos trechos `{...}` de cadeias o mesmo realce na IDE CodeMirror e no Web Playground, com variável de tema dedicada e texto do `<pre>` idêntico ao `<textarea>`.

**Architecture:** Manter os dois editores com suas arquiteturas atuais — `StreamLanguage` no desktop e regex + spans no playground — mas compartilhar a taxonomia de tokens do spec. Nenhuma mudança em frontend/HIR/backends: este plano consome a sintaxe já especificada e apenas a expor visualmente.

**Tech Stack:** TypeScript, CodeMirror 6, Lezer highlight, CSS custom properties, JavaScript ESM, `node:test`.

**Spec:** `docs/superpowers/specs/2026-11-26-pors-syntax-evolution-design.md`

## Global Constraints

- `escreval` é `builtin`, nunca `keyword`; `leia` continua `builtin`, inclusive em `leia()`.
- Não sugerir `leia<T>()` nesta fatia.
- Um `{` não escapado só inicia interpolação se houver `}` correspondente na mesma linha da cadeia e conteúdo não vazio. `\{` e `\}` são literais. `{}` e `{` sem fechamento permanecem `string`.
- IDE: braces são `interpolation`, identificador/inner tokens dentro das braces usam as classes normais. Playground: o segmento `{...}` inteiro recebe `pg-tok-interpolation`, sem span interno.
- Cores: `--syntax-interpolation` na IDE, `--pg-interpolation` no playground, ambos com fallback para cor de tipo/função existente.
- `<pre id="pg-highlight">` deve ter `textContent` **exatamente igual** a `<textarea id="pg-editor">.value`; sem `\n` extra de alinhamento.
- Nenhum uso de `innerHTML`; apenas `createTextNode`/`createElement`.
- Commits sem `Co-Authored-By`, sem menção a Claude/IA/Anthropic, sem emoji.
- Testes CodeMirror exigem dependências em `apps/desktop/node_modules`; se `@codemirror/state` não resolver, rodar `npm --prefix apps/desktop install` antes.

## Review Focus

- `\{nome}` e `\}` devem ser texto literal, sem token de interpolação nos dois editores — teste em `pors-interpolation-highlight.test.mjs` e no teste do playground.
- `{}` e uma cadeia com `{` sem fechamento devem permanecer `string` — teste no mesmo arquivo.
- `"Olá, {nome}!"` tem acentos: os spans devem cair nos bytes UTF-8 corretos — teste compara `sliceString` exato, não índices de caractere.
- Incluir `escreval` em `PORS_BUILTINS` sem criar `PORS_TOP_LEVEL_DOCS.escreval` quebra `PORS_WORDS` em `portugol.ts:191-192` — teste de completion e hover cobre a entrada.
- O `\n` extra em `app.mjs:76` não é detectado pela suíte atual — teste novo compara `textContent` do `<pre>` com `value` do `<textarea>`.

---

### Task 1: `escreval` e `leia()` como builtins documentados na IDE

**Files:**
- Modify: `apps/desktop/src/portugol.ts:46` (`PORS_BUILTINS`), `:73-101` (`PORS_TOP_LEVEL_DOCS`)
- Modify: `apps/desktop/src/help-navigation.ts:182-216` (`CONTEXTUAL_TOPIC_MAP`)
- Test: `apps/desktop/tests/pors-completions.test.mjs`

**Interfaces:**
- Consumes: `portugolRs()`, `porsHoverInfo(doc: string, pos: number)`, helper local `completeAt(doc, pos)`.
- Produces: `PORS_BUILTINS` com `escreval`; `PORS_TOP_LEVEL_DOCS.escreval` e `.leia` atualizada; mapeamento de ajuda para `escreval`.

- [ ] **Step 1: Write the failing test**

Adicionar em `apps/desktop/tests/pors-completions.test.mjs`:

```js
test("PORS documents escreval and leia() as builtins", async () => {
  const completions = await completeAt("escr");
  const entry = completions.options.find((o) => o.label === "escreval");
  assert.ok(entry);
  assert.equal(entry.apply, "escreval(");
  assert.match(entry.detail, /escreval\(/);

  const hover = porsHoverInfo('escreval("Olá");', 3);
  assert.equal(hover.title, "escreval");
  assert.match(hover.description, /quebra de linha/);

  const readHover = porsHoverInfo("leia();", 1);
  assert.match(readHover.signature, /leia\(\)/);
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm --prefix apps/desktop install` (se necessário) e, em `apps/desktop`, `node --experimental-strip-types --test tests/pors-completions.test.mjs`
Expected: FAIL com `entry` ausente (`PORS_BUILTINS` não contém `escreval`).

- [ ] **Step 3: Add the documentation entries**

Em `portugol.ts`, acrescentar `"escreval"` a `PORS_BUILTINS` e criar `PORS_TOP_LEVEL_DOCS.escreval` com `kind: "function"`, `signature: "escreval(valor, ...)"` e descrição declarando a quebra de linha e a equivalência com `escreva(..., "\n")`. Atualizar `PORS_TOP_LEVEL_DOCS.leia.signature` para cobrir as duas formas (`leia()` e `leia(variavel_mutavel)`) e a descrição para dizer que `leia()` devolve `cadeia`. Em `help-navigation.ts`, mapear `escreval` para o mesmo tópico de ajuda de `escreva`.

- [ ] **Step 4: Run test to verify it passes**

Run: `node --experimental-strip-types --test tests/pors-completions.test.mjs`
Expected: PASS, sem regressão nos testes existentes do arquivo.

- [ ] **Step 5: Commit**

```bash
git add apps/desktop/src/portugol.ts apps/desktop/src/help-navigation.ts apps/desktop/tests/pors-completions.test.mjs
git commit -m "feat(editor): documentar escreval e leia() como builtins"
```

---

### Task 2: Scanner de interpolação no CodeMirror

**Files:**
- Modify: `apps/desktop/src/portugol.ts:201-205`, `:236-237`, `:269-274`
- Test: `apps/desktop/tests/pors-interpolation-highlight.test.mjs` (criar)

**Interfaces:**
- Consumes: `portugolRs()`; convenção de que o nó da `syntaxTree` tem o nome devolvido por `StreamParser.token()`.
- Produces: tokens `string`, `interpolation` e `variableName` com spans exatos; `tokenTable.interpolation`.

- [ ] **Step 1: Write the failing tests**

Criar `apps/desktop/tests/pors-interpolation-highlight.test.mjs`:

```js
import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { portugolRs } from "../src/portugol.ts";

function tokens(doc) {
  const state = EditorState.create({ doc, extensions: [portugolRs()] });
  const found = [];
  syntaxTree(state).iterate({
    enter(node) {
      const slice = state.doc.sliceString(node.from, node.to);
      if (["string", "interpolation", "variableName", "number", "operator"].includes(node.name)) {
        found.push([node.name, slice]);
      }
    },
  });
  return found;
}

test("PORS highlights interpolated parts with exact UTF-8 spans", () => {
  assert.deepEqual(tokens('"Olá, {nome}!"'), [
    ["string", '"Olá, '],
    ["interpolation", "{"],
    ["variableName", "nome"],
    ["interpolation", "}"],
    ["string", '!"'],
  ]);
});

test("PORS keeps escaped and malformed braces as literal string", () => {
  for (const doc of [String.raw`escreval("\{nome}");`, 'escreval("{}");', 'escreval("abri {");']) {
    assert.ok(tokens(doc).every(([name]) => name !== "interpolation"), doc);
  }
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `node --experimental-strip-types --test tests/pors-interpolation-highlight.test.mjs`
Expected: FAIL — `"Olá, {nome}!"` produz um único nó `string`.

- [ ] **Step 3: Implement the scanner**

Em `PorsState`, acrescentar o estado necessário para a cadeia corrente e para a interpolação. Substituir os dois `stream.match` de cadeia em `token()` por um scanner que:
  - consome `\{` e `\}` como parte do `string`;
  - ao encontrar `{` não escapado com `}` correspondente e conteúdo não vazio na mesma linha, emite `interpolation` para `{` e `}`, classificando o conteúdo com as regras já existentes em `:239-260` (`variableName`, `number`, `operator`, `string`);
  - fecha a cadeia e devolve `string` quando encontra a aspas de fechamento;
  - mantém `{}` e `{` sem fechamento como `string`.
Adicionar `interpolation: tags.special(tags.string)` ao `tokenTable` em `:269-274`. Não alterar a ordem de prioridade de `keyword`/`typeName`/`builtin`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `node --experimental-strip-types --test tests/pors-interpolation-highlight.test.mjs tests/pors-editor-loops.test.mjs tests/pors-completions.test.mjs`
Expected: PASS nos três arquivos.

- [ ] **Step 5: Commit**

```bash
git add apps/desktop/src/portugol.ts apps/desktop/tests/pors-interpolation-highlight.test.mjs
git commit -m "feat(editor): realçar interpolação de cadeias com spans próprios"
```

---

### Task 3: Variável `--syntax-interpolation` nos temas da IDE

**Files:**
- Modify: `apps/desktop/src/editor/theme.ts:35-45`
- Modify: `apps/desktop/src/theme-presets.ts:1-8`, `:19-49`, `:51-87`, `:93-180`
- Modify: `apps/desktop/src/themes.ts:3-39`
- Modify: `apps/desktop/src/styles/tokens.css:4-38`, `:101-138`, `:141-176`, `:178-235`
- Test: `apps/desktop/tests/theme-presets.test.mjs`, `apps/desktop/tests/styles-modules.test.mjs`

**Interfaces:**
- Consumes: tag `tags.special(tags.string)` da Task 2; `THEME_COLOR_FIELDS`.
- Produces: `--syntax-interpolation` disponível em claro, escuro, sistema e `forced-colors`.

- [ ] **Step 1: Write the failing test**

Em `apps/desktop/tests/theme-presets.test.mjs`, acrescentar `"--syntax-interpolation"` à lista explícita de foregrounds com contraste (`:33-37`). Os testes genéricos de cobertura de `THEME_COLOR_FIELDS` e de `tokens.css` passam a exigir o campo automaticamente.

- [ ] **Step 2: Run tests to verify they fail**

Run: `node --experimental-strip-types --test tests/theme-presets.test.mjs tests/styles-modules.test.mjs`
Expected: FAIL por campo ausente nos presets e no CSS.

- [ ] **Step 3: Add the color to every palette and theme block**

Acrescentar `interpolation` ao tipo `Palette`, a todas as 11 paletas, ao objeto `colors`, a `THEME_COLOR_FIELDS`, a `fields` em `themes.ts` e a todos os blocos de `tokens.css` (claro, escuro por `prefers-color-scheme`, escuro explícito e `forced-colors`). Usar a cor de função/tipo existente como fallback. Em `editor/theme.ts`, acrescentar a regra `{ tag: tags.special(tags.string), color: "var(--syntax-interpolation)" }`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `node --experimental-strip-types --test tests/theme-presets.test.mjs tests/styles-modules.test.mjs tests/editor-theme-controller.test.mjs`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add apps/desktop/src/editor/theme.ts apps/desktop/src/theme-presets.ts apps/desktop/src/themes.ts apps/desktop/src/styles/tokens.css apps/desktop/tests/theme-presets.test.mjs
git commit -m "feat(editor): adicionar cor de interpolação aos temas"
```

---

### Task 4: Realce e igualdade de texto no Web Playground

**Files:**
- Modify: `apps/web-playground/app.mjs:35-37`, `:60-79`
- Modify: `apps/web-playground/style.css:29-34`, `:63-68`, `:422-448`
- Test: `apps/desktop/tests/web-playground.test.mjs`

**Interfaces:**
- Consumes: taxonomia compartilhada (`builtin`, `string`, `interpolation`).
- Produces: `pg-tok-interpolation`, `--pg-interpolation`, `renderHighlight()` sem `\n` extra.

- [ ] **Step 1: Write the failing tests**

Adicionar em `apps/desktop/tests/web-playground.test.mjs` um teste com DOM falso (padrão de `FakeElement` já presente no arquivo, estendido com `createTextNode`, `createDocumentFragment`, `replaceChildren` e `getElementById`) que:
  - faz `globalThis.document` apontar para o DOM falso e importa `../../web-playground/app.mjs?highlight-contract`;
  - verifica que `TOKEN_NAMES`/o renderizador produzem `pg-tok-interpolation` para `{nome}` em `escreval("Olá, {nome}!");` e `pg-tok-builtin` para `escreval`;
  - verifica que `\{nome}` e `{}` não produzem `pg-tok-interpolation`;
  - verifica `highlightEl.textContent === editorEl.value` após definir `editorEl.value` e disparar o evento `input` (sem chamar `setEditorValue`, que não é exportado);
  - verifica que o código-fonte de `app.mjs` não contém `.innerHTML`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `node --experimental-strip-types --test tests/web-playground.test.mjs`
Expected: FAIL por ausência de `pg-tok-interpolation` e pelo `\n` extra.

- [ ] **Step 3: Implement the playground renderer**

Em `app.mjs`, acrescentar `"interpolation"` a `TOKEN_NAMES`, incluir `escreval` no grupo `builtin` da regex e substituir o grupo `string` por um scanner de partes que emita spans `pg-tok-string` e `pg-tok-interpolation` (segmento `{...}` inteiro). Remover o `document.createTextNode("\n")` e seu comentário em `:75-76`. Manter `syncHighlightScroll()`. Em `style.css`, adicionar `--pg-interpolation` nos dois temas e a classe `.pg-tok-interpolation` ao lado das demais `.pg-tok-*`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `node --experimental-strip-types --test tests/web-playground.test.mjs`
Expected: PASS nos 8 testes do arquivo (7 atuais + 1 novo).

- [ ] **Step 5: Commit**

```bash
git add apps/web-playground/app.mjs apps/web-playground/style.css apps/desktop/tests/web-playground.test.mjs
git commit -m "feat(web): alinhar realce de PORS com a IDE e remover linha extra"
```

---

### Task 5: Verificação final e sincronização documental

**Files:**
- Modify: `docs/research/PORS_MIGRATION_MATRIX.md` (linha de realce/editor, se houver)

**Interfaces:**
- Consumes: Tasks 1–4.
- Produces: fatia pronta para revisão e PR separado.

- [ ] **Step 1: Run the full frontend test set**

Run (em `apps/desktop`):

```bash
node --experimental-strip-types --test tests/web-playground.test.mjs tests/pors-editor-loops.test.mjs tests/pors-completions.test.mjs tests/pors-interpolation-highlight.test.mjs tests/theme-presets.test.mjs tests/styles-modules.test.mjs tests/editor-theme-controller.test.mjs
```

Expected: 0 falhas.

- [ ] **Step 2: Verify the build and diff**

Run: `npm --prefix apps/desktop run build` e `git diff --check`.
Expected: build exit 0; nenhum whitespace error.

- [ ] **Step 3: Update the migration matrix**

Registrar que IDE e playground passam a realçar `escreval`, `leia()` e interpolação, sem alterar status dos backends.

- [ ] **Step 4: Commit e PR**

```bash
git add docs/research/PORS_MIGRATION_MATRIX.md
git commit -m "docs: registrar realce alinhado de PORS na IDE e no playground"
git push -u origin pors-language-evolution-specs
gh pr create --base main --title "feat(web): realce de sintaxe alinhado para evolução do PORS" --body "..."
```

Expected: PR aberto contra `main`, sem alterações em specs/ADRs e sem trailers de autoria por IA.
