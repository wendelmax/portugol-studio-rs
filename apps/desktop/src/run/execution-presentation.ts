import type { EditorView } from "@codemirror/view";
import { $ } from "../dom";
import * as api from "../api";
import { canDebugSource, canRunSource } from "../runtime-capabilities";
import type { Tab } from "../state/tab";
import type { ExecutionState } from "./execution-state";
import { createRunControls } from "./controls";
import { createDebuggerShell } from "./debugger-shell";
import { renderDebugInfo } from "./debug-view";
import { breakpointLines, setDebugLine } from "../editor/decorations";

export interface ExecutionPresentationDependencies {
  activeTab: () => Tab | undefined;
  tabs: () => Tab[];
  activeTabId: () => number;
  runState: ExecutionState;
  rustcAvailable: () => boolean | null;
  cargoAvailable?: () => boolean | null;
  view: Pick<EditorView, "dispatch">;
  rustSourceMap?: () => api.SourceMapInfo[];
  activate: (id: number) => void;
  goTo: (offset: number) => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  showUtilities: (visible: boolean) => void;
  showDebugPanel: () => void;
}

/** Liga os controles visuais da execução aos comandos e à apresentação do depurador. */
export function createExecutionPresentationShell(dependencies: ExecutionPresentationDependencies) {
  const { runState } = dependencies;
  const inputForm = $<HTMLFormElement>("console-form");
  const runControls = createRunControls({
    activeLanguage: () => dependencies.activeTab()?.language,
    canRun: (language, rustcAvailable) => {
      if (!canRunSource(language, rustcAvailable)) return false;
      const tab = dependencies.activeTab();
      if (language === "rust" && tab) {
        const cargoDetectionPending = Boolean(tab.workspaceId && tab.workspacePath) && tab.cargoProject === undefined;
        if (cargoDetectionPending) return false;
        if (tab.cargoProject === true && dependencies.cargoAvailable?.() === false) return false;
      }
      return true;
    },
    canDebug: canDebugSource,
    canPause: () => !runState.porsRunPending,
    rustcAvailable: dependencies.rustcAvailable,
    runId: () => runState.runId,
    paused: () => runState.paused,
    setButtonDisabled: (id, disabled) => { $<HTMLButtonElement>(id).disabled = disabled; },
    setInputHidden: (hidden) => { inputForm.hidden = hidden; },
    setDebugVarsHidden: (hidden) => { $("debug-vars").hidden = hidden; },
    setInspectorEmptyHidden: (hidden) => { $("inspector-empty").hidden = hidden; },
    setInspectorText: (text) => { $("inspector-empty").textContent = text; },
  });
  const setRunning = runControls.setRunning;
  const runTab = () => dependencies.tabs().find(({ id }) => id === runState.runTabId);
  const debuggerShell = createDebuggerShell({
    runId: () => runState.runId,
    runTabId: () => runState.runTabId,
    activeTabId: dependencies.activeTabId,
    breakpointLines,
    setBreakpoints: api.debugSetBreakpoints,
    setDebugLine: (line) => dependencies.view.dispatch({ effects: setDebugLine.of(line) }),
    paused: () => runState.paused,
    setPaused: (info) => { runState.paused = info; },
    setRunning,
    runLanguage: () => runTab()?.language ?? dependencies.activeTab()?.language,
    setStatus: dependencies.setStatus,
    stop: api.stop,
    runTab,
    renderDebugInfo,
    rustSourceMap: dependencies.rustSourceMap,
    activate: dependencies.activate,
    goTo: dependencies.goTo,
    showUtilities: dependencies.showUtilities,
    showDebugPanel: dependencies.showDebugPanel,
  });

  return { setRunning, ...debuggerShell };
}
