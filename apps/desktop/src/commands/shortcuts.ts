// Atalhos de teclado. Também alimentam a paleta de comandos e a lista exibida
// em "Atalhos de teclado" (F11).

import { $ } from "../dom";

export interface Shortcut {
  /** Texto do atalho exibido ao usuário, como "Ctrl+S". */
  label: string;
  command: string;
  matches: (event: KeyboardEvent) => boolean;
  action: () => unknown;
}

/** Atalho Ctrl+tecla (ou Ctrl+Shift+tecla), como os do menu Arquivo. */
export function ctrlShortcut(key: string, shift: boolean, command: string, action: () => unknown): Shortcut {
  return {
    label: shift ? `Ctrl+Shift+${key}` : `Ctrl+${key}`,
    command,
    matches: (e) => (e.ctrlKey || e.metaKey) && e.shiftKey === shift && e.key.toUpperCase() === key,
    action,
  };
}

/** Preenche a lista do diálogo "Atalhos de teclado". */
export function renderShortcutList(shortcuts: readonly Shortcut[]) {
  for (const { label, command } of shortcuts) {
    const term = document.createElement("dt");
    const shortcut = document.createElement("kbd");
    const description = document.createElement("dd");
    shortcut.textContent = label;
    description.textContent = command;
    term.append(shortcut);
    $("shortcut-list").append(term, description);
  }
}
