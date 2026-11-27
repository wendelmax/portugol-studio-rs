import { isolateHistory } from "@codemirror/commands";
import { $ } from "../dom";
import * as api from "../api";
import { semanticSymbolLanguage } from "../semantic-symbol-actions";
import { setStatus } from "../shell/status";
import type { EditorFormatterDependencies } from "./format";
import { createEditorFormatter } from "./format";
import type { SymbolEditorActionsDependencies } from "./symbol-editor-actions";
import { createSymbolEditorActions } from "./symbol-editor-actions";

export interface EditorActionsShellDependencies {
  view: SymbolEditorActionsDependencies["view"] & EditorFormatterDependencies["view"];
  activeTab: SymbolEditorActionsDependencies["activeTab"];
  runId: SymbolEditorActionsDependencies["runId"];
  runTabId: SymbolEditorActionsDependencies["runTabId"];
  rustfmtAvailable: EditorFormatterDependencies["rustfmtAvailable"];
  centerSelection: () => void;
  toggleCenterSource: () => void;
}

/** Compõe navegação semântica, renomeação, formatação e ações de seleção do editor. */
export function createEditorActionsShell(dependencies: EditorActionsShellDependencies) {
  const symbolActions = createSymbolEditorActions({
    activeTab: dependencies.activeTab,
    view: dependencies.view,
    runId: dependencies.runId,
    runTabId: dependencies.runTabId,
    hasOpenDialog: () => document.querySelector("dialog[open]") !== null,
    semanticSymbolLanguage,
    renameTargetPortugolRs: api.renameTargetPortugolRs,
    renameSymbolPortugolRs: api.renameSymbolPortugolRs,
    resolvePortugolRs: api.resolveSymbolPortugolRs,
    applyChanges: (changes) => dependencies.view.dispatch({
      changes,
      annotations: isolateHistory.of("full"),
      userEvent: "input.rename",
    }),
    focusEditor: () => dependencies.view.focus(),
    setStatus,
    ui: {
      dialog: $<HTMLDialogElement>("rename"),
      form: $<HTMLFormElement>("rename-form"),
      input: $<HTMLInputElement>("rename-input"),
      error: $("rename-error"),
      submit: $<HTMLButtonElement>("rename-submit"),
      description: $("rename-description"),
      cancelButton: $<HTMLButtonElement>("rename-cancel"),
    },
  });
  const formatter = createEditorFormatter({
    activeTab: dependencies.activeTab,
    isRunningTab: (tab) => dependencies.runTabId() === tab.id && dependencies.runId() !== null,
    rustfmtAvailable: dependencies.rustfmtAvailable,
    view: dependencies.view,
    formatRust: api.formatRustSource,
    formatPortugolRs: api.formatPortugolRs,
    formatMarkup: api.formatMarkup,
    setStatus,
  });

  return {
    centerSelection: dependencies.centerSelection,
    toggleCenterSource: dependencies.toggleCenterSource,
    renameAt: (offset: number) => symbolActions.renameAt(offset),
    goToDeclarationAt: (offset: number, quiet = false) => symbolActions.goToDeclarationAt(offset, quiet),
    renameSelection: () => symbolActions.renameAt(dependencies.view.state.selection.main.from),
    goToDeclarationSelection: () => symbolActions.goToDeclarationAt(dependencies.view.state.selection.main.head),
    formatCode: () => formatter.formatCode(),
  };
}
