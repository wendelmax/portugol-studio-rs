import type { SourceLanguage } from "../document-language";

export interface RunControlsDependencies {
  activeLanguage: () => SourceLanguage | undefined;
  canRun: (language: SourceLanguage, rustcAvailable: boolean | null) => boolean;
  canDebug: (language: SourceLanguage) => boolean;
  canPause?: () => boolean;
  rustcAvailable: () => boolean | null;
  runId: () => number | null;
  paused: () => { variables: unknown[] } | null;
  setButtonDisabled: (id: string, disabled: boolean) => void;
  setInputHidden: (hidden: boolean) => void;
  setDebugVarsHidden: (hidden: boolean) => void;
  setInspectorEmptyHidden: (hidden: boolean) => void;
  setInspectorText: (text: string) => void;
}

/** Sincroniza controles de execução e o estado vazio do inspetor. */
export function createRunControls(dependencies: RunControlsDependencies) {
  function setRunning(running: boolean) {
    const language = dependencies.activeLanguage();
    const canRun = language !== undefined && dependencies.canRun(language, dependencies.rustcAvailable());
    const canDebug = language !== undefined && dependencies.canDebug(language);
    const canPause = dependencies.canPause ? dependencies.canPause() : true;
    const paused = dependencies.paused();
    const isBusy = running && paused === null;
    dependencies.setButtonDisabled("btn-run", !canRun || isBusy);
    dependencies.setButtonDisabled("btn-debug", !canDebug || isBusy);
    dependencies.setButtonDisabled("btn-stop", !running || dependencies.runId() === null);
    dependencies.setButtonDisabled("btn-debug-pause", !canDebug || !canPause || !running || paused !== null);
    for (const id of [
      "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out",
    ]) {
      dependencies.setButtonDisabled(id, !canDebug || paused === null);
    }
    if (!running) dependencies.setInputHidden(true);
    const debugVarsHidden = paused === null || paused.variables.length === 0;
    dependencies.setDebugVarsHidden(debugVarsHidden);
    dependencies.setInspectorEmptyHidden(!debugVarsHidden);
    dependencies.setInspectorText(paused
      ? "Nenhuma variável neste ponto."
      : "Pause a execução para inspecionar as variáveis.");
  }

  return { setRunning };
}
