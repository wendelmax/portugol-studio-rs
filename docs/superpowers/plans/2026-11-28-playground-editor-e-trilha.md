# Playground editor-like com arquivos locais e trilha — plano

## Contexto

O Playground Web hoje é uma página com um `<select>` de 4 exemplos, um editor, terminal, prévia UI, Rust e referência dispostos na mesma grade. PR #294 já entregou autocomplete, referência e terminal. Este plano transforma a página numa casca de editor: lateral com arquivos/exemplos/trilha, abas no editor, painel inferior com abas e ciclo de arquivo local (abrir, criar, salvar). Nada de backend, npm, bundler ou `innerHTML`.

Especificação aprovada: `docs/superpowers/specs/2026-11-28-playground-editor-e-trilha-design.md`.

## Antes de começar

```bash
git switch -c playground-editor-trilha origin/main
git add docs/superpowers/specs/2026-11-28-playground-editor-e-trilha-design.md \
        docs/superpowers/plans/2026-11-28-playground-editor-e-trilha.md
git commit -m "docs: especifica playground com arquivos locais e trilha"
```

Commits sem `Co-Authored-By` e sem menção a IA/Claude/Anthropic.

## Mapa de arquivos

- `apps/web-playground/playground-core.mjs`: catálogo de 12 exemplos; runtime WASM inalterado.
- `apps/web-playground/file-system.mjs` (novo): abrir/salvar `.fer` com API nativa e fallback.
- `apps/web-playground/lessons.mjs` (novo): 6 lições, verificação por stdout e soluções para checagem automatizada.
- `apps/web-playground/index.html` / `style.css`: casca de editor com lateral, abas e painel inferior.
- `apps/web-playground/app.mjs`: estado de abas, arquivos, lições e painéis.
- `apps/desktop/tests/web-playground*.test.mjs`: testes Node com runtime WASM falso (exceto checagem real separada).
- `scripts/build-web-playground.sh`: copia os módulos novos para `dist/`.
- `scripts/check-web-lessons.mjs` (novo): verifica as soluções das lições com o WASM real.
- `apps/web-playground/README.md`: documentação curta da nova interface.

---

## Task 1 — Catálogo com 12 exemplos

**Teste RED:** em `apps/desktop/tests/web-playground.test.mjs`, trocar a primeira asserção por `PLAYGROUND_EXAMPLES.length === 12`, exigir `group` em todas as entradas e `id`/`title`/`file`/`mode`/`input`/`source` presentes. Rodar o teste e ver `expected: 12, actual: 4`.

**Implementação:** em `playground-core.mjs`, substituir o array por 12 entradas. Manter `createPlaygroundRuntime`, `loadPlaygroundRuntime` e `renderUiNodeTree` exatamente como estão.

| id | file | group | mode | fonte |
| --- | --- | --- | --- | --- |
| `ola` | `boas-vindas.fer` | Básico | console | `01-ola-variaveis.fer` |
| `entrada` | `entrada.fer` | Básico | console | `07-entrada-tipada.fer`, `input: "Ada\n17\n"` |
| `tipos` | `tipos.fer` | Básico | console | `11-matematica-texto-tipos.fer` |
| `decisao` | `decisao.fer` | Controle | console | `02-condicionais.fer` |
| `lacos` | `lacos.fer` | Controle | console | `03-lacos.fer` |
| `funcoes` | `funcoes.fer` | Controle | console | `04-funcoes-recursao.fer` |
| `vetores` | `vetores.fer` | Dados | console | `05-vetores.fer` |
| `registro` | `registro.fer` | Dados | console | `06-registros-enums.fer` |
| `valores` | `valores.fer` | Valores | console | `09-opcao-resultado.fer` |
| `bibliotecas` | `bibliotecas.fer` | Valores | console | `12-util-calendario.fer` |
| `ui-contador` | `ui-contador.fer` | Visual | ui | `08-ui-contador.fer` |
| `exercicio` | `exercicio.fer` | Exercício | console | novo starter com comentário curto e `funcao inicio() { ... }` |

