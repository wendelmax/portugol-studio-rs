import type { DebugFrame, DebugInfo, SourceMapInfo } from "../api";
import type { Tab } from "../state/tab";
import { createDebuggerCommands, type DebuggerCommandDependencies } from "./debugger-commands";
import { createPausePresentation } from "./pause-presentation";

export interface DebuggerShellDependencies extends DebuggerCommandDependencies {
  runTab: () => Tab | undefined;
  renderDebugInfo: (info: DebugInfo, onSelectFrame: (frame: DebugFrame) => void, links: SourceMapInfo[]) => void;
  rustSourceMap?: () => SourceMapInfo[];
  activate: (id: number) => void;
  goTo: (offset: number) => void;
  showUtilities: (visible: boolean) => void;
  showDebugPanel: () => void;
}

/** Compõe os comandos do depurador com a apresentação de uma pausa. */
export function createDebuggerShell(dependencies: DebuggerShellDependencies) {
  const debuggerCommands = createDebuggerCommands(dependencies);
  const pausePresentation = createPausePresentation({
    setPausedState: dependencies.setPaused,
    activeTabId: dependencies.activeTabId,
    runTabId: dependencies.runTabId,
    runId: dependencies.runId,
    runTab: dependencies.runTab,
    setDebugLine: dependencies.setDebugLine,
    setRunning: dependencies.setRunning,
    renderDebugInfo: dependencies.renderDebugInfo,
    rustSourceMap: dependencies.rustSourceMap,
    activate: dependencies.activate,
    goTo: dependencies.goTo,
    showUtilities: dependencies.showUtilities,
    showDebugPanel: dependencies.showDebugPanel,
    setStatus: (message) => dependencies.setStatus(message),
  });

  return {
    debuggerCommands,
    pausePresentation,
    syncBreakpoints: debuggerCommands.syncBreakpoints,
    setPaused: pausePresentation.setPaused,
    debugCommand: debuggerCommands.debugCommand,
  };
}
