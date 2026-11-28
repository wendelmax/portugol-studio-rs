import {
  PLAYGROUND_EXAMPLES,
  loadPlaygroundRuntime,
  renderUiNodeTree,
} from "./playground-core.mjs";
import { FERRO_SYNTAX_ENTRIES, completeFerro, wordRangeAt } from "./ferro-language.mjs";
import { openFerFile, saveFerFile } from "./file-system.mjs";
import { LESSONS, checkLesson } from "./lessons.mjs";

const byId = (id) => document.getElementById(id);

const statusEl = byId("pg-status");
const statusTextEl = byId("pg-status-text");
const themeButton = byId("pg-theme");
const editorEl = byId("pg-editor");
const completionsEl = byId("pg-completions");
const highlightEl = byId("pg-highlight");
const fileListEl = byId("pg-file-list");
const exampleListEl = byId("pg-example-list");
const lessonListEl = byId("pg-lesson-list");
const lessonProgressEl = byId("pg-lesson-progress");
const tabsEl = byId("pg-tabs");
const activeFileEl = byId("pg-active-file");
const cursorEl = byId("pg-cursor");
const lessonBarEl = byId("pg-lesson-bar");
const lessonTitleEl = byId("pg-lesson-title");
const lessonPromptEl = byId("pg-lesson-prompt");
const lessonHintEl = byId("pg-lesson-hint");
const lessonCheckButton = byId("pg-lesson-check");
const newFileButton = byId("pg-new-file");
const openFileButton = byId("pg-open-file");
const saveFileButton = byId("pg-save-file");
const panelTabsEl = byId("pg-panel-tabs");
const panelEls = {
  terminal: byId("pg-panel-terminal"),
  ui: byId("pg-panel-ui"),
  rust: byId("pg-panel-rust"),
  syntax: byId("pg-panel-syntax"),
};
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
let activeTabId = null;
let newFileCounter = 1;
const tabs = [];

