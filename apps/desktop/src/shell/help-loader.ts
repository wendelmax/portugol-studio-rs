// A ajuda é carregada sob demanda na primeira vez que é aberta.

import { $ } from "../dom";
import { setStatus } from "./status";

type Help = ReturnType<typeof import("../help").mountHelp>;

export function createHelpLoader(options: {
  /** Abre um exemplo da ajuda numa nova aba. */
  openExample: (source: string, file: string) => void;
  /** Abre um exemplo da ajuda numa nova aba e executa na VM. */
  runExample?: (source: string, file: string) => void;
  /** Abre o Laboratório de Ownership a partir da Ajuda. */
  openOwnershipLab?: () => void;
  /** A aba de ajuda está ativa (para focar a busca ao abrir). */
  isActive: () => boolean;
}) {
  let help: Help | undefined;
  let helpLoading: Promise<void> | undefined;

  async function ensureLoaded() {
    if (!help) {
      helpLoading ??= import("../help").then(({ mountHelp }) => {
        help = mountHelp({
          openExample: options.openExample,
          runExample: options.runExample,
          openOwnershipLab: options.openOwnershipLab,
        });
      }).catch((error) => {
        $("help-article").textContent = "Não foi possível carregar a ajuda.";
        setStatus(String(error), "error");
        helpLoading = undefined;
      });
      await helpLoading;
    }
  }

  async function open(topicId?: string) {
    await ensureLoaded();
    if (topicId) {
      help?.navigateTo(topicId);
    } else if (options.isActive()) {
      help?.focusSearch();
    }
  }

  return {
    open,
    /** Reaplica o tema à ajuda, se ela já foi carregada. */
    refreshTheme: () => help?.refreshTheme(),
  };
}
