// Painel de Preview Visual PORS (ADR 0017) independente do transporte:
// opera sobre o mesmo contrato JSON tanto via Tauri IPC (Desktop) quanto via WASM (Web).

import type { PorsUiExample } from "./pors-ui-examples";
import type {
  MountUiPreviewFn,
  PorsUiBudget,
  PorsUiFailure,
  PorsUiResult,
  PorsWasmRuntime,
} from "./pors-wasm-host";
import type { UiEvent, UiRenderLimits } from "./ui-preview-renderer";

export interface PorsUiPreviewBackend {
  startUi(source: string, budget?: PorsUiBudget): Promise<PorsUiResult> | PorsUiResult;
  dispatchUi(sessionId: number, event: UiEvent): Promise<PorsUiResult> | PorsUiResult;
  closeUi(sessionId: number): Promise<boolean> | boolean;
  cancelUi?(sessionId: number): Promise<boolean> | boolean;
}

export function createWasmUiBackend(runtime: PorsWasmRuntime): PorsUiPreviewBackend {
  return {
    startUi: (source, budget) => runtime.startUi(source, budget),
    dispatchUi: (sessionId, event) => runtime.dispatchUi(sessionId, event),
    closeUi: (sessionId) => runtime.closeUi(sessionId),
    cancelUi: (sessionId) => runtime.closeUi(sessionId),
  };
}

export interface PorsUiPreviewPanelElements {
  readonly panel: HTMLElement;
  readonly toggleButton: HTMLButtonElement;
  readonly runButton: HTMLButtonElement;
  readonly restartButton: HTMLButtonElement;
  readonly cancelButton: HTMLButtonElement;
  readonly closeButton: HTMLButtonElement;
  readonly exampleSelect: HTMLSelectElement;
  readonly loadExampleButton: HTMLButtonElement;
  readonly status: HTMLElement;
  readonly canvas: HTMLElement;
}

export interface PorsUiPreviewPanelOptions {
  readonly elements: PorsUiPreviewPanelElements;
  readonly backend: PorsUiPreviewBackend;
  readonly mount: MountUiPreviewFn;
  readonly examples: readonly PorsUiExample[];
  readonly limits?: Partial<UiRenderLimits>;
  readonly budget?: PorsUiBudget;
  readonly getActiveSource: () => { source: string; isPors: boolean } | null;
  readonly onLoadExample?: (example: PorsUiExample) => void;
  readonly onCloseFocus?: () => void;
}

export interface PorsUiPreviewPanelController {
  open(autoRun?: boolean): Promise<void>;
  close(): Promise<void>;
  toggle(): Promise<void>;
  runCurrentSource(): Promise<PorsUiResult | null>;
  restart(): Promise<PorsUiResult | null>;
  cancel(): Promise<void>;
  destroy(): Promise<void>;
  readonly isOpen: boolean;
  readonly active: boolean;
  readonly sessionId: number | null;
  readonly step: number;
}

function formatErrorMessage(failure: PorsUiFailure): string {
  switch (failure.code) {
    case "time":
      return `Orçamento de passos excedido: ${failure.error}`;
    case "string_memory":
      return `Limite de memória de texto excedido: ${failure.error}`;
    case "output":
      return `Limite de saída excedido: ${failure.error}`;
    case "node_limit":
    case "depth_limit":
    case "render_contract_error":
      return `Limite visual excedido: ${failure.error}`;
    case "missing_function":
      return `${failure.error} Carregue um exemplo visual para começar.`;
    default:
      return failure.error;
  }
}