Copiar o `source` dos arquivos de `crates/ferro-service/assets/exemplos-ferro/` literalmente (`JSON.stringify` para escapar quebras e aspas). `input: ""` salvo onde indicado.

**Verificação:** `node --test apps/desktop/tests/web-playground.test.mjs` (a primeira asserção passa; as demais ainda podem falhar até as Tasks 4–6).

**Commit:** `git commit -am "feat: amplia catálogo do playground para doze exemplos"`.

---

## Task 2 — Módulo de arquivos locais

**Teste RED:** criar `apps/desktop/tests/web-playground-files.test.mjs` com um host falso e importar `openFerFile`/`saveFerFile` de `../../web-playground/file-system.mjs`. Primeiro teste: `showOpenFilePicker` chamado com `types` contendo `.fer`, `getFile().text()` devolve o conteúdo e o resultado inclui `handle`. Rodar e ver `ERR_MODULE_NOT_FOUND`.

**Implementação:** criar `apps/web-playground/file-system.mjs` com:

```js
export async function openFerFile(host) // -> { name, content, handle } | null
export async function saveFerFile(host, { name, content, handle = null }) // -> { name, handle, downloaded }
```

- `host` aceita `{ window, document, createObjectURL, revokeObjectURL, setTimeout }`; valores ausentes caem em `globalThis`.
- Nativo: `showOpenFilePicker({ multiple: false, types: [{ description: "Arquivos Ferro", accept: { "text/plain": [".fer"] } }] })`; `AbortError` retorna `null`. `showSaveFilePicker` com `suggestedName`; `createWritable()` → `write(content)` → `close()`; `AbortError` retorna `null`.
- Fallback: `document.createElement("input")`, `type = "file"`, `accept = ".fer,text/plain"`, resolução no `change`; sem arquivo ou `cancel` retorna `null`. Salvar usa `Blob([content], { type: "text/plain" })`, `createObjectURL`, `<a download>` temporário, `remove()` e `revokeObjectURL` via `setTimeout(..., 0)`.
- Sem `innerHTML`; toda criação de nó usa `createElement`.

**Testes no mesmo arquivo:** cancelamento nativo, fallback de abertura, escrita nativa com `writable.calls`, fallback de download com `download` igual ao nome e URL revogada.

**Verificação:** `node --test apps/desktop/tests/web-playground-files.test.mjs` → 4/4.

**Commit:** `git commit -am "feat: abre e salva arquivos .fer no playground web"`.

---

## Task 3 — Módulo de lições

**Teste RED:** criar `apps/desktop/tests/web-playground-lessons.test.mjs` importando `LESSONS`, `LESSON_SOLUTIONS` e `checkLesson`. Primeiro teste: runtime falso devolve `stdout: "Olá, Ada\n"` para `lesson.stdin === ""` e `checkLesson` deve devolver `ok: true`; depois devolve `"errado"` e `ok: false`. Rodar e ver `ERR_MODULE_NOT_FOUND`.

**Implementação:** criar `apps/web-playground/lessons.mjs` com `LESSONS` (6 entradas), `LESSON_SOLUTIONS` (mesmas chaves, usadas só por `scripts/check-web-lessons.mjs`) e:

```js
export function checkLesson(runtime, lesson, source) {
  const result = runtime.run(source, lesson.stdin);
  const actual = String(result.stdout ?? "").split("\n");
  if (actual.length > 1 && actual.at(-1) === "") actual.pop();
  return {
    ok: Boolean(result.ok) && JSON.stringify(actual) === JSON.stringify(lesson.expected),
    actual,
    expected: lesson.expected,
    diagnostics: result.diagnostics ?? [],
  };
}
```

Lições (`stdin: ""` em todas):

