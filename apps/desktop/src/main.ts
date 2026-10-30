// IDE: abas de arquivos, editor CodeMirror, console e execução.
//
// Atalhos seguem o Portugol Studio original (TelaAtalhos.java):
// Shift+F6 executa, Shift+F7 interrompe, Ctrl+Q fecha a aba,
// Alt+←/→ troca de aba, Shift+Esc expande o editor.

import "./style.css";

import { EditorState, Compartment, StateEffect, StateField, type Extension } from "@codemirror/state";
import {
  EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter,
  drawSelection, highlightSpecialChars, Decoration, type DecorationSet,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import {
  syntaxHighlighting, defaultHighlightStyle, bracketMatching, indentOnInput,
} from "@codemirror/language";
import { linter, lintGutter, type Diagnostic } from "@codemirror/lint";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { oneDark } from "@codemirror/theme-one-dark";
import { open as openDialog, save as saveDialog, ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

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
const editorTheme = (): Extension => (isDark() ? oneDark : syntaxHighlighting(defaultHighlightStyle));

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
    theme.of(editorTheme()),
    lintGutter(),
    portugolLinter,
    runtimeError,
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
  if (!tab.path) return "Sem título";
  return tab.path.split(/[\\/]/).pop() ?? tab.path;
}

function isDirty(tab: Tab): boolean {
  const text = tab.id === activeId ? view.state.doc.toString() : tab.state.doc.toString();
  return text !== tab.saved;
}

function newTab(text = TEMPLATE, path: string | null = null): Tab {
  const tab: Tab = {
    id: nextTabId++,
    path,
    state: EditorState.create({ doc: text, extensions: extensions() }),
    saved: path ? text : TEMPLATE,
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
  if (!tab) return;
  activeId = id;
  view.setState(tab.state);
  view.dispatch({ effects: theme.reconfigure(editorTheme()) });
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
  if (tabs.length === 0) newTab();
  else if (id === activeId) activate(tabs[Math.max(0, index - 1)].id);
  else renderTabs();
  return true;
}

function renderTabs() {
  const bar = $("tabs");
  bar.replaceChildren(
    ...tabs.map((tab) => {
      const el = document.createElement("div");
      el.className = "tab" + (tab.id === activeId ? " active" : "");
      el.title = tab.path ?? "Arquivo ainda não salvo";
      el.setAttribute("role", "tab");
      const name = document.createElement("span");
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
      el.onclick = () => activate(tab.id);
      el.onauxclick = (e) => e.button === 1 && void closeTab(tab.id);
      return el;
    }),
  );
  const tab = activeTab();
  document.title = tab ? `${tabName(tab)} — Portugol Studio` : "Portugol Studio";
}

function renderCursor() {
  const head = view.state.selection.main.head;
  const line = view.state.doc.lineAt(head);
  $("cursor").textContent = `Linha ${line.number}, coluna ${head - line.from + 1}`;
}

// ---------------------------------------------------------------------------
// Arquivos
// ---------------------------------------------------------------------------

const FILTERS = [{ name: "Programa Portugol", extensions: ["por"] }];

async function openFiles() {
  const selected = await openDialog({ multiple: true, filters: FILTERS });
  if (!selected) return;
  for (const path of Array.isArray(selected) ? selected : [selected]) {
    const existing = tabs.find((t) => t.path === path);
    if (existing) {
      activate(existing.id);
      continue;
    }
    try {
      const text = await api.readFile(path);
      const current = activeTab();
      // Reaproveita a aba "Sem título" intocada.
      if (current && !current.path && !isDirty(current) && tabs.length === 1) tabs = [];
      newTab(text, path);
    } catch (error) {
      setStatus(String(error), "error");
    }
  }
}

async function save(as = false): Promise<boolean> {
  const tab = activeTab();
  if (!tab) return false;
  let path = tab.path;
  if (!path || as) {
    path = await saveDialog({ filters: FILTERS, defaultPath: tab.path ?? "programa.por" });
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
  renderTabs();
  setStatus(`Salvo em ${path}`, "ok");
  return true;
}

// ---------------------------------------------------------------------------
// Execução e console
// ---------------------------------------------------------------------------

let runId: number | null = null;
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
  $<HTMLButtonElement>("btn-stop").disabled = !running;
  if (!running) inputForm.hidden = true;
}

function parseArgs(text: string): string[] {
  // Aspas agrupam: `"João Silva" 19` -> ["João Silva", "19"].
  return [...text.matchAll(/"([^"]*)"|(\S+)/g)].map((m) => m[1] ?? m[2]);
}

async function runProgram() {
  if (runId !== null) return;
  const tab = activeTab();
  if (!tab) return;
  view.dispatch({ effects: setRuntimeError.of(null) });
  showPanel("console");
  output.replaceChildren();
  try {
    runId = await api.run(
      view.state.doc.toString(),
      parseArgs($<HTMLInputElement>("args").value),
      tab.path,
    );
    setRunning(true);
    setStatus("Executando…");
  } catch (errors) {
    tab.diagnostics = errors as api.DiagnosticInfo[];
    renderMessages();
    showPanel("messages");
    setStatus("O programa contém erros e não pode ser executado.", "error");
  }
}

async function stopProgram() {
  if (runId !== null) await api.stop(runId);
}

function showPanel(which: "console" | "messages") {
  $("tab-console").classList.toggle("active", which === "console");
  $("tab-messages").classList.toggle("active", which === "messages");
  consoleBox.hidden = which !== "console";
  $("messages").hidden = which !== "messages";
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

void api.listenRun({
  output(id, text) {
    if (id === runId) appendConsole(text);
  },
  clear(id) {
    if (id === runId) output.replaceChildren();
  },
  input(id, expected) {
    if (id !== runId) return;
    $("console-hint").textContent = `leia (${expected}):`;
    inputForm.hidden = false;
    inputField.value = "";
    inputField.focus();
    consoleBox.scrollTop = consoleBox.scrollHeight;
  },
  finished(id, outcome) {
    if (id !== runId) return;
    runId = null;
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
      view.dispatch({ effects: setRuntimeError.of({ from: start.offset, to: outcome.end.offset }) });
      goTo(start.offset);
    }
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
  const hidden = !panel.hidden;
  panel.hidden = hidden;
  $("splitter").hidden = hidden;
}

// ---------------------------------------------------------------------------
// Botões e atalhos
// ---------------------------------------------------------------------------

$("btn-new").onclick = () => newTab();
$("btn-open").onclick = () => void openFiles();
$("btn-save").onclick = () => void save();
$("btn-save-as").onclick = () => void save(true);
$("btn-run").onclick = () => void runProgram();
$("btn-stop").onclick = () => void stopProgram();
$("btn-clear").onclick = () => output.replaceChildren();
$("tab-console").onclick = () => showPanel("console");
$("tab-messages").onclick = () => showPanel("messages");

function cycleTab(step: number) {
  const index = tabs.findIndex((t) => t.id === activeId);
  activate(tabs[(index + step + tabs.length) % tabs.length].id);
}

window.addEventListener("keydown", (e) => {
  const ctrl = e.ctrlKey || e.metaKey;
  const key = e.key.toLowerCase();
  const actions: [boolean, () => unknown][] = [
    [e.shiftKey && e.key === "F6", runProgram],
    [ctrl && e.key === "Enter", runProgram],
    [e.shiftKey && e.key === "F7", stopProgram],
    [ctrl && !e.shiftKey && key === "n", () => newTab()],
    [ctrl && key === "o", openFiles],
    [ctrl && e.shiftKey && key === "s", () => save(true)],
    [ctrl && !e.shiftKey && key === "s", () => save()],
    [ctrl && !e.shiftKey && key === "q", () => closeTab(activeId)],
    [ctrl && e.shiftKey && key === "q", async () => {
      for (const tab of [...tabs]) if (!(await closeTab(tab.id))) break;
    }],
    [e.altKey && e.key === "ArrowRight", () => cycleTab(1)],
    [e.altKey && e.key === "ArrowLeft", () => cycleTab(-1)],
    [e.shiftKey && e.key === "Escape", toggleExpanded],
  ];
  const action = actions.find(([match]) => match);
  if (action) {
    e.preventDefault();
    void action[1]();
  }
});

// Confirma antes de fechar a janela com alterações não salvas.
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

newTab();
applyTheme(themeChoice());
showPanel("console");
