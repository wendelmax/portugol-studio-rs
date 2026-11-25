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

import { EditorView } from "@codemirror/view";
import { isolateHistory } from "@codemirror/commands";
import { forceLinting, setDiagnostics } from "@codemirror/lint";
import { openSearchPanel } from "@codemirror/search";
import { rust } from "@codemirror/lang-rust";
import { open as openDialog, save as saveDialog, ask } from "@tauri-apps/plugin-dialog";

import { renderDiagnostics } from "./diagnostics";
import { mountExamples } from "./examples";
import { mountHomeNewFileMenu } from "./home-new-file-menu";
import { readExercise, type ExerciseDocument } from "./exercises";
import { createExecutionPresentationShell } from "./run/execution-presentation";
import { createExecutionControllerShell } from "./run/execution-controller-shell";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline } from "./outline";
import { panelContentId, panelNameForKey, type PanelName } from "./panel-tabs";
import { LIBRARIES, portugol, portugolRs, SNIPPETS } from "./portugol";
import { detectSourceLanguage, ensureSourceExtension } from "./document-language";
import { canRunAsRust } from "./runtime-capabilities";
import { createExecutionState, isExecutionBusy } from "./run/execution-state";
import { mountRunEventsShell } from "./run/run-events-shell";
import { mountRunToolbar } from "./run/toolbar";
import { loadLearningMode, type LearningMode } from "./learning-mode";
import { createPaletteContext } from "./command-palette";
import { markRustPreviewStale } from "./rust-preview-state";
import { mountPorsUiPreviewShell } from "./pors-ui-preview-shell";
import * as api from "./api";
import type { Tab } from "./state/tab";
import { PORS_TEMPLATE, tabName } from "./state/tabs";
import { createTabsShell } from "./state/tabs-shell";
import { createLanguageControlsView } from "./state/language-controls-view";
import { createTabActivationShell } from "./state/tab-activation-shell";
import type { WorkspaceExerciseRef } from "./state/exercise-tab";
import { mountWorkspaceExplorer } from "./workspace/explorer";
import { mountRustPreview } from "./rust/preview";
import { mountLearningPanel } from "./learning/panel";
import { mountLearningModeShell } from "./learning/mode-shell";
import { createExerciseWorkbenchShell } from "./learning/exercise-workbench";
import { cycleIdeTab } from "./commands/keyboard-dispatch";
import { mountIdeCommandShell } from "./commands/ide-command-shell";
import { mountProjectDialogs } from "./shell/project-dialogs";
import { mountWindowControls } from "./shell/window";
import { mountActivityBar } from "./shell/activity-bar";
import { createHelpLoader } from "./shell/help-loader";
import { mountToolchainStatusShell } from "./shell/toolchain-status-shell";
import { createToolchainState } from "./shell/toolchain-state";
import { $ } from "./dom";
import { createRecentFiles } from "./files/recent";
import { createRecentFilesView } from "./files/recent-view";
import { createFileIO } from "./files/io";
import { mountSystemFilesShell } from "./files/system-files-shell";
import { mountPaneLayout } from "./layout/panes";
import { createPanelNavigation } from "./layout/panel-navigation";
import { setStatus } from "./shell/status";
import { mountFontSizeControls } from "./shell/font-size";
import { appendConsole, appendProcessOutput, clearConsole, consoleText } from "./run/console";
import { applyPageTheme, editorColors, editorTheme, onSystemThemeChange, themeChoice } from "./editor/theme";
import { createEditorUpdateController } from "./editor/update-controller";
import { createEditorExtensionFactory } from "./editor/extensions";
import { createEditorSetup } from "./editor/setup";
import { createEditorNavigationBindings } from "./editor/navigation-bindings";
import { createEditorThemeController } from "./editor/theme-controller";
import { createEditorActionsShell } from "./editor/actions-shell";
import { createUtilityInsertions } from "./editor/utility-insertions";
import {
  breakpoints, debugLine, runtimeError, setDebugLine,
} from "./editor/decorations";

const TEMPLATE = "programa\n{\n\tfuncao inicio()\n\t{\n\t\t\n\t}\n}\n";
let tabsShell: ReturnType<typeof createTabsShell>;
const toolchainState = createToolchainState();

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
  isRustcAvailable: () => toolchainState.rustcAvailable() === true,
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
    await exerciseWorkbench?.openById(exerciseId);
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
let exerciseWorkbench: ReturnType<typeof createExerciseWorkbenchShell> | null = null;
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
    rustcAvailable: toolchainState.rustcAvailable(),
    cargoAvailable: toolchainState.cargoAvailable(),
    rustfmtAvailable: toolchainState.rustfmtAvailable(),
    runId: runState.runId,
    paused: runState.paused,
    busy: isExecutionBusy(runState),
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

