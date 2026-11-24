import { isolateHistory } from "@codemirror/commands";
import type { EditorView as EditorViewType } from "@codemirror/view";
import type { Tab } from "../state/tab";

export interface FormatChange {
  from: number;
  to: number;
  insert: string;
}

export interface EditorFormatterDependencies {
  activeTab: () => Tab | undefined;
  isRunningTab: (tab: Tab) => boolean;
  rustfmtAvailable: () => boolean;
  view: Pick<EditorViewType, "state" | "dispatch" | "focus">;
  formatRust: (source: string) => Promise<string>;
  formatPortugolRs: (source: string) => Promise<FormatChange[] | null>;
  formatPortugol: (source: string) => Promise<FormatChange[] | null>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
}

/** Aplica o formatador adequado sem modificar abas que deixaram de ser atuais. */
export function createEditorFormatter(dependencies: EditorFormatterDependencies) {
  async function formatCode(): Promise<void> {
    const tab = dependencies.activeTab();
    if (!tab || dependencies.isRunningTab(tab)) return;
    if (tab.language === "rust" && !dependencies.rustfmtAvailable()) {
      dependencies.setStatus("Instale o componente rustfmt com `rustup component add rustfmt`.", "error");
      return;
    }
    const doc = dependencies.view.state.doc;
    try {
      const source = doc.toString();
      let changes: FormatChange[] | null;
      if (tab.language === "rust") {
        const formatted = await dependencies.formatRust(source);
        changes = formatted === source ? [] : [{ from: 0, to: doc.length, insert: formatted }];
      } else {
        changes = tab.language === "portugol-rs"
          ? await dependencies.formatPortugolRs(source)
          : await dependencies.formatPortugol(source);
      }
      if (dependencies.activeTab() !== tab || dependencies.view.state.doc !== doc) return;
      if (!changes) {
        dependencies.setStatus("Não foi possível formatar. Confira a sintaxe do programa.", "error");
        return;
      }
      if (changes.length) {
        dependencies.view.dispatch({ changes, annotations: isolateHistory.of("full"), userEvent: "input.format" });
      }
      dependencies.setStatus(changes.length ? "Código formatado." : "O código já está formatado.", "ok");
      dependencies.view.focus();
    } catch (error) {
      if (dependencies.activeTab() === tab && dependencies.view.state.doc === doc) {
        dependencies.setStatus(error instanceof Error ? error.message : String(error), "error");
      }
    }
  }

  return { formatCode };
}
