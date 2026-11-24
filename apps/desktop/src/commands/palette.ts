// Paleta de comandos (Ctrl+Shift+P): busca, navegação por teclado e execução.

import { filterCommands, nextCommandIndex, type CommandSearchItem } from "../command-palette";
import { $ } from "../dom";

export interface PaletteCommand extends CommandSearchItem {
  action: () => unknown;
}

/** `availableCommands` é consultado a cada abertura e antes de executar. */
export function mountCommandPalette<T extends PaletteCommand>(availableCommands: () => T[]) {
  const commandPalette = $<HTMLDialogElement>("command-palette");
  const commandSearch = $<HTMLInputElement>("command-search");
  const commandResults = $("command-results");
  const commandEmpty = $("command-empty");
  let paletteReturnFocus: HTMLElement | null = null;
  let paletteCommands: T[] = [];
  let selectedCommandIndex = -1;

  function renderCommandPalette() {
    paletteCommands = filterCommands(availableCommands(), commandSearch.value);
    selectedCommandIndex = paletteCommands.length > 0 ? Math.min(Math.max(selectedCommandIndex, 0), paletteCommands.length - 1) : -1;
    commandResults.replaceChildren();
    commandEmpty.hidden = paletteCommands.length > 0;
    commandSearch.setAttribute("aria-expanded", "true");
    for (const [index, item] of paletteCommands.entries()) {
      const option = document.createElement("button");
      option.type = "button";
      option.className = "command-option";
      option.id = `command-option-${index}`;
      option.setAttribute("role", "option");
      option.setAttribute("aria-selected", String(index === selectedCommandIndex));
      const name = document.createElement("span");
      name.textContent = item.command;
      const shortcut = document.createElement("kbd");
      shortcut.textContent = item.label;
      option.append(name, shortcut);
      option.addEventListener("mouseenter", () => {
        selectedCommandIndex = index;
        updateSelectedPaletteCommand();
      });
      option.addEventListener("click", () => executePaletteCommand(index));
      commandResults.append(option);
    }
    updateSelectedPaletteCommand();
  }

  function updateSelectedPaletteCommand() {
    for (const [index, option] of [...commandResults.querySelectorAll<HTMLElement>("[role=option]")].entries()) {
      option.setAttribute("aria-selected", String(index === selectedCommandIndex));
    }
    const id = selectedCommandIndex >= 0 ? `command-option-${selectedCommandIndex}` : "";
    if (id) commandSearch.setAttribute("aria-activedescendant", id);
    else commandSearch.removeAttribute("aria-activedescendant");
  }

  function executePaletteCommand(index: number) {
    const item = paletteCommands[index];
    if (!item) return;
    if (!availableCommands().includes(item)) {
      renderCommandPalette();
      return;
    }
    commandPalette.close();
    void item.action();
  }

  function showCommandPalette() {
    if (commandPalette.open) return;
    paletteReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    commandSearch.value = "";
    selectedCommandIndex = 0;
    renderCommandPalette();
    commandPalette.showModal();
    commandSearch.focus();
  }

  commandSearch.addEventListener("input", () => {
    selectedCommandIndex = 0;
    renderCommandPalette();
  });
  commandSearch.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      selectedCommandIndex = nextCommandIndex(paletteCommands.length, selectedCommandIndex, event.key);
      updateSelectedPaletteCommand();
    } else if (event.key === "Enter") {
      event.preventDefault();
      executePaletteCommand(selectedCommandIndex);
    }
  });
  commandPalette.addEventListener("close", () => {
    const focusTarget = paletteReturnFocus;
    paletteReturnFocus = null;
    if (focusTarget?.isConnected && !focusTarget.hasAttribute("disabled")) focusTarget.focus();
  });

  return { show: showCommandPalette };
}
