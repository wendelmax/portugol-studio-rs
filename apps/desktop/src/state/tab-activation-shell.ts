import type { EditorState } from "@codemirror/state";
import type { Tab } from "./tab";
import { $ } from "../dom";
import { createTabActivator } from "./tab-activation";

export interface TabActivationShellDependencies {
  tabs: {
    all: () => Tab[];
    current: () => Tab | undefined;
    setActiveId: (id: number) => void;
    render: () => void;
    renderCursor: () => void;
  };
  ids: { home: number; help: number };
  help: { setOpen: (open: boolean) => void; open: () => void };
  editor: {
    state: () => EditorState;
    setState: (tab: Tab) => void;
    configure: (tab: Tab) => void;
    clearRustDiagnostics: (tab: Tab) => void;
    centerSelectionEnabled: () => boolean;
    centerSelection: () => void;
    syncLanguageControls: (tab: Tab) => void;
    focus: () => void;
  };
  workspace: {
    select: (tab: Tab) => void;
    loadPanes: (tab: Tab) => void;
    applyPanes: () => void;
  };
  preview: {
    cancel: () => void;
    render: (tab: Tab) => void;
    renderMappingGutter: () => void;
    schedule: (tab: Tab) => void;
  };
  panels: {
    renderExerciseTests: () => void;
    renderMessages: () => void;
    showOutline: (tab: Tab, changed: boolean) => void;
    updateLibraryButtons: (tab: Tab) => void;
  };
  learning: {
    applyForTab: (tab: Tab) => void;
    bridgeMode: () => boolean;
  };
}

/** Adapta os módulos visuais e de estado ao ciclo de ativação de abas. */
export function createTabActivationShell(dependencies: TabActivationShellDependencies) {
  return createTabActivator({
    tabs: dependencies.tabs.all,
    currentTab: dependencies.tabs.current,
    setActiveId: dependencies.tabs.setActiveId,
    editorState: dependencies.editor.state,
    homeId: dependencies.ids.home,
    helpId: dependencies.ids.help,
    setHelpOpen: dependencies.help.setOpen,
    setPageVisibility: (id, tab) => {
      $("home").hidden = id !== dependencies.ids.home;
      $("help").hidden = id !== dependencies.ids.help;
      $("code").hidden = !tab;
      $("editor-exercise").hidden = !tab?.exercise;
    },
    cancelPreview: dependencies.preview.cancel,
    renderTabs: dependencies.tabs.render,
    renderExerciseTests: dependencies.panels.renderExerciseTests,
    clearCursor: () => { $("cursor").textContent = ""; },
    openHelp: dependencies.help.open,
    focusHome: () => $("home").focus(),
    selectWorkspace: dependencies.workspace.select,
    setEditorState: dependencies.editor.setState,
    loadPanes: dependencies.workspace.loadPanes,
    applyLearningMode: dependencies.learning.applyForTab,
    configureEditor: dependencies.editor.configure,
    applyPanes: dependencies.workspace.applyPanes,
    clearRustDiagnostics: dependencies.editor.clearRustDiagnostics,
    setRustPreviewHidden: (hidden) => { $("rust-preview").hidden = hidden; },
    renderRustPreview: dependencies.preview.render,
    renderMappingGutter: dependencies.preview.renderMappingGutter,
    renderMessages: dependencies.panels.renderMessages,
    showOutline: dependencies.panels.showOutline,
    updateLibraryButtons: dependencies.panels.updateLibraryButtons,
    renderCursor: dependencies.tabs.renderCursor,
    centerSelectionEnabled: dependencies.editor.centerSelectionEnabled,
    centerSelection: dependencies.editor.centerSelection,
    bridgeMode: dependencies.learning.bridgeMode,
    schedulePreview: dependencies.preview.schedule,
    syncLanguageControls: dependencies.editor.syncLanguageControls,
    focusEditor: dependencies.editor.focus,
  });
}
