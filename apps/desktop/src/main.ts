import { mountTips } from "./tips";
import { applyThemeColors, customTheme, mountThemeEditor } from "./themes";
// IDE: abas de arquivos, editor CodeMirror, console e execução.
//
// Atalhos seguem o Portugol Studio original (TelaAtalhos.java):
// Shift+F6 executa, Shift+F7 interrompe, Ctrl+Q fecha a aba,
// Alt+←/→ troca de aba, Shift+Esc expande o editor.

import "./style.css";
import license from "../../../LICENSE?raw";
import { version } from "../package.json";

import { EditorState, Compartment, RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import {
  EditorView, GutterMarker, gutter, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter,
  drawSelection, highlightSpecialChars, Decoration, type DecorationSet,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab, isolateHistory } from "@codemirror/commands";
import {
  syntaxHighlighting, HighlightStyle, bracketMatching, indentOnInput, foldGutter, foldKeymap,
} from "@codemirror/language";
import { linter, lintGutter, type Diagnostic } from "@codemirror/lint";
import { openSearchPanel, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { tags } from "@lezer/highlight";
import { open as openDialog, save as saveDialog, ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { renderDiagnostics } from "./diagnostics";
import { mountExamples } from "./examples";
import { mountExercises, type ExerciseDocument } from "./exercises";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline, symbolKindLabels } from "./outline";
import { LIBRARIES, portugol, SNIPPETS } from "./portugol";
import * as api from "./api";

const TEMPLATE = "programa\n{\n\tfuncao inicio()\n\t{\n\t\t\n\t}\n}\n";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------

interface Tab {
  id: number;
  path: string | null;
  exampleName?: string;
  examplePath?: string;
  exercise?: ExerciseDocument;
  state: EditorState;
  /** Texto da última gravação, para saber se há alterações. */
  saved: string;
  diagnostics: api.DiagnosticInfo[];
  symbols: api.SymbolInfo[];
}

// Tema: "system" segue o sistema operacional; a escolha fica salva.
const THEME_KEY = "portugol.theme";
const theme = new Compartment();
const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

function themeChoice(): string {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === "light" || saved === "dark" || saved === "system" || (saved && customTheme(saved))) return saved;
  } catch {
    // Sem armazenamento local: fica no padrão.
  }
  return "dark";
}

let appliedTheme = themeChoice();
const isDark = () => {
  const choice = customTheme(appliedTheme)?.base ?? appliedTheme;
  return choice === "dark" || (choice === "system" && darkQuery.matches);
};
const editorColors = syntaxHighlighting(HighlightStyle.define([
  { tag: tags.keyword, color: "var(--syntax-keyword)" },
  { tag: [tags.typeName, tags.namespace], color: "var(--syntax-type)" },
  { tag: tags.number, color: "var(--syntax-number)" },
  { tag: tags.bool, color: "var(--syntax-logical)" },
  { tag: tags.function(tags.variableName), color: "var(--syntax-function)" },
  { tag: tags.standard(tags.function(tags.variableName)), color: "var(--syntax-function)" },
  { tag: tags.string, color: "var(--syntax-string)" },
  { tag: tags.comment, color: "var(--syntax-comment)" },
  { tag: tags.operator, color: "var(--syntax-operator)" },
]));
const centerKey = "portugol.centerSource";
let centerActive = false;
try {
  centerActive = localStorage.getItem(centerKey) === "true";
} catch {
  // A opção também funciona sem armazenamento.
}

const darkEditor = EditorView.theme({}, { dark: true });
const editorTheme = (): Extension => isDark() ? darkEditor : [];

/** Destaque da linha onde ocorreu o erro de execução. */
const setRuntimeError = StateEffect.define<{ from: number; to: number } | null>();
const runtimeError = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setRuntimeError)) {
        if (!effect.value) return Decoration.none;
        const line = tr.state.doc.lineAt(Math.min(effect.value.from, tr.state.doc.length));
        return Decoration.set([Decoration.line({ class: "cm-runtime-error" }).range(line.from)]);
      }
    }
    return tr.docChanged ? Decoration.none : value.map(tr.changes);
  },
  provide: (field) => EditorView.decorations.from(field),
});

/** Offsets dos pontos de parada; acompanham as edições do documento. */
const toggleBreakpoint = StateEffect.define<number>();
const breakpoints = StateField.define<Set<number>>({
  create: () => new Set<number>(),
  update(value, tr) {
    let next = value;
    for (const effect of tr.effects) {
      if (effect.is(toggleBreakpoint)) {
        next = new Set(next);
        if (next.has(effect.value)) next.delete(effect.value);
        else next.add(effect.value);
      }
    }
    if (!tr.docChanged) return next;
    const mapped = new Set<number>();
    for (const pos of next) mapped.add(tr.changes.mapPos(pos, -1));
    return mapped;
  },
});

const breakpointMarker = new (class extends GutterMarker {
  toDOM() {
    const dot = icon("pequeno/bug");
    dot.classList.add("cm-breakpoint");
    dot.title = "Ponto de parada — clique para remover";
    return dot;
  }
})();

const breakpointGutter = gutter({
  class: "cm-breakpoint-gutter",
  markers: (view) => {
    const positions = [...view.state.field(breakpoints)]
      .map((pos) => view.state.doc.lineAt(Math.min(pos, view.state.doc.length)).from)
      .sort((a, b) => a - b);
    return RangeSet.of(positions.map((pos) => breakpointMarker.range(pos)));
  },
  initialSpacer: () => breakpointMarker,
  domEventHandlers: {
    mousedown(view, line, event) {
      if ((event as MouseEvent).button !== 0) return false;
      view.dispatch({ effects: toggleBreakpoint.of(line.from) });
      syncBreakpoints(view.state);
      return true;
    },
  },
});

/** Destaque da linha onde a VM está parada no depurador. */
const setDebugLine = StateEffect.define<number | null>();
const debugLine = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setDebugLine)) {
        if (effect.value === null) return Decoration.none;
        const line = tr.state.doc.line(Math.max(1, Math.min(effect.value, tr.state.doc.lines)));
        return Decoration.set([Decoration.line({ class: "cm-debug-line" }).range(line.from)]);
      }
    }
    return tr.docChanged ? Decoration.none : value.map(tr.changes);
  },
  provide: (field) => EditorView.decorations.from(field),
});