export function mountPorsUiPreviewPanel(
  options: PorsUiPreviewPanelOptions,
): PorsUiPreviewPanelController {
  const {
    elements,
    backend,
    mount,
    examples,
    limits,
    budget,
    getActiveSource,
    onLoadExample,
    onCloseFocus,
  } = options;

  let destroyed = false;
  let active = false;
  let busy = false;
  let sessionId: number | null = null;
  let step = 0;
  let generation = 0;
  let lastSource: string | null = null;

  const preview = mount(
    elements.canvas,
    (event) => {
      void handleEvent(event);
    },
    limits,
  );

  // Popula seletor de exemplos visuais com textContent seguro.
  const doc = elements.exampleSelect.ownerDocument;
  elements.exampleSelect.replaceChildren();
  for (const example of examples) {
    const option = doc.createElement("option");
    option.value = example.id;
    option.textContent = example.title;
    elements.exampleSelect.appendChild(option);
  }
  if (examples.length > 0) {
    elements.exampleSelect.value = examples[0].id;
  }

  function setStatus(message: string, isError = false) {
    elements.status.textContent = message;
    elements.status.setAttribute("data-error", String(isError));
  }

  function syncButtons() {
    elements.toggleButton.setAttribute("aria-pressed", String(!elements.panel.hidden));
    elements.restartButton.disabled = lastSource === null || busy;
    elements.cancelButton.disabled = sessionId === null && !busy;
    elements.loadExampleButton.disabled = examples.length === 0;
  }

  async function closeActiveSession(useCancel = false) {
    const closingId = sessionId;
    sessionId = null;
    active = false;
    if (closingId !== null) {
      try {
        if (useCancel && typeof backend.cancelUi === "function") {
          await backend.cancelUi(closingId);
        } else {
          await backend.closeUi(closingId);
        }
      } catch {
        // Ignora erro de transporte ao encerrar sessão já fechada.
      }
    }
  }

  async function applyResult(result: PorsUiResult, expectedGeneration: number): Promise<PorsUiResult> {
    if (destroyed || expectedGeneration !== generation) {
      if (result.ok) {
        try {
          await backend.closeUi(result.session);
        } catch {
          // Sessão obsoleta descartada.
        }
      }
      return result;
    }

    busy = false;
    if (!result.ok) {
      active = false;
      preview.setDisabled(true);
      setStatus(formatErrorMessage(result), true);
      syncButtons();
      return result;
    }

    sessionId = result.session;
    try {
      preview.render(result.tree);
    } catch (error) {
      await closeActiveSession(false);
      preview.setDisabled(true);
      const failure: PorsUiFailure = {
        ok: false,
        code: "render_contract_error",
        error: error instanceof Error ? error.message : String(error),
      };
      setStatus(formatErrorMessage(failure), true);
      syncButtons();
      return failure;
    }

    step = result.step;
    active = true;
    preview.setDisabled(false);
    setStatus(`Preview ativo — passo ${step}.`, false);
    syncButtons();
    return result;
  }

  async function runSource(source: string): Promise<PorsUiResult | null> {
    if (destroyed) return null;
    generation += 1;
    const currentGeneration = generation;
    await closeActiveSession(true);
    lastSource = source;
    step = 0;
    busy = true;
    preview.setDisabled(true);
    setStatus("Iniciando preview visual PORS…", false);
    syncButtons();

    try {
      const result = await backend.startUi(source, budget);
      return await applyResult(result, currentGeneration);
    } catch (error) {
      if (destroyed || currentGeneration !== generation) return null;
      busy = false;
      active = false;
      const failure: PorsUiFailure = {
        ok: false,
        code: "transport_error",
        error: error instanceof Error ? error.message : String(error),
      };
      setStatus(formatErrorMessage(failure), true);
      syncButtons();
      return failure;
    }
  }

  async function runCurrentSource(): Promise<PorsUiResult | null> {
    if (destroyed) return null;
    const current = getActiveSource();
    if (!current || !current.isPors) {
      generation += 1;
      await closeActiveSession(true);
      preview.setDisabled(true);
      setStatus(
        "Abra um arquivo .fer com iniciar/atualizar/visualizar ou carregue um exemplo visual acima.",
        false,
      );
      syncButtons();
      return null;
    }
    return runSource(current.source);
  }

  async function handleEvent(event: UiEvent): Promise<PorsUiResult | null> {
    if (destroyed || !active || sessionId === null) return null;
    const currentGeneration = generation;
    const targetSession = sessionId;
    busy = true;
    syncButtons();

    try {
      const result = await backend.dispatchUi(targetSession, event);
      return await applyResult(result, currentGeneration);
    } catch (error) {
      if (destroyed || currentGeneration !== generation) return null;
      busy = false;
      active = false;
      preview.setDisabled(true);
      const failure: PorsUiFailure = {
        ok: false,
        code: "transport_error",
        error: error instanceof Error ? error.message : String(error),
      };
      setStatus(formatErrorMessage(failure), true);
      syncButtons();
      return failure;
    }
  }

  async function restart(): Promise<PorsUiResult | null> {
    if (destroyed || lastSource === null) return null;
    return runSource(lastSource);
  }

  async function cancel(): Promise<void> {
    if (destroyed) return;
    generation += 1;
    busy = false;
    await closeActiveSession(true);
    preview.setDisabled(true);
    setStatus("Preview cancelado.", false);
    syncButtons();
  }

  async function open(autoRun = true): Promise<void> {
    if (destroyed) return;
    elements.panel.hidden = false;
    syncButtons();
    if (autoRun) {
      await runCurrentSource();
    }
  }

  async function close(): Promise<void> {
    if (destroyed) return;
    generation += 1;
    busy = false;
    await closeActiveSession(true);
    preview.setDisabled(true);
    elements.panel.hidden = true;
    syncButtons();
    onCloseFocus?.();
  }

  async function toggle(): Promise<void> {
    if (elements.panel.hidden) {
      await open(true);
    } else {
      await close();
    }
  }

  async function handleLoadExample() {
    const selectedId = elements.exampleSelect.value;
    const selected = examples.find((item) => item.id === selectedId) ?? examples[0];
    if (!selected) return;
    onLoadExample?.(selected);
    if (elements.panel.hidden) {
      elements.panel.hidden = false;
    }
    await runSource(selected.source);
  }

  const onToggleClick = () => {
    void toggle();
  };
  const onRunClick = () => {
    void runCurrentSource();
  };
  const onRestartClick = () => {
    void restart();
  };
  const onCancelClick = () => {
    void cancel();
  };
  const onCloseClick = () => {
    void close();
  };
  const onLoadExampleClick = () => {
    void handleLoadExample();
  };

  elements.toggleButton.addEventListener("click", onToggleClick);
  elements.runButton.addEventListener("click", onRunClick);
  elements.restartButton.addEventListener("click", onRestartClick);
  elements.cancelButton.addEventListener("click", onCancelClick);
  elements.closeButton.addEventListener("click", onCloseClick);
  elements.loadExampleButton.addEventListener("click", onLoadExampleClick);

  syncButtons();

  async function destroy(): Promise<void> {
    if (destroyed) return;
    destroyed = true;
    generation += 1;
    busy = false;
    elements.toggleButton.removeEventListener("click", onToggleClick);
    elements.runButton.removeEventListener("click", onRunClick);
    elements.restartButton.removeEventListener("click", onRestartClick);
    elements.cancelButton.removeEventListener("click", onCancelClick);
    elements.closeButton.removeEventListener("click", onCloseClick);
    elements.loadExampleButton.removeEventListener("click", onLoadExampleClick);
    await closeActiveSession(true);
    preview.destroy();
  }

  return {
    open,
    close,
    toggle,
    runCurrentSource,
    restart,
    cancel,
    destroy,
    get isOpen() {
      return !elements.panel.hidden;
    },
    get active() {
      return active;
    },
    get sessionId() {
      return sessionId;
    },
    get step() {
      return step;
    },
  };
}
