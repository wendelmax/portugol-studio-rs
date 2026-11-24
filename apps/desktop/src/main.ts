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
} from "./exercise-test-panel";
import { createTestsPanelController } from "./run/tests-panel";
import { createCargoProjectController } from "./run/cargo-project";
import { createRunControls } from "./run/controls";
import { createDebuggerCommands } from "./run/debugger-commands";
import { createRustExecutionController } from "./run/rust-execution";
import { createPortugolRsExecutionController } from "./run/portugol-rs-execution";
import { createClassicExecutionController } from "./run/classic-execution";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline } from "./outline";
import { panelContentId, panelNameForKey, type PanelName } from "./panel-tabs";
import { LIBRARIES, portugol, portugolRs, SNIPPETS } from "./portugol";
import { detectSourceLanguage, ensureSourceExtension, type SourceLanguage } from "./document-language";
import { semanticSymbolLanguage } from "./semantic-symbol-actions";
import { canDebugSource, canRunAsRust, canRunSource } from "./runtime-capabilities";
import { parseClassicArguments, parseProcessArguments } from "./process-arguments";
import { hasCargoExecutionConsent, hasRustExecutionConsent, rememberCargoExecutionConsent, rememberRustExecutionConsent } from "./rust-execution-consent";
import { createLatestOnlyRustSourceChecker } from "./rust-source-check";
import { replayRunEvents, type PendingRunEvent } from "./run-event-router";
import { createRunEventsController } from "./run/run-events";
import { loadLearningMode, saveLearningMode, type LearningMode } from "./learning-mode";
import { isPaletteCommandAvailable, type PaletteContext } from "./command-palette";
import { markRustPreviewStale } from "./rust-preview-state";
import { PORS_UI_EXAMPLES } from "./pors-ui-examples";
import { mountPorsUiPreviewPanel } from "./pors-ui-preview-panel";
import { mountUiPreview } from "./ui-preview-renderer";
import * as api from "./api";
import type { Tab } from "./state/tab";
import { createTabsView } from "./state/tabs-view";
import { createTabFactory, isTabDirty, PORS_TEMPLATE, tabName } from "./state/tabs";
import { createTabCloser } from "./state/tab-closer";
import { deriveLanguageControls, LANGUAGE_BUTTON_IDS } from "./state/language-capabilities";
import { createTabActivator } from "./state/tab-activation";
import { mountWorkspaceExplorer } from "./workspace/explorer";
import { mountRustPreview } from "./rust/preview";
import { mountLearningPanel } from "./learning/panel";
import { createLearningModeController } from "./learning/mode-controller";
import { mountCommandPalette } from "./commands/palette";
import { renderShortcutList } from "./commands/shortcuts";
import { createIdeShortcuts } from "./commands/ide-shortcuts";
import { mountProjectDialogs } from "./shell/project-dialogs";
import { mountWindowControls } from "./shell/window";
import { createHelpLoader } from "./shell/help-loader";
import { $ } from "./dom";
import { createRecentFiles } from "./files/recent";
import { createFileIO } from "./files/io";
import { createIncomingFileOpener } from "./files/system-files";
import { mountPaneLayout } from "./layout/panes";
import { setStatus } from "./shell/status";
import { mountFontSizeControls } from "./shell/font-size";
import { appendConsole, appendProcessOutput, clearConsole, consoleText } from "./run/console";
import { renderDebugInfo } from "./run/debug-view";
import { chooseCargoTarget } from "./run/cargo-target-dialog";
import { applyPageTheme, editorColors, editorTheme, onSystemThemeChange, themeChoice } from "./editor/theme";
import { createSymbolNavigator } from "./editor/symbol-navigation";
import { createEditorFormatter } from "./editor/format";
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
    EditorView.contentAttributes.of({ "aria-label": "Editor de código" }),
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
const help = createHelpLoader({
  openExample: (source, file) => newTab(source, null, file),
  isActive: () => activeId === helpId,
});
const openHelp = help.open;
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
  help.refreshTheme();
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
let learningMode: LearningMode = loadLearningMode();
let testsPanel: ReturnType<typeof createTestsPanelController> | null = null;

const activeTab = () => tabs.find((t) => t.id === activeId);

