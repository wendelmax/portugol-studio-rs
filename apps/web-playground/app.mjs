import {
  PLAYGROUND_EXAMPLES,
  loadPlaygroundRuntime,
  renderUiNodeTree,
} from "./playground-core.mjs";
import { FERRO_SYNTAX_ENTRIES, completeFerro, wordRangeAt } from "./ferro-language.mjs";

const byId = (id) => document.getElementById(id);

const statusEl = byId("pg-status");
const statusTextEl = byId("pg-status-text");
const themeButton = byId("pg-theme");
const exampleSelect = byId("pg-example");
const editorEl = byId("pg-editor");
const completionsEl = byId("pg-completions");
const highlightEl = byId("pg-highlight");
const syntaxSearchEl = byId("pg-syntax-search");
const syntaxListEl = byId("pg-syntax-list");
const syntaxEmptyEl = byId("pg-syntax-empty");
const stdinEl = byId("pg-stdin");
const terminalHistoryEl = byId("pg-terminal-history");
const terminalBadgeEl = byId("pg-terminal-badge");
const uiContainerEl = byId("pg-ui-container");
const uiSectionEl = byId("pg-ui-section");
const rustSectionEl = byId("pg-rust-section");
const rustCodeEl = byId("pg-rust-code");
const btnCopyRust = byId("pg-copy-rust");
const actionSelect = byId("pg-action");
const btnRun = byId("pg-run");

let runtime = null;
let activeUiSession = null;
let completionState = null;

