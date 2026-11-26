import { EditorView } from "@codemirror/view";
import type { EditorView as EditorViewType } from "@codemirror/view";
import type { SymbolLocation } from "../api";
import type { Tab } from "../state/tab";

export interface SymbolNavigationDependencies {
  activeTab: () => Tab | undefined;
  view: Pick<EditorViewType, "state" | "dispatch" | "focus">;
  resolvePortugolRs: (source: string, offset: number) => Promise<SymbolLocation>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
}

/** Resolve uma declaração e navega até ela somente se o documento ainda for atual. */
export function createSymbolNavigator(dependencies: SymbolNavigationDependencies) {
  let generation = 0;

  async function goToDeclarationAt(offset: number, quiet = false): Promise<void> {
    const tab = dependencies.activeTab();
    if (!tab) return;
    if (tab.language === "rust") {
      if (!quiet) dependencies.setStatus("Navegação de símbolos Rust ainda não está configurada.");
      return;
    }
    const doc = dependencies.view.state.doc;
    const request = ++generation;
    try {
      const location = await dependencies.resolvePortugolRs(doc.toString(), offset);
      if (request !== generation || dependencies.activeTab() !== tab || dependencies.view.state.doc !== doc) return;
      dependencies.view.dispatch({
        selection: { anchor: location.start.offset, head: location.end.offset },
        effects: EditorView.scrollIntoView(location.start.offset, { y: "center" }),
      });
      dependencies.view.focus();
    } catch (error) {
      if (!quiet && request === generation && dependencies.activeTab() === tab && dependencies.view.state.doc === doc) {
        dependencies.setStatus(typeof error === "string" ? error : "Não foi possível ir à declaração.", "error");
      }
    }
  }

  return { goToDeclarationAt };
}