function toCodeMirror(info: api.DiagnosticInfo, length: number): Diagnostic {
  const from = Math.min(info.start.offset, length);
  const to = Math.min(Math.max(info.end.offset, from + 1), length);
  return {
    from,
    to: Math.max(to, from),
    severity: info.severity,
    source: info.code,
    message: info.help ? `${info.message}\nAjuda: ${info.help}` : info.message,
  };
}

const portugolLinter = linter(
  async (view) => {
    const doc = view.state.doc;
    const tab = activeTab();
    const { diagnostics, symbols } = await api.inspect(doc.toString());
    if (tab && tab === activeTab() && view.state.doc === doc) {
      tab.diagnostics = diagnostics;
      tab.symbols = symbols;
      renderMessages();
      outline.show(symbols);
      updateLibraryButtons(symbols);
    }
    return diagnostics.map((d) => toCodeMirror(d, doc.length));
  },
  { delay: 350 },
);

function extensions(): Extension[] {
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    drawSelection(),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    autocompletion(),
    highlightActiveLine(),
    highlightSelectionMatches(),
    findDialog,
    EditorState.tabSize.of(4),
    portugol(),
    portugolFolding,
    foldGutter({ markerDOM: (open) => {
      const marker = document.createElement("span");
      marker.textContent = open ? "−" : "+";
      marker.title = open ? "Recolher bloco" : "Expandir bloco";
      marker.setAttribute("aria-label", marker.title);
      return marker;
    }}),
    editorColors,
    theme.of(editorTheme()),
    lintGutter(),
    portugolLinter,
    runtimeError,
    breakpoints,
    breakpointGutter,
    debugLine,
    keymap.of([
      ...closeBracketsKeymap, ...defaultKeymap, ...editorSearchKeys,
      ...historyKeymap, ...completionKeymap, ...foldKeymap, indentWithTab,
    ]),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) renderTabs();
      if (update.selectionSet || update.docChanged) renderCursor();
      if (centerActive && update.selectionSet) {
        queueMicrotask(() => {
          if (centerActive && view.state === update.state) centerSelection();
        });
      }
    }),
  ];
}

const view = new EditorView({ parent: $("editor") });

function applyTheme(choice: string) {
  appliedTheme = choice;
  try {
    localStorage.setItem(THEME_KEY, choice);
  } catch {
    // Ignora: a escolha vale só para esta sessão.
  }
  if (choice === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = customTheme(choice)?.base ?? choice;
  applyThemeColors(document.documentElement, customTheme(choice)?.colors);
  view.dispatch({ effects: theme.reconfigure(editorTheme()) });
  help?.refreshTheme();
}

darkQuery.addEventListener("change", () => {
  view.dispatch({ effects: theme.reconfigure(editorTheme()) });
  help?.refreshTheme();
});

// ---------------------------------------------------------------------------
// Abas
// ---------------------------------------------------------------------------

let tabs: Tab[] = [];
const homeId = 0;
const helpId = -1;
let activeId = homeId;
let helpOpen = false;
let nextTabId = 1;

const activeTab = () => tabs.find((t) => t.id === activeId);

function tabName(tab: Tab): string {
  if (!tab.path) return tab.exampleName ?? "Sem título";
  return tab.path.split(/[\\/]/).pop() ?? tab.path;
}

function isDirty(tab: Tab): boolean {
  const text = tab.id === activeId ? view.state.doc.toString() : tab.state.doc.toString();
  return text !== tab.saved;
}

const NEW_PROGRAM = TEMPLATE.replace("\t\t\n", '\t\tescreva("Olá Mundo")\n');

function newTab(text = NEW_PROGRAM, path: string | null = null, exampleName?: string, examplePath?: string, exercise?: ExerciseDocument): Tab {
  const tab: Tab = {
    id: nextTabId++,
    path,
    exampleName,
    examplePath,
    exercise,
    state: EditorState.create({ doc: text, extensions: extensions() }),
    saved: text,
    diagnostics: [],
    symbols: [],
  };
  tabs.push(tab);
  activate(tab.id);
  return tab;
}

function activate(id: number) {
  const current = activeTab();
  if (current) current.state = view.state;
  const tab = tabs.find((t) => t.id === id);
  if (!tab && id !== homeId && id !== helpId) return;
  activeId = id;
  if (id === helpId) helpOpen = true;
  $("home").hidden = id !== homeId;
  $("help").hidden = id !== helpId;
  $("code").hidden = !tab;
  $("editor-exercise").hidden = !tab?.exercise;
  if (!tab) {
    renderTabs();
    $("cursor").textContent = "";
    if (id === helpId) void openHelp();
    else $("home").focus();
    return;
  }
  view.setState(tab.state);
  view.dispatch({ effects: [
    theme.reconfigure(editorTheme()),
    setDebugLine.of(tab.id === runTabId ? paused?.position.line ?? null : null),
  ] });
  renderTabs();
  renderMessages();
  outline.show(tab.symbols, current !== tab);
  updateLibraryButtons(tab.symbols);
  renderCursor();
  if (centerActive) centerSelection();
  view.focus();
}

async function closeTab(id: number): Promise<boolean> {
  if (id === helpId) {
    helpOpen = false;
    if (activeId === helpId) activate(tabs[tabs.length - 1]?.id ?? homeId);
    else renderTabs();
    return true;
  }
  const tab = tabs.find((t) => t.id === id);
  if (!tab) return true;
  if (isDirty(tab)) {
    const discard = await ask(`"${tabName(tab)}" tem alterações não salvas. Fechar mesmo assim?`, {
      title: "Fechar aba", kind: "warning", okLabel: "Fechar sem salvar", cancelLabel: "Cancelar",
    });
    if (!discard) return false;
  }
  const index = tabs.indexOf(tab);
  tabs.splice(index, 1);
  if (id === activeId) activate(tabs[Math.max(0, index - 1)]?.id ?? 0);
  else renderTabs();
  return true;
}

function renderTabs() {
  const bar = $("tabs");
  const home = document.createElement("button");
  home.className = "tab home-tab" + (activeId === 0 ? " active" : "");
  home.append(icon("pequeno/light_pix"), "Portugol Studio");
  home.title = "Tela inicial (Alt+Home)";
  home.setAttribute("aria-current", String(activeId === 0));
  home.onclick = () => activate(0);
  const add = document.createElement("button");
  add.className = "tab add-tab";
  add.textContent = "+";
  add.title = "Novo arquivo (Ctrl+N)";
  add.setAttribute("aria-label", "Novo arquivo");
  add.onclick = () => newTab();
  const helpTab = document.createElement("div");
  helpTab.className = "file-tab tab" + (activeId === helpId ? " active" : "");
  const helpButton = document.createElement("button");
  helpButton.textContent = "Ajuda";
  helpButton.setAttribute("aria-current", String(activeId === helpId));
  helpButton.onclick = () => activate(helpId);
  const closeHelp = document.createElement("button");
  closeHelp.className = "close";
  closeHelp.textContent = "×";
  closeHelp.setAttribute("aria-label", "Fechar ajuda");
  closeHelp.onclick = () => void closeTab(helpId);
  helpTab.append(helpButton, closeHelp);
  bar.replaceChildren(
    home,
    ...(helpOpen ? [helpTab] : []),
    ...tabs.map((tab) => {
      const el = document.createElement("div");
      el.className = "file-tab tab" + (tab.id === activeId ? " active" : "");
      el.title = tab.path ?? "Arquivo ainda não salvo";
      const name = document.createElement("button");
      name.setAttribute("aria-current", String(tab.id === activeId));
      name.textContent = tabName(tab);
      if (isDirty(tab)) name.className = "dirty";
      const close = document.createElement("button");
      close.className = "close";
      close.textContent = "×";
      close.title = "Fechar (Ctrl+Q)";
      close.onclick = (e) => {
        e.stopPropagation();
        void closeTab(tab.id);
      };
      name.prepend(icon("pequeno/light_pix"));
      el.append(name, close);
      name.onclick = () => activate(tab.id);
      el.onauxclick = (e) => e.button === 1 && void closeTab(tab.id);
      return el;
    }),
    add,
  );
  const tab = activeTab();
  const title = tab ? tabName(tab) : activeId === helpId ? "Ajuda" : "";
  document.title = title ? `${title} — Portugol Studio` : "Portugol Studio";
}

function renderCursor() {
  if (!activeTab()) return;
  const head = view.state.selection.main.head;
  const line = view.state.doc.lineAt(head);
  $("cursor").textContent = `Linha ${line.number}, coluna ${head - line.from + 1}`;
}

// ---------------------------------------------------------------------------
// Arquivos
// ---------------------------------------------------------------------------

const FILTERS = [{ name: "Programa Portugol", extensions: ["por"] }];

const RECENT_KEY = "portugol.recent";

function loadRecent(): string[] {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(RECENT_KEY) ?? "[]");
    if (Array.isArray(saved)) return saved.filter((path): path is string => typeof path === "string").slice(0, 6);
  } catch {
    // A lista continua funcionando nesta sessão quando o armazenamento está indisponível.
  }
  return [];
}

