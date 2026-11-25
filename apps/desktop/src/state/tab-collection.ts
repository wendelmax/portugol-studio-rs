import type { Tab } from "./tab";

/** Owns the open tab list and selected tab id; lifecycle effects stay in shell controllers. */
export function createTabCollection(initialActiveId: number) {
  let openTabs: Tab[] = [];
  let selectedId = initialActiveId;

  return {
    tabs(): Tab[] {
      return [...openTabs];
    },
    activeId(): number {
      return selectedId;
    },
    activeTab(): Tab | undefined {
      return openTabs.find((tab) => tab.id === selectedId);
    },
    setActiveId(id: number): void {
      selectedId = id;
    },
    add(tab: Tab): void {
      openTabs.push(tab);
    },
    removeAt(index: number): void {
      openTabs.splice(index, 1);
    },
    replace(tabs: readonly Tab[]): void {
      openTabs = [...tabs];
    },
  };
}
