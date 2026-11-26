import type { EditorView } from "@codemirror/view";
import { setDiagnostics as applyDiagnostics } from "@codemirror/lint";
import { ask } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { sourceDiagnostic } from "../source-map-ui";
import { toCodeMirror, setDebugLine, setRuntimeError } from "../editor/decorations";
import { parseProcessArguments } from "../process-arguments";
import { hasCargoExecutionConsent, hasRustExecutionConsent, rememberCargoExecutionConsent, rememberRustExecutionConsent } from "../rust-execution-consent";
import { replayRunEvents } from "../run-event-router";
import { chooseCargoTarget } from "./cargo-target-dialog";
import { createCargoProjectController } from "./cargo-project";
import { createPortugolRsExecutionController } from "./portugol-rs-execution";
import { createRunDispatcher } from "./dispatcher";
import { createRustExecutionController } from "./rust-execution";
import type { ExecutionState } from "./execution-state";
import { isExecutionBusy } from "./execution-state";
import type { Tab } from "../state/tab";
import type { createExecutionPresentationShell } from "./execution-presentation";
import { breakpointLines } from "../editor/decorations";
import { $ } from "../dom";

type ExecutionPresentation = ReturnType<typeof createExecutionPresentationShell>;

export interface ExecutionControllerShellDependencies {
  runState: ExecutionState;
  runPresentation: ExecutionPresentation;
  view: EditorView;
  tabs: () => Tab[];
  activeTab: () => Tab | undefined;
  rustcAvailable: () => boolean | null;
  cargoAvailable: () => boolean | null;
  isDirty: (tab: Tab) => boolean;
  save: () => Promise<boolean>;
  runListeners: () => ReturnType<typeof api.listenRun>;
  syncLanguageControls: (tab: Tab) => void;
  showPanel: (panel: "console" | "messages" | "debug" | "tests") => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  clearConsole: () => void;
  appendConsole: (text: string) => void;
  appendProcessOutput: (result: api.PortugolRsRunResult | api.CargoProjectRunResult | api.RustSourceRunResult) => void;
  renderMessages: () => void;
  renderExerciseTests: () => void;
}

