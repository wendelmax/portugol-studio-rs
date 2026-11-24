import { mountTips } from "./tips";
import { mountThemeEditor } from "./themes";
// IDE: abas de arquivos, editor CodeMirror, console e execução.
//
// Atalhos seguem o Portugol Studio original (TelaAtalhos.java):
// Shift+F6 executa, Shift+F7 interrompe, Ctrl+Q fecha a aba,
// Alt+←/→ troca de aba, Shift+Esc expande o editor.

import "./style.css";
import license from "../../../LICENSE?raw";
import { version } from "../package.json";

import { EditorState, Compartment, type Extension } from "@codemirror/state";
import {
  EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter,
  drawSelection, highlightSpecialChars,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab, isolateHistory } from "@codemirror/commands";
import {
  bracketMatching, indentOnInput, foldGutter, foldKeymap,
} from "@codemirror/language";
import { forceLinting, linter, lintGutter, setDiagnostics } from "@codemirror/lint";
import { openSearchPanel, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { rust } from "@codemirror/lang-rust";
import { open as openDialog, save as saveDialog, ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { renderDiagnostics } from "./diagnostics";
import { sourceDiagnostic } from "./source-map-ui";
import { mountExamples } from "./examples";
import { mountHomeNewFileMenu } from "./home-new-file-menu";
import { mountExercises, readExercise, type ExerciseDocument } from "./exercises";
import {
  cargoTestPanelState,
  cargoTestStatus,
  exerciseTestPanelState,
  exerciseTestStatus,
  isCurrentExerciseTestRequest,
  type ExerciseTestRequest,
} from "./exercise-test-panel";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline, symbolKindLabels } from "./outline";
import { panelContentId, panelNameForKey, type PanelName } from "./panel-tabs";
import { LIBRARIES, portugol, portugolRs, SNIPPETS } from "./portugol";
import { detectSourceLanguage, ensureSourceExtension, type SourceLanguage } from "./document-language";
import { semanticSymbolLanguage } from "./semantic-symbol-actions";
import { canDebugSource, canFormatSource, canRunAsRust, canRunSource } from "./runtime-capabilities";
import { parseClassicArguments, parseProcessArguments } from "./process-arguments";
import { hasCargoExecutionConsent, hasRustExecutionConsent, rememberCargoExecutionConsent, rememberRustExecutionConsent } from "./rust-execution-consent";
import { createLatestOnlyRustSourceChecker } from "./rust-source-check";
import { replayRunEvents, routeRunEvent as routeRunEventById, type PendingRunEvent } from "./run-event-router";
import { loadLearningMode, saveLearningMode, type LearningMode } from "./learning-mode";
import { isPaletteCommandAvailable, type PaletteContext } from "./command-palette";
import { markRustPreviewStale } from "./rust-preview-state";
import { PORS_UI_EXAMPLES } from "./pors-ui-examples";
import { mountPorsUiPreviewPanel } from "./pors-ui-preview-panel";
import { mountUiPreview } from "./ui-preview-renderer";
import * as api from "./api";
import type { Tab } from "./state/tab";
import { mountWorkspaceExplorer } from "./workspace/explorer";
import { mountRustPreview } from "./rust/preview";
import { mountLearningPanel } from "./learning/panel";
import { mountCommandPalette } from "./commands/palette";
import { ctrlShortcut, renderShortcutList, type Shortcut } from "./commands/shortcuts";
import { $ } from "./dom";
import { createRecentFiles } from "./files/recent";
import { mountPaneLayout } from "./layout/panes";
import { setStatus } from "./shell/status";
import { mountFontSizeControls } from "./shell/font-size";
import { appendConsole, appendProcessOutput, clearConsole, consoleText } from "./run/console";
import { renderDebugInfo } from "./run/debug-view";
import { chooseCargoTarget } from "./run/cargo-target-dialog";
import { applyPageTheme, editorColors, editorTheme, onSystemThemeChange, themeChoice } from "./editor/theme";
import {
  breakpointLines, breakpoints, createBreakpointGutter, debugLine, runtimeError, setDebugLine, setRuntimeError, toCodeMirror,
} from "./editor/decorations";

const TEMPLATE = "programa\n{\n\tfuncao inicio()\n\t{\n\t\t\n\t}\n}\n";

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------


// Compartimentos reconfigurados ao trocar de aba, linguagem ou tema.
const theme = new Compartment();
const sourceLanguage = new Compartment();
const portugolTools = new Compartment();
const generatedRustTheme = new Compartment();

const centerKey = "portugol.centerSource";
let centerActive = false;
try {
  centerActive = localStorage.getItem(centerKey) === "true";
} catch {
  // A opção também funciona sem armazenamento.
}

const breakpointGutter = createBreakpointGutter((state) => syncBreakpoints(state), () => icon("pequeno/bug"));

const portugolLinter = linter(
  async (view) => {
    const doc = view.state.doc;
    const tab = activeTab();
    if (tab?.language === "rust") {
      tab.diagnostics = [];
      tab.symbols = [];
      renderMessages();
      outline.show([]);
      updateLibraryButtons([]);
      return [];
    }
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
    return [...diagnostics, ...(tab?.runtimeCompileDiagnostics ?? [])].map((d) => toCodeMirror(d, doc.length));
  },
  { delay: 350 },
);

const latestRustSourceCheck = createLatestOnlyRustSourceChecker(api.checkRustSource);
const rustSourceLinter = linter(
  async (editor) => {
    const doc = editor.state.doc;
    const tab = activeTab();
    if (tab?.language !== "rust" || tab.cargoProject === true || rustcAvailable !== true) {
      if (tab?.language === "rust") {
        tab.diagnostics = [];
        renderMessages();
      }
      return [];
    }

    const checked = await latestRustSourceCheck(doc.toString());
    if (activeTab() !== tab || editor.state.doc !== doc || checked.source !== doc.toString()) return [];
    const diagnostics: api.DiagnosticInfo[] = checked.error
      ? [{
        severity: "warning",
        code: "rustc.check",
        message: `Não foi possível analisar este arquivo Rust: ${checked.error}`,
        help: null,
        start: { offset: 0, line: 1, column: 1 },
        end: { offset: 0, line: 1, column: 1 },
      }]
      : checked.diagnostics.flatMap((diagnostic) => {
        if (diagnostic.severity !== "error" && diagnostic.severity !== "warning") return [];
        return [{
          severity: diagnostic.severity,
          code: diagnostic.code ?? "rustc",
          message: diagnostic.message,
          help: diagnostic.rendered,
          start: diagnostic.start,
          end: diagnostic.end,
        }];
      });
    tab.diagnostics = diagnostics;
    renderMessages();
    return [...diagnostics, ...(tab.runtimeCompileDiagnostics ?? [])]
      .map((diagnostic) => toCodeMirror(diagnostic, doc.length));
  },
  { delay: 500 },
);

function extensions(language: SourceLanguage): Extension[] {
  const isRust = language === "rust";
  const isPortugol = !isRust;
  const tools: Extension[] = isPortugol
    ? [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]
    : [rustSourceLinter];
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
    sourceLanguage.of(isRust ? rust() : language === "portugol-rs" ? portugolRs() : portugol()),
    portugolTools.of(tools),
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
    EditorView.domEventHandlers({
      click(event, editor) {
        const mouse = event as MouseEvent;
        if (activeTab()?.language === "rust") return false;
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
      if (update.docChanged && activeTab()?.runtimeCompileDiagnostics?.length) {
        activeTab()!.runtimeCompileDiagnostics = [];
        renderMessages();
      }
      if (update.docChanged) {
        const tab = activeTab();
        if (tab?.exercise) {
          tab.exerciseTestGeneration = (tab.exerciseTestGeneration ?? 0) + 1;
          tab.exerciseTestRun = undefined;
          renderExerciseTests();
        }
        renderTabs();
      }
      if (update.docChanged && activeTab()?.language !== "rust") {
        const tab = activeTab();
        if (tab?.rustPreview) {
          tab.rustPreview = markRustPreviewStale(tab.rustPreview, update.state.doc.toString());
        }
        if (learningMode === "bridge") rustPreview.schedule();
      }
      if (update.selectionSet) learningPanel.selectionChanged();
      if ((update.selectionSet || update.docChanged) && !$<HTMLElement>("learning-panel").hidden) learningPanel.render();
      if (update.selectionSet || update.docChanged) renderCursor();
      if (update.selectionSet || update.docChanged) rustPreview.renderMappingGutter();
      if (centerActive && update.selectionSet) {
        queueMicrotask(() => {
          if (centerActive && view.state === update.state) centerSelection();
        });
      }
    }),
  ];
}

const view = new EditorView({ parent: $("editor") });
const rustPreview = mountRustPreview({
  sourceView: view,
  activeTab: () => activeTab(),
  learningMode: () => learningMode,
  goTo,
  editorExtensions: [editorColors, generatedRustTheme.of(editorTheme())],
  onRendered: () => {
    if (!$<HTMLElement>("learning-panel").hidden) learningPanel.render();
  },
  openDetachedRust: (code) => {
    const tab = newTab(code, null, undefined, undefined, undefined, "rust");
    setStatus("Rust copiado para uma nova aba .rs independente. A origem e a prévia vinculada permanecem intactas.", "ok");
    view.focus();
    renderTabs();
    syncLanguageControls(tab);
  },
});
const generatedRustEditor = rustPreview.generatedEditor;
const learningPanel = mountLearningPanel({
  sourceView: view,
  activeTab: () => activeTab(),
  learningMode: () => learningMode,
  openExercise: async (exerciseId) => {
    await exercises.openById(exerciseId);
  },
});

/** Reaplica o tema atual aos editores e à ajuda. */
function refreshEditorThemes() {
  view.dispatch({ effects: theme.reconfigure(editorTheme()) });
  generatedRustEditor.dispatch({ effects: generatedRustTheme.reconfigure(editorTheme()) });
  help?.refreshTheme();
}

function applyTheme(choice: string) {
  applyPageTheme(choice);
  refreshEditorThemes();
}

onSystemThemeChange(refreshEditorThemes);

// ---------------------------------------------------------------------------
// Abas
// ---------------------------------------------------------------------------

let tabs: Tab[] = [];
const homeId = 0;
const helpId = -1;
let activeId = homeId;
let helpOpen = false;
let nextTabId = 1;
let definitionNavigationGeneration = 0;
let learningMode: LearningMode = loadLearningMode();

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

function newTab(text = PORS_TEMPLATE, path: string | null = null, exampleName?: string, examplePath?: string, exercise?: ExerciseDocument, explicitLanguage?: SourceLanguage, workspaceFile?: { id: string; path: string; preferenceKey: string }): Tab {
  const language = explicitLanguage ?? detectSourceLanguage(path ?? exampleName ?? "programa.pors");
  const tab: Tab = {
    id: nextTabId++,
    path,
    language,
    exampleName,
    examplePath,
    exercise,
    workspaceId: workspaceFile?.id,
    workspacePath: workspaceFile?.path,
    workspacePreferenceKey: workspaceFile?.preferenceKey,
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
    rustPreview.cancel();
    renderTabs();
    renderExerciseTests();
    $("cursor").textContent = "";
    if (id === helpId) void openHelp();
    else $("home").focus();
    return;
  }
  workspaceExplorer.select(tab.workspaceId, tab.workspacePath ?? tab.workspaceExercisePath);
  view.setState(tab.state);
  panes.load(tab.workspacePreferenceKey);
  applyLearningMode(loadLearningMode(undefined, tab.workspacePreferenceKey), false);
  view.dispatch({ effects: [
    theme.reconfigure(editorTheme()),
    sourceLanguage.reconfigure(tab.language === "rust" ? rust() : tab.language === "portugol-rs" ? portugolRs() : portugol()),
    portugolTools.reconfigure(tab.language === "rust" ? [rustSourceLinter] : [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]),
    setDebugLine.of(tab.id === runTabId ? paused?.position.line ?? null : null),
  ] });
  panes.applyAll();
  if (tab.language === "rust") view.dispatch(setDiagnostics(view.state, []));
  renderTabs();
  renderExerciseTests();
  $<HTMLElement>("rust-preview").hidden = learningMode !== "bridge" || tab.language === "rust";
  if (tab.language !== "rust") rustPreview.render(tab);
  rustPreview.renderMappingGutter();
  renderMessages();
  outline.show(tab.symbols, current !== tab);
  updateLibraryButtons(tab.symbols);
  renderCursor();
  if (centerActive) centerSelection();
  if (learningMode === "bridge" && tab.language !== "rust") rustPreview.schedule();
  if (tab.language === "rust") rustPreview.cancel();
  syncLanguageControls(tab);
  view.focus();
}

function syncLanguageControls(tab: Tab) {
  const semanticLanguage = semanticSymbolLanguage(tab.language);
  const pors = semanticLanguage === "portugol-rs";
  const rustFile = tab.language === "rust";
  const busy = runId !== null || pendingEvents !== null || porsRunPending;
  for (const id of ["btn-run", "btn-debug", "btn-debug-pause", "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out"]) {
    const button = $<HTMLButtonElement>(id);
    const isRun = id === "btn-run";
    const cargoDetectionPending = rustFile && Boolean(tab.workspaceId && tab.workspacePath) && tab.cargoProject === undefined;
    const runtimeUnavailable = isRun
      ? !canRunSource(tab.language, rustcAvailable) || cargoDetectionPending || (rustFile && tab.cargoProject === true && cargoAvailable === false)
      : !canDebugSource(tab.language);
    const stateDisabled = id === "btn-debug-pause"
      ? runId === null || paused !== null
      : id === "btn-run" || id === "btn-debug"
        ? busy && paused === null
        : paused === null;
    button.disabled = runtimeUnavailable || stateDisabled;
    button.dataset.defaultTitle ??= button.title;
    const unavailableReason = cargoDetectionPending
      ? "Verificando se este arquivo pertence a um projeto Cargo."
      : rustFile && rustcAvailable !== true
      ? "Instale a toolchain Rust para executar arquivos .rs."
      : rustFile && tab.cargoProject === true && cargoAvailable === false
        ? "Instale Cargo para executar o projeto Rust."
      : rustFile && !isRun
        ? "Depuração Rust ainda não está disponível."
          : "";
    if (unavailableReason) button.title = unavailableReason;
    else if (pors && isRun) button.title = "Executar na VM do Portugol-RS (Shift+F6)";
    else if (rustFile && isRun && tab.cargoProject === true) button.title = "Compilar e executar o projeto Cargo (scripts e dependências executam localmente)";
    else if (rustFile && isRun) button.title = "Compilar e executar Rust 2021 com rustc (execução sem sandbox)";
    else button.title = button.dataset.defaultTitle;
    button.setAttribute("aria-description", unavailableReason);
  }
  const args = $<HTMLInputElement>("args");
  args.disabled = pors;
  args.title = pors
    ? "Parâmetros ainda não são suportados em Portugol-RS."
    : rustFile
      ? "Espaços separam itens; aspas duplas agrupam. O programa recebe argv, sem shell."
      : "";
  const format = $<HTMLButtonElement>("editor-format");
  format.disabled = !canFormatSource(tab.language, rustfmtAvailable);
  format.title = rustFile && rustfmtAvailable !== true
    ? "Instale o componente rustfmt com `rustup component add rustfmt`."
    : "Formatar código-fonte";
  for (const [name, button] of libraryButtons) {
    button.disabled = pors || rustFile || tab.symbols.some((symbol) => symbol.kind === "library" && symbol.name.split(" → ", 1)[0] === name);
    button.title = rustFile ? "Bibliotecas Portugol não se aplicam a Rust." : pors ? "Bibliotecas clássicas não se aplicam ao Portugol-RS." : `Incluir biblioteca ${name}`;
  }
  for (const id of ["editor-definition", "editor-rename"]) {
    const button = $<HTMLButtonElement>(id);
    button.disabled = rustFile;
    button.setAttribute("aria-description", rustFile ? "Navegação e renomeação de símbolos Rust ainda não estão implementadas." : "");
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

const FILTERS = [
  { name: "Portugol-RS e Rust", extensions: ["pors", "rs"] },
  { name: "Portugol clássico (legado)", extensions: ["por"] },
  { name: "Todos os programas", extensions: ["pors", "rs", "por"] },
];

const workspaceExplorer = mountWorkspaceExplorer({
  tabs: () => tabs,
  replaceTabs: (next) => { tabs = next; },
  activeTab,
  activeId: () => activeId,
  homeId,
  activate,
  newTab: () => { newTab(); },
  openWorkspaceSource: (source, path, workspaceFile) =>
    newTab(source, path, undefined, undefined, undefined, undefined, workspaceFile),
  openExercise,
  isDirty,
  save: () => save(),
  renderTabs,
  setExplorerVisible: (visible) => setWorkspaceExplorerVisible(visible),
  cargoProjectDetected: (tab, checked) => {
    if (activeTab() !== tab) return;
    syncLanguageControls(tab);
    if (checked) forceLinting(view);
  },
  recentFiles: {
    remember: (path) => recentFiles.remember(path),
    rename: (oldPath, newPath) => recentFiles.rename(oldPath, newPath),
    forget: (path) => recentFiles.forget(path),
  },
});

workspaceExplorer.render();
void workspaceExplorer.restore();

function localStorageOrNull(): Storage | null {
  try {
    return localStorage;
  } catch {
    return null;
  }
}

const recentFiles = createRecentFiles(localStorageOrNull(), (list) => renderRecent(list));

function renderRecent(recent: readonly string[] = recentFiles.list()) {
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
  recentFiles.remember(path);
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
    const extension = tab.language === "portugol-rs" ? "pors" : tab.language === "rust" ? "rs" : "por";
    const exerciseName = tab.exercise
      ? `${tab.exercise.name.replace(/\.(pex|pors\.json|json)$/i, "")}.${extension}`
      : undefined;
    const filters = tab.language === "rust"
      ? [{ name: "Rust", extensions: ["rs"] }]
      : tab.language === "portugol-rs"
        ? [{ name: "Portugol-RS", extensions: ["pors"] }, ...FILTERS]
        : [{ name: "Portugol clássico", extensions: ["por"] }, ...FILTERS];
    path = await saveDialog({ filters, defaultPath: tab.path ?? exerciseName ?? tab.exampleName ?? `programa.${extension}` });
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
  const previousLanguage = tab.language;
  tab.language = detectSourceLanguage(path);
  const languageChanged = tab.language !== previousLanguage;
  if (languageChanged && tab.rustPreview?.artifact) {
    tab.rustPreview = { ...tab.rustPreview, pending: false, stale: true, error: undefined };
  }
  if (tab.id === activeId) {
    view.dispatch({ effects: [
      sourceLanguage.reconfigure(tab.language === "rust" ? rust() : tab.language === "portugol-rs" ? portugolRs() : portugol()),
      portugolTools.reconfigure(tab.language === "rust" ? [rustSourceLinter] : [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]),
    ] });
    syncLanguageControls(tab);
    if (languageChanged) {
      rustPreview.cancel();
      $<HTMLElement>("rust-preview").hidden = learningMode !== "bridge" || tab.language === "rust";
      rustPreview.render(tab);
      if (learningMode === "bridge" && tab.language !== "rust") rustPreview.schedule();
      else rustPreview.renderMappingGutter();
    }
  }
  if (tab.language === "rust") {
    if (tab.id === activeId) view.dispatch(setDiagnostics(view.state, []));
    tab.diagnostics = [];
    tab.symbols = [];
    renderMessages();
    outline.show([]);
    updateLibraryButtons([]);
  }
  tab.saved = text;
  recentFiles.remember(path);
  renderTabs();
  setStatus(`Salvo em ${path}`, "ok");
  return true;
}

// ---------------------------------------------------------------------------
// Execução e console
// ---------------------------------------------------------------------------

let runId: number | null = null;
let runTabId: number | null = null;
let pendingEvents: PendingRunEvent[] | null = null;
let rustcAvailable: boolean | null = null;
let cargoAvailable: boolean | null = null;
let rustfmtAvailable: boolean | null = null;
let porsRunPending = false;
let paused: api.DebugInfo | null = null;
const consoleBox = $("console");
const inputForm = $<HTMLFormElement>("console-form");
const inputField = $<HTMLInputElement>("console-input");

function setRunning(running: boolean) {
  const language = activeTab()?.language;
  const canRun = language !== undefined && canRunSource(language, rustcAvailable);
  const canDebug = language !== undefined && canDebugSource(language);
  const isBusy = running && paused === null;
  $<HTMLButtonElement>("btn-run").disabled = !canRun || isBusy;
  $<HTMLButtonElement>("btn-debug").disabled = !canDebug || isBusy;
  $<HTMLButtonElement>("btn-stop").disabled = !running || runId === null;
  $<HTMLButtonElement>("btn-debug-pause").disabled = !canDebug || !running || paused !== null;
  for (const id of [
    "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out",
  ]) {
    $<HTMLButtonElement>(id).disabled = !canDebug || paused === null;
  }
  if (!running) inputForm.hidden = true;
  $("debug-vars").hidden = paused === null || paused.variables.length === 0;
  $("inspector-empty").hidden = !$("debug-vars").hidden;
  $("inspector-empty").textContent = paused
    ? "Nenhuma variável neste ponto."
    : "Pause a execução para inspecionar as variáveis.";
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
    renderDebugInfo(info, (frame) => {
      if (runTabId === null || !tabs.some(({ id }) => id === runTabId)) return;
      if (activeId !== runTabId) activate(runTabId);
      goTo(frame.position.offset);
    });
    showUtilities(true);
    showPanel("debug");
    if (activeId === runTabId) goTo(info.position.offset);
    const activeRunTab = tabs.find(({ id }) => id === runTabId);
    setStatus(activeRunTab?.language === "portugol-rs"
      ? `VM Portugol-RS pausada na linha ${info.position.line}.`
      : `Pausado na linha ${info.position.line}.`);
  }
}

async function debugCommand(command: (id: number) => Promise<void>) {
  if (runId === null || paused === null) return;
  const id = runId;
  view.dispatch({ effects: setDebugLine.of(null) });
  paused = null;
  setRunning(true);
  setStatus(activeTab()?.language === "portugol-rs" ? "VM Portugol-RS executando…" : "Executando…");
  try {
    await command(id);
  } catch {
    // A execução pode ter terminado enquanto o comando era enviado.
  }
}

/**
 * Executa o arquivo ativo. Portugol-RS roda na VM educacional por padrão;
 * `asRust` compila o Rust gerado com `rustc`, quando disponível.
 */
async function runProgram(pauseOnStart = false, asRust = false) {
  if (runId !== null || pendingEvents !== null || porsRunPending) return;
  const tab = activeTab();
  if (!tab) return;
  if (tab.language === "rust") {
    if (tab.workspaceId && tab.workspacePath && tab.cargoProject === undefined) {
      setStatus("Verificando se este arquivo pertence a um projeto Cargo.");
      return;
    }
    if (pauseOnStart) {
      setStatus("Depuração de arquivos Rust ainda não está disponível.");
      return;
    }
    if (tab.cargoProject === true && cargoAvailable !== true) {
      setStatus("Instale Cargo para executar o projeto Rust.", "error");
      return;
    }
    if (rustcAvailable !== true) {
      setStatus("Instale a toolchain Rust para executar arquivos .rs.", "error");
      return;
    }
    let rustArgs: string[];
    try {
      rustArgs = parseProcessArguments($<HTMLInputElement>("args").value);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Argumentos inválidos.", "error");
      return;
    }
    if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
      const dirtyFiles = tabs.filter((candidate) =>
        candidate.workspaceId === tab.workspaceId && candidate.language === "rust" && candidate !== tab && isDirty(candidate),
      );
      if (dirtyFiles.length > 0) {
        setStatus("Salve os outros arquivos alterados deste projeto antes de executar Cargo.", "error");
        return;
      }
      if (isDirty(tab) && !(await save())) {
        setStatus("Salve o arquivo Rust antes de executar o projeto Cargo.", "error");
        return;
      }
    }
    if (activeTab() !== tab) return;
    const documentState = view.state.doc;
    const source = documentState.toString();
    porsRunPending = true;
    runTabId = tab.id;
    syncLanguageControls(tab);
    let cargoTarget: api.CargoRunTarget | null = null;
    try {
      if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
        setStatus("Identificando os binários do projeto Cargo…");
        const discoveryId = await api.registerRustSourceRun();
        runId = discoveryId;
        setRunning(true);
        let targetInfo: api.CargoRunTargets;
        try {
          targetInfo = await api.cargoProjectRunTargets(discoveryId, tab.workspaceId, tab.workspacePath);
        } finally {
          if (runId === discoveryId) runId = null;
          setRunning(false);
        }
        if (activeTab() !== tab || view.state.doc !== documentState) return;
        if (targetInfo.targets.length === 0) {
          setStatus("Este projeto Cargo não contém um binário para executar.", "error");
          return;
        }
        cargoTarget = targetInfo.targets.find((target) =>
          target.packageName === tab.cargoTarget?.packageName && target.targetName === tab.cargoTarget?.targetName,
        ) ?? targetInfo.defaultTarget ?? (targetInfo.targets.length === 1 ? targetInfo.targets[0] : null);
        if (!cargoTarget) {
          cargoTarget = await chooseCargoTarget(targetInfo.targets);
          if (!cargoTarget) {
            if (activeTab() === tab) setStatus("Execução Cargo cancelada.");
            return;
          }
        }
        tab.cargoTarget = cargoTarget;
      }
      const cargoConsent = tab.cargoProject === true;
      const hasConsent = cargoConsent
        ? hasCargoExecutionConsent(sessionStorage)
        : hasRustExecutionConsent(sessionStorage);
      if (!hasConsent) {
        const consentText = tab.cargoProject === true
          ? "Cargo pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta execução não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?"
          : "Arquivos Rust compilados e executados podem acessar arquivos, processos e recursos permitidos à sua conta. Esta execução não possui sandbox. Execute apenas código em que você confia. Deseja continuar?";
        const accepted = await ask(
          consentText,
          { title: tab.cargoProject === true ? "Executar projeto Cargo sem sandbox" : "Executar código Rust sem sandbox", kind: "warning", okLabel: "Executar Rust", cancelLabel: "Cancelar" },
        );
        if (!accepted) {
          if (activeTab() === tab) setStatus("Execução Rust cancelada.");
          return;
        }
        if (cargoConsent) rememberCargoExecutionConsent(sessionStorage, true);
        else rememberRustExecutionConsent(sessionStorage, true);
      }
      if (activeTab() !== tab || view.state.doc !== documentState) return;
      const id = await api.registerRustSourceRun();
      runId = id;
      view.dispatch({ effects: setRuntimeError.of(null) });
      view.dispatch({ effects: setDebugLine.of(null) });
      paused = null;
      showPanel("console");
      clearConsole();
      setRunning(true);
      if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
        setStatus("Compilando o projeto Cargo…");
        const result = await api.runCargoProject(id, tab.workspaceId, tab.workspacePath, cargoTarget, rustArgs);
        if (activeTab() !== tab || view.state.doc !== documentState) return;
        appendProcessOutput(result);
        if (result.status === "executed") {
          setStatus(result.exitCode === 0 ? "Projeto Cargo finalizado." : `Programa Rust terminou com código ${result.exitCode ?? "desconhecido"}.`, result.exitCode === 0 ? "ok" : "error");
        } else if (result.status === "cancelled") {
          setStatus(`Execução Cargo cancelada durante a ${result.phase === "compilation" ? "compilação" : "execução"}.`);
        } else if (result.status === "build_limit" || result.status === "runtime_limit") {
          const reason = result.reason === "time" ? (result.phase === "compilation" ? "20 segundos de compilação" : "3 segundos de execução") : "limite de saída de 1 MiB";
          setStatus(`Projeto Cargo interrompido: ${reason}.`, "error");
        } else {
          setStatus(result.phase === "compilation" ? "Cargo não conseguiu compilar o projeto." : "O programa Cargo terminou com erro.", "error");
        }
        return;
      }
      setStatus("Compilando e executando Rust 2021 com rustc…");
      const result = await api.runRustSource(id, source, rustArgs);
      if (activeTab() !== tab || view.state.doc !== documentState) return;
      if (result.status === "cancelled") {
        appendProcessOutput(result);
        const phase = result.phase === "compilation" ? "compilação" : "execução";
        setStatus(`Execução Rust cancelada durante a ${phase}.`);
      } else if (result.status === "executed") {
        appendProcessOutput(result);
        setStatus(
          result.exitCode === 0
            ? "Programa Rust finalizado com rustc."
            : `Programa Rust terminou com código ${result.exitCode ?? "desconhecido"}.`,
          result.exitCode === 0 ? "ok" : "error",
        );
      } else if (result.status === "runtime_limit") {
        appendProcessOutput(result);
        const reason = result.reason === "time" ? "tempo máximo de 3 segundos" : "limite de saída de 1 MiB";
        setStatus(`Execução Rust interrompida: ${reason}.`, "error");
      } else if (result.status === "compile_limit") {
        if (result.stderr) appendConsole(result.stderr, "error");
        const reason = result.reason === "time" ? "tempo máximo de compilação de 20 segundos" : "limite de saída de compilação de 1 MiB";
        setStatus(`Compilação Rust interrompida: ${reason}.`, "error");
      } else {
        const diagnostics: api.DiagnosticInfo[] = result.diagnostics.map((item) => ({
          severity: item.severity === "warning" ? "warning" : "error",
          code: item.code ?? "rustc",
          message: item.message,
          help: item.rendered,
          start: item.start,
          end: item.end,
        }));
        tab.runtimeCompileDiagnostics = diagnostics;
        if (result.stderr) appendConsole(result.stderr, "error");
        if (diagnostics.length) {
          renderMessages();
          view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
          showPanel("messages");
          setStatus("rustc encontrou erros no arquivo Rust.", "error");
        } else {
          setStatus("rustc encontrou erros; veja o detalhe no console.", "error");
        }
      }
    } catch (error) {
      if (activeTab() === tab && view.state.doc === documentState) {
        const message = String(error);
        if (message.includes("Descoberta de alvos Cargo cancelada")) {
          setStatus("Execução Cargo cancelada.");
        } else {
          appendConsole(message + "\n", "error");
          setStatus("Não foi possível compilar ou executar o arquivo Rust.", "error");
        }
      }
    } finally {
      runId = null;
      porsRunPending = false;
      runTabId = null;
      setRunning(runId !== null);
      const current = activeTab();
      if (current) syncLanguageControls(current);
    }
    return;
  }
  if (tab.language === "portugol-rs") {
    const documentState = view.state.doc;
    const source = documentState.toString();
    view.dispatch({ effects: setRuntimeError.of(null) });
    view.dispatch({ effects: setDebugLine.of(null) });
    paused = null;
    showPanel("console");
    clearConsole();
    inputForm.hidden = true;
    if (!asRust) {
      const events: PendingRunEvent[] = [];
      pendingEvents = events;
      runTabId = tab.id;
      setRunning(true);
      try {
        await runListeners;
        const id = await api.startPortugolRsVm(
          source,
          breakpointLines(view.state),
          pauseOnStart,
          tab.path ?? tab.examplePath ?? null,
        );
        runId = id;
        setRunning(true);
        setStatus(pauseOnStart ? "Depurando Portugol-RS na VM." : "Executando Portugol-RS na VM…");
        replayRunEvents(events, id);
      } catch (errors) {
        runTabId = null;
        setRunning(false);
        if (Array.isArray(errors)) {
          tab.diagnostics = errors;
          renderMessages();
          showPanel("messages");
          setStatus(
            pauseOnStart
              ? "O Portugol-RS contém erros e não pode ser depurado."
              : "O Portugol-RS contém erros e não pode ser executado.",
            "error",
          );
        } else {
          setStatus(String(errors), "error");
        }
      } finally {
        pendingEvents = null;
      }
      return;
    }
    if (rustcAvailable !== true) {
      setStatus("Instale a toolchain Rust para executar o Rust gerado.", "error");
      return;
    }
    porsRunPending = true;
    runTabId = tab.id;
    setRunning(true);
    setStatus("Compilando e executando com rustc…");
    let porsRunId: number | null = null;
    try {
      porsRunId = await api.registerPortugolRsRun();
      runId = porsRunId;
      setRunning(true);
      await runListeners;
      const result = await api.runPortugolRs(porsRunId, source);
      if (activeTab() !== tab || view.state.doc !== documentState) return;
      if (result.status === "cancelled") {
        appendProcessOutput(result);
        const phase = result.phase === "compilation" ? "compilação" : "execução";
        setStatus(`Execução Portugol-RS cancelada durante a ${phase}.`);
      } else if (result.status === "executed") {
        appendProcessOutput(result);
        setStatus(
          result.exitCode === 0
            ? "Programa Portugol-RS finalizado com rustc."
            : `Programa Portugol-RS terminou com código ${result.exitCode ?? "desconhecido"}.`,
          result.exitCode === 0 ? "ok" : "error",
        );
      } else if (result.status === "compile_limit") {
        if (result.stderr) appendConsole(result.stderr, "error");
        const reason = result.reason === "time" ? "tempo máximo de compilação de 20 segundos" : "limite de saída de compilação de 1 MiB";
        setStatus(`Compilação Portugol-RS interrompida: ${reason}.`, "error");
      } else if (result.status === "runtime_limit") {
        appendProcessOutput(result);
        const descriptions = {
          time: "tempo máximo de 3 segundos",
          output: "limite de saída de 1 MiB",
          string_memory: "limite de memória para cadeias de 16 MiB",
        } as const;
        setStatus(`Execução Portugol-RS interrompida: ${descriptions[result.reason]}.`, "error");
      } else {
        const diagnostics = result.diagnostics.map(sourceDiagnostic).filter((item): item is api.DiagnosticInfo => item !== null);
        tab.runtimeCompileDiagnostics = diagnostics;
        if (result.stderr) appendConsole(result.stderr, "error");
        if (diagnostics.length) {
          renderMessages();
          view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
          showPanel("messages");
          setStatus("rustc encontrou erros no Rust gerado. Selecione um diagnóstico para voltar à origem.", "error");
        } else {
          setStatus("rustc encontrou um erro no código gerado; veja o detalhe no console.", "error");
        }
      }
    } catch (error) {
      if (activeTab() === tab && view.state.doc === documentState) {
        appendConsole(String(error) + "\n", "error");
        setStatus("Não foi possível compilar ou executar o Portugol-RS.", "error");
      }
    } finally {
      inputForm.hidden = true;
      if (runId === porsRunId) runId = null;
      porsRunPending = false;
      runTabId = null;
      setRunning(runId !== null);
    }
    return;
  }
  view.dispatch({ effects: setRuntimeError.of(null) });
  view.dispatch({ effects: setDebugLine.of(null) });
  paused = null;
  showPanel("console");
  clearConsole();
  const source = view.state.doc.toString();
  const args = parseClassicArguments($<HTMLInputElement>("args").value);
  const lines = breakpointLines(view.state);
  const events: PendingRunEvent[] = [];
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
    replayRunEvents(events, id);
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
  if (runId === null) return;
  const id = runId;
  try {
    const accepted = await api.stop(id);
    if (accepted && runId === id && activeId === runTabId) {
      setStatus("Interrupção solicitada…");
    }
  } catch (error) {
    if (runId === id && activeId === runTabId) setStatus(`Não foi possível interromper: ${String(error)}`, "error");
  }
}

