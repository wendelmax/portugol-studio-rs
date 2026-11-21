import { mountTips } from "./tips";
import { applyThemeColors, mountThemeEditor, themeProfile } from "./themes";
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
import { mountExercises, readExercise, type ExerciseDocument } from "./exercises";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline, symbolKindLabels } from "./outline";
import { panelContentId, type PanelName } from "./panel-tabs";
import { LIBRARIES, portugol, portugolRs, SNIPPETS } from "./portugol";
import { detectSourceLanguage, ensureSourceExtension, type SourceLanguage } from "./document-language";
import { flattenWorkspaceEntries, workspaceFileKind, workspaceSourceFileName, type WorkspaceEntry } from "./workspace-explorer";
import { conceptsForMode, resolveConceptForMode } from "./learning-content";
import { loadLearningMode, saveLearningMode, type LearningMode } from "./learning-mode";
import {
  getOwnershipScenario,
  getOwnershipSnapshot,
  ownershipScenarioIds,
  type OwnershipBindingState,
  type OwnershipScenarioId,
} from "./ownership-lab";
import * as api from "./api";

const TEMPLATE = "programa\n{\n\tfuncao inicio()\n\t{\n\t\t\n\t}\n}\n";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------

interface Tab {
  id: number;
  path: string | null;
  language: SourceLanguage;
  exampleName?: string;
  examplePath?: string;
  exercise?: ExerciseDocument;
  workspaceId?: string;
  workspacePath?: string;
  workspaceExercisePath?: string;
  state: EditorState;
  /** Texto da última gravação, para saber se há alterações. */
  saved: string;
  diagnostics: api.DiagnosticInfo[];
  symbols: api.SymbolInfo[];
}

// Tema: "system" segue o sistema operacional; a escolha fica salva.
const THEME_KEY = "portugol.theme";
const theme = new Compartment();
const sourceLanguage = new Compartment();
const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

function themeChoice(): string {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === "light" || saved === "dark" || saved === "system" || (saved && themeProfile(saved))) return saved;
  } catch {
    // Sem armazenamento local: fica no padrão.
  }
  return "dark";
}

