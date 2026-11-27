import type { LearningMode } from "../learning-mode";

export interface LearningModeTab {
  language: string;
  workspacePreferenceKey?: string;
}

export interface LearningModeControllerDependencies {
  rustcAvailable: () => boolean | null;
  activeTab: () => LearningModeTab | undefined;
  setMode: (mode: LearningMode) => void;
  setSelectorValue: (mode: LearningMode) => void;
  persist: (mode: LearningMode, preferenceKey?: string) => void;
  panelVisible: () => boolean;
  renderPanel: () => void;
  setPreviewHidden: (hidden: boolean) => void;
  renderMappingGutter: () => void;
  setBridgePressed: (pressed: boolean) => void;
  schedulePreview: () => void;
  cancelPreview: () => void;
}

/** Coordena a seleção do modo e os efeitos do laboratório sem acoplar o fluxo ao DOM. */
export function createLearningModeController(dependencies: LearningModeControllerDependencies) {
  return (mode: LearningMode, persist = true): LearningMode => {
    const availableMode = mode === "rust" && dependencies.rustcAvailable() !== true ? "beginner" : mode;
    dependencies.setMode(availableMode);
    dependencies.setSelectorValue(availableMode);
    if (persist) dependencies.persist(availableMode, dependencies.activeTab()?.workspacePreferenceKey);
    if (dependencies.panelVisible()) dependencies.renderPanel();

    const bridgeEnabled = availableMode === "bridge" && dependencies.activeTab()?.language === "portugol-rs";
    dependencies.setPreviewHidden(!bridgeEnabled);
    dependencies.renderMappingGutter();
    dependencies.setBridgePressed(bridgeEnabled);
    if (bridgeEnabled && dependencies.activeTab()) dependencies.schedulePreview();
    else dependencies.cancelPreview();

    return availableMode;
  };
}