/** Compõe execução, despacho de linguagens e operações de projetos Cargo. */
export function createExecutionControllerShell(dependencies: ExecutionControllerShellDependencies) {
  const { runState, runPresentation } = dependencies;
  const currentDocument = () => dependencies.view.state.doc;
  const isCurrent = (tab: Tab, documentState: object) =>
    dependencies.activeTab() === tab && dependencies.view.state.doc === documentState;
  const runRustExecution = createRustExecutionController({
    activeTab: dependencies.activeTab,
    tabs: dependencies.tabs,
    isBusy: () => isExecutionBusy(runState),
    rustcAvailable: dependencies.rustcAvailable,
    cargoAvailable: dependencies.cargoAvailable,
    isDirty: dependencies.isDirty,
    save: dependencies.save,
    argumentsText: () => $<HTMLInputElement>("args").value,
    parseArguments: parseProcessArguments,
    currentDocument,
    isCurrent,
    runId: () => runState.runId,
    setRunId: (id) => { runState.runId = id; },
    setPending: (pending) => { runState.porsRunPending = pending; },
    setRunTabId: (id) => { runState.runTabId = id; },
    syncLanguageControls: dependencies.syncLanguageControls,
    setRunning: runPresentation.setRunning,
    hasConsent: (consent) => consent === "cargo"
      ? hasCargoExecutionConsent(sessionStorage)
      : hasRustExecutionConsent(sessionStorage),
    askConsent: (consent) => ask(
      consent === "cargo"
        ? "Cargo pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta execução não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?"
        : "Arquivos Rust compilados e executados podem acessar arquivos, processos e recursos permitidos à sua conta. Esta execução não possui sandbox. Execute apenas código em que você confia. Deseja continuar?",
      {
        title: consent === "cargo" ? "Executar projeto Cargo sem sandbox" : "Executar código Rust sem sandbox",
        kind: "warning",
        okLabel: "Executar Rust",
        cancelLabel: "Cancelar",
      },
    ),
    rememberConsent: (consent) => consent === "cargo"
      ? rememberCargoExecutionConsent(sessionStorage, true)
      : rememberRustExecutionConsent(sessionStorage, true),
    chooseCargoTarget,
    registerRun: api.registerRustSourceRun,
    cargoRunTargets: api.cargoProjectRunTargets,
    runCargoProject: api.runCargoProject,
    runRustSource: api.runRustSource,
    clearRuntimeError: () => dependencies.view.dispatch({ effects: setRuntimeError.of(null) }),
    clearDebugLine: () => dependencies.view.dispatch({ effects: setDebugLine.of(null) }),
    clearPaused: () => { runState.paused = null; },
    showConsole: () => dependencies.showPanel("console"),
    clearConsole: dependencies.clearConsole,
    appendProcessOutput: dependencies.appendProcessOutput,
    appendConsole: dependencies.appendConsole,
    setStatus: dependencies.setStatus,
    renderMessages: dependencies.renderMessages,
    setDiagnostics: (tab, diagnostics) => {
      tab.runtimeCompileDiagnostics = diagnostics;
      dependencies.view.dispatch(applyDiagnostics(dependencies.view.state, diagnostics.map((item) => toCodeMirror(item, dependencies.view.state.doc.length))));
    },
    showMessages: () => dependencies.showPanel("messages"),
  });

  const runPortugolRsExecution = createPortugolRsExecutionController({
    rustcAvailable: dependencies.rustcAvailable,
    isCurrent,
    runId: () => runState.runId,
    setRunId: (id) => { runState.runId = id; },
    setPending: (pending) => { runState.porsRunPending = pending; },
    setRunTabId: (id) => { runState.runTabId = id; },
    setPendingEvents: (events) => { runState.pendingEvents = events; },
    setRunning: runPresentation.setRunning,
    clearRuntimeError: () => dependencies.view.dispatch({ effects: setRuntimeError.of(null) }),
    clearDebugLine: () => dependencies.view.dispatch({ effects: setDebugLine.of(null) }),
    clearPaused: () => { runState.paused = null; },
    showConsole: () => dependencies.showPanel("console"),
    clearConsole: dependencies.clearConsole,
    hideInput: () => { $<HTMLFormElement>("console-form").hidden = true; },
    runListeners: dependencies.runListeners,
    startVm: api.startPortugolRsVm,
    registerRun: api.registerPortugolRsRun,
    runGeneratedRust: api.runPortugolRs,
    setStatus: dependencies.setStatus,
    appendProcessOutput: dependencies.appendProcessOutput,
    appendConsole: dependencies.appendConsole,
    renderMessages: dependencies.renderMessages,
    setDiagnostics: (tab, diagnostics) => {
      tab.runtimeCompileDiagnostics = diagnostics;
      dependencies.view.dispatch(applyDiagnostics(dependencies.view.state, diagnostics.map((item) => toCodeMirror(item, dependencies.view.state.doc.length))));
    },
    setVmDiagnostics: (tab, diagnostics) => { tab.diagnostics = diagnostics as api.DiagnosticInfo[]; },
    showMessages: () => dependencies.showPanel("messages"),
    sourceDiagnostic,
    toCodeMirror,
    replayEvents: replayRunEvents,
  });

  const runDispatcher = createRunDispatcher({
    isBusy: () => isExecutionBusy(runState),
    activeTab: dependencies.activeTab,
    currentDocument,
    editorState: () => dependencies.view.state,
    argumentsText: () => $<HTMLInputElement>("args").value,
    breakpointLines: (state) => breakpointLines(state),
    rustExecution: runRustExecution,
    portugolRsExecution: runPortugolRsExecution,
    isPaused: () => runState.paused !== null,
    debugContinue: () => runPresentation.debugCommand(api.debugContinue),
    debugStepOver: () => runPresentation.debugCommand(api.debugStepOver),
    stop: () => runPresentation.debuggerCommands.stopProgram(),
  });

  const cargoProjectController = createCargoProjectController({
    tabs: dependencies.tabs,
    activeTab: dependencies.activeTab,
    isBusy: () => isExecutionBusy(runState),
    cargoAvailable: dependencies.cargoAvailable,
    isDirty: dependencies.isDirty,
    save: dependencies.save,
    currentDocument,
    isCurrent,
    setPending: (pending) => { runState.porsRunPending = pending; },
    setRunTabId: (id) => { runState.runTabId = id; },
    runId: () => runState.runId,
    setRunId: (id) => { runState.runId = id; },
    syncLanguageControls: dependencies.syncLanguageControls,
    setRunning: runPresentation.setRunning,
    hasCargoConsent: () => hasCargoExecutionConsent(sessionStorage),
    askCargoConsent: (operation) => operation === "check"
      ? ask(
        "cargo check pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta análise não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?",
        { title: "Analisar projeto Cargo sem sandbox", kind: "warning", okLabel: "Analisar Cargo", cancelLabel: "Cancelar" },
      )
      : ask(
        "cargo test pode acessar a rede e executar scripts de build, macros procedurais e código de teste no seu usuário local. Esta execução não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?",
        { title: "Executar cargo test sem sandbox", kind: "warning", okLabel: "Executar testes", cancelLabel: "Cancelar" },
      ),
    rememberCargoConsent: () => rememberCargoExecutionConsent(sessionStorage, true),
    registerRun: api.registerRustSourceRun,
    checkCargoProject: api.checkCargoProject,
    testCargoProject: api.testCargoProject,
    clearRuntimeError: () => dependencies.view.dispatch({ effects: setRuntimeError.of(null) }),
    clearConsole: dependencies.clearConsole,
    showPanel: dependencies.showPanel,
    setStatus: dependencies.setStatus,
    setDiagnostics: (tab, diagnostics) => {
      tab.diagnostics = diagnostics;
      dependencies.renderMessages();
      dependencies.view.dispatch(applyDiagnostics(dependencies.view.state, diagnostics.map((item) => toCodeMirror(item, dependencies.view.state.doc.length))));
    },
    appendConsole: dependencies.appendConsole,
    renderExerciseTests: dependencies.renderExerciseTests,
  });

  return {
    ...runDispatcher,
    checkCargoProject: () => cargoProjectController.check(),
    runCargoTests: () => cargoProjectController.runTests(),
  };
}
