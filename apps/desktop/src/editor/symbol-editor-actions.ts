import type { EditorView as EditorViewType } from "@codemirror/view";
import type { RenameChange, RenameTarget, SymbolRenameUI } from "./symbol-rename";
import type { SemanticSymbolLanguage } from "../semantic-symbol-actions";
import type { SourceLanguage } from "../document-language";
import type { SymbolLocation } from "../api";
import type { Tab } from "../state/tab";
import { createSymbolNavigator } from "./symbol-navigation";
import { createSymbolRenamer } from "./symbol-rename";

export interface SymbolEditorActionsDependencies {
  activeTab: () => Tab | undefined;
  view: Pick<EditorViewType, "state" | "dispatch" | "focus">;
  runId: () => number | null;
  runTabId: () => number | null;
  hasOpenDialog: () => boolean;
  semanticSymbolLanguage: (language: SourceLanguage) => SemanticSymbolLanguage | null;
  renameTargetFerroRust: (source: string, offset: number) => Promise<RenameTarget>;
  renameSymbolFerroRust: (source: string, offset: number, name: string) => Promise<RenameChange[]>;
  resolveFerroRust: (source: string, offset: number) => Promise<SymbolLocation>;
  applyChanges: (changes: RenameChange[]) => void;
  focusEditor: () => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  ui: SymbolRenameUI;
}

/** Compõe a renomeação semântica e a navegação de declarações do editor. */
export function createSymbolEditorActions(dependencies: SymbolEditorActionsDependencies) {
  const renamer = createSymbolRenamer({
    activeTab: dependencies.activeTab,
    currentDocument: () => dependencies.view.state.doc,
    isRunning: (tab) => dependencies.runTabId() === tab.id && dependencies.runId() !== null,
    isCurrent: (tab, doc) => dependencies.activeTab() === tab && dependencies.view.state.doc === doc,
    hasOpenDialog: dependencies.hasOpenDialog,
    semanticSymbolLanguage: dependencies.semanticSymbolLanguage,
    renameTargetFerroRust: dependencies.renameTargetFerroRust,
    renameSymbolFerroRust: dependencies.renameSymbolFerroRust,
    applyChanges: dependencies.applyChanges,
    focusEditor: dependencies.focusEditor,
    setStatus: dependencies.setStatus,
    ui: dependencies.ui,
  });
  const navigator = createSymbolNavigator({
    activeTab: dependencies.activeTab,
    view: dependencies.view,
    resolveFerroRust: dependencies.resolveFerroRust,
    setStatus: dependencies.setStatus,
  });

  return {
    renameAt: (offset: number) => renamer.openAt(offset),
    goToDeclarationAt: (offset: number, quiet = false) => navigator.goToDeclarationAt(offset, quiet),
  };
}
