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

import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { isolateHistory } from "@codemirror/commands";
import { forceLinting, setDiagnostics } from "@codemirror/lint";
import { openSearchPanel } from "@codemirror/search";
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
import { createRunDispatcher } from "./run/dispatcher";
import { createPausePresentation } from "./run/pause-presentation";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline } from "./outline";
import { panelContentId, panelNameForKey, type PanelName } from "./panel-tabs";
import { LIBRARIES, portugol, portugolRs, SNIPPETS } from "./portugol";
import { detectSourceLanguage, ensureSourceExtension } from "./document-language";
import { semanticSymbolLanguage } from "./semantic-symbol-actions";
import { canDebugSource, canRunAsRust, canRunSource } from "./runtime-capabilities";
import { parseClassicArguments, parseProcessArguments } from "./process-arguments";
import { hasCargoExecutionConsent, hasRustExecutionConsent, rememberCargoExecutionConsent, rememberRustExecutionConsent } from "./rust-execution-consent";
import { replayRunEvents, type PendingRunEvent } from "./run-event-router";
import { createRunEventsController } from "./run/run-events";
import { mountRunToolbar } from "./run/toolbar";
import { loadLearningMode, saveLearningMode, type LearningMode } from "./learning-mode";
import { isPaletteCommandAvailable, type PaletteContext } from "./command-palette";
import { markRustPreviewStale } from "./rust-preview-state";
import { PORS_UI_EXAMPLES } from "./pors-ui-examples";
import { mountPorsUiPreviewPanel } from "./pors-ui-preview-panel";
import { mountUiPreview } from "./ui-preview-renderer";
import * as api from "./api";
import type { Tab } from "./state/tab";
import { PORS_TEMPLATE, tabName } from "./state/tabs";
import { createTabsShell } from "./state/tabs-shell";
import { createLanguageControlsView } from "./state/language-controls-view";
import { createTabActivator } from "./state/tab-activation";
import { createExerciseTabOpener, type WorkspaceExerciseRef } from "./state/exercise-tab";
import { mountWorkspaceExplorer } from "./workspace/explorer";
import { mountRustPreview } from "./rust/preview";
import { mountLearningPanel } from "./learning/panel";
import { createLearningModeController } from "./learning/mode-controller";
import { mountCommandPalette } from "./commands/palette";
import { renderShortcutList } from "./commands/shortcuts";
import { createIdeShortcuts } from "./commands/ide-shortcuts";
import { cycleIdeTab, mountKeyboardShortcutDispatch } from "./commands/keyboard-dispatch";
import { mountProjectDialogs } from "./shell/project-dialogs";
import { mountWindowControls } from "./shell/window";
import { mountActivityBar } from "./shell/activity-bar";
import { createHelpLoader } from "./shell/help-loader";
import { createToolchainStatusController } from "./shell/toolchain-status";
import { $ } from "./dom";
import { createRecentFiles } from "./files/recent";
import { createRecentFilesView } from "./files/recent-view";
import { createFileIO } from "./files/io";
import { createIncomingFileOpener } from "./files/system-files";
import { mountPaneLayout } from "./layout/panes";
import { createPanelNavigation } from "./layout/panel-navigation";
import { setStatus } from "./shell/status";
import { mountFontSizeControls } from "./shell/font-size";
import { appendConsole, appendProcessOutput, clearConsole, consoleText } from "./run/console";
import { renderDebugInfo } from "./run/debug-view";
import { chooseCargoTarget } from "./run/cargo-target-dialog";
import { applyPageTheme, editorColors, editorTheme, onSystemThemeChange, themeChoice } from "./editor/theme";
import { createSymbolNavigator } from "./editor/symbol-navigation";
import { createEditorFormatter } from "./editor/format";
import { createEditorUpdateController } from "./editor/update-controller";
import { createEditorExtensionFactory } from "./editor/extensions";
import { createEditorSetup } from "./editor/setup";
import { createEditorNavigationBindings } from "./editor/navigation-bindings";
import { createEditorThemeController } from "./editor/theme-controller";
import { createSymbolRenamer } from "./editor/symbol-rename";
import { createUtilityInsertions } from "./editor/utility-insertions";
import {
  breakpointLines, breakpoints, debugLine, runtimeError, setDebugLine, setRuntimeError, toCodeMirror,
} from "./editor/decorations";

