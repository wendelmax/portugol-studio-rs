import type { Tab } from "../state/tab";
import type { SourceLanguage } from "../document-language";
import type { SemanticSymbolLanguage } from "../semantic-symbol-actions";

export interface RenameTarget {
  name: string;
  occurrences: number;
}

export interface RenameChange {
  from: number;
  to: number;
  insert: string;
}

export interface SymbolRenameUI {
  dialog: Pick<HTMLDialogElement, "open" | "showModal" | "close">;
  form: Pick<HTMLFormElement, "onsubmit">;
  input: Pick<HTMLInputElement, "value" | "select">;
  error: Pick<HTMLElement, "hidden" | "textContent">;
  submit: Pick<HTMLButtonElement, "disabled">;
  description: Pick<HTMLElement, "textContent">;
  cancelButton: Pick<HTMLButtonElement, "onclick">;
}

export interface SymbolRenamerDependencies {
  activeTab: () => Tab | undefined;
  currentDocument: () => { toString(): string };
  isRunning: (tab: Tab) => boolean;
  isCurrent: (tab: Tab, document: { toString(): string }) => boolean;
  hasOpenDialog: () => boolean;
  semanticSymbolLanguage: (language: SourceLanguage) => SemanticSymbolLanguage | null;
  renameTargetPortugolRs: (source: string, offset: number) => Promise<RenameTarget>;
  renameSymbolPortugolRs: (source: string, offset: number, name: string) => Promise<RenameChange[]>;
  applyChanges: (changes: RenameChange[]) => void;
  focusEditor: () => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  ui: SymbolRenameUI;
}

/** Coordinates symbol rename requests, dialog state, and safe editor updates. */
export function createSymbolRenamer(dependencies: SymbolRenamerDependencies) {
  async function openAt(offset: number) {
    const tab = dependencies.activeTab();
    if (!tab || tab.language !== "portugol-rs" || dependencies.isRunning(tab)) return;
    const doc = dependencies.currentDocument();
    const language = dependencies.semanticSymbolLanguage(tab.language);
    if (!language) return;

    try {
      const target = await dependencies.renameTargetPortugolRs(doc.toString(), offset);
      if (!dependencies.isCurrent(tab, doc) || dependencies.hasOpenDialog()) return;

      const { dialog, form, input, error, submit, description, cancelButton } = dependencies.ui;
      description.textContent = `${target.name} · ${target.occurrences} ${target.occurrences === 1 ? "ocorrência" : "ocorrências"}`;
      input.value = target.name;
      error.hidden = true;
      cancelButton.onclick = () => dialog.close();
      form.onsubmit = async (event) => {
        event.preventDefault();
        error.hidden = true;
        submit.disabled = true;
        try {
          const changes = await dependencies.renameSymbolPortugolRs(doc.toString(), offset, input.value.trim());
          if (!dialog.open || !dependencies.isCurrent(tab, doc)) return;
          if (changes.length) dependencies.applyChanges(changes);
          dialog.close();
          dependencies.focusEditor();
        } catch (message) {
          error.textContent = typeof message === "string" ? message : "Não foi possível renomear o símbolo.";
          error.hidden = false;
        } finally {
          submit.disabled = false;
        }
      };
      dialog.showModal();
      input.select();
    } catch (message) {
      if (dependencies.isCurrent(tab, doc)) {
        dependencies.setStatus(typeof message === "string" ? message : "Não foi possível localizar o símbolo.", "error");
      }
    }
  }

  return { openAt };
}
