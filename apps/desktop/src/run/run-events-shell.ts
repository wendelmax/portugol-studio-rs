import * as api from "../api";
import { setDebugLine, setRuntimeError } from "../editor/decorations";
import { $ } from "../dom";
import type { EditorView } from "@codemirror/view";
import { createRunEventsController, type RunEventsDependencies } from "./run-events";
import type { ExecutionState } from "./execution-state";

export interface RunEventsShellDependencies {
  state: ExecutionState;
  activeTabId: () => number;
  setPausedInfo: RunEventsDependencies["setPausedInfo"];
  setRunning: (running: boolean) => void;
  view: EditorView;
  consoleBox: HTMLElement;
  inputForm: HTMLFormElement;
  inputField: HTMLInputElement;
  appendConsole: RunEventsDependencies["appendConsole"];
  clearConsole: () => void;
  setStatus: RunEventsDependencies["setStatus"];
  goTo: (offset: number) => void;
  activeTab: () => { id: number } | undefined;
}

/** Liga o controlador de eventos de execução ao estado, editor, console e API. */
export function mountRunEventsShell(dependencies: RunEventsShellDependencies) {
  const { state } = dependencies;
  const { handlers } = createRunEventsController({
    runId: () => state.runId,
    setRunId: (id) => { state.runId = id; },
    runTabId: () => state.runTabId,
    setRunTabId: (id) => { state.runTabId = id; },
    activeTabId: dependencies.activeTabId,
    pendingEvents: () => state.pendingEvents,
    setPaused: (info) => { state.paused = info; },
    setPausedInfo: dependencies.setPausedInfo,
    setRunning: dependencies.setRunning,
    setDebugLine: (line) => dependencies.view.dispatch({ effects: setDebugLine.of(line) }),
    setRuntimeError: (range) => dependencies.view.dispatch({ effects: setRuntimeError.of(range) }),
    appendConsole: dependencies.appendConsole,
    clearConsole: dependencies.clearConsole,
    setInputHint: (text) => { $("console-hint").textContent = text; },
    showInput: () => { dependencies.inputForm.hidden = false; },
    clearInput: () => { dependencies.inputField.value = ""; },
    focusInput: () => { if (dependencies.activeTab()) dependencies.inputField.focus(); },
    scrollConsole: () => { dependencies.consoleBox.scrollTop = dependencies.consoleBox.scrollHeight; },
    setStatus: dependencies.setStatus,
    goTo: dependencies.goTo,
  });

  return api.listenRun(handlers);
}