let recent = loadRecent();

function renderRecent() {
  $("recent").hidden = recent.length === 0;
  $("recent-files").replaceChildren(...recent.map((path) => {
    const button = document.createElement("button");
    const name = document.createElement("span");
    name.textContent = path.split(/[\\/]/).pop() ?? path;
    button.append(icon("grande/file"), name);
    button.title = path;
    button.onclick = () => void openFile(path).catch((error) => setStatus(String(error), "error"));
    return button;
  }));
}

function rememberFile(path: string) {
  recent = [path, ...recent.filter((entry) => entry !== path)].slice(0, 6);
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recent));
  } catch {
    // A lista em memória permanece disponível.
  }
  renderRecent();
}

async function openFile(path: string) {
  const existing = tabs.find((tab) => tab.path === path);
  if (existing) {
    activate(existing.id);
  } else {
    const text = await api.readFile(path);
    const current = activeTab();
    if (current && !current.path && !current.exampleName && !isDirty(current) && tabs.length === 1) {
      tabs = [];
    }
    newTab(text, path);
  }
  rememberFile(path);
}

async function openFiles() {
  try {
    const selected = await openDialog({ multiple: true, filters: FILTERS });
    if (!selected) return;
    for (const path of Array.isArray(selected) ? selected : [selected]) {
      await openFile(path);
    }
  } catch (error) {
    setStatus(String(error), "error");
  }
}

async function save(as = false): Promise<boolean> {
  const tab = activeTab();
  if (!tab) return false;
  let path = tab.path;
  if (!path || as) {
    path = await saveDialog({ filters: FILTERS, defaultPath: tab.path ?? tab.exampleName ?? "programa.por" });
    if (!path) return false;
    if (!path.endsWith(".por")) path += ".por";
  }
  const text = view.state.doc.toString();
  try {
    await api.writeFile(path, text);
  } catch (error) {
    setStatus(String(error), "error");
    return false;
  }
  tab.path = path;
  tab.saved = text;
  rememberFile(path);
  renderTabs();
  setStatus(`Salvo em ${path}`, "ok");
  return true;
}

// ---------------------------------------------------------------------------
// Execução e console
// ---------------------------------------------------------------------------

let runId: number | null = null;
let runTabId: number | null = null;
let pendingEvents: { id: number; handle: () => void }[] | null = null;
let paused: api.DebugInfo | null = null;
const output = $("console-output");
const consoleBox = $("console");
const inputForm = $<HTMLFormElement>("console-form");
const inputField = $<HTMLInputElement>("console-input");

function setStatus(text: string, kind: "" | "ok" | "error" = "") {
  const status = $("status");
  status.textContent = text;
  status.className = kind;
}

function appendConsole(text: string, className?: string) {
  const atBottom = consoleBox.scrollTop + consoleBox.clientHeight >= consoleBox.scrollHeight - 4;
  if (className) {
    const span = document.createElement("span");
    span.className = className;
    span.textContent = text;
    output.append(span);
  } else {
    output.append(text);
  }
  if (atBottom) consoleBox.scrollTop = consoleBox.scrollHeight;
}