async function checkCargoProject() {
  if (runId !== null || pendingEvents !== null || porsRunPending) return;
  const tab = activeTab();
  if (!tab || tab.language !== "rust") return;
  if (tab.workspaceId && tab.workspacePath && tab.cargoProject === undefined) {
    setStatus("Verificando se este arquivo pertence a um projeto Cargo.");
    return;
  }
  if (tab.cargoProject !== true || !tab.workspaceId || !tab.workspacePath) {
    setStatus("Abra um arquivo Rust pelo explorador dentro de um projeto Cargo.", "error");
    return;
  }
  if (cargoAvailable !== true) {
    setStatus("Instale Cargo para analisar o projeto Rust.", "error");
    return;
  }
  const dirtyFiles = tabs.filter((candidate) =>
    candidate.workspaceId === tab.workspaceId && candidate.language === "rust" && candidate !== tab && isDirty(candidate),
  );
  if (dirtyFiles.length > 0) {
    setStatus("Salve os outros arquivos Rust alterados deste workspace antes da análise Cargo.", "error");
    return;
  }
  if (isDirty(tab) && !(await save())) {
    setStatus("Salve o arquivo Rust antes de analisar o projeto Cargo.", "error");
    return;
  }
  if (activeTab() !== tab) return;
  const documentState = view.state.doc;
  let registeredId: number | null = null;
  porsRunPending = true;
  runTabId = tab.id;
  syncLanguageControls(tab);
  try {
    if (!hasCargoExecutionConsent(sessionStorage)) {
      const accepted = await ask(
        "cargo check pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta análise não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?",
        { title: "Analisar projeto Cargo sem sandbox", kind: "warning", okLabel: "Analisar Cargo", cancelLabel: "Cancelar" },
      );
      if (!accepted) {
        if (activeTab() === tab) setStatus("Análise Cargo cancelada.");
        return;
      }
      rememberCargoExecutionConsent(sessionStorage, true);
    }
    if (activeTab() !== tab || view.state.doc !== documentState) return;
    registeredId = await api.registerRustSourceRun();
    runId = registeredId;
    setRunning(true);
    view.dispatch({ effects: setRuntimeError.of(null) });
    clearConsole();
    showPanel("messages");
    setStatus("Analisando projeto com cargo check…");
    const result = await api.checkCargoProject(registeredId, tab.workspaceId, tab.workspacePath);
    if (activeTab() !== tab || view.state.doc !== documentState) return;
    const diagnostics: api.DiagnosticInfo[] = result.diagnostics.map((item) => ({
      severity: item.severity === "warning" ? "warning" : "error",
      code: item.code ?? "cargo",
      message: item.message,
      help: item.rendered,
      start: item.start,
      end: item.end,
    }));
    tab.diagnostics = diagnostics;
    renderMessages();
    view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
    if (result.stderr) appendConsole(result.stderr, result.status === "check_error" ? "error" : "");
    if (result.status === "cancelled") {
      setStatus("Análise Cargo cancelada.");
    } else if (result.status === "check_limit") {
      const reason = result.reason === "time" ? "20 segundos" : "limite de saída de 1 MiB";
      setStatus(`Análise Cargo interrompida: ${reason}.`, "error");
    } else if (result.status === "check_error") {
      showPanel(diagnostics.length ? "messages" : "console");
      setStatus(diagnostics.length
        ? "Cargo encontrou diagnósticos no arquivo aberto; confira o Console para detalhes do projeto."
        : "Cargo check terminou com erros em outros arquivos ou no manifesto.", "error");
    } else {
      showPanel(diagnostics.length ? "messages" : result.stderr ? "console" : "messages");
      const warnings = diagnostics.filter((item) => item.severity === "warning").length;
      setStatus(diagnostics.length ? `Análise concluída: ${warnings} aviso(s) no arquivo aberto.` : "Análise Cargo concluída sem diagnósticos no arquivo aberto.", "ok");
    }
  } catch (error) {
    if (activeTab() === tab && view.state.doc === documentState) {
      appendConsole(`${String(error)}\n`, "error");
      showPanel("console");
      setStatus("Não foi possível analisar o projeto Cargo.", "error");
    }
  } finally {
    if (registeredId !== null && runId === registeredId) runId = null;
    porsRunPending = false;
    runTabId = null;
    setRunning(false);
    const current = activeTab();
    if (current) syncLanguageControls(current);
  }
}

