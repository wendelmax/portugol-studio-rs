import type { RunEvents, RunOutcome } from "../api";
import { routeRunEvent, type PendingRunEvent } from "../run-event-router.ts";

export interface RunEventsDependencies {
  runId: () => number | null;
  setRunId: (id: number | null) => void;
  runTabId: () => number | null;
  setRunTabId: (id: number | null) => void;
  activeTabId: () => number;
  pendingEvents: () => PendingRunEvent[] | null;
  setPaused: (info: null) => void;
  setRunning: (running: boolean) => void;
  setDebugLine: (line: null) => void;
  setRuntimeError: (range: { from: number; to: number }) => void;
  appendConsole: (text: string, kind?: "error" | "info" | "echo") => void;
  clearConsole: () => void;
  setInputHint: (text: string) => void;
  showInput: () => void;
  clearInput: () => void;
  focusInput: () => void;
  scrollConsole: () => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  goTo: (offset: number) => void;
  setPausedInfo: (info: Parameters<RunEvents["paused"]>[1]) => void;
}

/** Encaminha eventos do backend e aplica efeitos de conclusão na aba da execução. */
export function createRunEventsController(dependencies: RunEventsDependencies) {
  function route(id: number, handle: () => void) {
    routeRunEvent(id, dependencies.runId(), dependencies.pendingEvents(), handle);
  }

  const handlers: RunEvents = {
    output(id, text) {
      route(id, () => dependencies.appendConsole(text));
    },
    clear(id) {
      route(id, dependencies.clearConsole);
    },
    input(id, expected) {
      route(id, () => {
        dependencies.setInputHint(`leia (${expected}):`);
        dependencies.showInput();
        dependencies.clearInput();
        dependencies.focusInput();
        dependencies.scrollConsole();
      });
    },
    paused(id, info) {
      route(id, () => dependencies.setPausedInfo(info));
    },
    finished(id, outcome) {
      route(id, () => finish(outcome));
    },
  };

  function finish(outcome: RunOutcome) {
    const isRunTab = dependencies.activeTabId() === dependencies.runTabId();
    dependencies.setRunId(null);
    dependencies.setRunTabId(null);
    dependencies.setPaused(null);
    if (isRunTab) dependencies.setDebugLine(null);
    dependencies.setRunning(false);
    if (outcome.status === "finished") {
      dependencies.appendConsole("\nPrograma finalizado.\n", "info");
      dependencies.setStatus("Programa finalizado.", "ok");
    } else if (outcome.status === "stopped") {
      dependencies.appendConsole("\nPrograma interrompido.\n", "info");
      dependencies.setStatus("Programa interrompido.");
    } else if (outcome.status === "runtime_limit") {
      const descriptions = {
        time: "tempo ativo de 3 segundos",
        output: "limite de saída de 1 MiB",
        string_memory: "limite cumulativo de cadeias de 16 MiB",
      } as const;
      const message = `VM Ferro interrompida: ${descriptions[outcome.reason]}.`;
      dependencies.appendConsole(`\n${message}\n`, "error");
      dependencies.setStatus(message, "error");
    } else {
      const { start, message, trace } = outcome;
      let text = `\nErro na linha ${start.line}, coluna ${start.column}: ${message}\n`;
      for (const frame of trace.slice(1)) text += `  chamado em ${frame.function} (linha ${frame.line})\n`;
      dependencies.appendConsole(text, "error");
      dependencies.setStatus(`Erro de execução na linha ${start.line}.`, "error");
      if (isRunTab) {
        dependencies.setRuntimeError({ from: start.offset, to: outcome.end.offset });
        dependencies.goTo(start.offset);
      }
    }
  }

  return { handlers };
}
