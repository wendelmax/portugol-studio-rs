import type { EditorState, Text } from "@codemirror/state";
import type { Tab } from "../state/tab";

export interface RunDispatcherDependencies {
  isBusy: () => boolean;
  activeTab: () => Tab | undefined;
  currentDocument: () => Text;
  editorState: () => EditorState;
  argumentsText: () => string;
  breakpointLines: (state: EditorState) => number[];
  rustExecution: { run: (pauseOnStart: boolean) => Promise<void> };
  ferroRustExecution: {
    run: (tab: Tab, document: Text, breakpoints: number[], pauseOnStart: boolean, asRust: boolean) => Promise<void>;
  };
  isPaused: () => boolean;
  debugContinue: () => Promise<void>;
  debugStepOver: () => Promise<void>;
  stop: () => Promise<void>;
  /** Telas desktop (ADR 0018): o `.design` ou o code-behind dele rodam numa janela própria. */
  screenExecution?: { handles: (tab: Tab) => boolean; run: (tab: Tab) => Promise<void> };
}

/** Routes IDE run commands to the controller for the active source language. */
export function createRunDispatcher(dependencies: RunDispatcherDependencies) {
  async function runProgram(pauseOnStart = false, asRust = false) {
    if (dependencies.isBusy()) return;
    const tab = dependencies.activeTab();
    if (!tab) return;
    // Uma tela roda numa janela própria, a partir do .design ou do código dela.
    if (dependencies.screenExecution?.handles(tab)) {
      await dependencies.screenExecution.run(tab);
      return;
    }
    // Outra marcação (.ferproj, .design fora de um workspace) não executa.
    if (tab.language === "markup") return;
    if (tab.language === "rust") {
      await dependencies.rustExecution.run(pauseOnStart);
      return;
    }
    const document = dependencies.currentDocument();
    const breakpoints = dependencies.breakpointLines(dependencies.editorState());
    await dependencies.ferroRustExecution.run(tab, document, breakpoints, pauseOnStart, asRust);
  }

  return {
    runProgram,
    runOrContinue: () => dependencies.isPaused() ? dependencies.debugContinue() : runProgram(),
    stepDebugOrStart: () => dependencies.isPaused() ? dependencies.debugStepOver() : runProgram(true),
    stopProgram: () => dependencies.stop(),
  };
}