function setRunning(running: boolean) {
  $<HTMLButtonElement>("btn-run").disabled = running && paused === null;
  $<HTMLButtonElement>("btn-debug").disabled = running && paused === null;
  $<HTMLButtonElement>("btn-stop").disabled = !running;
  $<HTMLButtonElement>("btn-debug-pause").disabled = !running || paused !== null;
  for (const id of [
    "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out",
  ]) {
    $<HTMLButtonElement>(id).disabled = paused === null;
  }
  if (!running) inputForm.hidden = true;
  $("debug-vars").hidden = paused === null || paused.variables.length === 0;
  $("inspector-empty").hidden = !$("debug-vars").hidden;
  $("inspector-empty").textContent = paused
    ? "Nenhuma variável neste ponto."
    : "Pause a execução para inspecionar as variáveis.";
}

/** Linhas (base 1) dos pontos de parada no estado do editor. */
function breakpointLines(state: EditorState): number[] {
  return [...state.field(breakpoints)]
    .map((pos) => state.doc.lineAt(Math.min(pos, state.doc.length)).number)
    .sort((a, b) => a - b);
}

/** Pontos de parada podem mudar enquanto o programa está em execução. */
function syncBreakpoints(state: EditorState) {
  if (runId === null || activeId !== runTabId) return;
  void api.debugSetBreakpoints(runId, state.doc.toString(), breakpointLines(state)).catch(() => {});
}

function setPaused(info: api.DebugInfo | null) {
  paused = info;
  if (activeId === runTabId) view.dispatch({ effects: setDebugLine.of(info?.position.line ?? null) });
  setRunning(runId !== null);
  if (info) {
    renderDebug(info);
    showUtilities(true);
    showPanel("debug");
    if (activeId === runTabId) goTo(info.position.offset);
    setStatus(`Pausado na linha ${info.position.line}.`);
  }
}

async function debugCommand(command: (id: number) => Promise<void>) {
  if (runId === null || paused === null) return;
  const id = runId;
  view.dispatch({ effects: setDebugLine.of(null) });
  paused = null;
  setRunning(true);
  setStatus("Executando…");
  try {
    await command(id);
  } catch {
    // A execução pode ter terminado enquanto o comando era enviado.
  }
}

function renderDebug(info: api.DebugInfo) {
  $("debug-stack").replaceChildren(...info.stack.map((frame, index) => {
    const li = document.createElement("li");
    li.textContent = `${frame.function} — linha ${frame.position.line}:${frame.position.column}`;
    if (index === 0) li.className = "current";
    li.onclick = () => {
      if (runTabId === null || !tabs.some(({ id }) => id === runTabId)) return;
      if (activeId !== runTabId) activate(runTabId);
      goTo(frame.position.offset);
    };
    return li;
  }));
  const body = document.querySelector<HTMLTableSectionElement>("#debug-vars tbody")!;
  body.replaceChildren(...info.variables.map((variable) => {
    const row = document.createElement("tr");
    for (const text of [variable.name, variable.ty, variable.value, variable.scope]) {
      const cell = document.createElement("td");
      cell.textContent = text;
      row.append(cell);
    }
    return row;
  }));
}

function parseArgs(text: string): string[] {
  // Aspas agrupam: `"João Silva" 19` -> ["João Silva", "19"].
  return [...text.matchAll(/"([^"]*)"|(\S+)/g)].map((m) => m[1] ?? m[2]);
}

async function runProgram(pauseOnStart = false) {
  if (runId !== null || pendingEvents !== null) return;
  const tab = activeTab();
  if (!tab) return;
  view.dispatch({ effects: setRuntimeError.of(null) });
  view.dispatch({ effects: setDebugLine.of(null) });
  paused = null;
  showPanel("console");
  output.replaceChildren();
  const source = view.state.doc.toString();
  const args = parseArgs($<HTMLInputElement>("args").value);
  const lines = breakpointLines(view.state);
  const events: { id: number; handle: () => void }[] = [];
  pendingEvents = events;
  runTabId = tab.id;
  $<HTMLButtonElement>("btn-run").disabled = true;
  $<HTMLButtonElement>("btn-debug").disabled = true;
  try {
    await runListeners;
    const id = await api.run(source, args, tab.path ?? tab.examplePath ?? null, lines, pauseOnStart);
    runId = id;
    setRunning(true);
    setStatus("Executando…");
    for (const event of events) {
      if (event.id === runId) event.handle();
    }
  } catch (errors) {
    runTabId = null;
    setRunning(false);
    if (Array.isArray(errors)) {
      tab.diagnostics = errors;
      renderMessages();
      showPanel("messages");
      setStatus("O programa contém erros e não pode ser executado.", "error");
    } else {
      setStatus(String(errors), "error");
    }
  } finally {
    pendingEvents = null;
  }
}

function runOrContinue() {
  return paused ? debugCommand(api.debugContinue) : runProgram();
}

function stepOrStart() {
  return paused ? debugCommand(api.debugStepOver) : runProgram(true);
}

async function stopProgram() {
  if (runId !== null) await api.stop(runId);
}

function showPanel(which: "console" | "messages" | "debug") {
  for (const name of ["console", "messages", "debug"]) {
    $(`tab-${name}`).classList.toggle("active", which === name);
    $(name).hidden = which !== name;
  }
  $("console-tools").hidden = which !== "console";
}

function goTo(offset: number) {
  const position = Math.min(offset, view.state.doc.length);
  view.dispatch({ selection: { anchor: position }, scrollIntoView: true });
  view.focus();
}

function renderMessages() {
  renderDiagnostics(activeTab()?.diagnostics ?? [], view.state.doc.lines, goTo);
}

function routeRunEvent(id: number, handle: () => void) {
  if (id === runId) handle();
  else if (pendingEvents) pendingEvents.push({ id, handle });
}