function isDirty(tab: Tab): boolean {
  return isTabDirty(tab, activeId, view.state.doc.toString());
}

const NEW_PROGRAM = TEMPLATE.replace("\t\t\n", '\t\tescreva("Olá Mundo")\n');
const tabFactory = createTabFactory({ extensions, addTab: (tab) => tabs.push(tab), activate });

function newTab(text = PORS_TEMPLATE, path: string | null = null, exampleName?: string, examplePath?: string, exercise?: ExerciseDocument, explicitLanguage?: SourceLanguage, workspaceFile?: { id: string; path: string; preferenceKey: string }): Tab {
  return tabFactory.create({
    text,
    path,
    exampleName,
    examplePath,
    exercise,
    explicitLanguage,
    workspaceFile,
  });
}

function activate(id: number) {
  return tabActivator.activate(id);
}

function syncLanguageControls(tab: Tab) {
  const defaultTitles: Partial<Record<(typeof LANGUAGE_BUTTON_IDS)[number], string>> = {};
  for (const id of LANGUAGE_BUTTON_IDS) {
    const button = $<HTMLButtonElement>(id);
    button.dataset.defaultTitle ??= button.title;
    defaultTitles[id] = button.dataset.defaultTitle;
  }
  const controls = deriveLanguageControls(tab, {
    rustcAvailable,
    cargoAvailable,
    rustfmtAvailable,
    runId,
    paused,
    busy: runId !== null || pendingEvents !== null || porsRunPending,
    defaultTitles,
    libraryNames: [...libraryButtons.keys()],
  });
  for (const id of LANGUAGE_BUTTON_IDS) {
    const button = $<HTMLButtonElement>(id);
    button.disabled = controls.buttons[id].disabled;
    button.title = controls.buttons[id].title;
    button.setAttribute("aria-description", controls.buttons[id].description);
  }
  const args = $<HTMLInputElement>("args");
  args.disabled = controls.arguments.disabled;
  args.title = controls.arguments.title;
  const format = $<HTMLButtonElement>("editor-format");
  format.disabled = controls.format.disabled;
  format.title = controls.format.title;
  for (const [name, button] of libraryButtons) {
    button.disabled = controls.libraries[name].disabled;
    button.title = controls.libraries[name].title;
  }
  for (const id of ["editor-definition", "editor-rename"]) {
    const button = $<HTMLButtonElement>(id);
    button.disabled = controls.symbolActions.disabled;
    button.setAttribute("aria-description", controls.symbolActions.description);
  }
}

const tabCloser = createTabCloser({
  tabs: () => tabs,
  activeId: () => activeId,
  homeId,
  helpId,
  setHelpOpen: (open) => { helpOpen = open; },
  isDirty,
  tabName,
  confirmDiscard: (tab) => ask(`"${tabName(tab)}" tem alterações não salvas. Fechar mesmo assim?`, {
    title: "Fechar aba", kind: "warning", okLabel: "Fechar sem salvar", cancelLabel: "Cancelar",
  }),
  removeTab: (index) => { tabs.splice(index, 1); },
  activate,
  renderTabs,
});

function closeTab(id: number): Promise<boolean> {
  return tabCloser.close(id);
}

const tabsView = createTabsView({
  container: $("tabs"),
  document,
  tabs: () => tabs,
  activeId: () => activeId,
  homeId,
  helpId,
  helpOpen: () => helpOpen,
  tabName,
  isDirty,
  icon,
  activate,
  newTab: () => { newTab(); },
  closeTab,
});

function renderTabs() {
  tabsView.render();
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
  save: () => fileIO.save(),
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
    button.onclick = () => void fileIO.openFile(path).catch((error) => setStatus(String(error), "error"));
    return button;
  }));
}

const fileIO = createFileIO({
  tabs: () => tabs,
  replaceTabs: (next) => { tabs = next; },
  activeTab,
  activate,
  newTab: (text, path) => { newTab(text, path); },
  isDirty,
  readFile: api.readFile,
  openDialog,
  saveDialog,
  writeFile: api.writeFile,
  writeWorkspaceFile: api.writeWorkspaceFile,
  currentText: () => view.state.doc.toString(),
  detectSourceLanguage,
  ensureSourceExtension,
  rememberRecent: (path) => recentFiles.remember(path),
  renderTabs,
  setStatus,
  afterSave: (tab, _previousLanguage, languageChanged) => {
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
  },
});

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