1. `variaveis` — starter `seja nome: cadeia = "???"; escreval("Olá, ", nome, "!");`, enunciado “Imprima `Olá, Ada`.”, dica “Troque `???` por `Ada`.”, `expected: ["Olá, Ada"]`, solução troca a string.
2. `decisoes` — starter com `nota = 6`, `se (nota >= 7) { escreval("Aprovado"); } senao { escreval("Recuperação"); }`, enunciado “Com nota 6, imprima `Aprovado`.”, dica “Ajuste a condição para 6.”, `expected: ["Aprovado"]`, solução `nota >= 6`.
3. `lacos` — starter `para (i = 1; i <= 3; i += 1) { total += 1; }` com `escreval(total)`, enunciado “Some de 1 até 4.”, dica “Inclua o 4 no laço.”, `expected: ["4"]`, solução `i <= 4`.
4. `vetores` — starter `seja notas: vetor<inteiro> = [7, 8, 9]; escreval(notas[0]);`, enunciado “Imprima o segundo elemento.”, dica “Use o índice 1.”, `expected: ["8"]`, solução `notas[1]`.
5. `cadeias` — starter `seja texto: cadeia = "ferro"; escreval(texto);`, enunciado “Imprima `FERRO`.”, dica “Use `Texto.caixa_alta`.”, `expected: ["FERRO"]`, solução `escreval(Texto.caixa_alta(texto));`.
6. `resultado` — starter com `dividir` retornando `Resultado::Erro("zero")` antes do `se`, `escreval(v)` no caso `Ok`; enunciado “Com `dividir(10, 2)`, imprima `5`.”, dica “Só retorne erro quando `b == 0`.”, `expected: ["5"]`, solução remove o retorno de erro inicial.

Cada entrada tem `id`, `title`, `prompt`, `hint`, `starter`, `stdin`, `expected`. Textos curtos em pt-BR.

**Verificação:** `node --test apps/desktop/tests/web-playground-lessons.test.mjs` → 3/3 (inclui teste de stdout com linha vazia no meio e diagnóstico).

**Commit:** `git commit -am "feat: adiciona trilha de seis lições ao playground"`.

---

## Task 4 — Casca de editor no HTML/CSS

**Teste RED:** em `web-playground.test.mjs`, trocar o teste do `<select>` por asserções de que `index.html` NÃO tem `id="pg-example"` e tem `pg-sidebar`, `pg-file-list`, `pg-example-list`, `pg-lesson-list`, `pg-tabs`, `pg-active-file`, `pg-panel-tabs`, `pg-panel-terminal`, `pg-panel-ui`, `pg-panel-rust`, `pg-panel-syntax`, `pg-new-file`, `pg-open-file`, `pg-save-file`, `pg-lesson-check`, `pg-lesson-progress`. Rodar e ver a falha.

**Implementação em `index.html`:** reorganizar sem trocar os ids já usados:

```
.pg-app
  header.pg-topbar  -> marca, #pg-new-file, #pg-open-file, #pg-save-file, #pg-theme, #pg-action, #pg-run
  main.pg-main > .pg-workspace
    aside#pg-sidebar
      "Arquivos" -> botões novos/abrir/salvar + #pg-file-list
      "Exemplos" -> #pg-example-list
      "Trilha"   -> #pg-lesson-list + #pg-lesson-progress
    section.pg-editor-area
      #pg-tabs (role=tablist) + span#pg-active-file
      #pg-lesson-bar[hidden] -> #pg-lesson-title, #pg-lesson-prompt, #pg-lesson-hint, #pg-lesson-check
      #pg-status / #pg-status-text (como hoje)
      .pg-editor-wrap -> #pg-highlight, #pg-editor, #pg-completions (como hoje)
      .pg-bottom
        #pg-panel-tabs -> botões data-panel="terminal|ui|rust|syntax"
        section#pg-panel-terminal -> #pg-terminal-history, #pg-stdin, #pg-terminal-badge
        section#pg-panel-ui[hidden] > section#pg-ui-section -> #pg-ui-container
        section#pg-panel-rust[hidden] > section#pg-rust-section -> #pg-rust-code, #pg-copy-rust
        section#pg-panel-syntax[hidden] > section#pg-syntax-panel -> #pg-syntax-search, #pg-syntax-list, #pg-syntax-empty
  footer.pg-footer (como hoje)
```

