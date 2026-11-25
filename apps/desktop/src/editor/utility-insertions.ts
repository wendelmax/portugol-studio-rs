import type { Completion } from "@codemirror/autocomplete";
import type { SymbolInfo } from "../api";
import type { SourceLanguage } from "../document-language";
import type { Tab } from "../state/tab";

type EditorButton = HTMLButtonElement;

export interface LibraryChange {
  from: number;
  to: number;
  insert: string;
}

export interface UtilityInsertionsDependencies {
  libraries: readonly string[];
  snippets: readonly Completion[];
  createButton: () => EditorButton;
  appendLibraryButton: (button: EditorButton) => void;
  appendSnippetButton: (button: EditorButton) => void;
  activeTab: () => Tab | undefined;
  isRunning: (tab: Tab) => boolean;
  currentDocument: () => { toString(): string };
  includeLibrary: (source: string, name: string) => Promise<LibraryChange | null>;
  isCurrent: (tab: Tab, document: { toString(): string }) => boolean;
  applyLibraryChange: (change: LibraryChange) => void;
  focusEditor: () => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  createProgramTab: () => void;
  selection: () => { from: number; to: number };
  applySnippet: (snippet: Completion, from: number, to: number) => void;
  utilityTabButton: (name: "libraries" | "models") => Pick<HTMLButtonElement, "onclick" | "setAttribute">;
  utilityPanel: (name: "libraries" | "models") => Pick<HTMLElement, "hidden">;
}

/** Wires library, snippet, and tab controls for the editor utilities panel. */
export function createUtilityInsertions(dependencies: UtilityInsertionsDependencies) {
  const libraryButtons = new Map<string, EditorButton>();

  function mount() {
    for (const name of dependencies.libraries) {
      const button = dependencies.createButton();
      button.textContent = name;
      button.title = `Incluir biblioteca ${name}`;
      button.onclick = async () => {
        const tab = dependencies.activeTab();
        if (!tab || dependencies.isRunning(tab)) return;
        if (tab.language === "portugol-rs") {
          dependencies.setStatus("Bibliotecas clássicas ainda não estão disponíveis no Portugol-RS.");
          return;
        }
        const document = dependencies.currentDocument();
        try {
          const change = await dependencies.includeLibrary(document.toString(), name);
          if (!dependencies.isCurrent(tab, document)) return;
          if (change) dependencies.applyLibraryChange(change);
          button.disabled = true;
          dependencies.focusEditor();
        } catch (message) {
          if (dependencies.activeTab() === tab) {
            dependencies.setStatus(typeof message === "string" ? message : "Não foi possível incluir a biblioteca.", "error");
          }
        }
      };
      dependencies.appendLibraryButton(button);
      libraryButtons.set(name, button);
    }

    for (const snippet of dependencies.snippets) {
      const button = dependencies.createButton();
      button.textContent = snippet.label;
      button.title = snippet.detail ?? snippet.label;
      button.onclick = () => {
        const tab = dependencies.activeTab();
        if (!tab || tab.language === "rust" || dependencies.isRunning(tab)) return;
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

    for (const selected of ["libraries", "models"] as const) {
      dependencies.utilityTabButton(selected).onclick = () => {
        for (const name of ["libraries", "models"] as const) {
          dependencies.utilityTabButton(name).setAttribute("aria-selected", String(name === selected));
          dependencies.utilityPanel(name).hidden = name !== selected;
        }
      };
    }
  }

  function updateLibraryButtons(symbols: SymbolInfo[], language: SourceLanguage | undefined) {
    const included = new Set(symbols.filter((symbol) => symbol.kind === "library")
      .map(({ name }) => name.split(" → ", 1)[0]));
    const unsupported = language !== "portugol";
    for (const [name, button] of libraryButtons) button.disabled = unsupported || included.has(name);
  }

  return {
    mount,
    libraryButtons: () => libraryButtons,
    updateLibraryButtons,
  };
}
