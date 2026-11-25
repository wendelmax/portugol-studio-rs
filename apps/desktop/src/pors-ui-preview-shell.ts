import * as api from "./api";
import { $ } from "./dom";
import { PORS_UI_EXAMPLES } from "./pors-ui-examples";
import { mountPorsUiPreviewPanel } from "./pors-ui-preview-panel";
import { mountUiPreview } from "./ui-preview-renderer";

export interface PorsUiPreviewShellDependencies {
  activeTab: () => { language: string } | undefined;
  source: () => string;
  openExample: (source: string, fileName: string) => void;
  focusEditor: () => void;
}

/** Monta a prévia visual PORS com elementos e comandos da aplicação desktop. */
export function mountPorsUiPreviewShell(dependencies: PorsUiPreviewShellDependencies) {
  return mountPorsUiPreviewPanel({
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
      const tab = dependencies.activeTab();
      if (!tab) return null;
      return { source: dependencies.source(), isPors: tab.language === "portugol-rs" };
    },
    onLoadExample: (example) => dependencies.openExample(example.source, example.fileName),
    onCloseFocus: dependencies.focusEditor,
  });
}