async function runCargoTests() {
  if (runId !== null || pendingEvents !== null || porsRunPending) return;
  const tab = activeTab();
  if (!tab || tab.language !== "rust") return;
  if (tab.workspaceId && tab.workspacePath && tab.cargoProject === undefined) {
    setStatus("Verificando se este arquivo pertence a um projeto Cargo.");
    return;
  }
  if (tab.cargoProject !== true || !tab.workspaceId || !tab.workspacePath) {
    setStatus("Abra um arquivo Rust pelo explorador dentro de um projeto Cargo.", "error");
    return;
  }
  if (cargoAvailable !== true) {
    setStatus("Instale Cargo para executar os testes do projeto Rust.", "error");
    return;
  }
  const dirtyFiles = tabs.filter((candidate) =>
    candidate.workspaceId === tab.workspaceId && candidate.language === "rust" && candidate !== tab && isDirty(candidate),
  );
  if (dirtyFiles.length > 0) {
    setStatus("Salve os outros arquivos Rust alterados deste workspace antes de executar `cargo test`.", "error");
    return;
  }
  if (isDirty(tab) && !(await save())) {
    setStatus("Salve o arquivo Rust antes de executar `cargo test`.", "error");
    return;
  }
  if (activeTab() !== tab) return;
  const documentState = view.state.doc;
  let registeredId: number | null = null;
  porsRunPending = true;
  runTabId = tab.id;
  syncLanguageControls(tab);
  try {
    if (!hasCargoExecutionConsent(sessionStorage)) {
      const accepted = await ask(
        "cargo test pode acessar a rede e executar scripts de build, macros procedurais e código de teste no seu usuário local. Esta execução não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?",
        { title: "Executar cargo test sem sandbox", kind: "warning", okLabel: "Executar testes", cancelLabel: "Cancelar" },
      );
      if (!accepted) {
        if (activeTab() === tab) setStatus("Execução de `cargo test` cancelada.");
        return;
      }
      rememberCargoExecutionConsent(sessionStorage, true);
    }
    if (activeTab() !== tab || view.state.doc !== documentState) return;
    registeredId = await api.registerRustSourceRun();
    runId = registeredId;
    setRunning(true);
    tab.cargoTestRun = { source: documentState, status: "running" };
    showPanel("tests");
    renderExerciseTests();
    clearConsole();
    setStatus("Executando testes do projeto com cargo test…");
    const result = await api.testCargoProject(registeredId, tab.workspaceId, tab.workspacePath);
    if (activeTab() !== tab || view.state.doc !== documentState) return;
    if (result.stdout) appendConsole(result.stdout);
    if (result.stderr) appendConsole(result.stderr, result.status === "build_error" ? "error" : "");
    tab.cargoTestRun = {
      source: documentState,
      status: "completed",
      result: {
        status: result.status,
        passed: result.passed,
        failed: result.failed,
        ignored: result.ignored,
        failures: result.failures,
        reason: result.reason,
      },
    };
    renderExerciseTests();
    if (result.status === "passed") {
      setStatus(`Cargo test aprovado (${result.passed} teste(s)).`, "ok");
    } else if (result.status === "cancelled") {
      setStatus("Execução de `cargo test` cancelada.");
    } else {
      setStatus(result.status === "build_error"
        ? "Falha de compilação ao executar `cargo test`."
        : `Cargo test falhou (${result.failed} falha(s)).`, "error");
    }
  } catch (error) {
    if (activeTab() === tab && view.state.doc === documentState) {
      const message = typeof error === "string" ? error : "Não foi possível executar `cargo test`.";
      tab.cargoTestRun = { source: documentState, status: "error", message };
      renderExerciseTests();
      appendConsole(`${message}\n`, "error");
      setStatus("Não foi possível executar `cargo test`.", "error");
    }
  } finally {
    if (registeredId !== null && runId === registeredId) runId = null;
    porsRunPending = false;
    runTabId = null;
    setRunning(false);
    const current = activeTab();
    if (current) syncLanguageControls(current);
    renderExerciseTests();
  }
}

