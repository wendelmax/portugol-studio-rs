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
  porsLibraries?: readonly string[];
  snippets: readonly Completion[];
  porsSnippets?: readonly Completion[];
  createButton: () => EditorButton;
  clearLibraryButtons?: () => void;
  clearSnippetButtons?: () => void;
  appendLibraryButton: (button: EditorButton) => void;
  appendSnippetButton: (button: EditorButton) => void;
  activeTab: () => Tab | undefined;
  isRunning: (tab: Tab) => boolean;
  currentDocument: () => { toString(): string };
  includeLibrary: (source: string, name: string) => Promise<LibraryChange | null>;
  isCurrent: (tab: Tab, document: { toString(): string }) => boolean;
  applyLibraryChange: (change: LibraryChange) => void;
  insertText?: (text: string) => void;
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
  let renderedPorsMode: boolean | null = null;

  function renderCatalog(porsMode: boolean) {
    if (renderedPorsMode === porsMode) return;
    renderedPorsMode = porsMode;
    libraryButtons.clear();
    dependencies.clearLibraryButtons?.();
    dependencies.clearSnippetButtons?.();

    const activeLibraries = porsMode && dependencies.porsLibraries?.length
      ? dependencies.porsLibraries
      : dependencies.libraries;
    for (const name of activeLibraries) {
      const button = dependencies.createButton();
      button.textContent = name;
      button.title = porsMode ? `Inserir módulo PORS ${name}.` : `Incluir biblioteca ${name}`;
      button.onclick = async () => {
        const tab = dependencies.activeTab();
        if (!tab || dependencies.isRunning(tab)) return;
        if (tab.language === "portugol-rs") {
          if (dependencies.insertText) {
            dependencies.insertText(`${name}.`);
            dependencies.focusEditor();
            dependencies.setStatus(`Módulo PORS inserido: ${name}. (use Ctrl+Espaço para completar).`, "ok");
          } else {
            dependencies.setStatus(`Use ${name}.funcao(...) diretamente em Portugol-RS.`);
          }
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

    const activeSnippets = porsMode && dependencies.porsSnippets?.length
      ? dependencies.porsSnippets
      : dependencies.snippets;
    for (const snippet of activeSnippets) {
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
  }

  function mount() {
    renderCatalog(Boolean(dependencies.porsLibraries?.length && dependencies.activeTab()?.language !== "portugol"));

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
    const porsMode = language === "portugol-rs" && Boolean(dependencies.porsLibraries?.length);
    if (dependencies.porsLibraries?.length || dependencies.porsSnippets?.length) {
      renderCatalog(porsMode);
    }
    if (porsMode) {
      for (const [name, button] of libraryButtons) {
        button.disabled = false;
        button.title = `Inserir módulo PORS ${name}.`;
      }
      return;
    }
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
