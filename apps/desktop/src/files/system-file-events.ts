export interface SystemFileEventsDependencies {
  pendingFiles: () => Promise<string[]>;
  listenForFiles: (listener: () => void) => Promise<unknown>;
  listenForDrop: (listener: (payload: { type: string; paths?: string[] }) => void) => Promise<unknown>;
  openFiles: (paths: string[], fromDrop?: boolean) => Promise<void>;
  setStatus: (message: string, kind: "error") => void;
}

/** Liga notificações de arquivos do sistema, replay inicial e drag-and-drop. */
export function mountSystemFileEvents(dependencies: SystemFileEventsDependencies): void {
  async function consumePendingFiles() {
    try {
      await dependencies.openFiles(await dependencies.pendingFiles());
    } catch {
      dependencies.setStatus("Não foi possível receber arquivos do sistema.", "error");
    }
  }

  void dependencies.listenForFiles(() => { void consumePendingFiles(); })
    .then(consumePendingFiles, consumePendingFiles);

  void dependencies.listenForDrop((payload) => {
    if (payload.type === "drop") void dependencies.openFiles(payload.paths ?? [], true);
  }).catch(() => {});
}