const TEMPLATE = "programa\n{\n\tfuncao inicio()\n\t{\n\t\t\n\t}\n}\n";
let tabsShell: ReturnType<typeof createTabsShell>;

function activeTab() { return tabsShell.activeTab(); }
function isDirty(tab: Tab) { return tabsShell.isDirty(tab); }
function newTab(...args: Parameters<typeof tabsShell.newTab>) { return tabsShell.newTab(...args); }
function closeTab(id: number) { return tabsShell.closeTab(id); }
function renderTabs() { tabsShell.renderTabs(); }
function renderCursor() { tabsShell.renderCursor(); }

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------


const editorSetup = createEditorSetup({
  storage: () => localStorage,
  hasActiveTab: () => activeTab() !== undefined,
  centerSelection,
  setCenterPressed: (pressed) => $("editor-center").setAttribute("aria-pressed", String(pressed)),
  focusEditor: () => view.focus(),
  syncBreakpoints: (state) => syncBreakpoints(state),
  bugIcon: () => icon("pequeno/bug"),
  activeTab,
  inspectPortugol: api.inspect,
  inspectPortugolRs: api.inspectPortugolRs,
  renderMessages,
  showOutline: (symbols) => outline.show(symbols),
  updateLibraryButtons,
  isRustcAvailable: () => rustcAvailable === true,
  checkRustSource: api.checkRustSource,
});
const {
  theme,
  sourceLanguage,
  portugolTools,
  generatedRustTheme,
  centerSource,
  breakpointGutter,
  portugolLinter,
  rustSourceLinter,
} = editorSetup;

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
  isActive: () => tabCollection.activeId() === helpId,
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

const editorUpdateHandler = createEditorUpdateController({
  activeTab,
  currentEditorState: () => view.state,
  renderMessages,
  renderExerciseTests,
  renderTabs,
  markPreviewStale: markRustPreviewStale,
  learningMode: () => learningMode,
  schedulePreview: () => rustPreview.schedule(),
  selectionChanged: () => learningPanel.selectionChanged(),
  learningPanelVisible: () => !$<HTMLElement>("learning-panel").hidden,
  renderLearningPanel: () => learningPanel.render(),
  renderCursor,
  renderMappingGutter: () => rustPreview.renderMappingGutter(),
  centerActive: () => centerSource.isActive(),
  centerSelection,
});

const editorNavigation = createEditorNavigationBindings({
  activeLanguage: () => activeTab()?.language,
  selectionHead: () => view.state.selection.main.head,
  goToDeclarationAt,
});

const extensions = createEditorExtensionFactory({
  sourceLanguage,
  portugolTools,
  theme,
  portugolExtensions: () => [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine],
  rustExtensions: () => [rustSourceLinter],
  portugolLanguage: () => portugol(),
  portugolRsLanguage: () => portugolRs(),
  rustLanguage: () => rust(),
  editorColors,
  editorTheme,
  findDialog,
  editorSearchKeys,
  onEditorClick: editorNavigation.onEditorClick,
  onGoToDeclaration: editorNavigation.onGoToDeclaration,
  onUpdate: editorUpdateHandler,
});

const editorThemeController = createEditorThemeController({
  sourceEditor: view,
  sourceTheme: theme,
  generatedEditor: generatedRustEditor,
  generatedTheme: generatedRustTheme,
  editorTheme,
  refreshHelpTheme: () => help.refreshTheme(),
  applyPageTheme,
  onSystemThemeChange,
});
const { applyTheme } = editorThemeController;

// ---------------------------------------------------------------------------
// Abas
// ---------------------------------------------------------------------------

const homeId = 0;
const helpId = -1;
let helpOpen = false;
let learningMode: LearningMode = loadLearningMode();
let testsPanel: ReturnType<typeof createTestsPanelController> | null = null;
const NEW_PROGRAM = TEMPLATE.replace("\t\t\n", '\t\tescreva("Olá Mundo")\n');
function activate(id: number) {
  return tabActivator.activate(id);
}