const runState = createExecutionState();
const consoleBox = $("console");
const inputForm = $<HTMLFormElement>("console-form");
const inputField = $<HTMLInputElement>("console-input");

const executionPresentation = createExecutionPresentationShell({
  activeTab,
  tabs: tabCollection.tabs,
  activeTabId: tabCollection.activeId,
  runState,
  rustcAvailable: toolchainState.rustcAvailable,
  view,
  activate,
  goTo,
  setStatus,
  showUtilities: (visible) => showUtilities(visible),
  showDebugPanel: () => showPanel("debug"),
});
const { setRunning, syncBreakpoints, setPaused, debugCommand } = executionPresentation;

const executionControllers = createExecutionControllerShell({
  runState,
  runPresentation: executionPresentation,
  view,
  tabs: tabCollection.tabs,
  activeTab,
  rustcAvailable: toolchainState.rustcAvailable,
  cargoAvailable: toolchainState.cargoAvailable,
  isDirty,
  save: () => fileIO.save(),
  runListeners: () => runListeners,
  syncLanguageControls,
  showPanel,
  setStatus,
  clearConsole,
  appendConsole,
  appendProcessOutput,
  renderMessages,
  renderExerciseTests,
});
const { runProgram, runOrContinue, stepOrStart, stopProgram, checkCargoProject, runCargoTests } = executionControllers;

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
  exerciseWorkbench?.renderTests();
}

