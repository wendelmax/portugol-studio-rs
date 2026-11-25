import type { Shortcut } from "./shortcuts.ts";

export interface KeyboardOptionMenu {
  open: boolean;
  focusSummary: () => void;
}

export interface KeyboardShortcutDispatchDependencies {
  target: Pick<Window, "addEventListener">;
  hasOpenDialog: () => boolean;
  optionMenus: readonly KeyboardOptionMenu[];
  shortcuts: () => readonly Shortcut[];
}

/** Installs the global keyboard guards and dispatches matching IDE shortcuts. */
export function mountKeyboardShortcutDispatch(dependencies: KeyboardShortcutDispatchDependencies): void {
  dependencies.target.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || dependencies.hasOpenDialog()) return;
    if (event.key === "Escape" && !event.shiftKey) {
      const menu = dependencies.optionMenus.find((options) => options.open);
      if (menu) {
        event.preventDefault();
        menu.open = false;
        menu.focusSummary();
        return;
      }
    }
    const shortcut = dependencies.shortcuts().find(({ matches }) => matches(event));
    if (!shortcut) return;
    event.preventDefault();
    void shortcut.action();
  });
}

export interface TabCycleDependencies {
  homeId: number;
  helpId: number;
  helpOpen: boolean;
  tabs: readonly { id: number }[];
  activeId: number;
  activate: (id: number) => void;
}

/** Cycles through home, optional help, and editor tabs. */
export function cycleIdeTab(step: number, dependencies: TabCycleDependencies): void {
  const ids = [
    dependencies.homeId,
    ...(dependencies.helpOpen ? [dependencies.helpId] : []),
    ...dependencies.tabs.map(({ id }) => id),
  ];
  dependencies.activate(ids[(ids.indexOf(dependencies.activeId) + step + ids.length) % ids.length]);
}
