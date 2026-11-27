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
import { EditorState } from "@codemirror/state";
import { forceLinting, setDiagnostics } from "@codemirror/lint";
import { openSearchPanel } from "@codemirror/search";
import { rust } from "@codemirror/lang-rust";
import { open as openDialog, save as saveDialog, ask } from "@tauri-apps/plugin-dialog";

import { renderDiagnostics } from "./diagnostics";
import { mountExamples } from "./examples";
import { mountHomeNewFileMenu } from "./home-new-file-menu";
import { readExercise, type ExerciseDocument } from "./exercises";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline } from "./outline";
import { panelContentId, panelNameForKey, type PanelName } from "./panel-tabs";
import { PORS_MODULES, PORS_SNIPPETS, portugolRs } from "./portugol";
import { markup } from "./editor/markup-language";
import { resolveHelpTopicForEditor } from "./help-navigation";
import { detectSourceLanguage, ensureSourceExtension, type SourceLanguage } from "./document-language";
import { canRunAsRust } from "./runtime-capabilities";
import { isExecutionBusy } from "./run/execution-state";
import { createRunShell } from "./run/run-shell";
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
import { mountLearningModeShell } from "./learning/mode-shell";
import { createExerciseWorkbenchShell } from "./learning/exercise-workbench";
import { cycleIdeTab } from "./commands/keyboard-dispatch";
import { mountIdeCommandShell } from "./commands/ide-command-shell";
import { mountProjectDialogs } from "./shell/project-dialogs";
import { mountWindowControls } from "./shell/window";
import { mountActivityBar } from "./shell/activity-bar";
import { mountToolchainStatusShell } from "./shell/toolchain-status-shell";
import { createToolchainState } from "./shell/toolchain-state";
import { $ } from "./dom";
import { createWorkspaceFilesShell } from "./files/workspace-files-shell";
import { mountSystemFilesShell } from "./files/system-files-shell";
import { mountPaneLayout } from "./layout/panes";
import { createPanelNavigation } from "./layout/panel-navigation";
import { setStatus } from "./shell/status";
import { mountStatusBar } from "./shell/status-bar";
import { mountFontSizeControls } from "./shell/font-size";
import { appendConsole, appendProcessOutput, clearConsole, consoleText } from "./run/console";
import { applyPageTheme, editorTheme, onSystemThemeChange, themeChoice } from "./editor/theme";
import { createEditorShell } from "./editor/editor-shell";
import { mountDesignerShell, textChange } from "./designer/designer-shell";
import { mountCanvasInteractions } from "./designer/interactions";
import { renderToolbox } from "./designer/toolbox";
import { createEditorActionsShell } from "./editor/actions-shell";
import {
  editorPreferenceExtensions,
  loadEditorPreferences,
  normalizeTabSize,
  saveEditorPreferences,
  type EditorPreferences,
} from "./editor/preferences";
import { createUtilityInsertions } from "./editor/utility-insertions";
import {
  breakpoints, debugLine, runtimeError, setDebugLine,
} from "./editor/decorations";

let tabsShell: ReturnType<typeof createTabsShell>;
const toolchainState = createToolchainState();

function activeTab() { return tabsShell.activeTab(); }
function isDirty(tab: Tab) { return tabsShell.isDirty(tab); }
function newTab(...args: Parameters<typeof tabsShell.newTab>) { return tabsShell.newTab(...args); }
function closeTab(id: number) { return tabsShell.closeTab(id); }
let renderStatusBar: () => void = () => {};
/** Modo visual das telas `.design` (ADR 0018); montado depois do Explorador. */
let designer: ReturnType<typeof mountDesignerShell> | null = null;
function renderTabs() {
  tabsShell.renderTabs();
  renderStatusBar();
}
function renderCursor() {
  tabsShell.renderCursor();
  renderStatusBar();
}

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------


let editorPreferences: EditorPreferences = loadEditorPreferences(localStorage);