const bottomPanelNames: PanelName[] = ["console", "tests", "messages", "debug"];

function setInspectorView(learning: boolean) {
  $("learning-panel").hidden = !learning;
  for (const id of ["utilities-insert", "outline", "variable-inspector"]) {
    $<HTMLElement>(id).hidden = learning;
  }
}

function showPanel(which: PanelName) {
  const learningActive = which === "learning";
  setInspectorView(learningActive);
  if (learningActive) {
    showUtilities(true);
    learningPanel.render();
    return;
  }
  for (const name of bottomPanelNames) {
    $(`tab-${name}`).classList.toggle("active", which === name);
    $(`tab-${name}`).setAttribute("aria-selected", String(which === name));
    $(panelContentId(name)).hidden = which !== name;
  }
  $("console-tools").hidden = which !== "console";
  if (which === "tests") renderExerciseTests();
}

document.querySelector<HTMLElement>(".panel-tabs")?.addEventListener("keydown", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  const current = bottomPanelNames.find((name) => target.id === `tab-${name}`);
  if (!current) return;
  const next = panelNameForKey(bottomPanelNames, current, event.key);
  if (!next) return;
  event.preventDefault();
  showPanel(next);
  $<HTMLButtonElement>(`tab-${next}`).focus();
});

function goTo(offset: number) {
  const position = Math.min(offset, view.state.doc.length);
  view.dispatch({ selection: { anchor: position }, scrollIntoView: true });
  view.focus();
}

