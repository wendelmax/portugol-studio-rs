import type { Diagnostic as CodeMirrorDiagnostic } from "@codemirror/lint";
import type { EditorView } from "@codemirror/view";
import type { DiagnosticInfo, Inspection, SymbolInfo } from "../api";
import type { Tab } from "../state/tab";

export interface PortugolLintDependencies {
  activeTab: () => Tab | undefined;
  inspectPortugol: (source: string) => Promise<Inspection>;
  inspectPortugolRs: (source: string) => Promise<Inspection>;
  renderMessages: () => void;
  showOutline: (symbols: SymbolInfo[]) => void;
  updateLibraryButtons: (symbols: SymbolInfo[]) => void;
  toCodeMirror: (diagnostic: DiagnosticInfo, length: number) => CodeMirrorDiagnostic;
}

/** Inspects Portugol/PORS source and synchronizes its diagnostics and symbols. */
export function createPortugolLintSourceCheck(dependencies: PortugolLintDependencies) {
  return async (view: EditorView): Promise<CodeMirrorDiagnostic[]> => {
    const doc = view.state.doc;
    const tab = dependencies.activeTab();
    if (tab?.language === "rust") {
      tab.diagnostics = [];
      tab.diagnosticsAnalysis = { source: doc.toString(), status: "unavailable" };
      tab.symbols = [];
      dependencies.renderMessages();
      dependencies.showOutline([]);
      dependencies.updateLibraryButtons([]);
      return [];
    }

    const language = tab?.language ?? "portugol";
    let inspection: Inspection;
    try {
      inspection = language === "portugol-rs"
        ? await dependencies.inspectPortugolRs(doc.toString())
        : await dependencies.inspectPortugol(doc.toString());
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
