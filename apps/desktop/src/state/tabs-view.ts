import type { Tab } from "./tab";

export interface TabsViewDependencies {
  container: HTMLElement;
  document: Pick<Document, "createElement"> & { title: string };
  tabs: () => Tab[];
  activeId: () => number;
  homeId: number;
  helpId: number;
  helpOpen: () => boolean;
  tabName: (tab: Tab) => string;
  isDirty: (tab: Tab) => boolean;
  icon: (file: string) => HTMLElement;
  activate: (id: number) => void;
  newTab: () => void;
  closeTab: (id: number) => Promise<boolean>;
}

/** Renderiza a faixa de abas; decisões de navegação continuam na shell. */
export function createTabsView(dependencies: TabsViewDependencies) {
  function render() {
    const { container, document } = dependencies;
    const activeId = dependencies.activeId();
    const home = document.createElement("button");
    home.className = "tab home-tab" + (activeId === dependencies.homeId ? " active" : "");
    home.append(dependencies.icon("home"), "Início");
    home.title = "Tela inicial (Alt+Home)";
    home.setAttribute("aria-current", String(activeId === dependencies.homeId));
    home.onclick = () => dependencies.activate(dependencies.homeId);

    const add = document.createElement("button");
    add.className = "tab add-tab";
    add.textContent = "+";
    add.title = "Novo arquivo (Ctrl+N)";
    add.setAttribute("aria-label", "Novo arquivo");
    add.onclick = dependencies.newTab;

    const helpTab = document.createElement("div");
    helpTab.className = "file-tab tab" + (activeId === dependencies.helpId ? " active" : "");
    const helpButton = document.createElement("button");
    helpButton.textContent = "Ajuda";
    helpButton.setAttribute("aria-current", String(activeId === dependencies.helpId));
    helpButton.onclick = () => dependencies.activate(dependencies.helpId);
    const closeHelp = document.createElement("button");
    closeHelp.className = "close";
    closeHelp.textContent = "×";
    closeHelp.setAttribute("aria-label", "Fechar ajuda");
    closeHelp.onclick = () => void dependencies.closeTab(dependencies.helpId);
    helpTab.append(helpButton, closeHelp);

    container.replaceChildren(
      home,
      ...(dependencies.helpOpen() ? [helpTab] : []),
      ...dependencies.tabs().map((tab) => {
        const element = document.createElement("div");
        element.className = "file-tab tab" + (tab.id === activeId ? " active" : "");
        element.title = tab.path ?? "Arquivo ainda não salvo";
        const name = document.createElement("button");
        name.setAttribute("aria-current", String(tab.id === activeId));
        name.textContent = dependencies.tabName(tab);
        if (dependencies.isDirty(tab)) name.className = "dirty";
        const close = document.createElement("button");
        close.className = "close";
        close.textContent = "×";
        close.title = "Fechar (Ctrl+Q)";
        close.onclick = (event) => {
          event.stopPropagation();
          void dependencies.closeTab(tab.id);
        };
        name.prepend(dependencies.icon("file-code"));
        element.append(name, close);
        name.onclick = () => dependencies.activate(tab.id);
        element.onauxclick = (event) => event.button === 1 && void dependencies.closeTab(tab.id);
        return element;
      }),
      add,
    );
    const tab = dependencies.tabs().find(({ id }) => id === activeId);
    const title = tab ? dependencies.tabName(tab) : activeId === dependencies.helpId ? "Ajuda" : "";
    document.title = title ? `${title} — Portugol Studio` : "Portugol Studio";
  }

  return { render };
}
