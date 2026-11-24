import type { DiagnosticInfo } from "../api";
import type { Tab } from "../state/tab";
import type { PendingRunEvent } from "../run-event-router";

export interface ClassicExecutionDependencies {
  parseArguments: (text: string) => string[];
  runListeners: () => Promise<unknown>;
  run: (source: string, args: string[], path: string | null, breakpoints: number[], pauseOnStart: boolean) => Promise<number>;
  setRuntimeError: () => void;
  setDebugLine: () => void;
  clearPaused: () => void;
  showConsole: () => void;
  clearConsole: () => void;
  setPendingEvents: (events: PendingRunEvent[] | null) => void;
  setRunTabId: (tabId: number | null) => void;
  setRunId: (runId: number | null) => void;
  setRunButtonsDisabled: (disabled: boolean) => void;
  setRunning: (running: boolean) => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  replayEvents: (events: PendingRunEvent[], runId: number) => void;
  setDiagnostics: (tab: Tab, diagnostics: DiagnosticInfo[]) => void;
  renderMessages: () => void;
  showMessages: () => void;
}

/** Orquestra execução do Portugol clássico sem acoplar o backend à shell. */
export function createClassicExecutionController(dependencies: ClassicExecutionDependencies) {
  async function run(tab: Tab, document: { toString(): string }, argumentsText: string, breakpoints: number[], pauseOnStart: boolean): Promise<void> {
    dependencies.setRuntimeError();
    dependencies.setDebugLine();
    dependencies.clearPaused();
    dependencies.showConsole();
    dependencies.clearConsole();
    const source = document.toString();
    const args = dependencies.parseArguments(argumentsText);
    const events: PendingRunEvent[] = [];
    dependencies.setPendingEvents(events);
    dependencies.setRunTabId(tab.id);
    dependencies.setRunButtonsDisabled(true);
    try {
      await dependencies.runListeners();
      const id = await dependencies.run(source, args, tab.path ?? tab.examplePath ?? null, breakpoints, pauseOnStart);
      dependencies.setRunId(id);
      dependencies.setRunning(true);
      dependencies.setStatus("Executando…");
      dependencies.replayEvents(events, id);
    } catch (errors) {
      dependencies.setRunTabId(null);
      dependencies.setRunning(false);
      if (Array.isArray(errors)) {
        dependencies.setDiagnostics(tab, errors as DiagnosticInfo[]);
        dependencies.renderMessages();
        dependencies.showMessages();
        dependencies.setStatus("O programa contém erros e não pode ser executado.", "error");
      } else {
        dependencies.setStatus(String(errors), "error");
      }
    } finally {
      dependencies.setPendingEvents(null);
    }
  }

  return { run };
}
