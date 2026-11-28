import type { DiagnosticInfo, PortugolRsCompileDiagnostic, PortugolRsRunResult } from "../api";
import type { Tab } from "../state/tab";
import type { PendingRunEvent } from "../run-event-router";

export interface PortugolRsExecutionDependencies {
  rustcAvailable: () => boolean | null;
  isCurrent: (tab: Tab, document: object) => boolean;
  runId: () => number | null;
  setRunId: (id: number | null) => void;
  setPending: (pending: boolean) => void;
  setRunTabId: (tabId: number | null) => void;
  setPendingEvents: (events: PendingRunEvent[] | null) => void;
  setRunning: (running: boolean) => void;
  clearRuntimeError: () => void;
  clearDebugLine: () => void;
  clearPaused: () => void;
  showConsole: () => void;
  clearConsole: () => void;
  hideInput: () => void;
  runListeners: () => Promise<unknown>;
  startVm: (source: string, lines: number[], pauseOnStart: boolean, path: string | null) => Promise<number>;
  registerRun: () => Promise<number>;
  runGeneratedRust: (id: number, source: string) => Promise<PortugolRsRunResult>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  appendProcessOutput: (result: PortugolRsRunResult) => void;
  appendConsole: (text: string, kind?: "error") => void;
  renderMessages: () => void;
  setDiagnostics: (tab: Tab, diagnostics: DiagnosticInfo[]) => void;
  setVmDiagnostics: (tab: Tab, diagnostics: unknown[]) => void;
  showMessages: () => void;
  sourceDiagnostic: (diagnostic: PortugolRsCompileDiagnostic) => DiagnosticInfo | null;
  toCodeMirror: (diagnostic: DiagnosticInfo, documentLength: number) => unknown;
  replayEvents: (events: PendingRunEvent[], id: number) => void;
}

/** Orquestra execução na VM educacional e execução do Rust gerado pelo PORS. */
export function createPortugolRsExecutionController(dependencies: PortugolRsExecutionDependencies) {
  async function run(
    tab: Tab,
    document: object,
    breakpointLines: number[],
    pauseOnStart: boolean,
    asRust: boolean,
  ): Promise<void> {
    const source = (document as { toString(): string }).toString();
    dependencies.clearRuntimeError();
    dependencies.clearDebugLine();
    dependencies.clearPaused();
    dependencies.showConsole();
    dependencies.clearConsole();
    dependencies.hideInput();
    if (!asRust) {
      const events: PendingRunEvent[] = [];
      dependencies.setPendingEvents(events);
      dependencies.setRunTabId(tab.id);
      dependencies.setRunning(true);
      try {
        await dependencies.runListeners();
        const id = await dependencies.startVm(source, breakpointLines, pauseOnStart, tab.path ?? tab.examplePath ?? null);
        dependencies.setRunId(id);
        dependencies.setRunning(true);
        dependencies.setStatus(pauseOnStart ? "Depurando Ferro na VM." : "Executando Ferro na VM…");
        dependencies.replayEvents(events, id);
      } catch (errors) {
        dependencies.setRunTabId(null);
        dependencies.setRunning(false);
        if (Array.isArray(errors)) {
          dependencies.setVmDiagnostics(tab, errors);
          dependencies.renderMessages();
          dependencies.showMessages();
          dependencies.setStatus(
            pauseOnStart
              ? "O Ferro contém erros e não pode ser depurado."
              : "O Ferro contém erros e não pode ser executado.",
            "error",
          );
        } else {
          dependencies.setStatus(String(errors), "error");
        }
      } finally {
        dependencies.setPendingEvents(null);
      }
      return;
    }
    if (dependencies.rustcAvailable() !== true) {
      dependencies.setStatus("Instale a toolchain Rust para executar o Rust gerado.", "error");
      return;
    }
    dependencies.setPending(true);
    dependencies.setRunTabId(tab.id);
    dependencies.setRunning(true);
    dependencies.setStatus("Compilando e executando com rustc…");
    let registeredId: number | null = null;
    try {
      registeredId = await dependencies.registerRun();
      dependencies.setRunId(registeredId);
      dependencies.setRunning(true);
      await dependencies.runListeners();
      const result = await dependencies.runGeneratedRust(registeredId, source);
      if (!dependencies.isCurrent(tab, document)) return;
      if (result.status === "cancelled") {
        dependencies.appendProcessOutput(result);
        const phase = result.phase === "compilation" ? "compilação" : "execução";
        dependencies.setStatus(`Execução Ferro cancelada durante a ${phase}.`);
      } else if (result.status === "executed") {
        dependencies.appendProcessOutput(result);
        dependencies.setStatus(
          result.exitCode === 0
            ? "Programa Ferro finalizado com rustc."
            : `Programa Ferro terminou com código ${result.exitCode ?? "desconhecido"}.`,
          result.exitCode === 0 ? "ok" : "error",
        );
      } else if (result.status === "compile_limit") {
        if (result.stderr) dependencies.appendConsole(result.stderr, "error");
        const reason = result.reason === "time" ? "tempo máximo de compilação de 20 segundos" : "limite de saída de compilação de 1 MiB";
        dependencies.setStatus(`Compilação Ferro interrompida: ${reason}.`, "error");
      } else if (result.status === "runtime_limit") {
        dependencies.appendProcessOutput(result);
        const descriptions = {
          time: "tempo máximo de 3 segundos",
          output: "limite de saída de 1 MiB",
          string_memory: "limite de memória para cadeias de 16 MiB",
        } as const;
        dependencies.setStatus(`Execução Ferro interrompida: ${descriptions[result.reason]}.`, "error");
      } else {
        const diagnostics = result.diagnostics.map(dependencies.sourceDiagnostic).filter((item): item is DiagnosticInfo => item !== null);
        dependencies.setDiagnostics(tab, diagnostics);
        if (result.stderr) dependencies.appendConsole(result.stderr, "error");
        if (diagnostics.length) {
          dependencies.renderMessages();
          dependencies.showMessages();
          dependencies.setStatus("rustc encontrou erros no Rust gerado. Selecione um diagnóstico para voltar à origem.", "error");
        } else {
          dependencies.setStatus("rustc encontrou um erro no código gerado; veja o detalhe no console.", "error");
        }
      }
    } catch (error) {
      if (dependencies.isCurrent(tab, document)) {
        dependencies.appendConsole(`${String(error)}\n`, "error");
        dependencies.setStatus("Não foi possível compilar ou executar o Ferro.", "error");
      }
    } finally {
      dependencies.hideInput();
      if (dependencies.runId() === registeredId) dependencies.setRunId(null);
      dependencies.setPending(false);
      dependencies.setRunTabId(null);
      dependencies.setRunning(dependencies.runId() !== null);
    }
  }

  return { run };
}
