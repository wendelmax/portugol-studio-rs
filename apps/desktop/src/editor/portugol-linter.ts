import type { Diagnostic as CodeMirrorDiagnostic } from "@codemirror/lint";
import type { EditorView } from "@codemirror/view";
import type { DiagnosticInfo, Inspection, SymbolInfo } from "../api";
import type { Tab } from "../state/tab";

export interface PortugolLintDependencies {
  activeTab: () => Tab | undefined;
  /** Inspeção do documento; recebe a aba para quem precisa do caminho (marcação). */
  inspectPortugolRs: (source: string, tab: Tab | undefined) => Promise<Inspection>;
  /** Linguagem atendida por este linter; abas de outra linguagem ficam sem diagnósticos dele. */
  language?: "portugol-rs" | "markup";
  renderMessages: () => void;
  showOutline: (symbols: SymbolInfo[]) => void;
  updateLibraryButtons: (symbols: SymbolInfo[]) => void;
  toCodeMirror: (diagnostic: DiagnosticInfo, length: number) => CodeMirrorDiagnostic;
}

/** Inspects Ferro (or desktop markup) source and synchronizes its diagnostics and symbols. */
export function createPortugolLintSourceCheck(dependencies: PortugolLintDependencies) {
  return async (view: EditorView): Promise<CodeMirrorDiagnostic[]> => {
    const doc = view.state.doc;
    const tab = dependencies.activeTab();
    if (tab && tab.language !== (dependencies.language ?? "portugol-rs")) {
      tab.diagnostics = [];
      tab.diagnosticsAnalysis = { source: doc.toString(), status: "unavailable" };
      tab.symbols = [];
      dependencies.renderMessages();
      dependencies.showOutline([]);
      dependencies.updateLibraryButtons([]);
      return [];
    }

    let inspection: Inspection;
    try {
      inspection = await dependencies.inspectPortugolRs(doc.toString(), tab);
    } catch {
      if (tab && tab === dependencies.activeTab() && view.state.doc === doc) {
        tab.diagnosticsAnalysis = { source: doc.toString(), status: "unavailable" };
        dependencies.renderMessages();
      }
      return [];
    }
    const { diagnostics, symbols } = inspection;
    if (tab && tab === dependencies.activeTab() && view.state.doc === doc) {
      tab.diagnostics = diagnostics;
      tab.diagnosticsAnalysis = { source: doc.toString(), status: "complete" };
      tab.symbols = symbols;
      dependencies.renderMessages();
      dependencies.showOutline(symbols);
      dependencies.updateLibraryButtons(symbols);
    }
    return [...diagnostics, ...(tab?.runtimeCompileDiagnostics ?? [])]
      .map((diagnostic) => dependencies.toCodeMirror(diagnostic, doc.length));
  };
}