Manter `lang="pt-BR"`, o script de tema no `<head>` e `<script type="module" src="./app.mjs">`. Nenhum `innerHTML`.

**Implementação em `style.css`:** `.pg-workspace` com `grid-template-columns: 240px minmax(0, 1fr)`; `.pg-editor-area` como flex column com `min-height: 0`; `.pg-bottom` com altura fixa/rolagem própria; estilos para lateral, abas, barra de lição e abas do painel reaproveitando `--pg-surface`, `--pg-border`, `--pg-radius`, `--pg-muted`, `--pg-accent`. Preservar `:root[data-theme="dark"]`, a regra `[hidden] { display: none !important; }` e os breakpoints 980/720/420 (980 recolhe a lateral; 720 empilha; 420 reduz paddings).

**Verificação:** asserções de HTML/CSS do teste passam; `git diff --check` limpo.

**Commit:** `git commit -am "feat: organiza playground como editor com lateral e abas"`.

---

## Task 5 — Estado de abas, arquivos e painéis no `app.mjs`

**Teste RED:** adicionar em `web-playground.test.mjs` um teste com `FakeElement` que importa `app.mjs?tabs-contract` com runtime falso, dispara clique no botão do exemplo `vetores` em `#pg-example-list` e espera `#pg-active-file === "vetores.fer"` com duas abas em `#pg-tabs`; depois edita, troca para `boas-vindas.fer` e volta, esperando o texto editado preservado. Rodar e ver `pg-example-list` sem listener/aba.

**Implementação:**

- Importar `openFerFile`, `saveFerFile`, `LESSONS`, `LESSON_SOLUTIONS`, `checkLesson`.
- Remover `exampleSelect` e `loadExample`. Estado: `tabs = []`, `activeTabId`, `completionState`, `runtime`, `activeUiSession`.
- `createTab({ id, name, source, stdin = "", mode = "console", handle = null, lessonId = null, dirty = false })`; `openTab(tab)` ativa a aba existente por `id` ou acrescenta; `activateTab(id)` salva o buffer atual, para a sessão UI, aplica `stdin`, terminal vazio e realça; `closeTab(id)` remove e ativa a vizinha; `renderTabs()` cria botões com `textContent` do nome e botão de fechar.
- `activeTab()`; `editorEl.addEventListener("input")` grava `activeTab().source`, marca `dirty`, realça, atualiza autocomplete e re-renderiza abas. `#pg-stdin` grava `activeTab().stdin`. `#pg-active-file` mostra o nome da aba ativa.
- `openExample(id)` cria aba com `file` do catálogo; `openFile()` chama `openFerFile(window)`, cria aba com o nome/handle recebidos; `newFile()` cria `sem-titulo-N.fer` vazio; `saveFile()` chama `saveFerFile(window, { name, content, handle })` e limpa `dirty`.
- `renderFileList()` mostra abas abertas; `renderExampleList()` agrupa por `group` e cria um botão por exemplo; `renderLessonList()` cria um botão por lição, marcando conclusão com `✓`.
- `startLesson(id)` cria ou ativa a aba `licao-<id>.fer`, aplica `starter`, `stdin` fixo, `stdinEl.disabled = true`, mostra `#pg-lesson-bar` com título/enunciado/dica. `#pg-lesson-check` chama `checkLesson`, grava `pg-lessons` em `localStorage`, atualiza `#pg-lesson-progress`, status e a marcação na lateral. Fechar/sair da aba da lição reabilita `#pg-stdin` e esconde a barra.
- `activatePanel(name)` alterna `hidden` dos quatro painéis e `aria-selected` dos botões; `renderSyntaxPanel` e `insertSyntaxEntry` continuam como hoje, apenas dentro de `#pg-panel-syntax`.
- `runConsole`, `runValidate`, `runFormat`, `runExportRust`, `runUiSession` passam a ler `activeTab().source` e `activeTab().stdin`; `runUiSession`, quando o fonte não tem `funcao iniciar(`, abre/ativa a aba `ui-contador` e roda a prévia — sem depender de `#pg-example`.
- `Ctrl/Cmd+Enter` usa a aba ativa (mesma regra de `funcao visualizar(` de hoje). `setControlsEnabled` continua habilitando só `#pg-run` e `#pg-action`.
-Estado inicial: `openExample(PLAYGROUND_EXAMPLES[0].id)`, `activatePanel("terminal")`, `renderTerminal("", "", [])`, `renderSyntaxPanel()`. No `.then` do runtime, `runConsole()` como hoje.

