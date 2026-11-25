import type { SourceLanguage } from "../document-language";
import { canDebugSource } from "../runtime-capabilities.ts";

export interface RunToolbarDependencies {
  button: (id: string) => Pick<HTMLButtonElement, "onclick">;
  form: Pick<HTMLFormElement, "hidden" | "addEventListener">;
  input: Pick<HTMLInputElement, "value">;
  runId: () => number | null;
  activeLanguage: () => SourceLanguage | null | undefined;
  runOrContinue: () => unknown;
  stepOrStart: () => unknown;
  stop: () => unknown;
  pause: (runId: number) => unknown;
  continueDebug: () => unknown;
  stepOver: () => unknown;
  stepInto: () => unknown;
  stepOut: () => unknown;
  clearConsole: () => void;
  consoleText: () => string;
  copyText: (text: string) => Promise<void>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  showPanel: (panel: "console" | "tests" | "messages" | "debug") => void;
  appendConsole: (text: string, kind?: string) => void;
  sendInput: (runId: number, value: string) => Promise<void>;
}

/** Binds execution, debugger, console, and panel controls to the run services. */
export function mountRunToolbar(dependencies: RunToolbarDependencies) {
  dependencies.button("btn-run").onclick = () => { void dependencies.runOrContinue(); };
  dependencies.button("btn-debug").onclick = () => { void dependencies.stepOrStart(); };
  dependencies.button("btn-stop").onclick = () => { void dependencies.stop(); };
  dependencies.button("btn-debug-pause").onclick = () => {
    const language = dependencies.activeLanguage();
    if (language === null || language === undefined || !canDebugSource(language)) return;
    const id = dependencies.runId();
    if (id !== null) void dependencies.pause(id);
  };
  dependencies.button("btn-debug-continue").onclick = () => { void dependencies.continueDebug(); };
  dependencies.button("btn-debug-step-over").onclick = () => { void dependencies.stepOver(); };
  dependencies.button("btn-debug-step-into").onclick = () => { void dependencies.stepInto(); };
  dependencies.button("btn-debug-step-out").onclick = () => { void dependencies.stepOut(); };
  dependencies.button("btn-clear").onclick = dependencies.clearConsole;
  dependencies.button("btn-copy").onclick = async () => {
    try {
      await dependencies.copyText(dependencies.consoleText());
      dependencies.setStatus("Saída copiada.", "ok");
    } catch (error) {
      dependencies.setStatus(String(error), "error");
    }
  };

  dependencies.button("tab-console").onclick = () => dependencies.showPanel("console");
  dependencies.button("tab-tests").onclick = () => dependencies.showPanel("tests");
  dependencies.button("tab-messages").onclick = () => dependencies.showPanel("messages");
  dependencies.button("tab-debug").onclick = () => dependencies.showPanel("debug");

  dependencies.form.addEventListener("submit", (event) => {
    event.preventDefault();
    const id = dependencies.runId();
    if (id === null) return;
    const line = dependencies.input.value;
    dependencies.appendConsole(line + "\n", "echo");
    dependencies.form.hidden = true;
    void dependencies.sendInput(id, line);
  });
}