const runListeners = mountRunEventsShell({
  state: runState,
  activeTabId: tabCollection.activeId,
  setPausedInfo: setPaused,
  setRunning,
  view,
  consoleBox,
  inputForm,
  inputField,
  appendConsole,
  clearConsole,
  setStatus,
  goTo,
  activeTab,
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

const utilityInsertions = createUtilityInsertions({
  libraries: LIBRARIES,
  snippets: SNIPPETS,
  createButton: () => document.createElement("button"),
  appendLibraryButton: (button) => $("utilities-libraries").append(button),
  appendSnippetButton: (button) => $("utilities-models").append(button),
  activeTab,
  isRunning: (tab) => tab.id === runState.runTabId && runState.runId !== null,
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

const tabActivator = createTabActivationShell({
  tabs: {
    all: tabCollection.tabs,
    current: activeTab,
    setActiveId: tabCollection.setActiveId,
    render: renderTabs,
    renderCursor,
  },
  ids: { home: homeId, help: helpId },
  help: { setOpen: (open) => { helpOpen = open; }, open: () => { void openHelp(); } },
  editor: {
    state: () => view.state,
    setState: (tab) => view.setState(tab.state),
    configure: (tab) => view.dispatch({ effects: [
      theme.reconfigure(editorTheme()),
      sourceLanguage.reconfigure(tab.language === "rust" ? rust() : tab.language === "portugol-rs" ? portugolRs() : portugol()),
      portugolTools.reconfigure(tab.language === "rust" ? [rustSourceLinter] : [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]),
      setDebugLine.of(tab.id === runState.runTabId ? runState.paused?.position.line ?? null : null),
    ] }),
    clearRustDiagnostics: () => view.dispatch(setDiagnostics(view.state, [])),
    centerSelectionEnabled: () => centerSource.isActive(),
    centerSelection,
    syncLanguageControls,
    focus: () => view.focus(),
  },
  workspace: {
    select: (tab) => workspaceExplorer.select(tab.workspaceId, tab.workspacePath ?? tab.workspaceExercisePath),
    loadPanes: (tab) => panes.load(tab.workspacePreferenceKey),
    applyPanes: () => panes.applyAll(),
  },
  preview: {
    cancel: () => rustPreview.cancel(),
    render: (tab) => rustPreview.render(tab),
    renderMappingGutter: () => rustPreview.renderMappingGutter(),
    schedule: () => rustPreview.schedule(),
  },
  panels: {
    renderExerciseTests,
    renderMessages,
    showOutline: (tab, changed) => outline.show(tab.symbols, changed),
    updateLibraryButtons: (tab) => updateLibraryButtons(tab.symbols),
  },
  learning: {
    applyForTab: (tab) => applyLearningMode(loadLearningMode(undefined, tab.workspacePreferenceKey), false),
    bridgeMode: () => learningMode === "bridge",
  },
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
  editorActions.centerSelection();
}

function toggleCenterSource() {
  editorActions.toggleCenterSource();
}

const editorActions = createEditorActionsShell({
  view,
  activeTab,
  runId: () => runState.runId,
  runTabId: () => runState.runTabId,
  rustfmtAvailable: () => toolchainState.rustfmtAvailable() === true,
  centerSelection: () => view.dispatch({ effects: EditorView.scrollIntoView(view.state.selection.main.head, { y: "center" }) }),
  toggleCenterSource: () => centerSource.toggle(),
});

function renameAt(offset: number) {
  return editorActions.renameAt(offset);
}

function goToDeclarationAt(offset: number, quiet = false) {
  return editorActions.goToDeclarationAt(offset, quiet);
}

function renameSelection() {
  return editorActions.renameSelection();
}

function goToDeclarationSelection() {
  return editorActions.goToDeclarationSelection();
}

function formatCode() {
  return editorActions.formatCode();
}

function editorMenuAction(id: string, action: () => unknown) {
  $(id).onclick = () => {
    editorOptions.open = false;
    action();
  };
}
function openExercise(exerciseDocument: ExerciseDocument, workspaceExercise?: WorkspaceExerciseRef) {
  exerciseWorkbench?.openExercise(exerciseDocument, workspaceExercise);
}
exerciseWorkbench = createExerciseWorkbenchShell({
  tabs: tabCollection.tabs,
  replaceTabs: (next) => tabCollection.replace(next),
  activeTab,
  activeId: tabCollection.activeId,
  isDirty,
  template: TEMPLATE,
  newTab,
  activate,
  selectWorkspace: (id, path) => workspaceExplorer.select(id, path),
  currentDocument: () => view.state.doc,
  documentForTab: (tab) => tab.state.doc,
  cargoAvailable: toolchainState.cargoAvailable,
  runId: () => runState.runId,
  runPending: () => runState.porsRunPending,
  tabName,
  runCargoTests,
});
editorMenuAction("editor-exercise", () => exerciseWorkbench?.gradeActiveExercise());
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
const applyLearningMode = mountLearningModeShell({
  initialMode: learningMode,
  currentMode: () => learningMode,
  rustcAvailable: toolchainState.rustcAvailable,
  activeTab,
  setMode: (mode) => { learningMode = mode; },
  panelVisible: () => !$<HTMLElement>("learning-panel").hidden,
  renderPanel: () => learningPanel.render(),
  setPreviewHidden: (hidden) => { $("rust-preview").hidden = hidden; },
  renderMappingGutter: () => rustPreview.renderMappingGutter(),
  setBridgePressed: (pressed) => $<HTMLButtonElement>("btn-rust").setAttribute("aria-pressed", String(pressed)),
  schedulePreview: () => rustPreview.schedule(),
  cancelPreview: () => rustPreview.cancel(),
  focusEditor: () => view.focus(),
});

mountPorsUiPreviewShell({
  activeTab,
  source: () => view.state.doc.toString(),
  openExample: (source, fileName) => newTab(source, null, fileName, undefined, undefined, "portugol-rs"),
  focusEditor: () => view.focus(),
});
mountRunToolbar({
  button: (id) => $<HTMLButtonElement>(id),
  form: inputForm,
  input: inputField,
  runId: () => runState.runId,
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

function paletteContext() {
  return createPaletteContext({
    activeTab: activeTab(),
    cargoAvailable: toolchainState.cargoAvailable(),
    rustcAvailable: toolchainState.rustcAvailable(),
    runActive: runState.runId !== null,
    paused: runState.paused !== null,
    porsRunPending: runState.porsRunPending,
    canRunAsRust,
  });
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
mountIdeCommandShell({
  paletteContext,
  optionMenus,
  actions: {
    runOrContinue,
    stepOrStart,
    stopProgram,
    pauseDebug: () => { if (runState.runId !== null) void api.debugPause(runState.runId); },
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
    home: () => activate(homeId),
    previousTab: () => cycleTab(-1),
    nextTab: () => cycleTab(1),
    toggleCenter: toggleCenterSource,
    toggleExpanded,
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
  },
});
mountProjectDialogs({ version, license });
$("home-help").onclick = $("btn-help").onclick = () => activate(helpId);

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
mountToolchainStatusShell({
  state: toolchainState,
  restoreLearningMode: () => applyLearningMode(loadLearningMode(undefined, activeTab()?.workspacePreferenceKey), false),
  activeTab,
  syncLanguageControls,
  forceRustLint: () => forceLinting(view),
});
const tips = mountTips(themeEditor.open);
$("home-tips").onclick = tips.open;


mountSystemFilesShell({
  readExercise,
  openExercise,
  openFile: (path) => fileIO.openFile(path),
  setStatus,
});