function renderMessages() {
  const tab = activeTab();
  renderDiagnostics([...(tab?.diagnostics ?? []), ...(tab?.runtimeCompileDiagnostics ?? [])], view.state.doc.lines, goTo);
}

function renderExerciseTests() {
  const tab = activeTab();
  const status = $("tests-status");
  const run = $<HTMLButtonElement>("tests-run");
  const failures = $<HTMLUListElement>("tests-results");
  failures.replaceChildren();

  if (!tab?.exercise && tab?.language === "rust" && tab.cargoProject === true) {
    const cargoState = cargoTestPanelState(view.state.doc, tab.cargoTestRun);
    $("tests-title").textContent = `Testes Cargo — ${tabName(tab)}`;
    const nextStatus = cargoTestStatus(cargoState);
    if (status.textContent !== nextStatus) status.textContent = nextStatus;
    run.hidden = false;
    run.disabled = cargoState.kind === "running" || cargoAvailable !== true || runId !== null || porsRunPending;
    run.textContent = cargoState.kind === "running" ? "Executando…" : "Executar testes";
    if (cargoState.kind === "completed") {
      for (const failure of cargoState.result.failures) {
        const item = document.createElement("li");
        item.textContent = failure;
        failures.append(item);
      }
    }
    failures.hidden = cargoState.kind !== "completed" || failures.childElementCount === 0;
    return;
  }

  const state = exerciseTestPanelState(tab?.exercise?.exercise, view.state.doc, tab?.exerciseTestRun);
  const exerciseLanguage = tab?.exercise?.pors ? "portugol-rs" : "portugol";
  $("tests-title").textContent = tab?.exercise ? `Testes — ${tabName(tab)}` : "Testes do projeto";
  const nextStatus = exerciseTestStatus(state, exerciseLanguage);
  if (status.textContent !== nextStatus) status.textContent = nextStatus;
  run.hidden = state.kind === "empty" || state.kind === "no-cases";
  run.disabled = state.kind === "running";
  run.textContent = state.kind === "running" ? "Executando…" : "Executar casos";
  if (state.kind === "completed") {
    for (const failure of state.result.failures) {
      const item = document.createElement("li");
      item.textContent = failure;
      failures.append(item);
    }
    for (const hint of state.result.hints ?? []) {
      const item = document.createElement("li");
      item.textContent = `Dica: ${hint}`;
      failures.append(item);
    }
  }
  failures.hidden = state.kind !== "completed" || failures.childElementCount === 0;
}