const runControls = createRunControls({
  activeLanguage: () => activeTab()?.language,
  canRun: canRunSource,
  canDebug: canDebugSource,
  rustcAvailable: () => rustcAvailable,
  runId: () => runId,
  paused: () => paused,
  setButtonDisabled: (id, disabled) => { $<HTMLButtonElement>(id).disabled = disabled; },
  setInputHidden: (hidden) => { inputForm.hidden = hidden; },
  setDebugVarsHidden: (hidden) => { $("debug-vars").hidden = hidden; },
  setInspectorEmptyHidden: (hidden) => { $("inspector-empty").hidden = hidden; },
  setInspectorText: (text) => { $("inspector-empty").textContent = text; },
});
const setRunning = runControls.setRunning;

const debuggerCommands = createDebuggerCommands({
  runId: () => runId,
  runTabId: () => runTabId,
  activeTabId: () => activeId,
  breakpointLines,
  setBreakpoints: api.debugSetBreakpoints,
  setDebugLine: (line) => view.dispatch({ effects: setDebugLine.of(line) }),
  paused: () => paused,
  setPaused: (info) => { paused = info; },
  setRunning,
  runLanguage: () => activeTab()?.language,
  setStatus,
  stop: api.stop,
});

/** Pontos de parada podem mudar enquanto o programa está em execução. */
function syncBreakpoints(state: EditorState) {
  debuggerCommands.syncBreakpoints(state);
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
  await debuggerCommands.debugCommand(command);
}

const rustExecution = createRustExecutionController({
  activeTab,
  tabs: () => tabs,
  isBusy: () => runId !== null || pendingEvents !== null || porsRunPending,
  rustcAvailable: () => rustcAvailable,
  cargoAvailable: () => cargoAvailable,
  isDirty,
  save: () => fileIO.save(),
  argumentsText: () => $<HTMLInputElement>("args").value,
  parseArguments: parseProcessArguments,
  currentDocument: () => view.state.doc,
  isCurrent: (tab, documentState) => activeTab() === tab && view.state.doc === documentState,
  runId: () => runId,
  setRunId: (id) => { runId = id; },
  setPending: (pending) => { porsRunPending = pending; },
  setRunTabId: (id) => { runTabId = id; },
  syncLanguageControls,
  setRunning,
  hasConsent: (consent) => consent === "cargo"
    ? hasCargoExecutionConsent(sessionStorage)
    : hasRustExecutionConsent(sessionStorage),
  askConsent: (consent) => ask(
    consent === "cargo"
      ? "Cargo pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta execução não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?"
      : "Arquivos Rust compilados e executados podem acessar arquivos, processos e recursos permitidos à sua conta. Esta execução não possui sandbox. Execute apenas código em que você confia. Deseja continuar?",
    {
      title: consent === "cargo" ? "Executar projeto Cargo sem sandbox" : "Executar código Rust sem sandbox",
      kind: "warning",
      okLabel: "Executar Rust",
      cancelLabel: "Cancelar",
    },
  ),
  rememberConsent: (consent) => consent === "cargo"
    ? rememberCargoExecutionConsent(sessionStorage, true)
    : rememberRustExecutionConsent(sessionStorage, true),
  chooseCargoTarget,
  registerRun: api.registerRustSourceRun,
  cargoRunTargets: api.cargoProjectRunTargets,
  runCargoProject: api.runCargoProject,
  runRustSource: api.runRustSource,
  clearRuntimeError: () => view.dispatch({ effects: setRuntimeError.of(null) }),
  clearDebugLine: () => view.dispatch({ effects: setDebugLine.of(null) }),
  clearPaused: () => { paused = null; },
  showConsole: () => showPanel("console"),
  clearConsole,
  appendProcessOutput,
  appendConsole,
  setStatus,
  renderMessages,
  setDiagnostics: (tab, diagnostics) => {
    tab.runtimeCompileDiagnostics = diagnostics;
    view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
  },
  showMessages: () => showPanel("messages"),
});

