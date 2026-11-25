import type { EditorView } from "@codemirror/view";
import type * as api from "../api";
import { $ } from "../dom";
import type { Tab } from "../state/tab";
import { createExecutionState } from "./execution-state";
import { createExecutionPresentationShell } from "./execution-presentation";
import { createExecutionControllerShell } from "./execution-controller-shell";
import { mountRunEventsShell } from "./run-events-shell";
import { mountRunToolbar } from "./toolbar";

export interface RunShellDependencies {
  activeTab: () => Tab | undefined;
  tabs: () => Tab[];
  activeTabId: () => number;
  view: EditorView;
  activate: (id: number) => void;
  goTo: (offset: number) => void;
  rustcAvailable: () => boolean | null;
  cargoAvailable: () => boolean | null;
  isDirty: (tab: Tab) => boolean;
  save: () => Promise<boolean>;
  syncLanguageControls: (tab: Tab) => void;
  showPanel: (panel: "console" | "tests" | "messages" | "debug") => void;
  showUtilities: (visible: boolean) => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  consoleBox: HTMLElement;
  inputForm: HTMLFormElement;
  inputField: HTMLInputElement;
  clearConsole: () => void;
  appendConsole: (text: string, kind?: string) => void;
  appendProcessOutput: (result: api.PortugolRsRunResult | api.CargoProjectRunResult | api.RustSourceRunResult) => void;
  consoleText: () => string;
  copyText: (text: string) => Promise<void>;
  sendInput: (runId: number, value: string) => Promise<void>;
  pause: (runId: number) => unknown;
  debugContinue: typeof api.debugContinue;
  debugStepOver: typeof api.debugStepOver;
  debugStepInto: typeof api.debugStepInto;
  debugStepOut: typeof api.debugStepOut;
  renderMessages: () => void;
  renderExerciseTests: () => void;
}

/** Compõe estado, eventos, controles e comandos do subsistema de execução. */
export function createRunShell(dependencies: RunShellDependencies) {
  const runState = createExecutionState();
  const presentation = createExecutionPresentationShell({
    activeTab: dependencies.activeTab,
    tabs: dependencies.tabs,
    activeTabId: dependencies.activeTabId,
    runState,
    rustcAvailable: dependencies.rustcAvailable,
    view: dependencies.view,
    activate: dependencies.activate,
    goTo: dependencies.goTo,
    setStatus: dependencies.setStatus,
    showUtilities: dependencies.showUtilities,
    showDebugPanel: () => dependencies.showPanel("debug"),
  });
  let runListeners!: ReturnType<typeof api.listenRun>;
  const controllers = createExecutionControllerShell({
    runState,
    runPresentation: presentation,
    view: dependencies.view,
    tabs: dependencies.tabs,
    activeTab: dependencies.activeTab,
    rustcAvailable: dependencies.rustcAvailable,
    cargoAvailable: dependencies.cargoAvailable,
    isDirty: dependencies.isDirty,
    save: dependencies.save,
    runListeners: () => runListeners,
    syncLanguageControls: dependencies.syncLanguageControls,
    showPanel: dependencies.showPanel,
    setStatus: dependencies.setStatus,
    clearConsole: dependencies.clearConsole,
    appendConsole: dependencies.appendConsole,
    appendProcessOutput: dependencies.appendProcessOutput,
    renderMessages: dependencies.renderMessages,
    renderExerciseTests: dependencies.renderExerciseTests,
  });
  runListeners = mountRunEventsShell({
    state: runState,
    activeTabId: dependencies.activeTabId,
    setPausedInfo: presentation.setPaused,
    setRunning: presentation.setRunning,
    view: dependencies.view,
    consoleBox: dependencies.consoleBox,
    inputForm: dependencies.inputForm,
    inputField: dependencies.inputField,
    appendConsole: dependencies.appendConsole,
    clearConsole: dependencies.clearConsole,
    setStatus: dependencies.setStatus,
    goTo: dependencies.goTo,
    activeTab: dependencies.activeTab,
  });

  mountRunToolbar({
    button: (id) => $<HTMLButtonElement>(id),
    form: dependencies.inputForm,
    input: dependencies.inputField,
    runId: () => runState.runId,
    activeLanguage: () => dependencies.activeTab()?.language ?? null,
    runOrContinue: controllers.runOrContinue,
    stepOrStart: controllers.stepOrStart,
    stop: controllers.stopProgram,
    pause: dependencies.pause,
    continueDebug: () => presentation.debugCommand(dependencies.debugContinue),
    stepOver: () => presentation.debugCommand(dependencies.debugStepOver),
    stepInto: () => presentation.debugCommand(dependencies.debugStepInto),
    stepOut: () => presentation.debugCommand(dependencies.debugStepOut),
    clearConsole: dependencies.clearConsole,
    consoleText: dependencies.consoleText,
    copyText: dependencies.copyText,
    setStatus: dependencies.setStatus,
    showPanel: dependencies.showPanel,
    appendConsole: dependencies.appendConsole,
    sendInput: dependencies.sendInput,
  });

  void runListeners.catch((error) => dependencies.setStatus(String(error), "error"));
  return {
    runState,
    syncBreakpoints: presentation.syncBreakpoints,
    debugCommand: presentation.debugCommand,
    ...controllers,
  };
}