async function runExerciseTests() {
  const tab = activeTab();
  if (!tab?.exercise && tab?.language === "rust" && tab.cargoProject === true) {
    await runCargoTests();
    return;
  }
  if (!tab?.exercise || tab.exercise.exercise.cases <= 0) return;
  const documentState = view.state.doc;
  const request: ExerciseTestRequest = {
    tabId: tab.id,
    document: documentState,
    generation: (tab.exerciseTestGeneration ?? 0) + 1,
  };
  tab.exerciseTestGeneration = request.generation;
  tab.exerciseTestRun = { source: documentState, status: "running" };
  renderExerciseTests();
  const isCurrent = () => {
    const currentDocument = tab.id === activeId ? view.state.doc : tab.state.doc;
    return isCurrentExerciseTestRequest(request, {
      tabId: tab.id,
      document: currentDocument,
      generation: tab.exerciseTestGeneration ?? 0,
    }, tabs.includes(tab));
  };
  try {
    const grade = await exercises.evaluate(tab.exercise, documentState.toString(), isCurrent);
    if (!isCurrent()) return;
    if (!grade.current) {
      tab.exerciseTestRun = {
        source: documentState,
        status: "error",
        message: "Esta avaliação foi substituída por outra. Execute os casos novamente.",
      };
      return;
    }
    tab.exerciseTestRun = {
      source: documentState,
      status: "completed",
      result: {
        total: grade.evaluation.total,
        passed: grade.evaluation.passed,
        failures: grade.evaluation.failures,
        hints: grade.evaluation.hints,
        completed: grade.progress.completed,
      },
    };
  } catch (message) {
    if (isCurrent()) {
      tab.exerciseTestRun = {
        source: documentState,
        status: "error",
        message: typeof message === "string" ? message : "Não foi possível executar os casos do exercício.",
      };
    }
  } finally {
    renderExerciseTests();
  }
}

