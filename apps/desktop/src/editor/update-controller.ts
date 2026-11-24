import type { LearningMode } from "../learning-mode";
import type { RustPreviewState } from "../rust-preview-state";
import type { Tab } from "../state/tab";

export interface EditorUpdateEvent {
  docChanged: boolean;
  selectionSet: boolean;
  state: { doc: { toString: () => string } };
}

export interface EditorUpdateDependencies {
  activeTab: () => Tab | undefined;
  currentEditorState: () => unknown;
  renderMessages: () => void;
  renderExerciseTests: () => void;
  renderTabs: () => void;
  markPreviewStale: (preview: RustPreviewState, source: string) => RustPreviewState;
  learningMode: () => LearningMode;
  schedulePreview: () => void;
  selectionChanged: () => void;
  learningPanelVisible: () => boolean;
  renderLearningPanel: () => void;
  renderCursor: () => void;
  renderMappingGutter: () => void;
  centerActive: () => boolean;
  centerSelection: () => void;
}

/** Atualiza as visualizações derivadas após alterações e seleções no editor. */
export function createEditorUpdateController(dependencies: EditorUpdateDependencies) {
  return function handleUpdate(update: EditorUpdateEvent): void {
    const currentTab = dependencies.activeTab();
    if (update.docChanged && currentTab?.runtimeCompileDiagnostics?.length) {
      currentTab.runtimeCompileDiagnostics = [];
      dependencies.renderMessages();
    }
    if (update.docChanged) {
      const tab = dependencies.activeTab();
      if (tab?.exercise) {
        tab.exerciseTestGeneration = (tab.exerciseTestGeneration ?? 0) + 1;
        tab.exerciseTestRun = undefined;
        dependencies.renderExerciseTests();
      }
      dependencies.renderTabs();
    }
    if (update.docChanged && dependencies.activeTab()?.language !== "rust") {
      const tab = dependencies.activeTab();
      if (tab?.rustPreview) {
        tab.rustPreview = dependencies.markPreviewStale(tab.rustPreview, update.state.doc.toString());
      }
      if (dependencies.learningMode() === "bridge") dependencies.schedulePreview();
    }
    if (update.selectionSet) dependencies.selectionChanged();
    if ((update.selectionSet || update.docChanged) && dependencies.learningPanelVisible()) {
      dependencies.renderLearningPanel();
    }
    if (update.selectionSet || update.docChanged) dependencies.renderCursor();
    if (update.selectionSet || update.docChanged) dependencies.renderMappingGutter();
    if (dependencies.centerActive() && update.selectionSet) {
      queueMicrotask(() => {
        if (dependencies.centerActive() && dependencies.currentEditorState() === update.state) {
          dependencies.centerSelection();
        }
      });
    }
  };
}