tabsShell = createTabsShell({
  homeId,
  helpId,
  document,
  container: $("tabs"),
  icon,
  extensions,
  activate,
  helpOpen: () => helpOpen,
  setHelpOpen: (open) => { helpOpen = open; },
  currentDocumentText: () => view.state.doc.toString(),
  editorState: () => view.state,
  setCursorText: (text) => { $("cursor").textContent = text; },
  confirmDiscard: (tab) => ask(`"${tabName(tab)}" tem alterações não salvas. Fechar mesmo assim?`, {
    title: "Fechar aba", kind: "warning", okLabel: "Fechar sem salvar", cancelLabel: "Cancelar",
  }),
});
const { tabCollection } = tabsShell;

const languageControlsView = createLanguageControlsView({
  button: (id) => $<HTMLButtonElement>(id),
  argumentsInput: () => $<HTMLInputElement>("args"),
  formatButton: () => $<HTMLButtonElement>("editor-format"),
  libraryButtons: () => utilityInsertions.libraryButtons(),
  state: () => ({
    rustcAvailable,
    cargoAvailable,
    rustfmtAvailable,
    runId,
    paused,
    busy: runId !== null || pendingEvents !== null || porsRunPending,
  }),
});

function syncLanguageControls(tab: Tab) {
  languageControlsView.sync(tab);
}

// ---------------------------------------------------------------------------
// Arquivos
// ---------------------------------------------------------------------------