const runListeners = api.listenRun({
  output(id, text) {
    routeRunEvent(id, () => appendConsole(text));
  },
  clear(id) {
    routeRunEvent(id, () => output.replaceChildren());
  },
  input(id, expected) {
    routeRunEvent(id, () => {
      $("console-hint").textContent = `leia (${expected}):`;
      inputForm.hidden = false;
      inputField.value = "";
      if (activeTab()) inputField.focus();
      consoleBox.scrollTop = consoleBox.scrollHeight;
    });
  },
  paused(id, info) {
    routeRunEvent(id, () => setPaused(info));
  },
  finished(id, outcome) {
    routeRunEvent(id, () => {
      const isRunTab = activeId === runTabId;
      runId = null;
      runTabId = null;
      paused = null;
      if (isRunTab) view.dispatch({ effects: setDebugLine.of(null) });
      setRunning(false);
      if (outcome.status === "finished") {
        appendConsole("\nPrograma finalizado.\n", "info");
        setStatus("Programa finalizado.", "ok");
      } else if (outcome.status === "stopped") {
        appendConsole("\nPrograma interrompido.\n", "info");
        setStatus("Programa interrompido.");
      } else {
        const { start, message, trace } = outcome;
        let text = `\nErro na linha ${start.line}, coluna ${start.column}: ${message}\n`;
        for (const frame of trace.slice(1, -1)) {
          text += `  chamado em ${frame.function} (linha ${frame.line})\n`;
        }
        appendConsole(text, "error");
        setStatus(`Erro de execução na linha ${start.line}.`, "error");
        if (isRunTab) {
          view.dispatch({ effects: setRuntimeError.of({ from: start.offset, to: outcome.end.offset }) });
          goTo(start.offset);
        }
      }
    });
  },
});

inputForm.addEventListener("submit", (e) => {
  e.preventDefault();
  if (runId === null) return;
  const line = inputField.value;
  appendConsole(line + "\n", "echo");
  inputForm.hidden = true;
  void api.sendInput(runId, line);
});

// ---------------------------------------------------------------------------
// Divisória redimensionável e modo editor expandido
// ---------------------------------------------------------------------------

const panel = document.querySelector<HTMLElement>(".panel")!;
$("splitter").addEventListener("pointerdown", (start) => {
  const splitter = start.currentTarget as HTMLElement;
  splitter.setPointerCapture(start.pointerId);
  const startY = start.clientY;
  const startHeight = panel.getBoundingClientRect().height;
  const move = (e: PointerEvent) => {
    const height = Math.max(90, Math.min(window.innerHeight - 200, startHeight - (e.clientY - startY)));
    panel.style.height = `${height}px`;
  };
  splitter.addEventListener("pointermove", move);
  splitter.addEventListener("pointerup", () => splitter.removeEventListener("pointermove", move), { once: true });
});

const utilities = $("utilities");
const utilitiesSplitter = $("utilities-splitter");
const outline = mountOutline(goTo, renameAt);

const libraryButtons = new Map<string, HTMLButtonElement>();
for (const name of Object.keys(LIBRARIES).sort((a, b) => a.localeCompare(b, "pt-BR"))) {
  const button = document.createElement("button");
  button.textContent = name;
  button.title = `Incluir biblioteca ${name}`;
  button.onclick = async () => {
    const tab = activeTab();
    if (!tab || (tab.id === runTabId && runId !== null)) return;
    const doc = view.state.doc;
    try {
      const change = await api.includeLibrary(doc.toString(), name);
      if (activeTab() !== tab || view.state.doc !== doc) return;
      if (change) view.dispatch({ changes: change, annotations: isolateHistory.of("full") });
      button.disabled = true;
      view.focus();
    } catch (message) {
      if (activeTab() === tab) setStatus(typeof message === "string" ? message : "Não foi possível incluir a biblioteca.", "error");
    }
  };
  $("utilities-libraries").append(button);
  libraryButtons.set(name, button);
}

function updateLibraryButtons(symbols: api.SymbolInfo[]) {
  const included = new Set(symbols.filter((symbol) => symbol.kind === "library")
    .map(({ name }) => name.split(" → ", 1)[0]));
  for (const [name, button] of libraryButtons) button.disabled = included.has(name);
}

for (const snippet of SNIPPETS) {
  const button = document.createElement("button");
  button.textContent = snippet.label;
  button.title = snippet.detail ?? snippet.label;
  button.onclick = () => {
    const tab = activeTab();
    if (!tab || (tab.id === runTabId && runId !== null)) return;
    if (snippet.label === "programa") {
      newTab();
      return;
    }
    const { from, to } = view.state.selection.main;
    if (typeof snippet.apply === "function") snippet.apply(view, snippet, from, to);
    view.focus();
  };
  $("utilities-models").append(button);
}

for (const selected of ["libraries", "models"]) {
  $(`utilities-${selected}-tab`).onclick = () => {
    for (const name of ["libraries", "models"]) {
      $(`utilities-${name}-tab`).setAttribute("aria-selected", String(name === selected));
      $(`utilities-${name}`).hidden = name !== selected;
    }
  };
}

function showUtilities(visible: boolean) {
  if (visible && $("code").classList.contains("expanded")) toggleExpanded();
  utilities.hidden = utilitiesSplitter.hidden = !visible;
  $("btn-utilities").setAttribute("aria-expanded", String(visible));
}

function toggleUtilities() {
  if (activeTab()) showUtilities(utilities.hidden === true);
}

utilitiesSplitter.addEventListener("pointerdown", (start) => {
  utilitiesSplitter.setPointerCapture(start.pointerId);
  const width = utilities.offsetWidth;
  const move = (event: PointerEvent) => {
    const next = Math.min(window.innerWidth * 0.55, Math.max(210, width + start.clientX - event.clientX));
    utilities.style.width = `${next}px`;
  };
  utilitiesSplitter.addEventListener("pointermove", move);
  utilitiesSplitter.addEventListener("lostpointercapture", () => {
    utilitiesSplitter.removeEventListener("pointermove", move);
  }, { once: true });
});

function toggleExpanded() {
  if (!activeTab()) return;
  const hidden = !panel.hidden;
  panel.hidden = hidden;
  $("splitter").hidden = hidden;
  $("code").classList.toggle("expanded", hidden);
  $("btn-unexpand").hidden = !hidden;
}

// ---------------------------------------------------------------------------
// Botões e atalhos
// ---------------------------------------------------------------------------

$("home-new").onclick = () => newTab();
$("home-open").onclick = () => void openFiles();
$("btn-open").onclick = () => void openFiles();
const editorOptions = $<HTMLDetailsElement>("editor-options");
const consoleOptions = $<HTMLDetailsElement>("console-tools");

const outlineOptions = $<HTMLDetailsElement>("outline-options");
const optionMenus = [editorOptions, consoleOptions, outlineOptions];
const centerButton = $("editor-center");
centerButton.setAttribute("aria-pressed", String(centerActive));

