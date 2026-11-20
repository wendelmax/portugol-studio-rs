// IDE: abas de arquivos, editor CodeMirror, console e execução.
//
// Atalhos seguem o Portugol Studio original (TelaAtalhos.java):
// Shift+F6 executa, Shift+F7 interrompe, Ctrl+Q fecha a aba,
// Alt+←/→ troca de aba, Shift+Esc expande o editor.

import "./style.css";

import { EditorState, Compartment, RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import {
  EditorView, GutterMarker, gutter, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter,
  drawSelection, highlightSpecialChars, Decoration, type DecorationSet,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import {
  syntaxHighlighting, HighlightStyle, bracketMatching, indentOnInput,
} from "@codemirror/language";
import { linter, lintGutter, type Diagnostic } from "@codemirror/lint";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { tags } from "@lezer/highlight";
import { open as openDialog, save as saveDialog, ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { mountExamples } from "./examples";
import { portugol } from "./portugol";
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
  state: EditorState;
  /** Texto da última gravação, para saber se há alterações. */
  saved: string;
  diagnostics: api.DiagnosticInfo[];
}

// Tema: "system" segue o sistema operacional; a escolha fica salva.
type ThemeChoice = "system" | "light" | "dark";
const THEME_KEY = "portugol.theme";
const theme = new Compartment();
const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

function themeChoice(): ThemeChoice {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === "light" || saved === "dark") return saved;
  } catch {
    // Sem armazenamento local: fica no padrão.
  }
  return "system";
}

const isDark = () => {
  const choice = themeChoice();
  return choice === "dark" || (choice === "system" && darkQuery.matches);
};
const editorColors = syntaxHighlighting(HighlightStyle.define([
  { tag: tags.keyword, color: "var(--syntax-keyword)" },
  { tag: [tags.typeName, tags.namespace], color: "var(--syntax-type)" },
  { tag: [tags.number, tags.bool], color: "var(--syntax-number)" },
  { tag: tags.string, color: "var(--syntax-string)" },
  { tag: tags.comment, color: "var(--syntax-comment)" },
  { tag: tags.operator, color: "var(--syntax-operator)" },
]));
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
    const dot = document.createElement("span");
    dot.className = "cm-breakpoint";
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
    const source = view.state.doc.toString();
    const diagnostics = await api.check(source);
    const tab = activeTab();
    if (tab && view.state.doc.toString() === source) {
      tab.diagnostics = diagnostics;
      renderMessages();
    }
    return diagnostics.map((d) => toCodeMirror(d, view.state.doc.length));
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
    EditorState.tabSize.of(4),
    portugol(),
    editorColors,
    theme.of(editorTheme()),
    lintGutter(),
    portugolLinter,
    runtimeError,
    breakpoints,
    breakpointGutter,
    debugLine,
    keymap.of([
      ...closeBracketsKeymap, ...defaultKeymap, ...searchKeymap,
      ...historyKeymap, ...completionKeymap, indentWithTab,
    ]),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) renderTabs();
      if (update.selectionSet || update.docChanged) renderCursor();
    }),
  ];
}

const view = new EditorView({ parent: $("editor") });

function applyTheme(choice: ThemeChoice) {
  try {
    localStorage.setItem(THEME_KEY, choice);
  } catch {
    // Ignora: a escolha vale só para esta sessão.
  }
  if (choice === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = choice;
  view.dispatch({ effects: theme.reconfigure(editorTheme()) });
}

darkQuery.addEventListener("change", () => view.dispatch({ effects: theme.reconfigure(editorTheme()) }));

// ---------------------------------------------------------------------------
// Abas
// ---------------------------------------------------------------------------

let tabs: Tab[] = [];
let activeId = 0;
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

function newTab(text = TEMPLATE, path: string | null = null, exampleName?: string): Tab {
  const tab: Tab = {
    id: nextTabId++,
    path,
    exampleName,
    state: EditorState.create({ doc: text, extensions: extensions() }),
    saved: text,
    diagnostics: [],
  };
  tabs.push(tab);
  activate(tab.id);
  return tab;
}

function activate(id: number) {
  const current = activeTab();
  if (current) current.state = view.state;
  const tab = tabs.find((t) => t.id === id);
  if (!tab && id !== 0) return;
  activeId = id;
  $("home").hidden = Boolean(tab);
  $("code").hidden = !tab;
  if (!tab) {
    renderTabs();
    $("cursor").textContent = "";
    $<HTMLButtonElement>("home-new").focus();
    return;
  }
  view.setState(tab.state);
  view.dispatch({ effects: [
    theme.reconfigure(editorTheme()),
    setDebugLine.of(tab.id === runTabId ? paused?.position.line ?? null : null),
  ] });
  renderTabs();
  renderMessages();
  renderCursor();
  view.focus();
}

async function closeTab(id: number): Promise<boolean> {
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
  home.className = "tab" + (activeId === 0 ? " active" : "");
  home.textContent = "Início";
  home.title = "Tela inicial (Alt+Home)";
  home.setAttribute("aria-current", String(activeId === 0));
  home.onclick = () => activate(0);
  const add = document.createElement("button");
  add.className = "tab";
  add.textContent = "+";
  add.title = "Novo arquivo (Ctrl+N)";
  add.setAttribute("aria-label", "Novo arquivo");
  add.onclick = () => newTab();
  bar.replaceChildren(
    home,
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
      el.append(name, close);
      name.onclick = () => activate(tab.id);
      el.onauxclick = (e) => e.button === 1 && void closeTab(tab.id);
      return el;
    }),
    add,
  );
  const tab = activeTab();
  document.title = tab ? `${tabName(tab)} — Portugol Studio` : "Portugol Studio";
}