const portugolRsExecution = createPortugolRsExecutionController({
  rustcAvailable: () => rustcAvailable,
  isCurrent: (tab, documentState) => activeTab() === tab && view.state.doc === documentState,
  runId: () => runId,
  setRunId: (id) => { runId = id; },
  setPending: (pending) => { porsRunPending = pending; },
  setRunTabId: (id) => { runTabId = id; },
  setPendingEvents: (events) => { pendingEvents = events; },
  setRunning,
  clearRuntimeError: () => view.dispatch({ effects: setRuntimeError.of(null) }),
  clearDebugLine: () => view.dispatch({ effects: setDebugLine.of(null) }),
  clearPaused: () => { paused = null; },
  showConsole: () => showPanel("console"),
  clearConsole,
  hideInput: () => { inputForm.hidden = true; },
  runListeners: () => runListeners,
  startVm: api.startPortugolRsVm,
  registerRun: api.registerPortugolRsRun,
  runGeneratedRust: api.runPortugolRs,
  setStatus,
  appendProcessOutput,
  appendConsole,
  renderMessages,
  setDiagnostics: (tab, diagnostics) => {
    tab.runtimeCompileDiagnostics = diagnostics;
    view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
  },
  setVmDiagnostics: (tab, diagnostics) => { tab.diagnostics = diagnostics as api.DiagnosticInfo[]; },
  showMessages: () => showPanel("messages"),
  sourceDiagnostic,
  toCodeMirror,
  replayEvents: replayRunEvents,
});

const classicExecution = createClassicExecutionController({
  parseArguments: parseClassicArguments,
  runListeners: () => runListeners,
  run: api.run,
  setRuntimeError: () => view.dispatch({ effects: setRuntimeError.of(null) }),
  setDebugLine: () => view.dispatch({ effects: setDebugLine.of(null) }),
  clearPaused: () => { paused = null; },
  showConsole: () => showPanel("console"),
  clearConsole,
  setPendingEvents: (events) => { pendingEvents = events; },
  setRunTabId: (id) => { runTabId = id; },
  setRunId: (id) => { runId = id; },
  setRunButtonsDisabled: (disabled) => {
    $<HTMLButtonElement>("btn-run").disabled = disabled;
    $<HTMLButtonElement>("btn-debug").disabled = disabled;
  },
  setRunning,
  setStatus,
  replayEvents: replayRunEvents,
  setDiagnostics: (tab, diagnostics) => { tab.diagnostics = diagnostics; },
  renderMessages,
  showMessages: () => showPanel("messages"),
});

/**
 * Executa o arquivo ativo. Portugol-RS roda na VM educacional por padrão;
 * `asRust` compila o Rust gerado com `rustc`, quando disponível.
 */
async function runProgram(pauseOnStart = false, asRust = false) {
  if (runId !== null || pendingEvents !== null || porsRunPending) return;
  const tab = activeTab();
  if (!tab) return;
  if (tab.language === "rust") {
    await rustExecution.run(pauseOnStart);
    return;
  }
  if (tab.language === "portugol-rs") {
    await portugolRsExecution.run(tab, view.state.doc, breakpointLines(view.state), pauseOnStart, asRust);
    return;
  }
  await classicExecution.run(tab, view.state.doc, $<HTMLInputElement>("args").value, breakpointLines(view.state), pauseOnStart);
}

function runOrContinue() {
  return paused ? debugCommand(api.debugContinue) : runProgram();
}

function stepOrStart() {
  return paused ? debugCommand(api.debugStepOver) : runProgram(true);
}

async function stopProgram() {
  await debuggerCommands.stopProgram();
}

