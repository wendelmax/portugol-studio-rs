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
  portugolRsExecution: {
    run: (tab: Tab, document: Text, breakpoints: number[], pauseOnStart: boolean, asRust: boolean) => Promise<void>;
  };
  classicExecution: {
    run: (tab: Tab, document: Text, argumentsText: string, breakpoints: number[], pauseOnStart: boolean) => Promise<void>;
  };
  isPaused: () => boolean;
  debugContinue: () => Promise<void>;
  debugStepOver: () => Promise<void>;
  stop: () => Promise<void>;
}

/** Routes IDE run commands to the controller for the active source language. */
export function createRunDispatcher(dependencies: RunDispatcherDependencies) {
  async function runProgram(pauseOnStart = false, asRust = false) {
    if (dependencies.isBusy()) return;
    const tab = dependencies.activeTab();
    if (!tab) return;
    if (tab.language === "rust") {
      await dependencies.rustExecution.run(pauseOnStart);
      return;
    }
    const document = dependencies.currentDocument();
    const breakpoints = dependencies.breakpointLines(dependencies.editorState());
    if (tab.language === "portugol-rs") {
      await dependencies.portugolRsExecution.run(tab, document, breakpoints, pauseOnStart, asRust);
      return;
    }
    await dependencies.classicExecution.run(tab, document, dependencies.argumentsText(), breakpoints, pauseOnStart);
  }

  return {
    runProgram,
    runOrContinue: () => dependencies.isPaused() ? dependencies.debugContinue() : runProgram(),
    stepOrStart: () => dependencies.isPaused() ? dependencies.debugStepOver() : runProgram(true),
    stopProgram: () => dependencies.stop(),
  };
}