const workspaceExplorer = mountWorkspaceExplorer({
  tabs: tabCollection.tabs,
  replaceTabs: (next) => tabCollection.replace(next),
  activeTab,
  activeId: tabCollection.activeId,
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

const recentFilesView = createRecentFilesView({
  recent: $("recent"),
  list: $("recent-files"),
  document,
  icon,
  openFile: (path) => fileIO.openFile(path),
  setStatus,
});
const recentFiles = createRecentFiles(localStorageOrNull(), (list) => recentFilesView.render(list));

const fileIO = createFileIO({
  tabs: tabCollection.tabs,
  replaceTabs: (next) => tabCollection.replace(next),
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
    if (tab.id === tabCollection.activeId()) {
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
      if (tab.id === tabCollection.activeId()) view.dispatch(setDiagnostics(view.state, []));
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
  activeTabId: tabCollection.activeId,
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

const pausePresentation = createPausePresentation({
  setPausedState: (info) => { paused = info; },
  activeTabId: tabCollection.activeId,
  runTabId: () => runTabId,
  runId: () => runId,
  runTab: () => tabCollection.tabs().find(({ id }) => id === runTabId),
  setDebugLine: (line) => view.dispatch({ effects: setDebugLine.of(line) }),
  setRunning,
  renderDebugInfo,
  activate,
  goTo,
  showUtilities: (visible) => showUtilities(visible),
  showDebugPanel: () => showPanel("debug"),
  setStatus,
});

function setPaused(info: api.DebugInfo | null) {
  pausePresentation.setPaused(info);
}

async function debugCommand(command: (id: number) => Promise<void>) {
  await debuggerCommands.debugCommand(command);
}

const rustExecution = createRustExecutionController({
  activeTab,
  tabs: tabCollection.tabs,
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

const runDispatcher = createRunDispatcher({
  isBusy: () => runId !== null || pendingEvents !== null || porsRunPending,
  activeTab,
  currentDocument: () => view.state.doc,
  editorState: () => view.state,
  argumentsText: () => $<HTMLInputElement>("args").value,
  breakpointLines: (state) => breakpointLines(state),
  rustExecution,
  portugolRsExecution,
  classicExecution,
  isPaused: () => paused !== null,
  debugContinue: () => debugCommand(api.debugContinue),
  debugStepOver: () => debugCommand(api.debugStepOver),
  stop: () => debuggerCommands.stopProgram(),
});
const { runProgram, runOrContinue, stepOrStart, stopProgram } = runDispatcher;

const cargoProjectController = createCargoProjectController({
  tabs: tabCollection.tabs,
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

const panelNavigation = createPanelNavigation({
  bottomPanels: bottomPanelNames,
  inspectorElements: ["utilities-insert", "outline", "variable-inspector"],
  setInspectorElementHidden: (id, hidden) => { $<HTMLElement>(id).hidden = hidden; },
  showUtilities: (visible) => panes.showUtilities(visible),
  renderLearningPanel: () => learningPanel.render(),
  setTabState: (name, active, selected) => {
    $(`tab-${name}`).classList.toggle("active", active);
    $(`tab-${name}`).setAttribute("aria-selected", String(selected));
  },
  setPanelContentHidden: (name, hidden) => { $(panelContentId(name)).hidden = hidden; },
  setConsoleToolsHidden: (hidden) => { $("console-tools").hidden = hidden; },
  renderExerciseTests,
});
const { setInspectorView } = panelNavigation;

function showPanel(which: PanelName) {
  panelNavigation.showPanel(which);
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
  activeTabId: tabCollection.activeId,
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

const utilityInsertions = createUtilityInsertions({
  libraries: LIBRARIES,
  snippets: SNIPPETS,
  createButton: () => document.createElement("button"),
  appendLibraryButton: (button) => $("utilities-libraries").append(button),
  appendSnippetButton: (button) => $("utilities-models").append(button),
  activeTab,
  isRunning: (tab) => tab.id === runTabId && runId !== null,
  currentDocument: () => view.state.doc,
  includeLibrary: api.includeLibrary,
  isCurrent: (tab, doc) => activeTab() === tab && view.state.doc === doc,
  applyLibraryChange: (change) => view.dispatch({ changes: change, annotations: isolateHistory.of("full") }),
  focusEditor: () => view.focus(),
  setStatus,
  createProgramTab: () => { newTab(); },
  selection: () => view.state.selection.main,
  applySnippet: (snippet, from, to) => {
    if (typeof snippet.apply === "function") snippet.apply(view, snippet, from, to);
  },
  utilityTabButton: (name) => $<HTMLButtonElement>(`utilities-${name}-tab`),
  utilityPanel: (name) => $<HTMLElement>(`utilities-${name}`),
});
utilityInsertions.mount();

function updateLibraryButtons(symbols: api.SymbolInfo[]) {
  utilityInsertions.updateLibraryButtons(symbols, activeTab()?.language);
}

const tabActivator = createTabActivator({
  tabs: tabCollection.tabs,
  currentTab: activeTab,
  setActiveId: tabCollection.setActiveId,
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
  centerSelectionEnabled: () => centerSource.isActive(),
  centerSelection,
  bridgeMode: () => learningMode === "bridge",
  schedulePreview: () => rustPreview.schedule(),
  syncLanguageControls,
  focusEditor: () => view.focus(),
});

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
function centerSelection() {
  view.dispatch({ effects: EditorView.scrollIntoView(view.state.selection.main.head, { y: "center" }) });
}

function toggleCenterSource() {
  centerSource.toggle();
}

const symbolRenamer = createSymbolRenamer({
  activeTab,
  currentDocument: () => view.state.doc,
  isRunning: (tab) => runTabId === tab.id && runId !== null,
  isCurrent: (tab, doc) => activeTab() === tab && view.state.doc === doc,
  hasOpenDialog: () => document.querySelector("dialog[open]") !== null,
  semanticSymbolLanguage,
  renameTargetPortugol: api.renameTarget,
  renameTargetPortugolRs: api.renameTargetPortugolRs,
  renameSymbolPortugol: api.renameSymbol,
  renameSymbolPortugolRs: api.renameSymbolPortugolRs,
  applyChanges: (changes) => view.dispatch({ changes, annotations: isolateHistory.of("full"), userEvent: "input.rename" }),
  focusEditor: () => view.focus(),
  setStatus,
  ui: {
    dialog: $<HTMLDialogElement>("rename"),
    form: $<HTMLFormElement>("rename-form"),
    input: $<HTMLInputElement>("rename-input"),
    error: $("rename-error"),
    submit: $<HTMLButtonElement>("rename-submit"),
    description: $("rename-description"),
    cancelButton: $<HTMLButtonElement>("rename-cancel"),
  },
});

function renameAt(offset: number) {
  return symbolRenamer.openAt(offset);
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
const exerciseTabOpener = createExerciseTabOpener({
  tabs: tabCollection.tabs,
  replaceTabs: (next) => tabCollection.replace(next),
  activeTab,
  isDirty,
  template: TEMPLATE,
  newTab,
  activate,
  selectWorkspace: (id, path) => workspaceExplorer.select(id, path),
});
function openExercise(exerciseDocument: ExerciseDocument, workspaceExercise?: WorkspaceExerciseRef) {
  exerciseTabOpener.open(exerciseDocument, workspaceExercise);
}
const exercises = mountExercises(openExercise);
testsPanel = createTestsPanelController({
  tabs: tabCollection.tabs,
  activeTab,
  activeId: tabCollection.activeId,
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
  const isCurrent = () => tabCollection.tabs().includes(tab)
    && (tab.id === tabCollection.activeId() ? view.state.doc : tab.state.doc) === documentState;
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
mountRunToolbar({
  button: (id) => $<HTMLButtonElement>(id),
  form: inputForm,
  input: inputField,
  runId: () => runId,
  activeLanguage: () => activeTab()?.language ?? null,
  runOrContinue,
  stepOrStart,
  stop: stopProgram,
  pause: (id) => { void api.debugPause(id); },
  continueDebug: () => debugCommand(api.debugContinue),
  stepOver: () => debugCommand(api.debugStepOver),
  stepInto: () => debugCommand(api.debugStepInto),
  stepOut: () => debugCommand(api.debugStepOut),
  clearConsole,
  consoleText,
  copyText: (text) => navigator.clipboard.writeText(text),
  setStatus,
  showPanel,
  appendConsole,
  sendInput: api.sendInput,
});
mountActivityBar(document.querySelectorAll<HTMLButtonElement>("[data-activity]"), {
  setInspectorView,
  openExplorer: () => $<HTMLButtonElement>("btn-explorer").click(),
  explorerExpanded: () => $("btn-explorer").getAttribute("aria-expanded"),
  showPanel,
  activateHome: () => activate(homeId),
  showUtilities,
  openSettings: () => {
    $("editor-options").setAttribute("open", "");
    $<HTMLSelectElement>("theme").focus();
  },
});
function cycleTab(step: number) {
  cycleIdeTab(step, {
    homeId,
    helpId,
    helpOpen,
    tabs: tabCollection.tabs(),
    activeId: tabCollection.activeId(),
    activate,
  });
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
  closeTab: () => closeTab(tabCollection.activeId()),
  closeAllTabs: async () => {
    for (const tab of tabCollection.tabs()) if (!(await closeTab(tab.id))) return;
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

mountKeyboardShortcutDispatch({
  target: window,
  hasOpenDialog: () => document.querySelector("dialog[open]") !== null,
  optionMenus: optionMenus.map((menu) => ({
    get open() { return menu.open; },
    set open(value: boolean) { menu.open = value; },
    focusSummary: () => menu.querySelector("summary")?.focus(),
  })),
  shortcuts: () => shortcuts,
});

// Confirma antes de fechar a janela com alterações não salvas.
void runListeners.catch((error) => setStatus(String(error), "error"));
mountWindowControls(() => {
  const current = activeTab();
  if (current) current.state = view.state;
  return tabCollection.tabs().filter(isDirty).map(tabName);
});

const themeSelect = $<HTMLSelectElement>("theme");
const themeEditor = mountThemeEditor(themeSelect, applyTheme);
themeSelect.value = themeChoice();
themeSelect.onchange = () => applyTheme(themeSelect.value);
editorMenuAction("editor-themes", themeEditor.open);

mountFontSizeControls(["editor-font", "console-font"]);

mountIcons();
recentFilesView.render(recentFiles.list());
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
const toolchainStatus = createToolchainStatusController({
  checkRust: api.rustToolchainAvailable,
  checkCargo: api.cargoToolchainAvailable,
  checkRustfmt: api.rustfmtAvailable,
  setRustcAvailable: (available) => { rustcAvailable = available; },
  setCargoAvailable: (available) => { cargoAvailable = available; },
  setRustfmtAvailable: (available) => { rustfmtAvailable = available; },
  setRustOptionDisabled: (disabled) => { $<HTMLOptionElement>("learning-mode-rust-option").disabled = disabled; },
  setRustHelpText: (text) => { $("learning-mode-rust-help").textContent = text; },
  restoreLearningMode: () => applyLearningMode(loadLearningMode(undefined, activeTab()?.workspacePreferenceKey), false),
  activeTab,
  syncLanguageControls,
  forceRustLint: () => forceLinting(view),
});
void toolchainStatus.initialize();
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