const {
  theme,
  preferences,
  sourceLanguage,
  portugolTools,
  centerSource,
  breakpointGutter,
  portugolLinter,
  markupLinter,
  rustSourceLinter,
  view,
  rustPreview,
  openHelp,
  learningPanel,
  extensions,
  applyTheme,
} = createEditorShell({
  parent: $("editor"),
  setup: {
    storage: () => localStorage,
    hasActiveTab: () => activeTab() !== undefined,
    centerSelection,
    setCenterPressed: (pressed) => $("editor-center").setAttribute("aria-pressed", String(pressed)),
    syncBreakpoints: (state) => syncBreakpoints(state),
    bugIcon: () => icon("bug"),
    activeTab,
    inspectPortugolRs: api.inspectPortugolRs,
    inspectMarkup: api.inspectMarkup,
    renderMessages,
    showOutline: (symbols) => outline.show(symbols),
    updateLibraryButtons,
    isRustcAvailable: () => toolchainState.rustcAvailable() === true,
    checkRustSource: api.checkRustSource,
  },
  activeTab,
  learningMode: () => learningMode,
  preview: {
    activeTab,
    learningMode: () => learningMode,
    goTo,
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
  },
  help: {
    openExample: (source, file) => newTab(source, null, file),
    runExample: (source, file) => {
      newTab(source, null, file);
      void runProgram(false, false);
    },
    openOwnershipLab: () => openOwnershipLabView(),
    isActive: () => tabCollection.activeId() === helpId,
  },
  learningPanel: {
    openPorsExercise: async (exerciseId) => { await exerciseWorkbench?.openPorsById(exerciseId); },
    openPorsExample: async (fileName) => {
      const examples = await api.listPorsExamples();
      const example = examples.find((item) => item.file_name === fileName);
      if (!example) throw new Error(`Exemplo ${fileName} não encontrado`);
      newTab(example.source, null, example.file_name, undefined, undefined, "portugol-rs");
    },
    openOwnershipLab: () => openOwnershipLabView(),
    ownership: {
      isRustcAvailable: () => toolchainState.rustcAvailable() === true,
      checkRustSource: api.checkRustSource,
      openRustScenario: (source, fileName) => {
        const tab = newTab(source, null, fileName, undefined, undefined, "rust");
        setStatus(`Cenário "${fileName}" aberto em uma nova aba .rs para testar com rustc.`, "ok");
        view.focus();
        renderTabs();
        syncLanguageControls(tab);
      },
    },
  },
  update: {
    renderMessages,
    renderExerciseTests,
    renderTabs,
    markPreviewStale: markRustPreviewStale,
    schedulePreview: () => rustPreview.schedule(),
    selectionChanged: () => learningPanel.selectionChanged(),
    learningPanelVisible: () => !$<HTMLElement>("learning-panel").hidden,
    renderLearningPanel: () => learningPanel.render(),
    renderCursor,
    renderMappingGutter: () => rustPreview.renderMappingGutter(),
    centerSelection,
    designerChanged: (docChanged, selectionSet) => designer?.editorChanged(docChanged, selectionSet),
  },
  navigation: { goToDeclarationAt },
  extensions: {
    editorPreferences: () => editorPreferenceExtensions(editorPreferences),
    portugolExtensions: () => [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine],
    rustExtensions: () => [rustSourceLinter],
    markupExtensions: () => [markupLinter],
    portugolRsLanguage: () => portugolRs(),
    rustLanguage: () => rust(),
    markupLanguage: () => markup(),
    findDialog,
    editorSearchKeys,
  },
  theme: { applyPageTheme, onSystemThemeChange },
});

/** Linguagem CodeMirror de cada tipo de documento. */
function languageSupportFor(language: SourceLanguage) {
  if (language === "rust") return rust();
  if (language === "markup") return markup();
  return portugolRs();
}

/** Linter e ferramentas de cada tipo de documento; marcação não tem depuração. */
function languageToolsFor(language: SourceLanguage) {
  if (language === "rust") return [rustSourceLinter];
  if (language === "markup") return [markupLinter];
  return [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine];
}

// ---------------------------------------------------------------------------
// Abas
// ---------------------------------------------------------------------------