function renderCursor() {
  if (activeId === 0) return;
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
    button.textContent = path.split(/[\\/]/).pop() ?? path;
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
  $<HTMLButtonElement>("btn-run").disabled = running;
  $<HTMLButtonElement>("btn-debug").disabled = running;
  $<HTMLButtonElement>("btn-stop").disabled = !running;
  $<HTMLButtonElement>("btn-debug-pause").disabled = !running || paused !== null;
  for (const id of [
    "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out",
  ]) {
    $<HTMLButtonElement>(id).disabled = paused === null;
  }
  if (!running) inputForm.hidden = true;
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
    li.onclick = () => goTo(frame.position.offset);
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
    const id = await api.run(source, args, tab.path, lines, pauseOnStart);
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
  const tab = activeTab();
  const list = $("messages");
  const diagnostics = tab?.diagnostics ?? [];
  const errors = diagnostics.filter((d) => d.severity === "error").length;
  $("message-count").textContent = errors ? String(errors) : "";
  if (diagnostics.length === 0) {
    const empty = document.createElement("li");
    empty.className = "empty";
    empty.textContent = "Nenhum problema encontrado.";
    list.replaceChildren(empty);
    return;
  }
  list.replaceChildren(
    ...diagnostics.map((d) => {
      const item = document.createElement("li");
      item.className = d.severity;
      const where = document.createElement("span");
      where.className = "where";
      where.textContent = `${d.start.line}:${d.start.column}`;
      const kind = document.createElement("span");
      kind.className = "kind";
      kind.textContent = d.severity === "error" ? "erro" : "aviso";
      const message = document.createElement("span");
      message.textContent = d.message;
      item.append(where, kind, message);
      if (d.help) {
        const help = document.createElement("span");
        help.className = "help";
        help.textContent = `— ${d.help}`;
        item.append(help);
      }
      item.onclick = () => goTo(d.start.offset);
      return item;
    }),
  );
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

function toggleExpanded() {
  if (!activeTab()) return;
  const hidden = !panel.hidden;
  panel.hidden = hidden;
  $("splitter").hidden = hidden;
}

// ---------------------------------------------------------------------------
// Botões e atalhos
// ---------------------------------------------------------------------------

$("home-new").onclick = () => newTab();
$("home-open").onclick = () => void openFiles();
$("btn-open").onclick = () => void openFiles();
$("btn-save").onclick = () => void save();
$("btn-save-as").onclick = () => void save(true);
$("btn-run").onclick = () => void runProgram();
$("btn-debug").onclick = () => void runProgram(true);
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
  const ids = [0, ...tabs.map((tab) => tab.id)];
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
    (e.shiftKey && e.key === "F6") || ((e.ctrlKey || e.metaKey) && e.key === "Enter"), action: runProgram },
  { label: "Shift+F5", command: "Depurar", matches: (e) => e.shiftKey && e.key === "F5",
    action: () => runProgram(true) },
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
  { label: "F11", command: "Atalhos de teclado", matches: (e) => !e.altKey && !e.shiftKey && e.key === "F11",
    action: showShortcuts },
  { label: "Alt+Home", command: "Tela inicial", matches: (e) => e.altKey && e.key === "Home", action: () => activate(0) },
  { label: "Alt+←", command: "Aba anterior", matches: (e) => e.altKey && e.key === "ArrowLeft", action: () => cycleTab(-1) },
  { label: "Alt+→", command: "Próxima aba", matches: (e) => e.altKey && e.key === "ArrowRight", action: () => cycleTab(1) },
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
fileShortcut("Q", false, "Fechar aba", () => closeTab(activeId));
fileShortcut("Q", true, "Fechar todas as abas", async () => {
  for (const tab of [...tabs]) if (!(await closeTab(tab.id))) break;
});

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

window.addEventListener("keydown", (e) => {
  if (e.defaultPrevented || shortcutDialog.open) return;
  const shortcut = shortcuts.find(({ matches }) => matches(e));
  if (!shortcut) return;
  e.preventDefault();
  void shortcut.action();
});

// Confirma antes de fechar a janela com alterações não salvas.
void runListeners.catch((error) => setStatus(String(error), "error"));
void getCurrentWindow().onCloseRequested(async (event) => {
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
themeSelect.value = themeChoice();
themeSelect.onchange = () => applyTheme(themeSelect.value as ThemeChoice);

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

renderRecent();
mountExamples((source, file) => newTab(source, null, file));
activate(0);
applyTheme(themeChoice());
showPanel("console");
