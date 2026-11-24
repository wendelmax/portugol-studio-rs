// Botões da barra de título e confirmação ao fechar com alterações não salvas.

import { ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { $ } from "../dom";
import { setStatus } from "./status";

/** `unsavedNames` lista as abas com alterações no momento do fechamento. */
export function mountWindowControls(unsavedNames: () => string[]) {
  const appWindow = getCurrentWindow();
  const windowAction = (action: () => Promise<void>) => {
    void action().catch((error) => setStatus(String(error), "error"));
  };
  $("window-min").onclick = () => windowAction(() => appWindow.minimize());
  $("window-max").onclick = () => windowAction(() => appWindow.toggleMaximize());
  $("window-close").onclick = () => windowAction(() => appWindow.close());
  void appWindow.onCloseRequested(async (event) => {
    const names = unsavedNames();
    if (names.length === 0) return;
    const quit = await ask(`Há alterações não salvas em: ${names.join(", ")}. Sair mesmo assim?`, {
      title: "Sair", kind: "warning", okLabel: "Sair sem salvar", cancelLabel: "Cancelar",
    });
    if (!quit) event.preventDefault();
  });
}
