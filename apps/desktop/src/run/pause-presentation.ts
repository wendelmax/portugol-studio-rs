import type { DebugFrame, DebugInfo, SourceMapInfo } from "../api";
import type { Tab } from "../state/tab";

export interface PausePresentationDependencies {
  setPausedState: (info: DebugInfo | null) => void;
  activeTabId: () => number;
  runTabId: () => number | null;
  runId: () => number | null;
  runTab: () => Tab | undefined;
  setDebugLine: (line: number | null) => void;
  setRunning: (running: boolean) => void;
  renderDebugInfo: (info: DebugInfo, onSelectFrame: (frame: DebugFrame) => void, links: SourceMapInfo[]) => void;
  rustSourceMap?: () => SourceMapInfo[];
  activate: (id: number) => void;
  goTo: (offset: number) => void;
  showUtilities: (visible: boolean) => void;
  showDebugPanel: () => void;
  setStatus: (message: string) => void;
}

/** Updates the editor and shell presentation when a debug session pauses. */
export function createPausePresentation(dependencies: PausePresentationDependencies) {
  function setPaused(info: DebugInfo | null) {
    const runTabId = dependencies.runTabId();
    dependencies.setPausedState(info);
    if (dependencies.activeTabId() === runTabId) dependencies.setDebugLine(info?.position.line ?? null);
    dependencies.setRunning(dependencies.runId() !== null);
    if (!info) return;

    dependencies.renderDebugInfo(info, (frame) => {
      if (runTabId === null || !dependencies.runTab()) return;
      if (dependencies.activeTabId() !== runTabId) dependencies.activate(runTabId);
      dependencies.goTo(frame.position.offset);
    }, dependencies.rustSourceMap?.() ?? []);
    dependencies.showUtilities(true);
    dependencies.showDebugPanel();
    if (dependencies.activeTabId() === runTabId) dependencies.goTo(info.position.offset);
    dependencies.setStatus(dependencies.runTab()?.language === "ferro-rust"
      ? `VM Ferro pausada na linha ${info.position.line}.`
      : `Pausado na linha ${info.position.line}.`);
  }

  return { setPaused };
}
