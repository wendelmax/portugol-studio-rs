import * as api from "./api";
import { $ } from "./dom";
import { FERRO_UI_EXAMPLES } from "./ferro-ui-examples";
import { mountFerroUiPreviewPanel } from "./ferro-ui-preview-panel";
import { mountUiPreview } from "./ui-preview-renderer";

export interface FerroUiPreviewShellDependencies {
  activeTab: () => { language: string } | undefined;
  source: () => string;
  openExample: (source: string, fileName: string) => void;
  focusEditor: () => void;
  /** Numa tela, a prévia visual é o próprio designer; devolve falso fora de telas. */
  showScreen?: () => Promise<boolean>;
  /** Se a aba ativa é o `.design` ou o código de uma tela. */
  isScreen?: () => boolean;
}

/** Monta a prévia visual FERRO com elementos e comandos da aplicação desktop. */
export function mountFerroUiPreviewShell(dependencies: FerroUiPreviewShellDependencies) {
  // Registrado antes do painel: numa tela, o botão abre o designer e o painel
  // do UI.* (obsoleto) não abre.
  const toggle = $<HTMLButtonElement>("btn-ui-preview");
  toggle.addEventListener("click", (event) => {
    if (!dependencies.isScreen?.()) return;
    event.stopImmediatePropagation();
    void dependencies.showScreen?.();
  });
  return mountFerroUiPreviewPanel({
    elements: {
      panel: $<HTMLElement>("ferro-ui-preview"),
      toggleButton: $<HTMLButtonElement>("btn-ui-preview"),
      runButton: $<HTMLButtonElement>("ferro-ui-run"),
      restartButton: $<HTMLButtonElement>("ferro-ui-restart"),
      cancelButton: $<HTMLButtonElement>("ferro-ui-cancel"),
      closeButton: $<HTMLButtonElement>("ferro-ui-close"),
      exampleSelect: $<HTMLSelectElement>("ferro-ui-example-select"),
      loadExampleButton: $<HTMLButtonElement>("ferro-ui-load-example"),
      status: $<HTMLElement>("ferro-ui-status"),
      canvas: $<HTMLElement>("ferro-ui-canvas"),
    },
    backend: {
      startUi: (source, budget) => api.ferroUiStart(source, budget),
      dispatchUi: (sessionId, event) => api.ferroUiDispatch(sessionId, event),
      cancelUi: (sessionId) => api.ferroUiCancel(sessionId),
      closeUi: (sessionId) => api.ferroUiClose(sessionId),
    },
    mount: mountUiPreview,
    examples: FERRO_UI_EXAMPLES,
    getActiveSource: () => {
      const tab = dependencies.activeTab();
      if (!tab) return null;
      return { source: dependencies.source(), isFerro: tab.language === "ferro-rust" };
    },
    onLoadExample: (example) => dependencies.openExample(example.source, example.fileName),
    onCloseFocus: dependencies.focusEditor,
  });
}