const homeId = 0;
const helpId = -1;
let helpOpen = false;
let learningMode: LearningMode = loadLearningMode();
let exerciseWorkbench: ReturnType<typeof createExerciseWorkbenchShell> | null = null;
function activate(id: number) {
  if (id > 0) lastCodeTabId = id;
  try {
    return tabActivator.activate(id);
  } finally {
    designer?.sync();
  }
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

const workspaceFilesShell = createWorkspaceFilesShell({
  state: {
    tabs: tabCollection.tabs,
    replaceTabs: (next) => tabCollection.replace(next),
    activeTab,
    activate,
    newWorkspaceTab: () => { newTab(); },
    newFileTab: (text, path) => { newTab(text, path); },
    isDirty,
  },
  workspace: {
    activeId: tabCollection.activeId,
    homeId,
    openWorkspaceSource: (source, path, workspaceFile) =>
      newTab(source, path, undefined, undefined, undefined, undefined, workspaceFile),
    openExercise,
    renderTabs,
    setExplorerVisible: (visible) => setWorkspaceExplorerVisible(visible),
    cargoProjectDetected: (tab, checked) => {
      if (activeTab() !== tab) return;
      syncLanguageControls(tab);
      if (checked) forceLinting(view);
    },
    screenOpened: () => designer?.screenOpened(),
  },
  files: {
    readFile: api.readFile,
    openDialog,
    saveDialog,
    writeFile: api.writeFile,
    writeWorkspaceFile: api.writeWorkspaceFile,
    currentText: () => view.state.doc.toString(),
    detectSourceLanguage,
    ensureSourceExtension,
    renderTabs,
    setStatus,
    afterSave: (tab, _previousLanguage, languageChanged) => {
      if (languageChanged && tab.rustPreview?.artifact) {
        tab.rustPreview = { ...tab.rustPreview, pending: false, stale: true, error: undefined };
      }
      if (languageChanged) tab.diagnosticsAnalysis = undefined;
      if (tab.id === tabCollection.activeId()) {
        view.dispatch({ effects: [
          sourceLanguage.reconfigure(languageSupportFor(tab.language)),
          portugolTools.reconfigure(languageToolsFor(tab.language)),
        ] });
        syncLanguageControls(tab);
        if (languageChanged) {
          rustPreview.cancel();
          $<HTMLElement>("rust-preview").hidden = learningMode !== "bridge" || tab.language !== "portugol-rs";
          rustPreview.render(tab);
          if (learningMode === "bridge" && tab.language === "portugol-rs") rustPreview.schedule();
          else rustPreview.renderMappingGutter();
        }
        renderStatusBar();
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
  },
  recentView: {
    recent: $("recent"),
    list: $("recent-files"),
    document,
    icon,
  },
});
const { fileIO, recentFiles, recentFilesView, workspaceExplorer } = workspaceFilesShell;

designer = mountDesignerShell({
  area: document.querySelector<HTMLElement>(".editor-area")!,
  toolbar: $("design-toolbar"),
  surface: $("design-surface"),
  viewport: $("design-viewport"),
  status: $("design-status"),
  buttons: {
    design: $<HTMLButtonElement>("design-view-design"),
    split: $<HTMLButtonElement>("design-view-split"),
    text: $<HTMLButtonElement>("design-view-text"),
    code: $<HTMLButtonElement>("design-view-code"),
  },
  activeTab,
  source: () => view.state.doc.toString(),
  cursorOffset: () => view.state.selection.main.head,
  selectInEditor: (from) => {
    const anchor = Math.min(from, view.state.doc.length);
    view.dispatch({ selection: { anchor }, scrollIntoView: true });
  },
  designTree: api.designTree,
  designSchema: api.designSchema,
  editDesign: api.editDesign,
  replaceSource: (source) => {
    const change = textChange(view.state.doc.toString(), source);
    if (change) view.dispatch({ changes: change });
  },
  mountInteractions: mountCanvasInteractions,
  hasFile: (workspaceId, path) => workspaceExplorer.hasFile(workspaceId, path),
  openFile: (workspaceId, path) => workspaceExplorer.openFile(workspaceId, path),
  setStatus,
});
void api.designSchema().then((schema) => {
  renderToolbox($("design-toolbox"), schema, (tag) => void designer?.insertTool(tag));
}).catch(() => undefined);

// ---------------------------------------------------------------------------
// Execução e console
// ---------------------------------------------------------------------------

const consoleBox = $("console");
const inputForm = $<HTMLFormElement>("console-form");
const inputField = $<HTMLInputElement>("console-input");

const runShell = createRunShell({
  activeTab,
  tabs: tabCollection.tabs,
  activeTabId: tabCollection.activeId,
  view,
  activate,
  goTo,
  rustSourceMap: () => activeTab()?.rustPreview?.artifact?.sourceMap ?? [],
  rustcAvailable: toolchainState.rustcAvailable,
  cargoAvailable: toolchainState.cargoAvailable,
  isDirty,
  save: () => fileIO.save(),
  syncLanguageControls,
  showPanel,
  showUtilities: (visible) => showUtilities(visible),
  setStatus,
  consoleBox,
  inputForm,
  inputField,
  clearConsole,
  appendConsole,
  appendProcessOutput,
  consoleText,
  copyText: (text) => navigator.clipboard.writeText(text),
  sendInput: api.sendInput,
  pause: (id) => { void api.debugPause(id); },
  debugContinue: api.debugContinue,
  debugStepOver: api.debugStepOver,
  debugStepInto: api.debugStepInto,
  debugStepOut: api.debugStepOut,
  renderMessages,
  renderExerciseTests,
});
const {
  runState,
  syncBreakpoints,
  debugCommand,
  runProgram,
  runOrContinue,
  stepOrStart,
  stopProgram,
  checkCargoProject,
  runCargoTests,
} = runShell;

const bottomPanelNames: PanelName[] = ["console", "tests", "messages", "debug"];

const panelNavigation = createPanelNavigation({
  bottomPanels: bottomPanelNames,
  inspectorElements: ["utilities-insert", "outline", "variable-inspector"],
  setInspectorElementHidden: (id, hidden) => { $<HTMLElement>(id).hidden = hidden; },
  setInspectorTitle: (title) => { $("utilities-title").textContent = title; },
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

const statusBar = mountStatusBar({
  activeTab,
  source: () => view.state.doc.toString(),
  tabSize: () => view.state.facet(EditorState.tabSize),
  learningMode: () => learningMode,
  rustcAvailable: toolchainState.rustcAvailable,
  openMessages: () => showPanel("messages"),
});
renderStatusBar = statusBar.render;

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
  renderStatusBar();
}

function renderExerciseTests() {
  exerciseWorkbench?.renderTests();
}

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
  porsLibraries: PORS_MODULES,
  porsSnippets: PORS_SNIPPETS,
  createButton: () => document.createElement("button"),
  clearLibraryButtons: () => $("utilities-libraries").replaceChildren(),
  clearSnippetButtons: () => $("utilities-models").replaceChildren(),
  appendLibraryButton: (button) => $("utilities-libraries").append(button),
  appendSnippetButton: (button) => $("utilities-models").append(button),
  activeTab,
  isRunning: (tab) => tab.id === runState.runTabId && runState.runId !== null,
  insertText: (text) => {
    const { from, to } = view.state.selection.main;
    view.dispatch({ changes: { from, to, insert: text }, selection: { anchor: from + text.length } });
  },
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
      preferences.reconfigure(editorPreferenceExtensions(editorPreferences)),
      sourceLanguage.reconfigure(languageSupportFor(tab.language)),
      portugolTools.reconfigure(languageToolsFor(tab.language)),
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
});
$("home-open").onclick = () => void fileIO.openFiles();
$("btn-open").onclick = () => void fileIO.openFiles();
const editorOptions = $<HTMLDetailsElement>("editor-options");
const consoleOptions = $<HTMLDetailsElement>("console-tools");

const outlineOptions = $<HTMLDetailsElement>("outline-options");
const homeNewMenu = $<HTMLDetailsElement>("home-new-menu");
const optionMenus = [editorOptions, consoleOptions, outlineOptions, homeNewMenu];
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
  template: PORS_TEMPLATE,
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
const settingsDialog = $<HTMLDialogElement>("settings-dialog");
function openSettings(focusId = "theme") {
  editorOptions.open = false;
  if (!settingsDialog.open) settingsDialog.showModal();
  $(focusId)?.focus();
}
function resetLayout() {
  panes.reset();
  rustPreview.resetComparisonBasis();
  setStatus("Layout dos painéis restaurado para o padrão.", "ok");
}
$("settings-reset-layout").onclick = resetLayout;

editorMenuAction("editor-exercise", () => exerciseWorkbench?.gradeActiveExercise());
editorMenuAction("editor-search", () => openSearchPanel(view));
editorMenuAction("editor-definition", goToDeclarationSelection);
editorMenuAction("editor-rename", renameSelection);
editorMenuAction("editor-format", formatCode);
editorMenuAction("editor-center", toggleCenterSource);
editorMenuAction("editor-expand", toggleExpanded);
editorMenuAction("btn-utilities", toggleUtilities);
editorMenuAction("editor-open-settings", () => openSettings("theme"));
$("btn-unexpand").onclick = toggleExpanded;

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Node)) return;
  for (const options of optionMenus) {
    if (options.open && !options.contains(event.target)) options.open = false;
  }
});
let activityBarControls: { setPressed: (activity: string | null) => void } | null = null;
let lastCodeTabId: number | null = null;
function returnToEditor() {
  const openTabs = tabCollection.tabs();
  if (!openTabs.length) {
    activate(homeId);
    return;
  }
  const target = openTabs.find(({ id }) => id === lastCodeTabId) ?? openTabs[0];
  activate(target.id);
}
function openOwnershipLabView() {
  if (!tabCollection.tabs().length) newTab();
  else returnToEditor();
  if (learningMode === "beginner") applyLearningMode("bridge");
  showPanel("learning");
  activityBarControls?.setPressed("learn");
  learningPanel.selectConcept("ownership");
  $("ownership-lab").scrollIntoView?.({ block: "start" });
}
function applyEditorPreferences(next: EditorPreferences) {
  editorPreferences = next;
  saveEditorPreferences(localStorage, next);
  $<HTMLInputElement>("editor-word-wrap").checked = next.wordWrap;
  $<HTMLSelectElement>("editor-tab-size").value = String(next.tabSize);
  const effect = preferences.reconfigure(editorPreferenceExtensions(next));
  const current = activeTab();
  if (current) {
    view.dispatch({ effects: effect });
    current.state = view.state;
  }
  for (const tab of tabCollection.tabs()) {
    if (tab !== current) tab.state = tab.state.update({ effects: effect }).state;
  }
  renderStatusBar();
}
settingsDialog.addEventListener("close", () => {
  const learningOpen = panes.utilitiesVisible() && !$<HTMLElement>("learning-panel").hidden;
  activityBarControls?.setPressed(learningOpen ? "learn" : null);
});
$("utilities-close").onclick = () => {
  showUtilities(false);
  activityBarControls?.setPressed(null);
  view.focus();
};
$("btn-save").onclick = () => void fileIO.save();
$("btn-save-as").onclick = () => void fileIO.save(true);
const applyLearningMode = mountLearningModeShell({
  initialMode: learningMode,
  currentMode: () => learningMode,
  rustcAvailable: toolchainState.rustcAvailable,
  activeTab,
  setMode: (mode) => { learningMode = mode; renderStatusBar(); },
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
activityBarControls = mountActivityBar(document.querySelectorAll<HTMLButtonElement>("[data-activity]"), {
  setInspectorView,
  showPanel,
  activateHome: () => {
    activityBarControls?.setPressed(null);
    activate(homeId);
  },
  showUtilities,
  openSettings: () => openSettings("theme"),
  isLearningActive: () => panes.utilitiesVisible() && !$<HTMLElement>("learning-panel").hidden,
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
function openContextualHelp(explicitTopicId?: string) {
  const active = activeTab();
  const topicId = explicitTopicId
    ?? (active?.language === "portugol-rs"
      ? resolveHelpTopicForEditor(view.state.doc.toString(), view.state.selection.main.head) ?? undefined
      : undefined);
  activate(helpId);
  if (topicId) void openHelp(topicId);
}
view.dom.addEventListener("pors-open-help", (event) => {
  const detail = (event as CustomEvent<{ topicId?: string }>).detail;
  openContextualHelp(detail?.topicId);
});
mountIdeCommandShell({
  paletteContext,
  optionMenus,
  actions: {
    runOrContinue,
    stepOrStart,
    stopProgram,
    pauseDebug: () => { if (runState.runId !== null && !runState.porsRunPending) void api.debugPause(runState.runId); },
    continueDebug: () => debugCommand(api.debugContinue),
    stepOver: () => debugCommand(api.debugStepOver),
    stepInto: () => debugCommand(api.debugStepInto),
    stepOut: () => debugCommand(api.debugStepOut),
    help: () => openContextualHelp(),
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
      openSettings("args");
    },
    openSettings: () => openSettings("theme"),
    resetLayout,
    openOwnershipLab: openOwnershipLabView,
    toggleRustPreview: () => $<HTMLButtonElement>("btn-rust").click(),
    toggleUiPreview: () => $<HTMLButtonElement>("btn-ui-preview").click(),
    openFolder: () => $<HTMLButtonElement>("home-open-folder").click(),
    newDesktopProject: () => $<HTMLButtonElement>("home-new-project").click(),
    toggleScreenCode: () => designer?.toggleCode(),
    newRustFile: () => newTab("fn main() {\n    \n}\n", null, undefined, undefined, undefined, "rust"),
    openExercises: () => $<HTMLButtonElement>("home-exercises").click(),
    openThemeEditor: () => {
      if (settingsDialog.open) settingsDialog.close();
      themeEditor.open();
    },
    openLearningPanel: () => {
      showPanel("learning");
      activityBarControls?.setPressed("learn");
    },
    showConsolePanel: () => showPanel("console"),
    showMessagesPanel: () => showPanel("messages"),
    showTestsPanel: () => showPanel("tests"),
    showDebugPanel: () => showPanel("debug"),
    clearConsoleAction: () => $<HTMLButtonElement>("btn-clear").click(),
    copyConsoleAction: () => $<HTMLButtonElement>("btn-copy").click(),
    toggleWordWrap: () => {
      applyEditorPreferences({ ...editorPreferences, wordWrap: !editorPreferences.wordWrap });
      setStatus(editorPreferences.wordWrap ? "Quebra de linha ativada." : "Quebra de linha desativada.", "ok");
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
$("home-help").onclick = () => activate(helpId);
$("btn-help").onclick = () => openContextualHelp();
$("home-return-editor").onclick = $("help-back-editor").onclick = returnToEditor;
$("help-go-home").onclick = () => activate(homeId);
$("help-close").onclick = () => void closeTab(helpId);

// Confirma antes de fechar a janela com alterações não salvas.
mountWindowControls(() => {
  const current = activeTab();
  if (current) current.state = view.state;
  return tabCollection.tabs().filter(isDirty).map(tabName);
});

const themeSelect = $<HTMLSelectElement>("theme");
const themeEditor = mountThemeEditor(themeSelect, applyTheme);
themeSelect.value = themeChoice();
themeSelect.onchange = () => applyTheme(themeSelect.value);
$("editor-themes").onclick = () => {
  if (settingsDialog.open) settingsDialog.close();
  themeEditor.open();
};

mountFontSizeControls(["editor-font", "console-font"]);
{
  const consoleFont = $<HTMLInputElement>("console-font");
  const settingsConsoleFont = $<HTMLInputElement>("settings-console-font");
  const syncConsoleFont = (source: HTMLInputElement, target: HTMLInputElement) => {
    target.value = source.value;
  };
  syncConsoleFont(consoleFont, settingsConsoleFont);
  consoleFont.addEventListener("change", () => syncConsoleFont(consoleFont, settingsConsoleFont));
  settingsConsoleFont.onchange = () => {
    consoleFont.value = settingsConsoleFont.value;
    consoleFont.dispatchEvent(new Event("change"));
    syncConsoleFont(consoleFont, settingsConsoleFont);
  };

  const wordWrapInput = $<HTMLInputElement>("editor-word-wrap");
  const tabSizeSelect = $<HTMLSelectElement>("editor-tab-size");
  wordWrapInput.checked = editorPreferences.wordWrap;
  tabSizeSelect.value = String(editorPreferences.tabSize);
  wordWrapInput.onchange = () => applyEditorPreferences({ ...editorPreferences, wordWrap: wordWrapInput.checked });
  tabSizeSelect.onchange = () => applyEditorPreferences({ ...editorPreferences, tabSize: normalizeTabSize(tabSizeSelect.value) });
}

mountIcons();
recentFilesView.render(recentFiles.list());
mountExamples((source, file, workPath) =>
  newTab(
    source,
    null,
    file,
    workPath || undefined,
    undefined,
    "portugol-rs",
  ),
);
activate(0);
applyTheme(themeChoice());
showPanel("console");
setWorkspaceExplorerVisible(window.innerWidth >= 1440);
showUtilities(window.innerWidth >= 1440);
mountToolchainStatusShell({
  state: toolchainState,
  restoreLearningMode: () => {
    applyLearningMode(loadLearningMode(undefined, activeTab()?.workspacePreferenceKey), false);
    renderStatusBar();
  },
  activeTab,
  syncLanguageControls,
  forceRustLint: () => forceLinting(view),
});
const tips = mountTips(() => {
  const tipsDialog = $<HTMLDialogElement>("tips");
  if (tipsDialog.open) tipsDialog.close();
  themeEditor.open();
});
$("home-tips").onclick = tips.open;
{
  const tipsStartup = $<HTMLInputElement>("tips-startup");
  const settingsTipsStartup = $<HTMLInputElement>("settings-tips-startup");
  settingsTipsStartup.checked = tipsStartup.checked;
  tipsStartup.addEventListener("change", () => { settingsTipsStartup.checked = tipsStartup.checked; });
  settingsTipsStartup.onchange = () => {
    tipsStartup.checked = settingsTipsStartup.checked;
    tipsStartup.dispatchEvent(new Event("change"));
  };
}


mountSystemFilesShell({
  readExercise,
  openExercise,
  openFile: (path) => fileIO.openFile(path),
  setStatus,
});
