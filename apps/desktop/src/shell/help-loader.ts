// A ajuda é carregada sob demanda na primeira vez que é aberta.

import { $ } from "../dom";
import { setStatus } from "./status";

type Help = ReturnType<typeof import("../help").mountHelp>;

export function createHelpLoader(options: {
  /** Abre um exemplo da ajuda numa nova aba. */
  openExample: (source: string, file: string) => void;
  /** A aba de ajuda está ativa (para focar a busca ao abrir). */
  isActive: () => boolean;
}) {
  let help: Help | undefined;
  let helpLoading: Promise<void> | undefined;

  async function open() {
    if (!help) {
      helpLoading ??= import("../help").then(({ mountHelp }) => {
        help = mountHelp(options.openExample);
      }).catch((error) => {
        $("help-article").textContent = "Não foi possível carregar a ajuda.";
        setStatus(String(error), "error");
        helpLoading = undefined;
      });
      await helpLoading;
    }
    if (options.isActive()) help?.focusSearch();
  }

  return {
    open,
    /** Reaplica o tema à ajuda, se ela já foi carregada. */
    refreshTheme: () => help?.refreshTheme(),
  };
}