let appliedTheme = themeChoice();
const isDark = () => {
  const choice = themeProfile(appliedTheme)?.base ?? appliedTheme;
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
    const language = tab?.language ?? "portugol";
    const inspection = language === "portugol-rs"
      ? await api.inspectPortugolRs(doc.toString())
      : await api.inspect(doc.toString());
    const { diagnostics, symbols } = inspection;
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

function extensions(language: SourceLanguage): Extension[] {
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
    sourceLanguage.of(language === "portugol-rs" ? portugolRs() : portugol()),
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
    EditorView.domEventHandlers({
      click(event, editor) {
        const mouse = event as MouseEvent;
        if (mouse.button !== 0 || (!mouse.ctrlKey && !mouse.metaKey)) return false;
        const offset = editor.posAtCoords({ x: mouse.clientX, y: mouse.clientY });
        if (offset === null) return false;
        void goToDeclarationAt(offset, true);
        return false;
      },
    }),
    keymap.of([
      { key: "F12", run: () => { void goToDeclarationAt(view.state.selection.main.head); return true; } },
      ...closeBracketsKeymap, ...defaultKeymap, ...editorSearchKeys,
      ...historyKeymap, ...completionKeymap, ...foldKeymap, indentWithTab,
    ]),
    EditorView.updateListener.of((update) => {
      if (update.docChanged) renderTabs();
      if (update.docChanged && learningMode === "bridge") scheduleRustExport();
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
  else document.documentElement.dataset.theme = themeProfile(choice)?.base ?? choice;
  applyThemeColors(document.documentElement, themeProfile(choice)?.colors);
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
let workspaceState: api.WorkspaceListing | null = null;
let workspaceLoading = false;
let workspaceRequest = 0;
let selectedWorkspacePath: string | null = null;
const expandedWorkspaceFolders = new Set<string>();
const homeId = 0;
const helpId = -1;
let activeId = homeId;
let helpOpen = false;
let nextTabId = 1;
let rustExportTimer: number | undefined;
let rustExportGeneration = 0;
let definitionNavigationGeneration = 0;
let learningMode: LearningMode = loadLearningMode();
let selectedLearningConceptId = "variables";
let ownershipScenarioId: OwnershipScenarioId = "move";
let ownershipStepIndex = 0;

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
const PORS_TEMPLATE = "funcao inicio() {\n    seja mutavel idade: inteiro = 16;\n    idade += 1;\n    escreva(idade);\n}\n";

function newTab(text = NEW_PROGRAM, path: string | null = null, exampleName?: string, examplePath?: string, exercise?: ExerciseDocument, explicitLanguage?: SourceLanguage, workspaceFile?: { id: string; path: string }): Tab {
  const language = explicitLanguage ?? detectSourceLanguage(path ?? "programa.por");
  const tab: Tab = {
    id: nextTabId++,
    path,
    language,
    exampleName,
    examplePath,
    exercise,
    workspaceId: workspaceFile?.id,
    workspacePath: workspaceFile?.path,
    state: EditorState.create({ doc: text, extensions: extensions(language) }),
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
    rustExportGeneration++;
    renderTabs();
    $("cursor").textContent = "";
    if (id === helpId) void openHelp();
    else $("home").focus();
    return;
  }
  const workspacePath = tab.workspacePath ?? tab.workspaceExercisePath;
  if (tab.workspaceId === workspaceState?.id && workspacePath) {
    selectedWorkspacePath = workspacePath;
    renderWorkspaceExplorer();
  }
  view.setState(tab.state);
  view.dispatch({ effects: [
    theme.reconfigure(editorTheme()),
    sourceLanguage.reconfigure(tab.language === "portugol-rs" ? portugolRs() : portugol()),
    setDebugLine.of(tab.id === runTabId ? paused?.position.line ?? null : null),
  ] });
  renderTabs();
  renderMessages();
  outline.show(tab.symbols, current !== tab);
  updateLibraryButtons(tab.symbols);
  renderCursor();
  if (centerActive) centerSelection();
  if (learningMode === "bridge") scheduleRustExport();
  syncLanguageControls(tab);
  view.focus();
}

function syncLanguageControls(tab: Tab) {
  const pors = tab.language === "portugol-rs";
  const reason = "Disponível quando o Portugol-RS tiver execução e depuração integradas.";
  for (const id of ["btn-run", "btn-debug", "btn-debug-pause", "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out"]) {
    const button = $<HTMLButtonElement>(id);
    button.disabled = pors || (id === "btn-debug-pause"
      ? runId === null || paused !== null
      : id === "btn-run" || id === "btn-debug"
        ? (runId !== null || pendingEvents !== null) && paused === null
        : paused === null);
    button.dataset.defaultTitle ??= button.title;
    if (pors) button.title = reason;
    else button.title = button.dataset.defaultTitle;
    button.setAttribute("aria-description", pors ? reason : "");
  }
  const format = $<HTMLButtonElement>("editor-format");
  format.disabled = pors;
  format.title = pors ? "Formatador clássico indisponível para Portugol-RS." : "Formatar código-fonte";
  for (const [name, button] of libraryButtons) {
    button.disabled = pors || tab.symbols.some((symbol) => symbol.kind === "library" && symbol.name.split(" → ", 1)[0] === name);
    button.title = pors ? "Bibliotecas clássicas não se aplicam ao Portugol-RS." : `Incluir biblioteca ${name}`;
  }
  for (const id of ["editor-definition", "editor-rename"]) {
    const button = $<HTMLButtonElement>(id);
    button.disabled = pors;
    button.setAttribute("aria-description", pors ? "Navegação e renomeação Portugol-RS ainda não estão implementadas." : "");
  }
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

const FILTERS = [{ name: "Programas Portugol", extensions: ["por", "pors"] }];

let workspaceError: string | null = null;

function setWorkspaceExplorerVisible(visible: boolean) {
  $("workspace-explorer").hidden = !visible;
  $("btn-explorer").setAttribute("aria-expanded", String(visible));
}

function renderWorkspaceExplorer() {
  $("workspace-explorer").setAttribute("aria-busy", String(workspaceLoading));
  const tree = $<HTMLUListElement>("workspace-tree");
  const message = $("workspace-tree-state");
  const name = $("workspace-name");
  const openButtons = [$<HTMLButtonElement>("home-open-folder"), $<HTMLButtonElement>("workspace-open-folder")];
  openButtons.forEach((button) => { button.disabled = workspaceLoading; });
  $<HTMLButtonElement>("workspace-new-file").disabled = !workspaceState || workspaceLoading;
  name.hidden = !workspaceState;
  name.textContent = workspaceState?.name ?? "";

  const rows = flattenWorkspaceEntries(workspaceState?.entries ?? [], expandedWorkspaceFolders);
  tree.setAttribute("aria-label", workspaceState ? `Arquivos em ${workspaceState.name}` : "Arquivos do projeto");
  const items = rows.map(({ entry, depth, position, setSize }) => {
    const item = document.createElement("li");
    item.setAttribute("role", "none");
    const button = document.createElement("button");
    const isFolder = entry.kind === "folder";
    const isExpanded = isFolder && expandedWorkspaceFolders.has(entry.path);
    const marker = document.createElement("span");
    marker.className = "workspace-tree-marker";
    marker.setAttribute("aria-hidden", "true");
    marker.textContent = isFolder ? (isExpanded ? "▾" : "▸") : workspaceFileKind(entry.path) === "portugol-rs" ? "RS" : workspaceFileKind(entry.path) === "exercise" ? "EX" : "POR";
    const label = document.createElement("span");
    label.className = "workspace-tree-label";
    label.textContent = entry.name;
    button.type = "button";
    button.className = `workspace-tree-item${isFolder ? " is-folder" : " is-file"}`;
    button.dataset.path = entry.path;
    button.dataset.depth = String(depth);
    button.setAttribute("role", "treeitem");
    button.setAttribute("aria-level", String(depth + 1));
    button.setAttribute("aria-posinset", String(position));
    button.setAttribute("aria-setsize", String(setSize));
    button.setAttribute("aria-selected", String(!isFolder && selectedWorkspacePath === entry.path));
    button.style.paddingInlineStart = `${8 + depth * 16}px`;
    if (isFolder) {
      button.setAttribute("aria-expanded", String(isExpanded));
      button.setAttribute("aria-label", `${entry.name}, pasta${isExpanded ? ", expandida" : ", recolhida"}`);
      button.onclick = () => {
        if (expandedWorkspaceFolders.has(entry.path)) expandedWorkspaceFolders.delete(entry.path);
        else expandedWorkspaceFolders.add(entry.path);
        renderWorkspaceExplorer();
        tree.querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(entry.path)}"]`)?.focus();
      };
    } else {
      button.setAttribute("aria-label", `${entry.name}, arquivo`);
      button.onclick = () => {
        selectedWorkspacePath = entry.path;
        renderWorkspaceExplorer();
        void openWorkspaceFile(entry).catch((error) => {
          workspaceError = String(error);
          renderWorkspaceExplorer();
          setStatus(String(error), "error");
        });
      };
    }
    button.append(marker, label);
    item.append(button);
    return item;
  });
  tree.replaceChildren(...items);
  tree.hidden = rows.length === 0;

  if (workspaceLoading) {
    message.textContent = "Abrindo pasta…";
    message.hidden = false;
  } else if (workspaceError) {
    message.textContent = workspaceError;
    message.hidden = false;
  } else if (!workspaceState) {
    message.textContent = "Abra uma pasta para navegar pelos arquivos do projeto.";
    message.hidden = false;
  } else if (rows.length === 0) {
    message.textContent = "Esta pasta não contém arquivos .por, .pors ou .pex.";
    message.hidden = false;
  } else {
    message.textContent = "";
    message.hidden = true;
  }
}

async function openWorkspaceFolder() {
  if (workspaceLoading) return;
  const request = ++workspaceRequest;
  workspaceLoading = true;
  workspaceError = null;
  renderWorkspaceExplorer();
  try {
    const selected = await api.openWorkspace();
    if (request !== workspaceRequest || !selected) return;
    workspaceState = selected;
    selectedWorkspacePath = null;
    expandedWorkspaceFolders.clear();
    workspaceError = null;
    setWorkspaceExplorerVisible(true);
    if (!activeTab()) {
      const previousTab = tabs[tabs.length - 1];
      if (previousTab) activate(previousTab.id);
      else newTab();
    }
    renderWorkspaceExplorer();
    setStatus(`Pasta aberta: ${selected.name}`, "ok");
  } catch (error) {
    if (request === workspaceRequest) {
      workspaceError = String(error);
      renderWorkspaceExplorer();
      setStatus(String(error), "error");
    }
  } finally {
    if (request === workspaceRequest) {
      workspaceLoading = false;
      renderWorkspaceExplorer();
    }
  }
}

async function openWorkspaceFile(entry: Extract<WorkspaceEntry, { kind: "file" }>) {
  const selected = workspaceState;
  if (!selected) return;
  const kind = workspaceFileKind(entry.path);
  if (!kind) return;
  if (kind === "exercise") {
    const existingExercise = tabs.find((tab) =>
      tab.workspaceId === selected.id && tab.workspaceExercisePath === entry.path,
    );
    if (existingExercise) {
      activate(existingExercise.id);
      return;
    }
    const loaded = await api.readWorkspaceFile(selected.id, entry.path);
    if (workspaceState?.id !== selected.id) return;
    const exercise = await api.inspectExercise(loaded.source);
    if (workspaceState?.id !== selected.id) return;
    openExercise({ exercise, name: loaded.path.split("/").pop() ?? entry.name, contents: loaded.source }, {
      id: selected.id,
      path: loaded.relativePath,
      exercise: true,
    });
    setStatus(`Exercício aberto: ${entry.path}`, "ok");
    return;
  }

  const existing = tabs.find((tab) => tab.workspaceId === selected.id && tab.workspacePath === entry.path);
  if (existing) {
    activate(existing.id);
    return;
  }
  const loaded = await api.readWorkspaceFile(selected.id, entry.path);
  if (workspaceState?.id !== selected.id) return;
  const current = activeTab();
  if (current && !current.path && !current.exampleName && !isDirty(current) && tabs.length === 1) tabs = [];
  const tab = newTab(loaded.source, loaded.path, undefined, undefined, undefined, undefined, {
    id: selected.id,
    path: loaded.relativePath,
  });
  tab.workspacePath = loaded.relativePath;
  rememberFile(loaded.path);
  setStatus(`Aberto: ${entry.path}`, "ok");
}

function showWorkspaceCreateError(message: string | null) {
  const error = $("workspace-create-error");
  error.textContent = message ?? "";
  error.hidden = !message;
  $("workspace-create-name").setAttribute("aria-invalid", String(Boolean(message)));
}

function openWorkspaceCreateDialog() {
  if (!workspaceState) return;
  const dialog = $<HTMLDialogElement>("workspace-create-dialog");
  showWorkspaceCreateError(null);
  $<HTMLFormElement>("workspace-create-form").reset();
  dialog.showModal();
  $("workspace-create-name").focus();
}

async function submitWorkspaceCreateDialog(event: SubmitEvent) {
  event.preventDefault();
  const selected = workspaceState;
  if (!selected) return;

  const name = $<HTMLInputElement>("workspace-create-name");
  const language = $<HTMLSelectElement>("workspace-create-language").value as "por" | "pors";
  let fileName: string;
  try {
    fileName = workspaceSourceFileName(name.value, language);
  } catch (error) {
    showWorkspaceCreateError(String(error).replace(/^Error: /, ""));
    name.focus();
    return;
  }

  const submit = $<HTMLButtonElement>("workspace-create-submit");
  submit.disabled = true;
  showWorkspaceCreateError(null);
  try {
    const refreshed = await api.createWorkspaceFile(selected.id, fileName);
    if (workspaceState?.id !== selected.id) return;
    workspaceState = refreshed;
    workspaceError = null;
    selectedWorkspacePath = fileName;
    setWorkspaceExplorerVisible(true);
    renderWorkspaceExplorer();
    $<HTMLDialogElement>("workspace-create-dialog").close();
    $<HTMLFormElement>("workspace-create-form").reset();
    const created = refreshed.entries.find((entry): entry is Extract<WorkspaceEntry, { kind: "file" }> =>
      entry.kind === "file" && entry.path === fileName,
    );
    if (created) await openWorkspaceFile(created);
    else setStatus(`Arquivo criado: ${fileName}`, "ok");
  } catch (error) {
    const message = String(error).replace(/^Error: /, "");
    if ($<HTMLDialogElement>("workspace-create-dialog").open) {
      showWorkspaceCreateError(message);
      name.focus();
    } else {
      workspaceError = message;
      renderWorkspaceExplorer();
      setStatus(message, "error");
    }
  } finally {
    submit.disabled = false;
  }
}

$("home-open-folder").onclick = () => void openWorkspaceFolder();
$("workspace-open-folder").onclick = () => void openWorkspaceFolder();
$("workspace-new-file").onclick = openWorkspaceCreateDialog;
$("workspace-create-form").addEventListener("submit", (event) => void submitWorkspaceCreateDialog(event as SubmitEvent));
$("workspace-create-cancel").onclick = () => {
  $<HTMLDialogElement>("workspace-create-dialog").close();
  $<HTMLFormElement>("workspace-create-form").reset();
  showWorkspaceCreateError(null);
};
$("btn-explorer").onclick = () => {
  if (!workspaceState) {
    void openWorkspaceFolder();
    return;
  }
  setWorkspaceExplorerVisible(Boolean($("workspace-explorer").hidden));
};

$("workspace-tree").addEventListener("keydown", (event) => {
  if (!(event instanceof KeyboardEvent)) return;
  const current = event.target instanceof HTMLButtonElement ? event.target : null;
  if (!current) return;
  const items = [...$("workspace-tree").querySelectorAll<HTMLButtonElement>("[role=treeitem]")];
  const index = items.indexOf(current);
  const focus = (item?: HTMLButtonElement) => {
    if (item) {
      event.preventDefault();
      item.focus();
    }
  };
  if (event.key === "ArrowDown") focus(items[index + 1]);
  else if (event.key === "ArrowUp") focus(items[index - 1]);
  else if (event.key === "Home") focus(items[0]);
  else if (event.key === "End") focus(items[items.length - 1]);
  else if (event.key === "ArrowRight" && current.classList.contains("is-folder")) {
    const path = current.dataset.path!;
    if (!expandedWorkspaceFolders.has(path)) {
      event.preventDefault();
      expandedWorkspaceFolders.add(path);
      renderWorkspaceExplorer();
      $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(path)}"]`)?.focus();
    } else if (items[index + 1]?.dataset.depth === String(Number(current.dataset.depth) + 1)) {
      focus(items[index + 1]);
    }
  } else if (event.key === "ArrowLeft") {
    const path = current.dataset.path!;
    if (current.classList.contains("is-folder") && expandedWorkspaceFolders.has(path)) {
      event.preventDefault();
      expandedWorkspaceFolders.delete(path);
      renderWorkspaceExplorer();
      $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(path)}"]`)?.focus();
    } else {
      const depth = Number(current.dataset.depth);
      focus(items.slice(0, index).reverse().find((item) => Number(item.dataset.depth) === depth - 1));
    }
  }
});

renderWorkspaceExplorer();

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
  let workspaceFile = Boolean(tab.workspaceId && tab.workspacePath);
  if (!path || as) {
    const extension = tab.language === "portugol-rs" ? "pors" : "por";
    const exerciseName = tab.exercise ? tab.exercise.name.replace(/\.pex$/i, "") + ".por" : undefined;
    path = await saveDialog({ filters: FILTERS, defaultPath: tab.path ?? exerciseName ?? tab.exampleName ?? `programa.${extension}` });
    if (!path) return false;
    path = ensureSourceExtension(path, tab.language);
    workspaceFile = false;
  }
  const text = view.state.doc.toString();
  try {
    if (workspaceFile && tab.workspaceId && tab.workspacePath) {
      await api.writeWorkspaceFile(tab.workspaceId, tab.workspacePath, text);
    } else {
      await api.writeFile(path, text);
    }
  } catch (error) {
    setStatus(String(error), "error");
    return false;
  }
  tab.path = path;
  if (!workspaceFile) {
    tab.workspaceId = undefined;
    tab.workspacePath = undefined;
    tab.workspaceExercisePath = undefined;
  }
  tab.language = detectSourceLanguage(path);
  if (tab.id === activeId) view.dispatch({ effects: sourceLanguage.reconfigure(tab.language === "portugol-rs" ? portugolRs() : portugol()) });
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
  const pors = activeTab()?.language === "portugol-rs";
  $<HTMLButtonElement>("btn-run").disabled = pors || (running && paused === null);
  $<HTMLButtonElement>("btn-debug").disabled = pors || (running && paused === null);
  $<HTMLButtonElement>("btn-stop").disabled = !running;
  $<HTMLButtonElement>("btn-debug-pause").disabled = pors || !running || paused !== null;
  for (const id of [
    "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out",
  ]) {
    $<HTMLButtonElement>(id).disabled = pors || paused === null;
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
  if (activeTab()?.language === "portugol-rs") {
    setStatus("O Portugol-RS ainda não possui execução e depuração integradas.");
    return;
  }
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
  if (tab.language === "portugol-rs") {
    setStatus("O Portugol-RS ainda não possui execução e depuração integradas.");
    return;
  }
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

const ownershipStateLabels: Record<OwnershipBindingState, string> = {
  owner: "Dono",
  "shared-reference": "Empréstimo compartilhado (&T)",
  "mutable-reference": "Empréstimo mutável exclusivo (&mut T)",
  moved: "Movido; valor indisponível",
  expired: "Empréstimo encerrado",
  dropped: "Liberado ao sair do escopo",
};

function renderOwnershipLab() {
  const scenario = getOwnershipScenario(ownershipScenarioId);
  const snapshot = getOwnershipSnapshot(ownershipScenarioId, ownershipStepIndex);
  if (!scenario || !snapshot) return;

  const picker = $<HTMLSelectElement>("ownership-scenario");
  picker.replaceChildren(...ownershipScenarioIds().map((id) => {
    const option = document.createElement("option");
    const item = getOwnershipScenario(id);
    option.value = id;
    option.textContent = item?.title ?? id;
    return option;
  }));
  picker.value = scenario.id;

  const code = $<HTMLElement>("ownership-source");
  code.replaceChildren(...scenario.source.split("\n").map((text, index) => {
    const line = document.createElement("span");
    line.className = "ownership-code-line";
    line.dataset.line = String(index + 1);
    if (index + 1 === snapshot.currentLine) {
      line.dataset.current = "true";
      line.setAttribute("aria-current", "location");
    }
    line.textContent = text || " ";
    return line;
  }));

  const bindings = $<HTMLUListElement>("ownership-bindings");
  bindings.replaceChildren(...snapshot.bindings.map((binding) => {
    const item = document.createElement("li");
    item.className = "ownership-binding";
    item.dataset.state = binding.state;

    const heading = document.createElement("div");
    heading.className = "ownership-binding-heading";
    const name = document.createElement("code");
    name.textContent = binding.name;
    const type = document.createElement("span");
    type.textContent = binding.type;
    const state = document.createElement("span");
    state.className = "ownership-state";
    state.textContent = ownershipStateLabels[binding.state];
    heading.append(name, type, state);

    const description = document.createElement("p");
    description.textContent = binding.description;
    item.append(heading, description);
    if (binding.target) {
      const relation = document.createElement("p");
      relation.className = "ownership-relation";
      relation.textContent = binding.state === "owner"
        ? `Controla: ${binding.target}`
        : `Empresta de: ${binding.target}`;
      item.append(relation);
    }
    if (binding.value !== undefined) {
      const value = document.createElement("p");
      value.className = "ownership-value";
      value.textContent = `Valor: ${binding.value}`;
      item.append(value);
    }
    return item;
  }));

  const heap = $<HTMLUListElement>("ownership-heap");
  heap.replaceChildren(...snapshot.heap.map((block) => {
    const item = document.createElement("li");
    item.className = "ownership-heap-block";
    item.dataset.state = block.state;
    const title = document.createElement("strong");
    title.textContent = block.id;
    const value = document.createElement("p");
    value.textContent = block.state === "allocated" ? `Conteúdo UTF-8: ${block.value}` : `Conteúdo liberado: ${block.value}`;
    const allocation = document.createElement("p");
    allocation.textContent = block.state === "allocated"
      ? `Alocação ativa; owner: ${block.owner ?? "não identificado"}.`
      : "Alocação liberada quando o owner saiu do escopo.";
    item.append(title, value, allocation);
    return item;
  }));

  $("ownership-step-title").textContent = snapshot.title;
  $("ownership-explanation").textContent = snapshot.explanation;
  $("ownership-step-status").textContent = `Passo ${ownershipStepIndex + 1} de ${scenario.snapshots.length}: ${snapshot.title}`;
  $("ownership-provenance").textContent = snapshot.provenance === "verified-simulation"
    ? "Simulação didática — não é execução nem trace de memória Rust."
    : "Proveniência indisponível.";
  const diagnostic = $<HTMLElement>("ownership-diagnostic");
  diagnostic.hidden = !snapshot.diagnostic;
  diagnostic.textContent = snapshot.diagnostic ?? "";

  $<HTMLButtonElement>("ownership-previous").disabled = ownershipStepIndex === 0;
  $<HTMLButtonElement>("ownership-next").disabled = ownershipStepIndex >= scenario.snapshots.length - 1;
  $<HTMLButtonElement>("ownership-restart").disabled = ownershipStepIndex === 0;
}

function renderLearningPanel() {
  const concepts = conceptsForMode(learningMode);
  const concept = resolveConceptForMode(learningMode, selectedLearningConceptId);
  selectedLearningConceptId = concept.id;

  const picker = $<HTMLSelectElement>("learning-concept");
  picker.replaceChildren(...concepts.map((item) => {
    const option = document.createElement("option");
    option.value = item.id;
    option.textContent = item.title;
    return option;
  }));
  picker.value = concept.id;
  $("learning-mode-description").textContent = learningMode === "bridge"
    ? "Ponte didática: compare os conceitos de Portugol com exemplos Rust editoriais."
    : "Fundamentos de lógica e programação em Portugol.";
  $("learning-title").textContent = concept.title;
  $("learning-definition").textContent = concept.definition;
  $("learning-provenance").textContent = "Conceito selecionado manualmente";
  $("learning-portugol-caption").textContent = concept.portugolExample.caption;
  $("learning-portugol-code").textContent = concept.portugolExample.code;

  const rustSection = $<HTMLElement>("learning-rust-example");
  const showRustExample = learningMode === "bridge" && concept.rustExample !== undefined;
  rustSection.hidden = !showRustExample;
  if (showRustExample && concept.rustExample) {
    $("learning-rust-caption").textContent = concept.rustExample.caption;
    $("learning-rust-code").textContent = concept.rustExample.code;
  }

  const ownershipLab = $<HTMLElement>("ownership-lab");
  const showOwnershipLab = learningMode === "bridge" && concept.id === "ownership";
  ownershipLab.hidden = !showOwnershipLab;
  if (showOwnershipLab) renderOwnershipLab();
}

function showPanel(which: PanelName) {
  for (const name of ["console", "messages", "debug", "learning"] as const) {
    $(`tab-${name}`).classList.toggle("active", which === name);
    $(`tab-${name}`).setAttribute("aria-selected", String(which === name));
    $(panelContentId(name)).hidden = which !== name;
  }
  $("console-tools").hidden = which !== "console";
  if (which === "learning") renderLearningPanel();
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
for (const name of LIBRARIES) {
  const button = document.createElement("button");
  button.textContent = name;
  button.title = `Incluir biblioteca ${name}`;
  button.onclick = async () => {
    const tab = activeTab();
    if (!tab || (tab.id === runTabId && runId !== null)) return;
    if (tab.language === "portugol-rs") {
      setStatus("Bibliotecas clássicas ainda não estão disponíveis no Portugol-RS.");
      return;
    }
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
  const pors = activeTab()?.language === "portugol-rs";
  for (const [name, button] of libraryButtons) button.disabled = pors || included.has(name);
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
const newPorsButton = document.createElement("button");
newPorsButton.id = "home-new-pors";
newPorsButton.title = "Novo arquivo Portugol-RS (.pors)";
newPorsButton.textContent = "Novo Portugol-RS";
newPorsButton.onclick = () => newTab(PORS_TEMPLATE, null, undefined, undefined, undefined, "portugol-rs");
$("home-new").parentElement!.append(newPorsButton);
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
  if (tab.language === "portugol-rs") {
    setStatus("Renomeação semântica ainda não está disponível no Portugol-RS.");
    return;
  }
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

async function goToDeclarationAt(offset: number, quiet = false) {
  const tab = activeTab();
  if (!tab) return;
  if (tab.language === "portugol-rs") {
    if (!quiet) setStatus("Navegação semântica ainda não está disponível no Portugol-RS.");
    return;
  }
  const doc = view.state.doc;
  const request = ++definitionNavigationGeneration;
  try {
    const location = await api.resolveSymbol(doc.toString(), offset);
    if (request !== definitionNavigationGeneration || activeTab() !== tab || view.state.doc !== doc) return;
    view.dispatch({
      selection: { anchor: location.start.offset, head: location.end.offset },
      effects: EditorView.scrollIntoView(location.start.offset, { y: "center" }),
    });
    view.focus();
  } catch (error) {
    if (!quiet && request === definitionNavigationGeneration && activeTab() === tab && view.state.doc === doc) {
      setStatus(typeof error === "string" ? error : "Não foi possível ir à declaração.", "error");
    }
  }
}

function renameSelection() {
  return renameAt(view.state.selection.main.from);
}

function goToDeclarationSelection() {
  return goToDeclarationAt(view.state.selection.main.head);
}

async function formatCode() {
  const tab = activeTab();
  if (!tab || (runTabId === tab.id && runId !== null)) return;
  if (tab.language === "portugol-rs") {
    setStatus("O formatador clássico não atende à sintaxe Portugol-RS.");
    return;
  }
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
function openExercise(exerciseDocument: ExerciseDocument, workspaceExercise?: { id: string; path: string; exercise: true }) {
  if (workspaceExercise) {
    const existing = tabs.find((tab) =>
      tab.workspaceId === workspaceExercise.id && tab.workspaceExercisePath === workspaceExercise.path,
    );
    if (existing) {
      activate(existing.id);
      return;
    }
  }
  const source = `/*
${exerciseDocument.exercise.statement.replaceAll("*/", "* /")}
*/

${TEMPLATE}`;
  const current = activeTab();
  if (current && !current.path && !current.exampleName && !isDirty(current) && tabs.length === 1) tabs = [];
  const tab = newTab(source, null, exerciseDocument.name, undefined, exerciseDocument);
  if (workspaceExercise) {
    tab.workspaceId = workspaceExercise.id;
    tab.workspaceExercisePath = workspaceExercise.path;
    selectedWorkspacePath = workspaceExercise.path;
    renderWorkspaceExplorer();
  }
}
const exercises = mountExercises(openExercise);
$("home-exercises").onclick = exercises.browse;
editorMenuAction("editor-exercise", () => {
  const exercise = activeTab()?.exercise;
  if (exercise) void exercises.grade(exercise, view.state.doc.toString());
});
editorMenuAction("editor-search", () => openSearchPanel(view));
editorMenuAction("editor-definition", goToDeclarationSelection);
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
function applyLearningMode(mode: LearningMode, persist = true) {
  learningMode = mode;
  const selector = $<HTMLSelectElement>("learning-mode");
  selector.value = mode;
  if (persist) saveLearningMode(mode);
  if (!$<HTMLElement>("learning-panel").hidden) renderLearningPanel();

  const preview = $<HTMLElement>("rust-preview");
  const bridgeEnabled = mode === "bridge";
  preview.hidden = !bridgeEnabled;
  $<HTMLButtonElement>("btn-rust").setAttribute("aria-pressed", String(bridgeEnabled));
  if (bridgeEnabled) {
    if (activeTab()) scheduleRustExport();
  } else {
    rustExportGeneration++;
    if (rustExportTimer !== undefined) window.clearTimeout(rustExportTimer);
    rustExportTimer = undefined;
  }
}

$<HTMLSelectElement>("learning-mode").onchange = (event) => {
  const selected = (event.currentTarget as HTMLSelectElement).value;
  if (selected === "beginner" || selected === "bridge") applyLearningMode(selected);
};
applyLearningMode(learningMode, false);
$<HTMLButtonElement>("btn-rust").onclick = () => {
  applyLearningMode(learningMode === "bridge" ? "beginner" : "bridge");
};
$<HTMLButtonElement>("rust-close").onclick = () => {
  applyLearningMode("beginner");
  view.focus();
};
$<HTMLButtonElement>("rust-copy").onclick = async () => {
  try {
    await navigator.clipboard.writeText($<HTMLElement>("rust-output").textContent ?? "");
    setStatus("Código Rust copiado.", "ok");
  } catch (error) {
    setStatus(String(error), "error");
  }
};
$<HTMLButtonElement>("rust-save").onclick = async () => {
  const code = $<HTMLElement>("rust-output").textContent ?? "";
  try {
    const path = await saveDialog({ defaultPath: "programa.rs", filters: [{ name: "Rust", extensions: ["rs"] }] });
    if (!path) return;
    await api.writeFile(path, code);
    setStatus("Rust salvo em " + path + ".", "ok");
  } catch (error) {
    setStatus(String(error), "error");
  }
};
$("btn-debug").onclick = () => void stepOrStart();
$("btn-stop").onclick = () => void stopProgram();
$("btn-debug-pause").onclick = () => {
  if (activeTab()?.language === "portugol-rs") return;
  if (runId !== null) void api.debugPause(runId);
};
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
$("tab-learning").onclick = () => showPanel("learning");
$<HTMLSelectElement>("learning-concept").onchange = (event) => {
  selectedLearningConceptId = (event.currentTarget as HTMLSelectElement).value;
  renderLearningPanel();
};
$<HTMLSelectElement>("ownership-scenario").onchange = (event) => {
  const selected = (event.currentTarget as HTMLSelectElement).value;
  if (ownershipScenarioIds().includes(selected as OwnershipScenarioId)) ownershipScenarioId = selected as OwnershipScenarioId;
  ownershipStepIndex = 0;
  renderOwnershipLab();
};
$<HTMLButtonElement>("ownership-previous").onclick = () => {
  ownershipStepIndex = Math.max(0, ownershipStepIndex - 1);
  renderOwnershipLab();
};
$<HTMLButtonElement>("ownership-next").onclick = () => {
  const scenario = getOwnershipScenario(ownershipScenarioId);
  ownershipStepIndex = Math.min(scenario ? scenario.snapshots.length - 1 : 0, ownershipStepIndex + 1);
  renderOwnershipLab();
};
$<HTMLButtonElement>("ownership-restart").onclick = () => {
  ownershipStepIndex = 0;
  renderOwnershipLab();
};

function scheduleRustExport() {
  const generation = ++rustExportGeneration;
  if (rustExportTimer !== undefined) window.clearTimeout(rustExportTimer);
  const status = $<HTMLElement>("rust-status");
  status.textContent = "Atualizando tradução…";
  delete status.dataset.error;
  $<HTMLButtonElement>("rust-copy").disabled = true;
  $<HTMLButtonElement>("rust-save").disabled = true;
  $<HTMLElement>("rust-output").textContent = "";
  rustExportTimer = window.setTimeout(async () => {
    let result: api.RustExport;
    try {
      const tab = activeTab();
      result = tab?.language === "portugol-rs"
        ? await api.exportPortugolRs(view.state.doc.toString())
        : await api.exportRust(view.state.doc.toString());
    } catch (error) {
      if (generation !== rustExportGeneration) return;
      status.textContent = "Falha ao solicitar a tradução: " + String(error);
      status.dataset.error = "true";
      $<HTMLButtonElement>("rust-copy").disabled = true;
      $<HTMLButtonElement>("rust-save").disabled = true;
      $<HTMLElement>("rust-output").textContent = "";
      return;
    }
    if (generation !== rustExportGeneration) return;
    const output = $<HTMLElement>("rust-output");
    const copy = $<HTMLButtonElement>("rust-copy");
    const save = $<HTMLButtonElement>("rust-save");
    if (result.code !== null) {
      output.textContent = result.code;
      status.textContent = "Tradução disponível para copiar ou salvar.";
      copy.disabled = save.disabled = false;
      delete status.dataset.error;
    } else {
      output.textContent = "";
      const diagnostic = result.diagnostic;
      status.textContent = diagnostic
        ? "Linha " + diagnostic.start.line + ", coluna " + diagnostic.start.column + ": " + diagnostic.message
        : "Não foi possível gerar Rust para este programa.";
      copy.disabled = save.disabled = true;
      status.dataset.error = "true";
    }
  }, 220);
}

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
  { label: "F12", command: "Sobre (fora do editor); ir à declaração (no editor)", matches: (e) => e.key === "F12", action: () =>
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
fileShortcut("E", true, "Mostrar ou ocultar o explorador", () => {
  if (!workspaceState) void openWorkspaceFolder();
  else setWorkspaceExplorerVisible(Boolean($("workspace-explorer").hidden));
});
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


async function openIncomingFiles(paths: string[], warnUnsupported = false) {
  let supported = false;
  for (const path of paths) {
    try {
      if (/\.pex$/i.test(path)) {
        supported = true;
        openExercise(await readExercise(path));
      } else if (/\.por$/i.test(path)) {
        supported = true;
        await openFile(path);
      } else if (/\.pors$/i.test(path)) {
        supported = true;
        await openFile(path);
      }
    } catch (error) {
      setStatus(typeof error === "string" ? error : "Não foi possível abrir o arquivo.", "error");
    }
  }
  if (warnUnsupported && !supported && paths.length) setStatus("Escolha um arquivo .por, .pors ou .pex.", "error");
}

async function consumeSystemFiles() {
  try {
    await openIncomingFiles(await api.pendingFiles());
  } catch {
    setStatus("Não foi possível receber arquivos do sistema.", "error");
  }
}

void api.onSystemFiles(() => { void consumeSystemFiles(); }).then(consumeSystemFiles, consumeSystemFiles);
void getCurrentWindow().onDragDropEvent(({ payload }) => {
  if (payload.type === "drop") void openIncomingFiles(payload.paths, true);
}).catch(() => {});
