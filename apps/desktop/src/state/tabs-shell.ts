import type { Extension, EditorState } from "@codemirror/state";
import type { SourceLanguage } from "../document-language.ts";
import type { ExerciseDocument } from "../exercises.ts";
import { createTabFactory, isTabDirty, PORS_TEMPLATE, tabName } from "./tabs.ts";
import { createTabCollection } from "./tab-collection.ts";
import { createTabCloser } from "./tab-closer.ts";
import { createTabsView } from "./tabs-view.ts";
import type { Tab } from "./tab.ts";

export interface TabsShellDependencies {
  homeId: number;
  helpId: number;
  document: Pick<Document, "createElement"> & { title: string };
  container: HTMLElement;
  icon: (file: string) => HTMLElement;
  extensions: (language: SourceLanguage) => Extension[];
  activate: (id: number) => void;
  helpOpen: () => boolean;
  setHelpOpen: (open: boolean) => void;
  currentDocumentText: () => string;
  editorState: () => EditorState;
  setCursorText: (text: string) => void;
  confirmDiscard: (tab: Tab) => Promise<boolean>;
}

/** Composes tab state, lifecycle, rendering, and cursor presentation for the IDE shell. */
export function createTabsShell(dependencies: TabsShellDependencies) {
  const tabCollection = createTabCollection(dependencies.homeId);
  const activeTab = () => tabCollection.activeTab();
  const isDirty = (tab: Tab) => isTabDirty(tab, tabCollection.activeId(), dependencies.currentDocumentText());
  const tabFactory = createTabFactory({
    extensions: dependencies.extensions,
    addTab: (tab) => tabCollection.add(tab),
    activate: dependencies.activate,
  });

  function newTab(
    text = PORS_TEMPLATE,
    path: string | null = null,
    exampleName?: string,
    examplePath?: string,
    exercise?: ExerciseDocument,
    explicitLanguage?: SourceLanguage,
    workspaceFile?: { id: string; path: string; preferenceKey: string },
  ): Tab {
    return tabFactory.create({ text, path, exampleName, examplePath, exercise, explicitLanguage, workspaceFile });
  }

  let tabsView: ReturnType<typeof createTabsView>;
  function renderTabs() {
    tabsView.render();
  }

  const tabCloser = createTabCloser({
    tabs: tabCollection.tabs,
    activeId: tabCollection.activeId,
    homeId: dependencies.homeId,
    helpId: dependencies.helpId,
    setHelpOpen: dependencies.setHelpOpen,
    isDirty,
    tabName,
    confirmDiscard: dependencies.confirmDiscard,
    removeTab: tabCollection.removeAt,
    activate: dependencies.activate,
    renderTabs,
  });
  function closeTab(id: number): Promise<boolean> {
    return tabCloser.close(id);
  }

  tabsView = createTabsView({
    container: dependencies.container,
    document: dependencies.document,
    tabs: tabCollection.tabs,
    activeId: tabCollection.activeId,
    homeId: dependencies.homeId,
    helpId: dependencies.helpId,
    helpOpen: dependencies.helpOpen,
    tabName,
    isDirty,
    icon: dependencies.icon,
    activate: dependencies.activate,
    newTab: () => { newTab(); },
    closeTab,
  });

  function renderCursor() {
    if (!activeTab()) return;
    const state = dependencies.editorState();
    const head = state.selection.main.head;
    const line = state.doc.lineAt(head);
    dependencies.setCursorText(`Linha ${line.number}, coluna ${head - line.from + 1}`);
  }

  return { tabCollection, activeTab, isDirty, newTab, closeTab, renderTabs, renderCursor };
}
