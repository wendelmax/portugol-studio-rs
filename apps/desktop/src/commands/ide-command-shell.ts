import { isPaletteCommandAvailable, type PaletteContext } from "../command-palette";
import { $ } from "../dom";
import { mountCommandPalette } from "./palette";
import { createIdeShortcuts, type IdeShortcutActions } from "./ide-shortcuts";
import { mountKeyboardShortcutDispatch } from "./keyboard-dispatch";
import { renderShortcutList } from "./shortcuts";

export interface IdeCommandShellDependencies {
  actions: Omit<IdeShortcutActions, "showShortcuts" | "commandPalette">;
  paletteContext: () => PaletteContext;
  optionMenus: readonly HTMLDetailsElement[];
}

/** Compõe paleta, catálogo visível e captura global dos atalhos da IDE. */
export function mountIdeCommandShell(dependencies: IdeCommandShellDependencies): void {
  const shortcutDialog = $<HTMLDialogElement>("shortcuts");
  const showShortcuts = () => {
    if (!shortcutDialog.open) shortcutDialog.showModal();
  };

  let shortcuts: ReturnType<typeof createIdeShortcuts> = [];
  const commandPalette = mountCommandPalette(() =>
    shortcuts.filter(({ command }) => isPaletteCommandAvailable(command, dependencies.paletteContext())),
  );
  shortcuts = createIdeShortcuts({
    ...dependencies.actions,
    showShortcuts,
    commandPalette: commandPalette.show,
  });

  renderShortcutList(shortcuts);
  $("home-shortcuts").onclick = showShortcuts;
  mountKeyboardShortcutDispatch({
    target: window,
    hasOpenDialog: () => document.querySelector("dialog[open]") !== null,
    optionMenus: dependencies.optionMenus.map((menu) => ({
      get open() { return menu.open; },
      set open(value: boolean) { menu.open = value; },
      focusSummary: () => { menu.querySelector("summary")?.focus(); },
    })),
    shortcuts: () => shortcuts,
  });
}