const cargoProjectController = createCargoProjectController({
  tabs: () => tabs,
  activeTab,
  isBusy: () => runId !== null || pendingEvents !== null || porsRunPending,
  cargoAvailable: () => cargoAvailable,
  isDirty,
  save: () => fileIO.save(),
  currentDocument: () => view.state.doc,
  isCurrent: (tab, documentState) => activeTab() === tab && view.state.doc === documentState,
  setPending: (pending) => { porsRunPending = pending; },
  setRunTabId: (id) => { runTabId = id; },
  runId: () => runId,
  setRunId: (id) => { runId = id; },
  syncLanguageControls,
  setRunning,
  hasCargoConsent: () => hasCargoExecutionConsent(sessionStorage),
  askCargoConsent: (operation) => operation === "check"
    ? ask(
      "cargo check pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta análise não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?",
      { title: "Analisar projeto Cargo sem sandbox", kind: "warning", okLabel: "Analisar Cargo", cancelLabel: "Cancelar" },
    )
    : ask(
      "cargo test pode acessar a rede e executar scripts de build, macros procedurais e código de teste no seu usuário local. Esta execução não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?",
      { title: "Executar cargo test sem sandbox", kind: "warning", okLabel: "Executar testes", cancelLabel: "Cancelar" },
    ),
  rememberCargoConsent: () => rememberCargoExecutionConsent(sessionStorage, true),
  registerRun: api.registerRustSourceRun,
  checkCargoProject: api.checkCargoProject,
  testCargoProject: api.testCargoProject,
  clearRuntimeError: () => view.dispatch({ effects: setRuntimeError.of(null) }),
  clearConsole,
  showPanel,
  setStatus,
  setDiagnostics: (tab, diagnostics) => {
    tab.diagnostics = diagnostics;
    renderMessages();
    view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
  },
  appendConsole,
  renderExerciseTests,
});

async function checkCargoProject() {
  await cargoProjectController.check();
}

async function runCargoTests() {
  await cargoProjectController.runTests();
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
  testsPanel?.render();
}

async function runExerciseTests() {
  await testsPanel?.runExerciseTests();
}

const runEvents = createRunEventsController({
  runId: () => runId,
  setRunId: (id) => { runId = id; },
  runTabId: () => runTabId,
  setRunTabId: (id) => { runTabId = id; },
  activeTabId: () => activeId,
  pendingEvents: () => pendingEvents,
  setPaused: (info) => { paused = info; },
  setPausedInfo: setPaused,
  setRunning,
  setDebugLine: (line) => view.dispatch({ effects: setDebugLine.of(line) }),
  setRuntimeError: (range) => view.dispatch({ effects: setRuntimeError.of(range) }),
  appendConsole,
  clearConsole,
  setInputHint: (text) => { $("console-hint").textContent = text; },
  showInput: () => { inputForm.hidden = false; },
  clearInput: () => { inputField.value = ""; },
  focusInput: () => { if (activeTab()) inputField.focus(); },
  scrollConsole: () => { consoleBox.scrollTop = consoleBox.scrollHeight; },
  setStatus,
  goTo,
});

const runListeners = api.listenRun(runEvents.handlers);

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

const tabActivator = createTabActivator({
  tabs: () => tabs,
  currentTab: activeTab,
  setActiveId: (id) => { activeId = id; },
  editorState: () => view.state,
  homeId,
  helpId,
  setHelpOpen: (open) => { helpOpen = open; },
  setPageVisibility: (id, tab) => {
    $("home").hidden = id !== homeId;
    $("help").hidden = id !== helpId;
    $("code").hidden = !tab;
    $("editor-exercise").hidden = !tab?.exercise;
  },
  cancelPreview: () => rustPreview.cancel(),
  renderTabs,
  renderExerciseTests,
  clearCursor: () => { $("cursor").textContent = ""; },
  openHelp: () => { void openHelp(); },
  focusHome: () => $("home").focus(),
  selectWorkspace: (tab) => workspaceExplorer.select(tab.workspaceId, tab.workspacePath ?? tab.workspaceExercisePath),
  setEditorState: (tab) => view.setState(tab.state),
  loadPanes: (tab) => panes.load(tab.workspacePreferenceKey),
  applyLearningMode: (tab) => applyLearningMode(loadLearningMode(undefined, tab.workspacePreferenceKey), false),
  configureEditor: (tab) => view.dispatch({ effects: [
    theme.reconfigure(editorTheme()),
    sourceLanguage.reconfigure(tab.language === "rust" ? rust() : tab.language === "portugol-rs" ? portugolRs() : portugol()),
    portugolTools.reconfigure(tab.language === "rust" ? [rustSourceLinter] : [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]),
    setDebugLine.of(tab.id === runTabId ? paused?.position.line ?? null : null),
  ] }),
  applyPanes: () => panes.applyAll(),
  clearRustDiagnostics: () => view.dispatch(setDiagnostics(view.state, [])),
  setRustPreviewHidden: (hidden) => { $<HTMLElement>("rust-preview").hidden = hidden; },
  renderRustPreview: (tab) => rustPreview.render(tab),
  renderMappingGutter: () => rustPreview.renderMappingGutter(),
  renderMessages,
  showOutline: (tab, changed) => outline.show(tab.symbols, changed),
  updateLibraryButtons: (tab) => updateLibraryButtons(tab.symbols),
  renderCursor,
  centerSelectionEnabled: () => centerActive,
  centerSelection,
  bridgeMode: () => learningMode === "bridge",
  schedulePreview: () => rustPreview.schedule(),
  syncLanguageControls,
  focusEditor: () => view.focus(),
});

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
$("home-open").onclick = () => void fileIO.openFiles();
$("btn-open").onclick = () => void fileIO.openFiles();
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

