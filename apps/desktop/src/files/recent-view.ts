export interface RecentFilesViewDependencies {
  recent: Pick<HTMLElement, "hidden">;
  list: Pick<HTMLElement, "replaceChildren">;
  document: Pick<Document, "createElement">;
  icon: (name: string) => HTMLElement;
  openFile: (path: string) => Promise<unknown>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
}

/** Renders and wires the recent-files list without owning its persisted state. */
export function createRecentFilesView(dependencies: RecentFilesViewDependencies) {
  function render(paths: readonly string[]) {
    dependencies.recent.hidden = paths.length === 0;
    dependencies.list.replaceChildren(...paths.map((path) => {
      const button = dependencies.document.createElement("button");
      const name = dependencies.document.createElement("span");
      name.textContent = path.split(/[\\/]/).pop() ?? path;
      button.append(dependencies.icon("grande/file"), name);
      button.title = path;
      button.onclick = () => {
        void dependencies.openFile(path).catch((error) => dependencies.setStatus(String(error), "error"));
      };
      return button;
    }));
  }

  return { render };
}