**Testes adicionais no mesmo arquivo:** clique em arquivo aberto ativa aba; fechar aba; lição bloqueia `#pg-stdin` e grava progresso; clique em aba do painel mostra `#pg-panel-syntax` e esconde terminal; botão salvar com fallback chama `saveFerFile` (runtime falso + host falso). Adaptar os fakes para guardar múltiplos listeners por tipo onde necessário e incluir os novos ids.

**Atualização do teste de prévia UI:** remover asserção em `pg-example` e passar a verificar `#pg-active-file === "ui-contador.fer"` e `#pg-tabs` com duas abas.

**Verificação:** `node --test apps/desktop/tests/web-playground.test.mjs apps/desktop/tests/web-playground-files.test.mjs apps/desktop/tests/web-playground-lessons.test.mjs apps/desktop/tests/ferro-language.test.mjs`.

**Commit:** `git commit -am "feat: conecta abas, arquivos locais e trilha no playground"`.

---

## Task 6 — Build, checagem real e documentação

**Teste RED:** `grep` em `scripts/build-web-playground.sh` por `file-system.mjs` e `lessons.mjs` falha; script `scripts/check-web-lessons.mjs` ainda não existe.

**Implementação:**

- Adicionar `file-system.mjs` e `lessons.mjs` à lista de `cp` do build.
- Criar `scripts/check-web-lessons.mjs`: lê `apps/web-playground/dist/ferro_wasm.wasm` com `readFileSync`, usa `WebAssembly.instantiate(bytes)` + `createPlaygroundRuntime(instance.exports)` e para cada `LESSONS` roda `checkLesson(runtime, lesson, LESSON_SOLUTIONS[lesson.id])`, imprimindo `ok <id>` ou lançando com `actual`/`expected`.
- Reescrever `apps/web-playground/README.md` em poucas linhas: abrir `.fer`, abas, salvar nativo/fallback, exemplos, trilha, atalho, build. Sem menção a IA.
- Atualizar o plano/spec se algum id mudar durante a implementação.

**Verificação completa (tudo precisa estar verde antes de declarar pronto):**

```bash
node --test apps/desktop/tests/ferro-language.test.mjs \
             apps/desktop/tests/web-playground.test.mjs \
             apps/desktop/tests/web-playground-files.test.mjs \
             apps/desktop/tests/web-playground-lessons.test.mjs
cargo test -p ferro-service --test ferro_examples_migration
bash -n scripts/build-web-playground.sh
git diff --check
bash scripts/build-web-playground.sh
node scripts/check-web-lessons.mjs
```

A suíte ampla `node --experimental-strip-types --test apps/desktop/tests/*.test.mjs` continua com as 13 falhas pré-existentes por `node_modules` ausente; conferir que a lista não cresceu.

**Commit:** `git commit -am "chore: documenta e verifica playground editor-like"`.

---

## Task 7 — PR

```bash
git push -u origin playground-editor-trilha
gh pr create --repo wendelmax/ferro-studio --base main --head playground-editor-trilha \
  --title "Playground web como editor: arquivos .fer, abas e trilha" \
  --body "Abre e salva .fer local, organiza a tela em lateral/abas/painel inferior, amplia para 12 exemplos e adiciona trilha de 6 lições com verificação por saída."
```

Depois: `gh pr checks <num> --repo wendelmax/ferro-studio` e `gh pr view <num> --json state,mergeable,headRefOid`. Só declarar completo com testes focados verdes, checagem real das lições verde e CI do PR verde.
