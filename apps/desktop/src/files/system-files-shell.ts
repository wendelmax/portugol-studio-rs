import { getCurrentWindow } from "@tauri-apps/api/window";
import * as api from "../api";
import { createIncomingFileOpener, type IncomingFileDependencies } from "./system-files";
import { mountSystemFileEvents } from "./system-file-events";

export type SystemFilesShellDependencies = Pick<IncomingFileDependencies, "readExercise" | "openExercise" | "openFile" | "setStatus">;

/** Compõe abertura recebida do sistema, eventos da plataforma e drag-and-drop. */
export function mountSystemFilesShell(dependencies: SystemFilesShellDependencies): void {
  const openFiles = createIncomingFileOpener(dependencies);
  mountSystemFileEvents({
    pendingFiles: api.pendingFiles,
    listenForFiles: api.onSystemFiles,
    listenForDrop: (listener) => getCurrentWindow().onDragDropEvent(({ payload }) => listener(payload)),
    openFiles,
    setStatus: (message) => dependencies.setStatus(message, "error"),
  });
}