const THEME_STORAGE_KEY = "pg-theme";
const TOKEN_NAMES = ["comment", "string", "interpolation", "number", "keyword", "type", "builtin"];
const PORS_TOKEN_RE =
  /(?<comment>\/\/[^\n]*|\/\*[\s\S]*?\*\/)|(?<string>"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')|(?<number>\b\d+(?:\.\d+)?\b)|(?<keyword>\b(?:funcao|retorne|se|senao|senao_se|seja|mutavel|enquanto|faca_enquanto|para|escolha|caso|registro|enum|verdadeiro|falso|nulo|e|ou|nao)\b)|(?<type>\b(?:inteiro|real|logico|cadeia|caracter|vetor|Resultado|Opcao)\b)|(?<builtin>\b(?:UI|Matematica|Texto|Tipos|Util|Calendario|Arquivos|Json|Internet|Graficos|Teclado|Mouse|Sons|escreva|escreval|leia|adicionar|remover|obter|tamanho)\b)/g;
const UI_START_RE = /\bfuncao\s+iniciar\s*\(/;

function setStatus(message, tone = "info") {
  statusTextEl.textContent = message;
  statusEl.dataset.tone = tone;
}

function applyTheme(theme, persist = true) {
  const resolved = theme === "dark" ? "dark" : "light";
  const usingDark = resolved === "dark";
  document.documentElement.dataset.theme = resolved;
  themeButton.textContent = usingDark ? "Tema: escuro" : "Tema: claro";
  themeButton.setAttribute("aria-label", usingDark ? "Ativar tema claro" : "Ativar tema escuro");
  themeButton.setAttribute("aria-pressed", String(usingDark));
  if (persist) {
    try {
      localStorage.setItem(THEME_STORAGE_KEY, resolved);
    } catch {
      // Ambientes com armazenamento bloqueado continuam usando o tema da sessão.
    }
  }
}

function appendToken(fragment, name, text) {
  if (!text) return;
  const span = document.createElement("span");
  span.className = `pg-tok-${name}`;
  span.textContent = text;
  fragment.append(span);
}

/** Índice do `}` que fecha a interpolação, ou -1 quando o trecho é literal. */
function interpolationEnd(literal, from) {
  const quote = literal[0];
  for (let i = from; i < literal.length - 1; i += 1) {
    const ch = literal[i];
    if (ch === "\\") {
      i += 1;
      continue;
    }
    if (ch === quote) return -1;
    if (ch === "}") return i > from ? i : -1;
  }
  return -1;
}

function appendStringTokens(fragment, literal) {
  let cursor = 0;
  for (let i = 1; i < literal.length - 1; ) {
    if (literal[i] === "\\") {
      i += 2;
      continue;
    }
    if (literal[i] === "{") {
      const end = interpolationEnd(literal, i + 1);
      if (end >= 0) {
        appendToken(fragment, "string", literal.slice(cursor, i));
        appendToken(fragment, "interpolation", literal.slice(i, end + 1));
        cursor = end + 1;
        i = end + 1;
        continue;
      }
    }
    i += 1;
  }
  appendToken(fragment, "string", literal.slice(cursor));
}

function renderHighlight(source) {
  const fragment = document.createDocumentFragment();
  let cursor = 0;
  for (const match of source.matchAll(PORS_TOKEN_RE)) {
    if (match.index > cursor) {
      fragment.append(document.createTextNode(source.slice(cursor, match.index)));
    }
    const token = TOKEN_NAMES.find((name) => match.groups?.[name]);
    if (token === "string") appendStringTokens(fragment, match[0]);
    else appendToken(fragment, token, match[0]);
    cursor = match.index + match[0].length;
  }
  fragment.append(document.createTextNode(source.slice(cursor)));
  highlightEl.replaceChildren(fragment);
  syncHighlightScroll();
}

function syncHighlightScroll() {
  highlightEl.scrollTop = editorEl.scrollTop;
  highlightEl.scrollLeft = editorEl.scrollLeft;
}

function setEditorValue(value) {
  editorEl.value = value;
  renderHighlight(value);
}

function closeCompletions() {
  completionState = null;
  completionsEl.replaceChildren();
  completionsEl.hidden = true;
}

function renderCompletions(entries, activeIndex, range) {
  completionState = { entries, activeIndex, range };
  const fragment = document.createDocumentFragment();
  entries.forEach((entry, index) => {
    const item = document.createElement("li");
    item.className = "pg-completion-item";
    item.setAttribute("role", "option");
    item.setAttribute("aria-selected", String(index === activeIndex));
    item.textContent = entry.label;
    item.addEventListener("click", () => applyCompletion(index));
    fragment.append(item);
  });
  completionsEl.replaceChildren(fragment);
  completionsEl.hidden = false;
}

function applyCompletion(index) {
  const state = completionState;
  const entry = state?.entries[index];
  if (!entry) return;
  const { start, end } = state.range;
  const caret = start + entry.insertText.length;
  setEditorValue(`${editorEl.value.slice(0, start)}${entry.insertText}${editorEl.value.slice(end)}`);
  editorEl.selectionStart = editorEl.selectionEnd = caret;
  closeCompletions();
  editorEl.focus();
}

function updateCompletions() {
  const caret = editorEl.selectionStart ?? editorEl.value.length;
  const entries = completeFerro(editorEl.value, caret);
  if (entries.length === 0) {
    closeCompletions();
    return;
  }
  renderCompletions(entries, 0, wordRangeAt(editorEl.value, caret));
}

function insertSyntaxEntry(entry) {
  const caret = editorEl.selectionStart ?? editorEl.value.length;
  const { start, end } = wordRangeAt(editorEl.value, caret);
  const nextCaret = start + entry.insertText.length;
  setEditorValue(`${editorEl.value.slice(0, start)}${entry.insertText}${editorEl.value.slice(end)}`);
  editorEl.selectionStart = editorEl.selectionEnd = nextCaret;
  closeCompletions();
  editorEl.focus();
}

function renderSyntaxPanel(filter = "") {
  const query = filter.trim().toLocaleLowerCase("pt-BR");
  const matching = query
    ? FERRO_SYNTAX_ENTRIES.filter((entry) =>
        `${entry.label} ${entry.group} ${entry.signature} ${entry.description}`
          .toLocaleLowerCase("pt-BR")
          .includes(query),
      )
    : FERRO_SYNTAX_ENTRIES;
  const groups = new Map();
  for (const entry of matching) {
    if (!groups.has(entry.group)) groups.set(entry.group, []);
    groups.get(entry.group).push(entry);
  }
  syntaxListEl.replaceChildren();
  syntaxEmptyEl.hidden = matching.length > 0;
  for (const [name, entries] of groups) {
    const group = document.createElement("section");
    group.className = "pg-syntax-group";
    const heading = document.createElement("h3");
    heading.textContent = name;
    group.append(heading);
    for (const entry of entries) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "pg-syntax-item";
      for (const [className, text] of [
        ["pg-syntax-label", entry.label],
        ["pg-syntax-signature", entry.signature],
        ["pg-syntax-description", entry.description],
        ["pg-syntax-example", entry.example],
      ]) {
        const part = document.createElement("span");
        part.className = className;
        part.textContent = text;
        button.append(part);
      }
      button.addEventListener("click", () => insertSyntaxEntry(entry));
      group.append(button);
    }
    syntaxListEl.append(group);
  }
}

function terminalLines(value) {
  if (!value) return [];
  const lines = String(value).split("\n");
  if (lines.length > 1 && lines.at(-1) === "") lines.pop();
  return lines;
}

function renderTerminal(stdin, stdout, diagnostics) {
  const entries = [
    ["input", terminalLines(stdin)],
    ["output", terminalLines(stdout)],
    ["error", (diagnostics ?? []).map((diag) => `[${diag.code}] ${diag.message}`)],
  ];
  terminalHistoryEl.replaceChildren();
  for (const [kind, lines] of entries) {
    for (const text of lines) {
      const line = document.createElement("div");
      line.className = "pg-terminal-line";
      line.dataset.kind = kind;
      line.textContent = text;
      terminalHistoryEl.append(line);
    }
  }
  if (terminalHistoryEl.children.length === 0) {
    const line = document.createElement("div");
    line.className = "pg-terminal-line";
    line.dataset.kind = "empty";
    line.textContent = "Execute para ver entrada e saída aqui.";
    terminalHistoryEl.append(line);
  }
  const count = diagnostics?.length ?? 0;
  terminalBadgeEl.textContent = count > 0 ? `${count} diagnóstico${count === 1 ? "" : "s"}` : "Terminal";
  terminalBadgeEl.dataset.tone = count > 0 ? "error" : "info";
}

function setControlsEnabled(enabled) {
  btnRun.disabled = !enabled;
  actionSelect.disabled = !enabled;
}

function closeActiveUi() {
  if (runtime && activeUiSession !== null) {
    runtime.closeUi(activeUiSession);
    activeUiSession = null;
  }
  uiSectionEl.hidden = true;
  uiContainerEl.replaceChildren();
}

function loadExample(id) {
  const example = PLAYGROUND_EXAMPLES.find((item) => item.id === id) ?? PLAYGROUND_EXAMPLES[0];
  if (!example) return;
  closeActiveUi();
  closeCompletions();
  setEditorValue(example.source);
  stdinEl.value = example.input;
  renderTerminal(stdinEl.value, "", []);
  if (runtime) {
    if (example.mode === "ui") {
      runUiSession();
    } else {
      runConsole();
    }
  }
}

function runValidate() {
  if (!runtime) return;
  const result = runtime.validate(editorEl.value);
  renderTerminal(stdinEl.value, "", result.diagnostics);
  if (result.ok) {
    setStatus("Código .fer válido sem diagnósticos.", "ok");
  } else {
    setStatus(`Encontrados ${result.diagnostics.length} diagnóstico(s).`, "error");
  }
}

function runFormat() {
  if (!runtime) return;
  closeCompletions();
  const result = runtime.format(editorEl.value);
  if (result.ok) {
    setEditorValue(result.formatted);
    renderTerminal(stdinEl.value, "", []);
    setStatus(result.changed ? "Código formatado." : "O código já está formatado.", "ok");
  } else {
    renderTerminal(stdinEl.value, "", result.diagnostics ?? []);
    setStatus(result.error, "error");
  }
}

function runConsole() {
  if (!runtime) return;
  closeActiveUi();
  const result = runtime.run(editorEl.value, stdinEl.value);
  renderTerminal(stdinEl.value, result.stdout ?? "", result.diagnostics ?? []);
  if (result.ok) {
    setStatus("Execução concluída na VM WebAssembly.", "ok");
  } else {
    setStatus(result.error, "error");
  }
}

function applyUiResult(result) {
  if (!result.ok) {
    renderTerminal(stdinEl.value, result.stdout ?? "", result.diagnostics ?? []);
    setStatus(result.error, "error");
    return;
  }
  activeUiSession = result.session;
  renderTerminal(stdinEl.value, result.stdout ?? "", []);
  uiSectionEl.hidden = false;
  renderUiNodeTree(document, uiContainerEl, result.tree, (event) => {
    if (!runtime || activeUiSession === null) return;
    applyUiResult(runtime.dispatchUi(activeUiSession, event));
  });
  setStatus(`Sessão visual MVU ativa (passo ${result.step}).`, "ok");
}

function runUiSession() {
  if (!runtime) return;
  // O playground abre com um exemplo de console. Sem um programa MVU carregado,
  // a ação apenas reportaria a ausência de "iniciar" e não mostraria prévia.
  if (!UI_START_RE.test(editorEl.value)) {
    const uiExample = PLAYGROUND_EXAMPLES.find((item) => item.mode === "ui");
    if (uiExample) {
      exampleSelect.value = uiExample.id;
      loadExample(uiExample.id);
      return;
    }
  }
  closeActiveUi();
  applyUiResult(runtime.startUi(editorEl.value));
}

function runExportRust(announce = true) {
  if (!runtime) return;
  const result = runtime.exportRust(editorEl.value);
  if (result.ok) {
    renderTerminal(stdinEl.value, "", []);
    rustCodeEl.textContent = result.code;
    rustSectionEl.hidden = false;
    if (announce) setStatus("Código Rust gerado em WebAssembly.", "ok");
  } else {
    renderTerminal(stdinEl.value, "", result.diagnostics ?? []);
    rustSectionEl.hidden = true;
    rustCodeEl.textContent = "";
    if (announce) setStatus(result.error, "error");
  }
}

for (const example of PLAYGROUND_EXAMPLES) {
  const option = document.createElement("option");
  option.value = example.id;
  option.textContent = example.title;
  exampleSelect.append(option);
}

applyTheme(document.documentElement.dataset.theme === "dark" ? "dark" : "light", false);
themeButton.addEventListener("click", () => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  applyTheme(next);
});
syntaxSearchEl.addEventListener("input", () => renderSyntaxPanel(syntaxSearchEl.value));

