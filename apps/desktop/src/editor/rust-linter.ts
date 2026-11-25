import type { Diagnostic as CodeMirrorDiagnostic } from "@codemirror/lint";
import type { EditorView } from "@codemirror/view";
import type { DiagnosticInfo, RustSourceCompileDiagnostic } from "../api";
import type { RustSourceCheckResult } from "../rust-source-check";
import type { Tab } from "../state/tab";

export interface RustLintDependencies {
  activeTab: () => Tab | undefined;
  isRustcAvailable: () => boolean;
  checkSource: (source: string) => Promise<RustSourceCheckResult<RustSourceCompileDiagnostic>>;
  renderMessages: () => void;
  toCodeMirror: (diagnostic: DiagnosticInfo, length: number) => CodeMirrorDiagnostic;
}

/** Produces CodeMirror diagnostics for the active standalone Rust source. */
export function createRustLintSourceCheck(dependencies: RustLintDependencies) {
  return async (editor: EditorView): Promise<CodeMirrorDiagnostic[]> => {
    const doc = editor.state.doc;
    const tab = dependencies.activeTab();
    if (tab?.language !== "rust" || tab.cargoProject === true || !dependencies.isRustcAvailable()) {
      if (tab?.language === "rust") {
        tab.diagnostics = [];
        tab.diagnosticsAnalysis = { source: doc.toString(), status: "unavailable" };
        dependencies.renderMessages();
      }
      return [];
    }

    const checked = await dependencies.checkSource(doc.toString());
    if (dependencies.activeTab() !== tab || editor.state.doc !== doc || checked.source !== doc.toString()) return [];
    const diagnostics: DiagnosticInfo[] = checked.error
      ? [{
        severity: "warning",
        code: "rustc.check",
        message: `Não foi possível analisar este arquivo Rust: ${checked.error}`,
        help: null,
        start: { offset: 0, line: 1, column: 1 },
        end: { offset: 0, line: 1, column: 1 },
      }]
      : checked.diagnostics.flatMap((diagnostic) => {
        if (diagnostic.severity !== "error" && diagnostic.severity !== "warning") return [];
        return [{
          severity: diagnostic.severity,
          code: diagnostic.code ?? "rustc",
          message: diagnostic.message,
          help: diagnostic.rendered,
          start: diagnostic.start,
          end: diagnostic.end,
        }];
      });
    tab.diagnostics = diagnostics;
    tab.diagnosticsAnalysis = {
      source: doc.toString(),
      status: checked.error ? "unavailable" : "complete",
    };
    dependencies.renderMessages();
    return [...diagnostics, ...(tab.runtimeCompileDiagnostics ?? [])]
      .map((diagnostic) => dependencies.toCodeMirror(diagnostic, doc.length));
  };
}