function centerSelection() {
  view.dispatch({ effects: EditorView.scrollIntoView(view.state.selection.main.head, { y: "center" }) });
}

function toggleCenterSource() {
  if (!activeTab()) return;
  centerActive = !centerActive;
  centerButton.setAttribute("aria-pressed", String(centerActive));
  try {
    localStorage.setItem(centerKey, String(centerActive));
  } catch {
    // A preferência permanece válida nesta sessão.
  }
  if (centerActive) centerSelection();
  view.focus();
}

async function renameAt(offset: number) {
  const tab = activeTab();
  if (!tab || (runTabId === tab.id && runId !== null)) return;
  const doc = view.state.doc;
  const dialog = $<HTMLDialogElement>("rename");
  try {
    const target = await api.renameTarget(doc.toString(), offset);
    if (activeTab() !== tab || view.state.doc !== doc || document.querySelector("dialog[open]")) return;

    const form = $<HTMLFormElement>("rename-form");
    const input = $<HTMLInputElement>("rename-input");
    const error = $("rename-error");
    const submit = $<HTMLButtonElement>("rename-submit");
    $("rename-description").textContent = `${target.name} · ${target.occurrences} ${target.occurrences === 1 ? "ocorrência" : "ocorrências"}`;
    input.value = target.name;
    error.hidden = true;
    $("rename-cancel").onclick = () => dialog.close();
    form.onsubmit = async (event) => {
      event.preventDefault();
      error.hidden = true;
      submit.disabled = true;
      try {
        const changes = await api.renameSymbol(doc.toString(), offset, input.value.trim());
        if (!dialog.open || activeTab() !== tab || view.state.doc !== doc) return;
        if (changes.length) view.dispatch({ changes, annotations: isolateHistory.of("full"), userEvent: "input.rename" });
        dialog.close();
        view.focus();
      } catch (message) {
        error.textContent = typeof message === "string" ? message : "Não foi possível renomear o símbolo.";
        error.hidden = false;
      } finally {
        submit.disabled = false;
      }
    };
    dialog.showModal();
    input.select();
  } catch (message) {
    if (activeTab() === tab && view.state.doc === doc) {
      setStatus(typeof message === "string" ? message : "Não foi possível localizar o símbolo.", "error");
    }
  }
}

function renameSelection() {
  return renameAt(view.state.selection.main.from);
}

async function formatCode() {
  const tab = activeTab();
  if (!tab || (runTabId === tab.id && runId !== null)) return;
  const doc = view.state.doc;
  try {
    const changes = await api.formatSource(doc.toString());
    if (activeTab() !== tab || view.state.doc !== doc) return;
    if (!changes) {
      setStatus("Não foi possível formatar. Confira a sintaxe do programa.", "error");
      return;
    }
    if (changes.length) view.dispatch({ changes, annotations: isolateHistory.of("full"), userEvent: "input.format" });
    setStatus(changes.length ? "Código formatado." : "O código já está formatado.", "ok");
    view.focus();
  } catch {
    if (activeTab() === tab) setStatus("Não foi possível executar o formatador.", "error");
  }
}

function editorMenuAction(id: string, action: () => unknown) {
  $(id).onclick = () => {
    editorOptions.open = false;
    action();
  };
}
const exercises = mountExercises((exerciseDocument) => {
  const source = `/*
${exerciseDocument.exercise.statement.replaceAll("*/", "* /")}
*/

${TEMPLATE}`;
  newTab(source, null, exerciseDocument.name, undefined, exerciseDocument);
});
$("home-exercises").onclick = exercises.browse;
editorMenuAction("editor-exercise", () => {
  const exercise = activeTab()?.exercise;
  if (exercise) void exercises.grade(exercise, view.state.doc.toString());
});
editorMenuAction("editor-search", () => openSearchPanel(view));
editorMenuAction("editor-rename", renameSelection);
editorMenuAction("editor-format", formatCode);
editorMenuAction("editor-center", toggleCenterSource);
editorMenuAction("editor-expand", toggleExpanded);
editorMenuAction("btn-utilities", toggleUtilities);
$("btn-unexpand").onclick = toggleExpanded;

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Node)) return;
  for (const options of optionMenus) {
    if (options.open && !options.contains(event.target)) options.open = false;
  }
});
$("utilities-close").onclick = () => {
  showUtilities(false);
  view.focus();
};
$("btn-save").onclick = () => void save();
$("btn-save-as").onclick = () => void save(true);
$("btn-run").onclick = () => void runOrContinue();
$("btn-debug").onclick = () => void stepOrStart();
$("btn-stop").onclick = () => void stopProgram();
$("btn-debug-pause").onclick = () => { if (runId !== null) void api.debugPause(runId); };
$("btn-debug-continue").onclick = () => void debugCommand(api.debugContinue);
$("btn-debug-step-over").onclick = () => void debugCommand(api.debugStepOver);
$("btn-debug-step-into").onclick = () => void debugCommand(api.debugStepInto);
$("btn-debug-step-out").onclick = () => void debugCommand(api.debugStepOut);
$("btn-clear").onclick = () => output.replaceChildren();
$("btn-copy").onclick = async () => {
  try {
    await navigator.clipboard.writeText(output.textContent ?? "");
    setStatus("Saída copiada.", "ok");
  } catch (error) {
    setStatus(String(error), "error");
  }
};
$("tab-console").onclick = () => showPanel("console");
$("tab-messages").onclick = () => showPanel("messages");
$("tab-debug").onclick = () => showPanel("debug");

function cycleTab(step: number) {
  const ids = [homeId, ...(helpOpen ? [helpId] : []), ...tabs.map((tab) => tab.id)];
  activate(ids[(ids.indexOf(activeId) + step + ids.length) % ids.length]);
}

const shortcutDialog = $<HTMLDialogElement>("shortcuts");

function showShortcuts() {
  if (!shortcutDialog.open) shortcutDialog.showModal();
}