exampleSelect.addEventListener("change", () => {
  const rustWasOpen = !rustSectionEl.hidden && runtime !== null;
  loadExample(exampleSelect.value);
  // A troca de exemplo já atualiza o status com a execução do novo código.
  if (rustWasOpen) runExportRust(false);
});
btnRun.addEventListener("click", runConsole);
const ACTION_HANDLERS = {
  validate: runValidate,
  format: runFormat,
  ui: runUiSession,
  rust: () => runExportRust(),
};
actionSelect.addEventListener("change", () => {
  const action = ACTION_HANDLERS[actionSelect.value];
  actionSelect.value = "";
  if (action) action();
});
btnCopyRust.addEventListener("click", async () => {
  if (!rustCodeEl.textContent) return;
  try {
    await navigator.clipboard.writeText(rustCodeEl.textContent);
    setStatus("Código Rust copiado para a área de transferência.", "ok");
  } catch {
    setStatus("Não foi possível copiar automaticamente; selecione o código manualmente.", "error");
  }
});

editorEl.addEventListener("input", () => {
  renderHighlight(editorEl.value);
  updateCompletions();
});
editorEl.addEventListener("scroll", syncHighlightScroll);
editorEl.addEventListener("blur", closeCompletions);
editorEl.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    closeCompletions();
    event.preventDefault();
    const current = PLAYGROUND_EXAMPLES.find((item) => item.id === exampleSelect.value);
    if (current?.mode === "ui" || editorEl.value.includes("funcao visualizar(")) {
      runUiSession();
    } else {
      runConsole();
    }
    return;
  }
  const state = completionState;
  if (!state) return;
  if (event.key === "Escape") {
    event.preventDefault();
    closeCompletions();
  } else if (event.key === "ArrowDown") {
    event.preventDefault();
    const next = (state.activeIndex + 1) % state.entries.length;
    renderCompletions(state.entries, next, state.range);
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    const next = (state.activeIndex - 1 + state.entries.length) % state.entries.length;
    renderCompletions(state.entries, next, state.range);
  } else if (event.key === "Enter" || event.key === "Tab") {
    event.preventDefault();
    applyCompletion(state.activeIndex);
  }
});

setControlsEnabled(false);
setEditorValue(PLAYGROUND_EXAMPLES[0].source);
stdinEl.value = PLAYGROUND_EXAMPLES[0].input;
renderTerminal(stdinEl.value, "", []);
renderSyntaxPanel();

loadPlaygroundRuntime("./ferro_wasm.wasm")
  .then((loaded) => {
    runtime = loaded;
    setControlsEnabled(true);
    setStatus("VM WebAssembly Ferro pronta.", "ok");
    runConsole();
  })
  .catch((error) => {
    setStatus(
      `Módulo ./ferro_wasm.wasm indisponível (${error instanceof Error ? error.message : String(error)}). Execute scripts/build-web-playground.sh para gerar o bundle WASM.`,
      "error",
    );
  });
