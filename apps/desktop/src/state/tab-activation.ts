import type { EditorState } from "@codemirror/state";
import type { Tab } from "./tab";

export interface TabActivationDependencies {
  tabs: () => Tab[];
  currentTab: () => Tab | undefined;
  setActiveId: (id: number) => void;
  editorState: () => EditorState;
  homeId: number;
  helpId: number;
  setHelpOpen: (open: boolean) => void;
  setPageVisibility: (id: number, tab: Tab | undefined) => void;
  cancelPreview: () => void;
  renderTabs: () => void;
  renderExerciseTests: () => void;
  clearCursor: () => void;
  openHelp: () => void;
  focusHome: () => void;
  selectWorkspace: (tab: Tab) => void;
  setEditorState: (tab: Tab) => void;
  loadPanes: (tab: Tab) => void;
  applyLearningMode: (tab: Tab) => void;
  configureEditor: (tab: Tab) => void;
  applyPanes: () => void;
  clearRustDiagnostics: (tab: Tab) => void;
  setRustPreviewHidden: (hidden: boolean) => void;
  renderRustPreview: (tab: Tab) => void;
  renderMappingGutter: () => void;
  renderMessages: () => void;
  showOutline: (tab: Tab, changed: boolean) => void;
  updateLibraryButtons: (tab: Tab) => void;
  renderCursor: () => void;
  centerSelectionEnabled: () => boolean;
  centerSelection: () => void;
  bridgeMode: () => boolean;
  schedulePreview: (tab: Tab) => void;
  syncLanguageControls: (tab: Tab) => void;
  focusEditor: () => void;
}

/** Persiste a aba atual e coordena a troca de aba com os módulos da IDE. */
export function createTabActivator(dependencies: TabActivationDependencies) {
  function activate(id: number): boolean {
    const current = dependencies.currentTab();
    if (current) current.state = dependencies.editorState();
    const tab = dependencies.tabs().find((candidate) => candidate.id === id);
    if (!tab && id !== dependencies.homeId && id !== dependencies.helpId) return false;
    dependencies.setActiveId(id);
    if (id === dependencies.helpId) dependencies.setHelpOpen(true);
    dependencies.setPageVisibility(id, tab);
    if (!tab) {
      dependencies.cancelPreview();
      dependencies.renderTabs();
      dependencies.renderExerciseTests();
      dependencies.clearCursor();
      if (id === dependencies.helpId) dependencies.openHelp();
      else dependencies.focusHome();
      return true;
    }
    dependencies.selectWorkspace(tab);
    dependencies.setEditorState(tab);
    dependencies.loadPanes(tab);
    dependencies.applyLearningMode(tab);
    dependencies.configureEditor(tab);
    dependencies.applyPanes();
    if (tab.language === "rust") dependencies.clearRustDiagnostics(tab);
    dependencies.renderTabs();
    dependencies.renderExerciseTests();
    dependencies.setRustPreviewHidden(!dependencies.bridgeMode() || tab.language !== "ferro-rust");
    if (tab.language === "ferro-rust") dependencies.renderRustPreview(tab);
    dependencies.renderMappingGutter();
    dependencies.renderMessages();
    dependencies.showOutline(tab, current !== tab);
    dependencies.updateLibraryButtons(tab);
    dependencies.renderCursor();
    if (dependencies.centerSelectionEnabled()) dependencies.centerSelection();
    if (dependencies.bridgeMode() && tab.language === "ferro-rust") dependencies.schedulePreview(tab);
    if (tab.language !== "ferro-rust") dependencies.cancelPreview();
    dependencies.syncLanguageControls(tab);
    dependencies.focusEditor();
    return true;
  }

  return { activate };
}