function routeRunEvent(id: number, handle: () => void) {
  routeRunEventById(id, runId, pendingEvents, handle);
}

const runListeners = api.listenRun({
  output(id, text) {
    routeRunEvent(id, () => appendConsole(text));
  },
  clear(id) {
    routeRunEvent(id, () => clearConsole());
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
      } else if (outcome.status === "runtime_limit") {
        const descriptions = {
          time: "tempo ativo de 3 segundos",
          output: "limite de saída de 1 MiB",
          string_memory: "limite cumulativo de cadeias de 16 MiB",
        } as const;
        const message = `VM Portugol-RS interrompida: ${descriptions[outcome.reason]}.`;
        appendConsole(`\n${message}\n`, "error");
        setStatus(message, "error");
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

const panes = mountPaneLayout({
  preferenceKey: () => activeTab()?.workspacePreferenceKey,
  hasActiveTab: () => activeTab() !== undefined,
});
const { showUtilities, toggleUtilities, toggleExpanded } = panes;
const setWorkspaceExplorerVisible = panes.setExplorerVisible;

$("learning-inspector-slot").append($("learning-panel"));
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
  const unsupported = activeTab()?.language !== "portugol";
  for (const [name, button] of libraryButtons) button.disabled = unsupported || included.has(name);
}

for (const snippet of SNIPPETS) {
  const button = document.createElement("button");
  button.textContent = snippet.label;
  button.title = snippet.detail ?? snippet.label;
  button.onclick = () => {
    const tab = activeTab();
    if (!tab || tab.language === "rust" || (tab.id === runTabId && runId !== null)) return;
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



// ---------------------------------------------------------------------------
// Botões e atalhos
// ---------------------------------------------------------------------------

$("home-new").onclick = () => newTab();
mountHomeNewFileMenu($<HTMLDetailsElement>("home-new-menu"), {
  "portugol-rs": () => newTab(PORS_TEMPLATE, null, undefined, undefined, undefined, "portugol-rs"),
  rust: () => newTab("fn main() {\n    \n}\n", null, undefined, undefined, undefined, "rust"),
  portugol: () => newTab(NEW_PROGRAM, null, undefined, undefined, undefined, "portugol"),
});
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
  if (!tab || tab.language === "rust" || (runTabId === tab.id && runId !== null)) return;
  const doc = view.state.doc;
  const dialog = $<HTMLDialogElement>("rename");
  try {
    const target = semanticSymbolLanguage(tab.language) === "portugol-rs"
      ? await api.renameTargetPortugolRs(doc.toString(), offset)
      : await api.renameTarget(doc.toString(), offset);
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
        const changes = semanticSymbolLanguage(tab.language) === "portugol-rs"
          ? await api.renameSymbolPortugolRs(doc.toString(), offset, input.value.trim())
          : await api.renameSymbol(doc.toString(), offset, input.value.trim());
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
  if (tab.language === "rust") {
    if (!quiet) setStatus("Navegação de símbolos Rust ainda não está configurada.");
    return;
  }
  const doc = view.state.doc;
  const request = ++definitionNavigationGeneration;
  try {
    const location = semanticSymbolLanguage(tab.language) === "portugol-rs"
      ? await api.resolveSymbolPortugolRs(doc.toString(), offset)
      : await api.resolveSymbol(doc.toString(), offset);
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
  if (tab.language === "rust" && rustfmtAvailable !== true) {
    setStatus("Instale o componente rustfmt com `rustup component add rustfmt`.", "error");
    return;
  }
  const doc = view.state.doc;
  try {
    const source = doc.toString();
    const changes = tab.language === "rust"
      ? await api.formatRustSource(source).then((formatted) =>
        formatted === source ? [] : [{ from: 0, to: doc.length, insert: formatted }])
      : tab.language === "portugol-rs"
        ? await api.formatPortugolRs(source)
        : await api.formatSource(source);
    if (activeTab() !== tab || view.state.doc !== doc) return;
    if (!changes) {
      setStatus("Não foi possível formatar. Confira a sintaxe do programa.", "error");
      return;
    }
    if (changes.length) view.dispatch({ changes, annotations: isolateHistory.of("full"), userEvent: "input.format" });
    setStatus(changes.length ? "Código formatado." : "O código já está formatado.", "ok");
    view.focus();
  } catch (error) {
    if (activeTab() === tab && view.state.doc === doc) {
      setStatus(error instanceof Error ? error.message : String(error), "error");
    }
  }
}

function editorMenuAction(id: string, action: () => unknown) {
  $(id).onclick = () => {
    editorOptions.open = false;
    action();
  };
}
function openExercise(exerciseDocument: ExerciseDocument, workspaceExercise?: { id: string; path: string; preferenceKey: string; exercise: true }) {
  if (workspaceExercise) {
    const existing = tabs.find((tab) =>
      tab.workspaceId === workspaceExercise.id && tab.workspaceExercisePath === workspaceExercise.path,
    );
    if (existing) {
      activate(existing.id);
      return;
    }
  }
  const template = exerciseDocument.pors ? exerciseDocument.pors.starter_code : TEMPLATE;
  const source = `/*
${exerciseDocument.exercise.statement.replaceAll("*/", "* /")}
*/

${template}`;
  const current = activeTab();
  if (current && !current.path && !current.exampleName && !isDirty(current) && tabs.length === 1) tabs = [];
  const tab = newTab(
    source,
    null,
    exerciseDocument.name,
    undefined,
    exerciseDocument,
    exerciseDocument.pors ? "portugol-rs" : "portugol",
  );
  if (workspaceExercise) {
    tab.workspaceId = workspaceExercise.id;
    tab.workspaceExercisePath = workspaceExercise.path;
    tab.workspacePreferenceKey = workspaceExercise.preferenceKey;
    workspaceExplorer.select(workspaceExercise.id, workspaceExercise.path);
    activate(tab.id);
  }
}
const exercises = mountExercises(openExercise);
$("home-exercises").onclick = exercises.browse;
$("tests-run").onclick = () => void runExerciseTests();
editorMenuAction("editor-exercise", () => {
  const tab = activeTab();
  const exercise = tab?.exercise;
  if (!tab || !exercise) return;
  const documentState = view.state.doc;
  const isCurrent = () => tabs.includes(tab)
    && (tab.id === activeId ? view.state.doc : tab.state.doc) === documentState;
  void exercises.grade(exercise, documentState.toString(), isCurrent);
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
  const availableMode = mode === "rust" && rustcAvailable !== true ? "beginner" : mode;
  learningMode = availableMode;
  const selector = $<HTMLSelectElement>("learning-mode");
  selector.value = availableMode;
  if (persist) saveLearningMode(availableMode, undefined, activeTab()?.workspacePreferenceKey);
  if (!$<HTMLElement>("learning-panel").hidden) learningPanel.render();

  const preview = $<HTMLElement>("rust-preview");
  const bridgeEnabled = availableMode === "bridge" && activeTab()?.language !== "rust";
  preview.hidden = !bridgeEnabled;
  rustPreview.renderMappingGutter();
  $<HTMLButtonElement>("btn-rust").setAttribute("aria-pressed", String(bridgeEnabled));
  if (bridgeEnabled) {
    if (activeTab()) rustPreview.schedule();
  } else {
    rustPreview.cancel();
  }
}

$<HTMLSelectElement>("learning-mode").onchange = (event) => {
  const selected = (event.currentTarget as HTMLSelectElement).value;
  if (selected === "beginner" || selected === "bridge" || selected === "rust") applyLearningMode(selected);
};
applyLearningMode(learningMode, false);
$<HTMLButtonElement>("btn-rust").onclick = () => {
  applyLearningMode(learningMode === "bridge" ? "beginner" : "bridge");
};
$<HTMLButtonElement>("rust-close").onclick = () => {
  applyLearningMode("beginner");
  view.focus();
};

mountPorsUiPreviewPanel({
  elements: {
    panel: $<HTMLElement>("pors-ui-preview"),
    toggleButton: $<HTMLButtonElement>("btn-ui-preview"),
    runButton: $<HTMLButtonElement>("pors-ui-run"),
    restartButton: $<HTMLButtonElement>("pors-ui-restart"),
    cancelButton: $<HTMLButtonElement>("pors-ui-cancel"),
    closeButton: $<HTMLButtonElement>("pors-ui-close"),
    exampleSelect: $<HTMLSelectElement>("pors-ui-example-select"),
    loadExampleButton: $<HTMLButtonElement>("pors-ui-load-example"),
    status: $<HTMLElement>("pors-ui-status"),
    canvas: $<HTMLElement>("pors-ui-canvas"),
  },
  backend: {
    startUi: (source, budget) => api.porsUiStart(source, budget),
    dispatchUi: (sessionId, event) => api.porsUiDispatch(sessionId, event),
    cancelUi: (sessionId) => api.porsUiCancel(sessionId),
    closeUi: (sessionId) => api.porsUiClose(sessionId),
  },
  mount: mountUiPreview,
  examples: PORS_UI_EXAMPLES,
  getActiveSource: () => {
    const tab = activeTab();
    if (!tab) return null;
    return {
      source: view.state.doc.toString(),
      isPors: tab.language === "portugol-rs",
    };
  },
  onLoadExample: (example) => {
    newTab(example.source, null, example.fileName, undefined, undefined, "portugol-rs");
  },
  onCloseFocus: () => view.focus(),
});
$("btn-debug").onclick = () => void stepOrStart();
$("btn-stop").onclick = () => void stopProgram();
$("btn-debug-pause").onclick = () => {
  if (activeTab()?.language !== "portugol") return;
  if (runId !== null) void api.debugPause(runId);
};
$("btn-debug-continue").onclick = () => void debugCommand(api.debugContinue);
$("btn-debug-step-over").onclick = () => void debugCommand(api.debugStepOver);
$("btn-debug-step-into").onclick = () => void debugCommand(api.debugStepInto);
$("btn-debug-step-out").onclick = () => void debugCommand(api.debugStepOut);
$("btn-clear").onclick = () => clearConsole();
$("btn-copy").onclick = async () => {
  try {
    await navigator.clipboard.writeText(consoleText());
    setStatus("Saída copiada.", "ok");
  } catch (error) {
    setStatus(String(error), "error");
  }
};
$("tab-console").onclick = () => showPanel("console");
$("tab-tests").onclick = () => showPanel("tests");
$("tab-messages").onclick = () => showPanel("messages");
$("tab-debug").onclick = () => showPanel("debug");
document.querySelectorAll<HTMLButtonElement>("[data-activity]").forEach((button) => {
  button.onclick = () => {
    document.querySelectorAll<HTMLButtonElement>("[data-activity]").forEach((activity) => {
      activity.setAttribute("aria-pressed", String(activity === button));
    });
    if (button.dataset.activity !== "learn") setInspectorView(false);
    switch (button.dataset.activity) {
      case "project":
        $("btn-explorer").click();
        button.setAttribute("aria-pressed", $("btn-explorer").getAttribute("aria-expanded") ?? "false");
        break;
      case "learn":
        showPanel("learning");
        break;
      case "examples":
        activate(homeId);
        break;
      case "debug":
        showUtilities(true);
        showPanel("debug");
        break;
      case "settings":
        $("editor-options").setAttribute("open", "");
        $<HTMLSelectElement>("theme").focus();
        break;
    }
  };
});
function cycleTab(step: number) {
  const ids = [homeId, ...(helpOpen ? [helpId] : []), ...tabs.map((tab) => tab.id)];
  activate(ids[(ids.indexOf(activeId) + step + ids.length) % ids.length]);
}

const shortcutDialog = $<HTMLDialogElement>("shortcuts");

function paletteContext(): PaletteContext {
  const tab = activeTab();
  return {
    language: tab?.language ?? null,
    cargoProject: tab?.cargoProject === true,
    cargoAvailable: cargoAvailable === true,
    canRunAsRust: tab !== undefined && canRunAsRust(tab.language, rustcAvailable),
    runActive: runId !== null,
    paused: paused !== null,
    porsRunPending,
  };
}

const commandPalette = mountCommandPalette(() =>
  shortcuts.filter(({ command }) => isPaletteCommandAvailable(command, paletteContext())),
);
const showCommandPalette = commandPalette.show;

function showShortcuts() {
  if (!shortcutDialog.open) shortcutDialog.showModal();
}

const shortcuts: Shortcut[] = [
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
  { label: "via paleta de comandos", command: "Executar como Rust", matches: () => false, action: () => runProgram(false, true) },
  { label: "via paleta de comandos", command: "Analisar projeto Cargo", matches: () => false, action: checkCargoProject },
  { label: "via paleta de comandos", command: "Testar projeto Cargo", matches: () => false, action: runCargoTests },
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
  { label: "Ctrl+Shift+P", command: "Paleta de comandos", matches: (e) =>
    (e.ctrlKey || e.metaKey) && e.shiftKey && !e.altKey && e.code === "KeyP", action: showCommandPalette },
];

function fileShortcut(key: string, shift: boolean, command: string, action: () => unknown) {
  shortcuts.push(ctrlShortcut(key, shift, command, action));
}

fileShortcut("N", false, "Novo arquivo", () => newTab());
fileShortcut("O", false, "Abrir arquivo", openFiles);
fileShortcut("E", true, "Mostrar ou ocultar o explorador", () => {
  workspaceExplorer.toggleExplorer();
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
shortcuts.push({ label: "Ctrl+F", command: "Localizar e substituir", matches: (e) =>
  (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "KeyF", action: () => openSearchPanel(view) });

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

renderShortcutList(shortcuts);
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

mountFontSizeControls(["editor-font", "console-font"]);

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
mountExamples((source, file, workPath) =>
  newTab(
    source,
    null,
    file,
    workPath || undefined,
    undefined,
    file.toLowerCase().endsWith(".pors") ? "portugol-rs" : undefined,
  ),
);
activate(0);
applyTheme(themeChoice());
showPanel("console");
setWorkspaceExplorerVisible(window.innerWidth >= 1440);
showUtilities(window.innerWidth >= 1440);
void api.rustToolchainAvailable().then((available) => {
  rustcAvailable = available;
  const rustOption = $<HTMLOptionElement>("learning-mode-rust-option");
  rustOption.disabled = !available;
  $("learning-mode-rust-help").textContent = available
    ? "Toolchain rustc disponível. Novo Rust cria um arquivo .rs; trocar de nível não altera o arquivo aberto."
    : "Instale a toolchain Rust (rustc) para habilitar este nível e executar arquivos .rs.";
  applyLearningMode(loadLearningMode(undefined, activeTab()?.workspacePreferenceKey), false);
  const tab = activeTab();
  if (!tab) return;
  syncLanguageControls(tab);
  if (tab.language === "rust") forceLinting(view);
}).catch(() => {
  rustcAvailable = false;
  $<HTMLOptionElement>("learning-mode-rust-option").disabled = true;
  $("learning-mode-rust-help").textContent = "Não foi possível verificar rustc. Instale ou configure a toolchain Rust para habilitar este nível.";
  applyLearningMode(loadLearningMode(undefined, activeTab()?.workspacePreferenceKey), false);
  const tab = activeTab();
  if (tab) {
    syncLanguageControls(tab);
    if (tab.language === "rust") forceLinting(view);
  }
});
void api.cargoToolchainAvailable().then((available) => {
  cargoAvailable = available;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
}).catch(() => {
  cargoAvailable = false;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
});
void api.rustfmtAvailable().then((available) => {
  rustfmtAvailable = available;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
}).catch(() => {
  rustfmtAvailable = false;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
});
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
      } else if (/\.rs$/i.test(path)) {
        supported = true;
        await openFile(path);
      }
    } catch (error) {
      setStatus(typeof error === "string" ? error : "Não foi possível abrir o arquivo.", "error");
    }
  }
  if (warnUnsupported && !supported && paths.length) setStatus("Escolha um arquivo .por, .pors, .rs ou .pex.", "error");
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