const shortcuts: {
  label: string;
  command: string;
  matches: (event: KeyboardEvent) => boolean;
  action: () => unknown;
}[] = [
  { label: "Shift+F6 / Ctrl+Enter", command: "Executar", matches: (e) =>
    (e.shiftKey && e.key === "F6") || ((e.ctrlKey || e.metaKey) && e.key === "Enter"), action: runOrContinue },
  { label: "Shift+F5", command: "Depurar", matches: (e) => e.shiftKey && e.key === "F5",
    action: stepOrStart },
  { label: "Shift+F7", command: "Interromper", matches: (e) => e.shiftKey && e.key === "F7", action: stopProgram },
  { label: "F9", command: "Pausar", matches: (e) => !e.shiftKey && e.key === "F9",
    action: () => { if (runId !== null) void api.debugPause(runId); } },
  { label: "F5", command: "Continuar", matches: (e) => !e.shiftKey && e.key === "F5",
    action: () => debugCommand(api.debugContinue) },
  { label: "Shift+F9 / F10", command: "Próxima instrução",
    matches: (e) => (e.shiftKey && e.key === "F9") || e.key === "F10", action: () => debugCommand(api.debugStepOver) },
  { label: "Alt+F11", command: "Entrar na função", matches: (e) => e.altKey && !e.shiftKey && e.key === "F11",
    action: () => debugCommand(api.debugStepInto) },
  { label: "Shift+F11", command: "Sair da função", matches: (e) => e.shiftKey && e.key === "F11",
    action: () => debugCommand(api.debugStepOut) },
  { label: "F1", command: "Ajuda", matches: (e) => e.key === "F1", action: () => activate(helpId) },
  { label: "Ctrl+R", command: "Renomear símbolo",
    matches: (e) => (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "KeyR", action: renameSelection },
  { label: "Ctrl+Alt+F", command: "Formatar código-fonte",
    matches: (e) => (e.ctrlKey || e.metaKey) && e.altKey && !e.shiftKey && e.code === "KeyF", action: formatCode },
  { label: "F3", command: "Dicas de interface", matches: (e) => e.key === "F3", action: () => tips.open() },
  { label: "F12", command: "Sobre", matches: (e) => e.key === "F12", action: () =>
    $<HTMLDialogElement>("about").showModal() },
  { label: "F11", command: "Atalhos de teclado", matches: (e) => !e.altKey && !e.shiftKey && e.key === "F11",
    action: showShortcuts },
  { label: "Alt+Home", command: "Tela inicial", matches: (e) => e.altKey && e.key === "Home", action: () => activate(0) },
  { label: "Alt+←", command: "Aba anterior", matches: (e) => e.altKey && e.key === "ArrowLeft", action: () => cycleTab(-1) },
  { label: "Alt+→", command: "Próxima aba", matches: (e) => e.altKey && e.key === "ArrowRight", action: () => cycleTab(1) },
  { label: "Shift+Pause", command: "Ativar/desativar centralização",
    matches: (e) => e.shiftKey && e.key === "Pause", action: toggleCenterSource },
  { label: "Shift+Esc", command: "Expandir/restaurar editor",
    matches: (e) => e.shiftKey && e.key === "Escape", action: toggleExpanded },
];

function fileShortcut(key: string, shift: boolean, command: string, action: () => unknown) {
  shortcuts.push({
    label: shift ? `Ctrl+Shift+${key}` : `Ctrl+${key}`,
    command,
    matches: (e) => (e.ctrlKey || e.metaKey) && e.shiftKey === shift && e.key.toUpperCase() === key,
    action,
  });
}

fileShortcut("N", false, "Novo arquivo", () => newTab());
fileShortcut("O", false, "Abrir arquivo", openFiles);
fileShortcut("S", false, "Salvar", () => save());
fileShortcut("S", true, "Salvar como", () => save(true));
fileShortcut("E", false, "Parâmetros de execução", () => {
  if (!activeTab()) return;
  editorOptions.open = true;
  $("args").focus();
});
fileShortcut("B", false, "Painel Utilitários", toggleUtilities);
fileShortcut("L", false, "Buscar na estrutura", () => {
  if (!activeTab()) return;
  showUtilities(true);
  outline.focus();
});
fileShortcut("Q", false, "Fechar aba", () => closeTab(activeId));
fileShortcut("Q", true, "Fechar todas as abas", async () => {
  for (const tab of [...tabs]) if (!(await closeTab(tab.id))) return;
  if (helpOpen) await closeTab(helpId);
});

for (const [key, ty] of Object.entries({ I: "inteiro", C: "caracter", R: "real", A: "cadeia", L: "logico", V: "vazio", T: "" })) {
  shortcuts.push({
    label: `Alt+Shift+${key}`, command: ty ? `Filtrar tipo ${ty}` : "Mostrar todos os tipos",
    matches: (e) => e.altKey && e.shiftKey && e.code === `Key${key}`,
    action: () => { if (activeTab()) { showUtilities(true); outline.filterType(ty); } },
  });
}

const symbolShortcuts = { A: "variable", E: "array", M: "matrix", F: "function", T: "constant" } as const;
for (const [key, kind] of Object.entries(symbolShortcuts)) {
  fileShortcut(key, true, `Filtrar ${symbolKindLabels[kind].toLowerCase()}`, () => {
    if (!activeTab()) return;
    showUtilities(true);
    outline.filterKind(kind);
  });
}

for (const { label, command } of [...shortcuts, { label: "Ctrl+F", command: "Localizar e substituir" }]) {
  const term = document.createElement("dt");
  const shortcut = document.createElement("kbd");
  const description = document.createElement("dd");
  shortcut.textContent = label;
  description.textContent = command;
  term.append(shortcut);
  $("shortcut-list").append(term, description);
}
$("home-shortcuts").onclick = showShortcuts;
const aboutDialog = $<HTMLDialogElement>("about");
const reportDialog = $<HTMLDialogElement>("report");
const reportError = $("report-error");
$("home-about").onclick = () => aboutDialog.showModal();
const updates = $<HTMLDialogElement>("updates");
const updateStatus = $("update-status");
const updateNotes = $("update-notes");
const updateDownload = $<HTMLButtonElement>("update-download");
let updateRequest = 0;
$("about-updates").onclick = async () => {
  const request = ++updateRequest;
  updateNotes.hidden = true;
  updateDownload.textContent = "Ver versões no GitHub";
  updateStatus.textContent = "Verificando atualizações…";
  $("update-error").hidden = true;
  updates.showModal();
  try {
    const result = await api.checkUpdates();
    if (request !== updateRequest) return;
    if (result.status === "available") {
      updateStatus.textContent = `Versão ${result.latest} disponível. Versão instalada ${result.current}.`;
      updateNotes.textContent = result.notes;
      updateNotes.hidden = !result.notes;
      updateDownload.textContent = "Baixar atualização";
    } else if (result.status === "current") {
      updateStatus.textContent = `O Portugol Studio RS ${result.current} está atualizado.`;
    } else {
      updateStatus.textContent = "Ainda não há uma versão estável publicada no GitHub.";
    }
  } catch (message) {
    if (request === updateRequest) {
      updateStatus.textContent = typeof message === "string" ? message : "Não foi possível verificar atualizações.";
    }
  }
};
updateDownload.onclick = async () => {
  updateDownload.disabled = true;
  $("update-error").hidden = true;
  try {
    await api.openProjectPage("releases");
  } catch {
    $("update-error").hidden = false;
  } finally {
    updateDownload.disabled = false;
  }
};
let alertRequest = 0;
$("home-alerts").onclick = async () => {
  const request = ++alertRequest;
  const dialog = $<HTMLDialogElement>("alerts");
  const status = $("alerts-status");
  const list = $("alerts-list");
  status.hidden = false;
  status.textContent = "Consultando alertas…";
  list.hidden = true;
  list.replaceChildren();
  dialog.showModal();
  try {
    const alerts = await api.projectAlerts();
    if (request !== alertRequest) return;
    if (!alerts.length) {
      status.textContent = "Não há alertas ou avisos.";
      return;
    }
    for (const alert of alerts) {
      const article = document.createElement("article");
      const title = document.createElement("h3");
      title.textContent = alert.title;
      const message = document.createElement("p");
      message.textContent = alert.message;
      article.append(title, message);
      list.append(article);
    }
    status.hidden = true;
    list.hidden = false;
  } catch (message) {
    if (request === alertRequest) {
      status.textContent = typeof message === "string" ? message : "Não foi possível consultar os alertas.";
    }
  }
};
$("home-report").onclick = $("about-report").onclick = () => {
  reportError.hidden = true;
  reportDialog.showModal();
};
$("home-contribute").onclick = () => {
  void api.openProjectPage("contribute").catch(() => setStatus("Não foi possível abrir o navegador.", "error"));
};
$("report-open").onclick = async () => {
  const button = $<HTMLButtonElement>("report-open");
  button.disabled = true;
  reportError.hidden = true;
  try {
    await api.openProjectPage("bug_report");
    reportDialog.close();
  } catch {
    reportError.textContent = "Não foi possível abrir o navegador. Use github.com/wendelmax/portugol-studio-rs/issues/new.";
    reportError.hidden = false;
  } finally {
    button.disabled = false;
  }
};
$("home-help").onclick = $("btn-help").onclick = () => activate(helpId);

window.addEventListener("keydown", (e) => {
  if (e.defaultPrevented || document.querySelector("dialog[open]")) return;
  if (e.key === "Escape" && !e.shiftKey) {
    const menu = optionMenus.find((options) => options.open);
    if (menu) {
      e.preventDefault();
      menu.open = false;
      menu.querySelector("summary")?.focus();
      return;
    }
  }
  const shortcut = shortcuts.find(({ matches }) => matches(e));
  if (!shortcut) return;
  e.preventDefault();
  void shortcut.action();
});

// Confirma antes de fechar a janela com alterações não salvas.
void runListeners.catch((error) => setStatus(String(error), "error"));
const appWindow = getCurrentWindow();
function windowAction(action: () => Promise<void>) {
  void action().catch((error) => setStatus(String(error), "error"));
}
$("window-min").onclick = () => windowAction(() => appWindow.minimize());
$("window-max").onclick = () => windowAction(() => appWindow.toggleMaximize());
$("window-close").onclick = () => windowAction(() => appWindow.close());
void appWindow.onCloseRequested(async (event) => {
  const current = activeTab();
  if (current) current.state = view.state;
  const dirty = tabs.filter(isDirty);
  if (dirty.length === 0) return;
  const names = dirty.map(tabName).join(", ");
  const quit = await ask(`Há alterações não salvas em: ${names}. Sair mesmo assim?`, {
    title: "Sair", kind: "warning", okLabel: "Sair sem salvar", cancelLabel: "Cancelar",
  });
  if (!quit) event.preventDefault();
});

const themeSelect = $<HTMLSelectElement>("theme");
const themeEditor = mountThemeEditor(themeSelect, applyTheme);
themeSelect.value = themeChoice();
themeSelect.onchange = () => applyTheme(themeSelect.value);
editorMenuAction("editor-themes", themeEditor.open);

for (const id of ["editor-font", "console-font"]) {
  const input = $<HTMLInputElement>(id);
  const key = `portugol.${id}`;
  const applyFontSize = () => {
    const size = Math.round(Math.max(10, Math.min(50, input.valueAsNumber || 12)));
    input.value = String(size);
    document.documentElement.style.setProperty(`--${id}`, `${size}px`);
    return input.value;
  };
  try {
    input.value = localStorage.getItem(key) ?? input.value;
  } catch {
    // O ajuste manual permanece disponível.
  }
  applyFontSize();
  input.onchange = () => {
    const value = applyFontSize();
    try {
      localStorage.setItem(key, value);
    } catch {
      // A preferência vale nesta sessão.
    }
  };
}

let help: ReturnType<typeof import("./help").mountHelp> | undefined;
let helpLoading: Promise<void> | undefined;

async function openHelp() {
  if (!help) {
    helpLoading ??= import("./help").then(({ mountHelp }) => {
      help = mountHelp((source, file) => newTab(source, null, file));
    }).catch((error) => {
      $("help-article").textContent = "Não foi possível carregar a ajuda.";
      setStatus(String(error), "error");
      helpLoading = undefined;
    });
    await helpLoading;
  }
  if (activeId === helpId) help?.focusSearch();
}
$("home-version").textContent = `v${version}`;
$("about-version").textContent = `Versão ${version}`;
$("about-license").textContent = license;
mountIcons();
renderRecent();
mountExamples((source, file, workPath) => newTab(source, null, file, workPath));
activate(0);
applyTheme(themeChoice());
showPanel("console");
showUtilities(true);
const tips = mountTips(themeEditor.open);
$("home-tips").onclick = tips.open;
