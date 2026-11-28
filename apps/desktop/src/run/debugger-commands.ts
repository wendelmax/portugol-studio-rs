import type { EditorState } from "@codemirror/state";
import type { DebugInfo } from "../api";
import type { SourceLanguage } from "../document-language";

export interface DebuggerCommandDependencies {
  runId: () => number | null;
  runTabId: () => number | null;
  activeTabId: () => number;
  breakpointLines: (state: EditorState) => number[];
  setBreakpoints: (id: number, source: string, lines: number[]) => Promise<void>;
  setDebugLine: (line: number | null) => void;
  paused: () => DebugInfo | null;
  setPaused: (info: DebugInfo | null) => void;
  setRunning: (running: boolean) => void;
  runLanguage: () => SourceLanguage | undefined;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  stop: (id: number) => Promise<boolean>;
}

/** Encaminha breakpoints e ações do depurador para a execução ainda ativa. */
export function createDebuggerCommands(dependencies: DebuggerCommandDependencies) {
  function syncBreakpoints(state: EditorState) {
    const id = dependencies.runId();
    if (id === null || dependencies.activeTabId() !== dependencies.runTabId()) return;
    void dependencies.setBreakpoints(id, state.doc.toString(), dependencies.breakpointLines(state)).catch(() => {});
  }

  async function debugCommand(command: (id: number) => Promise<void>) {
    const id = dependencies.runId();
    if (id === null || dependencies.paused() === null) return;
    dependencies.setDebugLine(null);
    dependencies.setPaused(null);
    dependencies.setRunning(true);
    dependencies.setStatus(dependencies.runLanguage() === "ferro-rust" ? "VM Ferro executando…" : "Executando…");
    try {
      await command(id);
    } catch {
      // A execução pode ter terminado enquanto o comando era enviado.
    }
  }

  async function stopProgram() {
    const id = dependencies.runId();
    if (id === null) return;
    try {
      const accepted = await dependencies.stop(id);
      if (accepted && dependencies.runId() === id && dependencies.activeTabId() === dependencies.runTabId()) {
        dependencies.setStatus("Interrupção solicitada…");
      }
    } catch (error) {
      if (dependencies.runId() === id && dependencies.activeTabId() === dependencies.runTabId()) {
        dependencies.setStatus(`Não foi possível interromper: ${String(error)}`, "error");
      }
    }
  }

  return { syncBreakpoints, debugCommand, stopProgram };
}