const symbolNavigator = createSymbolNavigator({
  activeTab,
  view,
  resolvePortugol: api.resolveSymbol,
  resolvePortugolRs: api.resolveSymbolPortugolRs,
  setStatus,
});

async function goToDeclarationAt(offset: number, quiet = false) {
  return symbolNavigator.goToDeclarationAt(offset, quiet);
}

function renameSelection() {
  return renameAt(view.state.selection.main.from);
}

function goToDeclarationSelection() {
  return goToDeclarationAt(view.state.selection.main.head);
}

const editorFormatter = createEditorFormatter({
  activeTab,
  isRunningTab: (tab) => runTabId === tab.id && runId !== null,
  rustfmtAvailable: () => rustfmtAvailable === true,
  view,
  formatRust: api.formatRustSource,
  formatPortugolRs: api.formatPortugolRs,
  formatPortugol: api.formatSource,
  setStatus,
});

function formatCode() {
  return editorFormatter.formatCode();
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
testsPanel = createTestsPanelController({
  tabs: () => tabs,
  activeTab,
  activeId: () => activeId,
  currentDocument: () => view.state.doc,
  documentForTab: (tab) => tab.state.doc,
  cargoAvailable: () => cargoAvailable,
  runId: () => runId,
  runPending: () => porsRunPending,
  tabName,
  evaluateExercise: (exercise, source, isCurrent) => exercises.evaluate(exercise, source, isCurrent),
  cargoTestPanelState,
  cargoTestStatus,
  exerciseTestPanelState,
  exerciseTestStatus,
  isCurrentExerciseTestRequest,
  runCargoTests,
  view: {
    clearResults: () => $<HTMLUListElement>("tests-results").replaceChildren(),
    setTitle: (title) => { $("tests-title").textContent = title; },
    setStatus: (status) => {
      const element = $("tests-status");
      if (element.textContent !== status) element.textContent = status;
    },
    setRunHidden: (hidden) => { $<HTMLButtonElement>("tests-run").hidden = hidden; },
    setRunDisabled: (disabled) => { $<HTMLButtonElement>("tests-run").disabled = disabled; },
    setRunText: (text) => { $<HTMLButtonElement>("tests-run").textContent = text; },
    appendResult: (text) => {
      const item = document.createElement("li");
      item.textContent = text;
      $<HTMLUListElement>("tests-results").append(item);
    },
    setResultsHidden: (hidden) => { $<HTMLUListElement>("tests-results").hidden = hidden; },
  },
});
renderExerciseTests();
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
$("btn-save").onclick = () => void fileIO.save();
$("btn-save-as").onclick = () => void fileIO.save(true);
$("btn-run").onclick = () => void runOrContinue();
const applyLearningMode = createLearningModeController({
  rustcAvailable: () => rustcAvailable,
  activeTab,
  setMode: (mode) => { learningMode = mode; },
  setSelectorValue: (mode) => { $<HTMLSelectElement>("learning-mode").value = mode; },
  persist: (mode, preferenceKey) => { saveLearningMode(mode, undefined, preferenceKey); },
  panelVisible: () => !$<HTMLElement>("learning-panel").hidden,
  renderPanel: () => learningPanel.render(),
  setPreviewHidden: (hidden) => { $<HTMLElement>("rust-preview").hidden = hidden; },
  renderMappingGutter: () => rustPreview.renderMappingGutter(),
  setBridgePressed: (pressed) => $<HTMLButtonElement>("btn-rust").setAttribute("aria-pressed", String(pressed)),
  schedulePreview: () => rustPreview.schedule(),
  cancelPreview: () => rustPreview.cancel(),
});

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

const filterOutlineType = (type: string) => () => {
  if (!activeTab()) return;
  showUtilities(true);
  outline.filterType(type);
};
const filterOutlineKind = (kind: "variable" | "array" | "matrix" | "function" | "constant") => () => {
  if (!activeTab()) return;
  showUtilities(true);
  outline.filterKind(kind);
};
const shortcuts = createIdeShortcuts({
  runOrContinue,
  stepOrStart,
  stopProgram,
  pauseDebug: () => { if (runId !== null) void api.debugPause(runId); },
  continueDebug: () => debugCommand(api.debugContinue),
  stepOver: () => debugCommand(api.debugStepOver),
  stepInto: () => debugCommand(api.debugStepInto),
  stepOut: () => debugCommand(api.debugStepOut),
  help: () => activate(helpId),
  renameSymbol: renameSelection,
  formatCode,
  runAsRust: () => runProgram(false, true),
  checkCargoProject,
  runCargoTests,
  openTips: () => tips.open(),
  about: () => $<HTMLDialogElement>("about").showModal(),
  showShortcuts,
  home: () => activate(homeId),
  previousTab: () => cycleTab(-1),
  nextTab: () => cycleTab(1),
  toggleCenter: toggleCenterSource,
  toggleExpanded,
  commandPalette: showCommandPalette,
  newFile: () => newTab(),
  openFile: () => fileIO.openFiles(),
  toggleExplorer: () => workspaceExplorer.toggleExplorer(),
  save: () => fileIO.save(),
  saveAs: () => fileIO.save(true),
  executionParameters: () => {
    if (!activeTab()) return;
    editorOptions.open = true;
    $("args").focus();
  },
  utilities: toggleUtilities,
  searchOutline: () => {
    if (!activeTab()) return;
    showUtilities(true);
    outline.focus();
  },
  closeTab: () => closeTab(activeId),
  closeAllTabs: async () => {
    for (const tab of [...tabs]) if (!(await closeTab(tab.id))) return;
    if (helpOpen) await closeTab(helpId);
  },
  findReplace: () => openSearchPanel(view),
  filterTypeInteger: filterOutlineType("inteiro"),
  filterTypeCharacter: filterOutlineType("caracter"),
  filterTypeReal: filterOutlineType("real"),
  filterTypeString: filterOutlineType("cadeia"),
  filterTypeLogical: filterOutlineType("logico"),
  filterTypeVoid: filterOutlineType("vazio"),
  showAllTypes: filterOutlineType(""),
  filterVariable: filterOutlineKind("variable"),
  filterArray: filterOutlineKind("array"),
  filterMatrix: filterOutlineKind("matrix"),
  filterFunction: filterOutlineKind("function"),
  filterConstant: filterOutlineKind("constant"),
});

renderShortcutList(shortcuts);
$("home-shortcuts").onclick = showShortcuts;
mountProjectDialogs({ version, license });
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
mountWindowControls(() => {
  const current = activeTab();
  if (current) current.state = view.state;
  return tabs.filter(isDirty).map(tabName);
});

const themeSelect = $<HTMLSelectElement>("theme");
const themeEditor = mountThemeEditor(themeSelect, applyTheme);
themeSelect.value = themeChoice();
themeSelect.onchange = () => applyTheme(themeSelect.value);
editorMenuAction("editor-themes", themeEditor.open);

mountFontSizeControls(["editor-font", "console-font"]);

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


const openIncomingFiles = createIncomingFileOpener({
  readExercise,
  openExercise,
  openFile: (path) => fileIO.openFile(path),
  setStatus,
});

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