const THEME_STORAGE_KEY = "pg-theme";
const LESSONS_STORAGE_KEY = "pg-lessons";
const PANEL_NAMES = ["terminal", "ui", "rust", "syntax"];
const TOKEN_NAMES = ["comment", "string", "interpolation", "number", "keyword", "type", "builtin"];
const FERRO_TOKEN_RE =
  /(?<comment>\/\/[^\n]*|\/\*[\s\S]*?\*\/)|(?<string>"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')|(?<number>\b\d+(?:\.\d+)?\b)|(?<keyword>\b(?:funcao|retorne|se|senao|senao_se|seja|mutavel|enquanto|faca_enquanto|para|escolha|caso|registro|enum|verdadeiro|falso|nulo|e|ou|nao)\b)|(?<type>\b(?:inteiro|real|logico|cadeia|caracter|vetor|Resultado|Opcao)\b)|(?<builtin>\b(?:UI|Matematica|Texto|Tipos|Util|Calendario|Arquivos|Json|Internet|Graficos|Teclado|Mouse|Sons|escreva|escreval|leia|adicionar|remover|obter|tamanho)\b)/g;
const UI_START_RE = /\bfuncao\s+iniciar\s*\(/;

function completedLessonIds() {
  try {
    const stored = JSON.parse(localStorage.getItem(LESSONS_STORAGE_KEY) ?? "[]");
    return new Set(Array.isArray(stored) ? stored.filter((id) => LESSONS.some((lesson) => lesson.id === id)) : []);
  } catch {
    return new Set();
  }
}

const completedLessons = completedLessonIds();

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
  for (const match of source.matchAll(FERRO_TOKEN_RE)) {
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

function updateCursor() {
  const caret = editorEl.selectionStart ?? editorEl.value.length;
  const before = editorEl.value.slice(0, caret);
  const line = before.split("\n").length;
  const column = before.length - before.lastIndexOf("\n");
  cursorEl.textContent = `linha ${line} · coluna ${column}`;
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
    item.addEventListener("mousedown", (event) => event.preventDefault());
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
  commitEditorChange();
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

function commitEditorChange() {
  const tab = activeTab();
  if (!tab) return;
  tab.source = editorEl.value;
  tab.dirty = true;
  updateCursor();
  renderTabs();
  renderFileList();
}

function insertSyntaxEntry(entry) {
  const caret = editorEl.selectionStart ?? editorEl.value.length;
  const { start, end } = wordRangeAt(editorEl.value, caret);
  const nextCaret = start + entry.insertText.length;
  setEditorValue(`${editorEl.value.slice(0, start)}${entry.insertText}${editorEl.value.slice(end)}`);
  editorEl.selectionStart = editorEl.selectionEnd = nextCaret;
  commitEditorChange();
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

function activatePanel(name) {
  for (const button of panelTabsEl.children) {
    button.setAttribute("aria-selected", String(button.dataset.panel === name));
  }
  for (const panel of PANEL_NAMES) panelEls[panel].hidden = panel !== name;
}

function renderPanelTabs() {
  panelTabsEl.replaceChildren();
  for (const [name, label] of [
    ["terminal", "Terminal"],
    ["ui", "Prévia UI"],
    ["rust", "Rust"],
    ["syntax", "Referência"],
  ]) {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.panel = name;
    button.textContent = label;
    button.setAttribute("role", "tab");
    button.setAttribute("aria-selected", "false");
    button.addEventListener("click", () => activatePanel(name));
    panelTabsEl.append(button);
  }
}

function activeTab() {
  return tabs.find((tab) => tab.id === activeTabId) ?? null;
}

function createTab({ id, name, source, stdin = "", mode = "console", handle = null, lessonId = null, dirty = false }) {
  return { id, name, source, stdin, mode, handle, lessonId, dirty };
}

function renderTabs() {
  tabsEl.replaceChildren();
  for (const tab of tabs) {
    const item = document.createElement("div");
    item.className = "pg-tab-item";
    const button = document.createElement("button");
    button.type = "button";
    button.className = "pg-tab";
    button.dataset.tabId = tab.id;
    button.textContent = `${tab.dirty ? "*" : ""}${tab.name}`;
    button.setAttribute("role", "tab");
    button.setAttribute("aria-selected", String(tab.id === activeTabId));
    button.addEventListener("click", () => activateTab(tab.id));
    const close = document.createElement("button");
    close.type = "button";
    close.className = "pg-tab-close";
    close.dataset.closeTabId = tab.id;
    close.textContent = "×";
    close.setAttribute("aria-label", `Fechar ${tab.name}`);
    close.addEventListener("click", (event) => {
      event.stopPropagation();
      closeTab(tab.id);
    });
    item.append(button, close);
    tabsEl.append(item);
  }
}

function renderFileList() {
  fileListEl.replaceChildren();
  for (const tab of tabs) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "pg-sidebar-item";
    button.dataset.fileTabId = tab.id;
    button.textContent = `${tab.dirty ? "*" : ""}${tab.name}`;
    button.setAttribute("role", "tab");
    button.setAttribute("aria-selected", String(tab.id === activeTabId));
    button.addEventListener("click", () => activateTab(tab.id));
    fileListEl.append(button);
  }
}

function renderExampleList() {
  exampleListEl.replaceChildren();
  const groups = new Map();
  for (const example of PLAYGROUND_EXAMPLES) {
    if (!groups.has(example.group)) groups.set(example.group, []);
    groups.get(example.group).push(example);
  }
  for (const [groupName, examples] of groups) {
    const heading = document.createElement("h3");
    heading.className = "pg-sidebar-heading";
    heading.textContent = groupName;
    exampleListEl.append(heading);
    for (const example of examples) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "pg-sidebar-item";
      button.dataset.exampleId = example.id;
      button.textContent = example.title;
      button.addEventListener("click", () => openExample(example.id));
      exampleListEl.append(button);
    }
  }
}

function renderLessonList() {
  lessonListEl.replaceChildren();
  for (const lesson of LESSONS) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "pg-sidebar-item";
    button.dataset.lessonId = lesson.id;
    button.textContent = completedLessons.has(lesson.id) ? `✓ ${lesson.title}` : lesson.title;
    button.setAttribute("aria-selected", String(activeTab()?.lessonId === lesson.id));
    button.addEventListener("click", () => startLesson(lesson.id));
    lessonListEl.append(button);
  }
  const done = LESSONS.filter((lesson) => completedLessons.has(lesson.id)).length;
  lessonProgressEl.textContent = `${done} de ${LESSONS.length} lições concluídas.`;
}

function updateLessonBar(tab) {
  const lesson = tab?.lessonId ? LESSONS.find((item) => item.id === tab.lessonId) : null;
  stdinEl.disabled = Boolean(lesson);
  if (!lesson) {
    lessonBarEl.hidden = true;
    return;
  }
  lessonTitleEl.textContent = lesson.title;
  lessonPromptEl.textContent = lesson.prompt;
  lessonHintEl.textContent = lesson.hint;
  lessonBarEl.hidden = false;
}

function activateTab(id, { run = true } = {}) {
  const previous = activeTab();
  if (previous && previous.id !== id) {
    previous.source = editorEl.value;
    previous.stdin = stdinEl.value;
  }
  const tab = tabs.find((item) => item.id === id);
  if (!tab) return;
  const rustWasOpen = !rustSectionEl.hidden && runtime !== null;
  closeActiveUi();
  closeCompletions();
  activatePanel("terminal");
  activeTabId = id;
  setEditorValue(tab.source);
  stdinEl.value = tab.stdin;
  activeFileEl.textContent = tab.name;
  updateLessonBar(tab);
  updateCursor();
  renderTerminal(tab.stdin, "", []);
  renderTabs();
  renderFileList();
  renderLessonList();
  if (run && runtime) {
    if (tab.mode === "ui" || UI_START_RE.test(tab.source)) runUiSession();
    else runConsole();
  }
  if (rustWasOpen) runExportRust(false);
}

function openTab(tab) {
  if (tabs.some((item) => item.id === tab.id)) {
    activateTab(tab.id);
    return;
  }
  tabs.push(createTab(tab));
  activateTab(tab.id);
}

function closeTab(id) {
  const index = tabs.findIndex((tab) => tab.id === id);
  if (index < 0) return;
  tabs.splice(index, 1);
  if (activeTabId !== id) {
    renderTabs();
    renderFileList();
    return;
  }
  activeTabId = null;
  const next = tabs[index] ?? tabs[index - 1];
  if (next) {
    activateTab(next.id);
    return;
  }
  const name = `sem-titulo-${newFileCounter}.fer`;
  newFileCounter += 1;
  tabs.push(createTab({ id: name, name, source: "" }));
  activateTab(name, { run: false });
  setStatus("Nenhum programa carregado.");
}

function openExample(id) {
  const example = PLAYGROUND_EXAMPLES.find((item) => item.id === id) ?? PLAYGROUND_EXAMPLES[0];
  if (!example) return;
  openTab(createTab({
    id: example.file,
    name: example.file,
    source: example.source,
    stdin: example.input,
    mode: example.mode,
  }));
}

async function openFile() {
  try {
    const opened = await openFerFile(window);
    if (!opened) return;
    openTab(createTab({
      id: opened.name,
      name: opened.name,
      source: opened.content,
      handle: opened.handle,
    }));
    setStatus(`Arquivo ${opened.name} aberto.`, "ok");
  } catch (error) {
    setStatus(`Falha ao abrir: ${error instanceof Error ? error.message : String(error)}`, "error");
  }
}

function newFile() {
  const name = `sem-titulo-${newFileCounter}.fer`;
  newFileCounter += 1;
  openTab(createTab({ id: name, name, source: "" }));
  setStatus(`${name} criado.`, "ok");
}

async function saveFile() {
  const tab = activeTab();
  if (!tab) return;
  try {
    const saved = await saveFerFile(window, { name: tab.name, content: editorEl.value, handle: tab.handle });
    if (!saved) return;
    tab.name = saved.name;
    tab.handle = saved.handle;
    tab.dirty = false;
    activeFileEl.textContent = tab.name;
    renderTabs();
    renderFileList();
    setStatus(saved.downloaded ? `${saved.name} baixado.` : `${saved.name} salvo.`, "ok");
  } catch (error) {
    setStatus(`Falha ao salvar: ${error instanceof Error ? error.message : String(error)}`, "error");
  }
}

function startLesson(id) {
  const lesson = LESSONS.find((item) => item.id === id);
  if (!lesson) return;
  const file = `licao-${id}.fer`;
  openTab(createTab({ id: file, name: file, source: lesson.starter, stdin: lesson.stdin, lessonId: id }));
  setStatus(`${lesson.title}: ${lesson.prompt}`);
}

function checkCurrentLesson() {
  const tab = activeTab();
  if (!runtime || !tab?.lessonId) return;
  const lesson = LESSONS.find((item) => item.id === tab.lessonId);
  if (!lesson) return;
  let result;
  try {
    result = checkLesson(runtime, lesson, editorEl.value);
  } catch (error) {
    setStatus(`Falha ao verificar: ${error instanceof Error ? error.message : String(error)}`, "error");
    return;
  }
  renderTerminal(tab.stdin, result.actual.join("\n"), result.diagnostics);
  if (result.ok) {
    completedLessons.add(lesson.id);
    try {
      localStorage.setItem(LESSONS_STORAGE_KEY, JSON.stringify([...completedLessons]));
    } catch {
      // Progresso não persistido ainda assim vale para a sessão.
    }
    renderLessonList();
    setStatus("Lição concluída.", "ok");
  } else {
    setStatus(`Ainda não: esperado ${result.expected.join(" | ")}.`, "error");
  }
}

function runValidate() {
  if (!runtime) return;
  const tab = activeTab();
  if (!tab) return;
  activatePanel("terminal");
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
  const tab = activeTab();
  if (!tab) return;
  closeCompletions();
  const result = runtime.format(editorEl.value);
  if (result.ok) {
    setEditorValue(result.formatted);
    tab.source = result.formatted;
    tab.dirty = true;
    renderTabs();
    activatePanel("terminal");
    renderTerminal(stdinEl.value, "", []);
    setStatus(result.changed ? "Código formatado." : "O código já está formatado.", "ok");
  } else {
    activatePanel("terminal");
    renderTerminal(stdinEl.value, "", result.diagnostics ?? []);
    setStatus(result.error, "error");
  }
}

function runConsole() {
  if (!runtime) return;
  const tab = activeTab();
  if (!tab) return;
  closeActiveUi();
  activatePanel("terminal");
  tab.stdin = stdinEl.value;
  const result = runtime.run(tab.source, stdinEl.value);
  renderTerminal(stdinEl.value, result.stdout ?? "", result.diagnostics ?? []);
  if (result.ok) {
    setStatus("Execução concluída na VM WebAssembly.", "ok");
  } else {
    setStatus(result.error, "error");
  }
}

function applyUiResult(result) {
  if (!result.ok) {
    activatePanel("terminal");
    renderTerminal(stdinEl.value, result.stdout ?? "", result.diagnostics ?? []);
    setStatus(result.error, "error");
    return;
  }
  activeUiSession = result.session;
  activatePanel("ui");
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
  const tab = activeTab();
  if (!tab) return;
  if (!UI_START_RE.test(tab.source)) {
    const uiExample = PLAYGROUND_EXAMPLES.find((item) => item.id === "ui-contador")
      ?? PLAYGROUND_EXAMPLES.find((item) => item.mode === "ui");
    if (uiExample) {
      openExample(uiExample.id);
      return;
    }
  }
  closeActiveUi();
  applyUiResult(runtime.startUi(tab.source));
}

function runExportRust(announce = true) {
  if (!runtime) return;
  const tab = activeTab();
  if (!tab) return;
  const result = runtime.exportRust(tab.source);
  if (result.ok) {
    activatePanel("rust");
    renderTerminal(stdinEl.value, "", []);
    rustCodeEl.textContent = result.code;
    rustSectionEl.hidden = false;
    if (announce) setStatus("Código Rust gerado em WebAssembly.", "ok");
  } else {
    activatePanel("terminal");
    renderTerminal(stdinEl.value, "", result.diagnostics ?? []);
    rustSectionEl.hidden = true;
    rustCodeEl.textContent = "";
    if (announce) setStatus(result.error, "error");
  }
}

renderPanelTabs();
renderExampleList();
renderLessonList();

applyTheme(document.documentElement.dataset.theme === "dark" ? "dark" : "light", false);
themeButton.addEventListener("click", () => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  applyTheme(next);
});
syntaxSearchEl.addEventListener("input", () => renderSyntaxPanel(syntaxSearchEl.value));

newFileButton.addEventListener("click", newFile);
openFileButton.addEventListener("click", () => { void openFile(); });
saveFileButton.addEventListener("click", () => { void saveFile(); });
lessonCheckButton.addEventListener("click", checkCurrentLesson);
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
  commitEditorChange();
  renderHighlight(editorEl.value);
  updateCompletions();
});
stdinEl.addEventListener("input", () => {
  const tab = activeTab();
  if (tab) tab.stdin = stdinEl.value;
});
editorEl.addEventListener("click", updateCursor);
editorEl.addEventListener("keyup", updateCursor);
editorEl.addEventListener("scroll", syncHighlightScroll);
editorEl.addEventListener("blur", closeCompletions);
editorEl.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    closeCompletions();
    event.preventDefault();
    const tab = activeTab();
    if (!tab) return;
    if (tab.mode === "ui" || tab.source.includes("funcao visualizar(")) runUiSession();
    else runConsole();
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
activatePanel("terminal");
openExample(PLAYGROUND_EXAMPLES[0].id);
renderTerminal("", "", []);
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
