import type { PanelName } from "../panel-tabs";

export interface PanelNavigationDependencies {
  bottomPanels: readonly PanelName[];
  inspectorElements: readonly string[];
  setInspectorElementHidden: (id: string, hidden: boolean) => void;
  showUtilities: (visible: boolean) => void;
  renderLearningPanel: () => void;
  setTabState: (name: PanelName, active: boolean, selected: boolean) => void;
  setPanelContentHidden: (name: PanelName, hidden: boolean) => void;
  setConsoleToolsHidden: (hidden: boolean) => void;
  renderExerciseTests: () => void;
}

/** Coordinates visible IDE inspector and bottom-panel views. */
export function createPanelNavigation(dependencies: PanelNavigationDependencies) {
  function setInspectorView(learning: boolean) {
    dependencies.setInspectorElementHidden("learning-panel", !learning);
    for (const id of dependencies.inspectorElements) dependencies.setInspectorElementHidden(id, learning);
  }

  function showPanel(which: PanelName) {
    const learningActive = which === "learning";
    setInspectorView(learningActive);
    if (learningActive) {
      dependencies.showUtilities(true);
      dependencies.renderLearningPanel();
      return;
    }
    for (const name of dependencies.bottomPanels) {
      const active = which === name;
      dependencies.setTabState(name, active, active);
      dependencies.setPanelContentHidden(name, !active);
    }
    dependencies.setConsoleToolsHidden(which !== "console");
    if (which === "tests") dependencies.renderExerciseTests();
  }

  return { setInspectorView, showPanel };
}
