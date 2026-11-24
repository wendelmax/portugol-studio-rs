import type { Tab } from "./tab";

export interface TabCloserDependencies {
  tabs: () => Tab[];
  activeId: () => number;
  homeId: number;
  helpId: number;
  setHelpOpen: (open: boolean) => void;
  isDirty: (tab: Tab) => boolean;
  tabName: (tab: Tab) => string;
  confirmDiscard: (tab: Tab) => Promise<boolean>;
  removeTab: (index: number) => void;
  activate: (id: number) => void;
  renderTabs: () => void;
}

/** Trata fechamento das abas sem acoplar confirmação e coleção à shell. */
export function createTabCloser(dependencies: TabCloserDependencies) {
  async function close(id: number): Promise<boolean> {
    if (id === dependencies.helpId) {
      dependencies.setHelpOpen(false);
      if (dependencies.activeId() === dependencies.helpId) {
        const tabs = dependencies.tabs();
        dependencies.activate(tabs[tabs.length - 1]?.id ?? dependencies.homeId);
      } else {
        dependencies.renderTabs();
      }
      return true;
    }
    const tab = dependencies.tabs().find((candidate) => candidate.id === id);
    if (!tab) return true;
    if (dependencies.isDirty(tab) && !(await dependencies.confirmDiscard(tab))) return false;
    const index = dependencies.tabs().indexOf(tab);
    dependencies.removeTab(index);
    if (id === dependencies.activeId()) {
      dependencies.activate(dependencies.tabs()[Math.max(0, index - 1)]?.id ?? dependencies.homeId);
    } else {
      dependencies.renderTabs();
    }
    return true;
  }

  return { close };
}
