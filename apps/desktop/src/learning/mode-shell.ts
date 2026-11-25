import type { LearningMode } from "../learning-mode";
import { saveLearningMode } from "../learning-mode";
import { $ } from "../dom";
import { createLearningModeController } from "./mode-controller";

export interface LearningModeShellDependencies {
  initialMode: LearningMode;
  currentMode: () => LearningMode;
  rustcAvailable: () => boolean | null;
  activeTab: () => { language: string; workspacePreferenceKey?: string } | undefined;
  setMode: (mode: LearningMode) => void;
  panelVisible: () => boolean;
  renderPanel: () => void;
  setPreviewHidden: (hidden: boolean) => void;
  renderMappingGutter: () => void;
  setBridgePressed: (pressed: boolean) => void;
  schedulePreview: () => void;
  cancelPreview: () => void;
  focusEditor: () => void;
}

/** Compõe seleção, persistência e controles visuais do modo de aprendizado. */
export function mountLearningModeShell(dependencies: LearningModeShellDependencies) {
  const modeSelectors = [
    $<HTMLSelectElement>("learning-mode"),
    document.querySelector<HTMLSelectElement>("#titlebar-learning-mode"),
  ].filter((element): element is HTMLSelectElement => element !== null);

  const applyLearningMode = createLearningModeController({
    rustcAvailable: dependencies.rustcAvailable,
    activeTab: dependencies.activeTab,
    setMode: dependencies.setMode,
    setSelectorValue: (mode) => {
      for (const selector of modeSelectors) selector.value = mode;
    },
    persist: (mode, preferenceKey) => saveLearningMode(mode, undefined, preferenceKey),
    panelVisible: dependencies.panelVisible,
    renderPanel: dependencies.renderPanel,
    setPreviewHidden: dependencies.setPreviewHidden,
    renderMappingGutter: dependencies.renderMappingGutter,
    setBridgePressed: dependencies.setBridgePressed,
    schedulePreview: dependencies.schedulePreview,
    cancelPreview: dependencies.cancelPreview,
  });

  for (const selector of modeSelectors) {
    selector.onchange = (event) => {
      const selected = (event.currentTarget as HTMLSelectElement).value;
      if (selected === "beginner" || selected === "bridge" || selected === "rust") applyLearningMode(selected);
    };
  }
  applyLearningMode(dependencies.initialMode, false);
  $<HTMLButtonElement>("btn-rust").onclick = () => {
    applyLearningMode(dependencies.currentMode() === "bridge" ? "beginner" : "bridge");
  };
  $<HTMLButtonElement>("rust-close").onclick = () => {
    applyLearningMode("beginner");
    dependencies.focusEditor();
  };

  return applyLearningMode;
}
