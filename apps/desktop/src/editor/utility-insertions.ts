import type { Completion } from "@codemirror/autocomplete";
import type { SymbolInfo } from "../api";
import type { SourceLanguage } from "../document-language";
import type { Tab } from "../state/tab";

type EditorButton = HTMLButtonElement;

export interface UtilityInsertionsDependencies {
  porsLibraries: readonly string[];
  porsSnippets: readonly Completion[];
  createButton: () => EditorButton;
  clearLibraryButtons?: () => void;
  clearSnippetButtons?: () => void;
  appendLibraryButton: (button: EditorButton) => void;
  appendSnippetButton: (button: EditorButton) => void;
  activeTab: () => Tab | undefined;
  isRunning: (tab: Tab) => boolean;
  insertText?: (text: string) => void;
  focusEditor: () => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  createProgramTab: () => void;
  selection: () => { from: number; to: number };
  applySnippet: (snippet: Completion, from: number, to: number) => void;
  utilityTabButton: (name: "libraries" | "models") => Pick<HTMLButtonElement, "onclick" | "setAttribute">;
  utilityPanel: (name: "libraries" | "models") => Pick<HTMLElement, "hidden">;
}

/** Wires PORS module, snippet, and tab controls for the editor utilities panel. */
export function createUtilityInsertions(dependencies: UtilityInsertionsDependencies) {
  const libraryButtons = new Map<string, EditorButton>();

  function renderCatalog() {
    libraryButtons.clear();
    dependencies.clearLibraryButtons?.();
    dependencies.clearSnippetButtons?.();

    for (const name of dependencies.porsLibraries) {
      const button = dependencies.createButton();
      button.textContent = name;
      button.title = `Inserir módulo PORS ${name}.`;
      button.onclick = () => {
        const tab = dependencies.activeTab();
        if (!tab || tab.language !== "portugol-rs" || dependencies.isRunning(tab)) return;
        if (dependencies.insertText) {
          dependencies.insertText(`${name}.`);
          dependencies.focusEditor();
          dependencies.setStatus(`Módulo PORS inserido: ${name}. (use Ctrl+Espaço para completar).`, "ok");
        } else {
          dependencies.setStatus(`Use ${name}.funcao(...) diretamente em Ferro.`);
        }
      };
      dependencies.appendLibraryButton(button);
      libraryButtons.set(name, button);
    }

    for (const snippet of dependencies.porsSnippets) {
      const button = dependencies.createButton();
      button.textContent = snippet.label;
      button.title = snippet.detail ?? snippet.label;
      button.onclick = () => {
        const tab = dependencies.activeTab();
        if (!tab || tab.language !== "portugol-rs" || dependencies.isRunning(tab)) return;
        if (snippet.label === "programa") {
          dependencies.createProgramTab();
          return;
        }
        const { from, to } = dependencies.selection();
        dependencies.applySnippet(snippet, from, to);
        dependencies.focusEditor();
      };
      dependencies.appendSnippetButton(button);
    }
  }

  function mount() {
    renderCatalog();

    for (const selected of ["libraries", "models"] as const) {
      dependencies.utilityTabButton(selected).onclick = () => {
        for (const name of ["libraries", "models"] as const) {
          dependencies.utilityTabButton(name).setAttribute("aria-selected", String(name === selected));
          dependencies.utilityPanel(name).hidden = name !== selected;
        }
      };
    }
  }

  function updateLibraryButtons(_symbols: SymbolInfo[], language: SourceLanguage | undefined) {
    const porsMode = language === "portugol-rs";
    for (const [name, button] of libraryButtons) {
      button.disabled = !porsMode;
      button.title = porsMode ? `Inserir módulo PORS ${name}.` : "Módulos PORS não se aplicam a Rust.";
    }
  }

  return {
    mount,
    libraryButtons: () => libraryButtons,
    updateLibraryButtons,
  };
}
