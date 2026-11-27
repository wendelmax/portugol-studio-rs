/**
 * Modo visual das telas (ADR 0018, Fase 2): as vistas de uma aba `.design`.
 *
 * - *Design*: só a janela desenhada;
 * - *Dividido*: a janela em cima e o pseudo-XAML embaixo;
 * - *Marcação*: só o pseudo-XAML (é como o `.design` abre direto);
 * - *Código* (F7): abre o code-behind `.pors` da tela.
 *
 * Abrir o nó de tela no Explorador vai para *Design*. Nesta etapa o canvas só
 * desenha e seleciona; arrastar, redimensionar e o painel de propriedades vêm
 * nos próximos PRs.
 */
import type { DesignNode, DesignSchema } from "../api.ts";
import { fileExtension, fileKind, stripFileExtension } from "../file-formats.ts";
import type { Tab } from "../state/tab.ts";
import { buildCanvasModel, nodeAt, pathAtOffset, pathFromKey, pathKey, type DefaultSizes } from "./canvas-model.ts";
import { markSelection, renderCanvas } from "./canvas-dom.ts";

export type DesignView = "design" | "split" | "text";

export interface DesignerShellDependencies {
  area: HTMLElement;
  toolbar: HTMLElement;
  surface: HTMLElement;
  status: HTMLElement;
  buttons: Record<DesignView | "code", HTMLButtonElement>;
  activeTab: () => Tab | undefined;
  source: () => string;
  cursorOffset: () => number;
  selectInEditor: (from: number) => void;
  designTree: (source: string) => Promise<DesignNode | null>;
  designSchema: () => Promise<DesignSchema>;
  hasFile: (workspaceId: string | undefined, path: string) => boolean;
  openFile: (workspaceId: string | undefined, path: string) => Promise<boolean>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  setTimeout?: (callback: () => void, delay: number) => unknown;
  clearTimeout?: (handle: unknown) => void;
}

const RENDER_DELAY_MS = 120;
const VIEW_LABELS: Record<DesignView, string> = { design: "Design", split: "Dividido", text: "Marcação" };

/** Se a aba é o desenho de uma tela (e não o manifesto). */
export function isDesignTab(tab: Tab | undefined): boolean {
  return Boolean(tab && fileKind(tab.workspacePath ?? tab.path ?? "") === "design");
}

/** Caminho do code-behind de um `.design`: `App/JanelaPrincipal.design` → `App/JanelaPrincipal.pors`. */
export function codeBehindPath(designPath: string): string {
  const slash = designPath.lastIndexOf("/");
  const folder = slash >= 0 ? designPath.slice(0, slash + 1) : "";
  return `${folder}${stripFileExtension(designPath.slice(slash + 1))}.${fileExtension("source")}`;
}

/** Caminho do `.design` de um code-behind, se o arquivo for código-fonte. */
export function designPathForCode(codePath: string): string | null {
  if (fileKind(codePath) !== "source") return null;
  const slash = codePath.lastIndexOf("/");
  const folder = slash >= 0 ? codePath.slice(0, slash + 1) : "";
  return `${folder}${stripFileExtension(codePath.slice(slash + 1))}.${fileExtension("design")}`;
}

export function mountDesignerShell(dependencies: DesignerShellDependencies) {
  const setTimer = dependencies.setTimeout ?? ((callback, delay) => setTimeout(callback, delay));
  const clearTimer = dependencies.clearTimeout ?? ((handle) => clearTimeout(handle as number));
  const views = new Map<number, DesignView>();
  const selections = new Map<number, string | null>();
  let tree: DesignNode | null = null;
  let sizes: DefaultSizes = {};
  let timer: unknown = null;
  let renderGeneration = 0;

  void dependencies.designSchema().then((schema) => {
    sizes = Object.fromEntries(
      schema.elements.map((element) => [element.name, [element.defaultWidth, element.defaultHeight] as const]),
    );
    refresh();
  }).catch(() => undefined);

  const viewOf = (tab: Tab) => views.get(tab.id) ?? "text";

  function applyView(tab: Tab | undefined) {
    dependencies.toolbar.hidden = !tab || !isDesignTab(tab);
    if (!tab || !isDesignTab(tab)) {
      dependencies.surface.hidden = true;
      delete dependencies.area.dataset.designView;
      return;
    }
    const view = viewOf(tab);
    dependencies.surface.hidden = view === "text";
    if (view === "text") delete dependencies.area.dataset.designView;
    else dependencies.area.dataset.designView = view;
    for (const [name, button] of Object.entries(dependencies.buttons)) {
      if (name === "code") continue;
      button.setAttribute("aria-pressed", String(name === view));
    }
    const code = codeBehindPath(tab.workspacePath ?? "");
    const hasCode = Boolean(tab.workspaceId && dependencies.hasFile(tab.workspaceId, code));
    dependencies.buttons.code.disabled = !hasCode;
    dependencies.buttons.code.title = hasCode
      ? `Abrir o código da tela (${code.split("/").pop()}) — F7`
      : "Esta tela ainda não tem código: ele é criado quando você liga o primeiro evento";
  }

  async function render() {
    const tab = dependencies.activeTab();
    if (!tab || !isDesignTab(tab) || viewOf(tab) === "text") return;
    const generation = ++renderGeneration;
    const next = await dependencies.designTree(dependencies.source()).catch(() => null);
    if (generation !== renderGeneration || dependencies.activeTab() !== tab) return;
    if (!next) {
      dependencies.surface.classList.add("is-stale");
      dependencies.status.textContent = "Corrija o erro na marcação para atualizar o desenho.";
      return;
    }
    tree = next;
    dependencies.surface.classList.remove("is-stale");
    dependencies.status.textContent = "";
    const frame = renderCanvas(buildCanvasModel(next, sizes));
    dependencies.surface.replaceChildren(frame);
    const selected = selections.get(tab.id) ?? null;
    markSelection(frame, selected !== null && nodeAt(next, pathFromKey(selected)) ? selected : null);
  }

  function refresh() {
    if (timer !== null) clearTimer(timer);
    timer = null;
    void render();
  }

  function scheduleRender() {
    if (timer !== null) clearTimer(timer);
    timer = setTimer(() => {
      timer = null;
      void render();
    }, RENDER_DELAY_MS);
  }

  function setView(view: DesignView) {
    const tab = dependencies.activeTab();
    if (!tab || !isDesignTab(tab)) return;
    views.set(tab.id, view);
    applyView(tab);
    if (view !== "text") refresh();
    dependencies.setStatus(`Vista ${VIEW_LABELS[view]}`, "ok");
  }

  function select(path: string | null, moveCursor: boolean) {
    const tab = dependencies.activeTab();
    if (!tab) return;
    selections.set(tab.id, path);
    const frame = dependencies.surface.firstElementChild as HTMLElement | null;
    if (frame) markSelection(frame, path);
    if (moveCursor && path !== null && tree) {
      const node = nodeAt(tree, pathFromKey(path));
      if (node) dependencies.selectInEditor(node.from);
    }
  }

  /** F7 e o botão *Código*: do desenho para o code-behind, e de volta. */
  async function toggleCode(): Promise<boolean> {
    const tab = dependencies.activeTab();
    if (!tab?.workspaceId || !tab.workspacePath) return false;
    if (isDesignTab(tab)) {
      const code = codeBehindPath(tab.workspacePath);
      if (!dependencies.hasFile(tab.workspaceId, code)) {
        dependencies.setStatus("Esta tela ainda não tem código: ele é criado quando você liga o primeiro evento.");
        return true;
      }
      return dependencies.openFile(tab.workspaceId, code);
    }
    const design = designPathForCode(tab.workspacePath);
    if (!design || !dependencies.hasFile(tab.workspaceId, design)) return false;
    const opened = await dependencies.openFile(tab.workspaceId, design);
    const designTab = dependencies.activeTab();
    if (opened && designTab && isDesignTab(designTab) && viewOf(designTab) === "text") {
      views.set(designTab.id, "design");
      sync();
    }
    return opened;
  }

  function sync() {
    const tab = dependencies.activeTab();
    applyView(tab);
    tree = null;
    dependencies.surface.replaceChildren();
    if (tab && isDesignTab(tab) && viewOf(tab) !== "text") refresh();
  }

  dependencies.buttons.design.addEventListener("click", () => setView("design"));
  dependencies.buttons.split.addEventListener("click", () => setView("split"));
  dependencies.buttons.text.addEventListener("click", () => setView("text"));
  dependencies.buttons.code.addEventListener("click", () => void toggleCode());

  dependencies.surface.addEventListener("click", (event) => {
    const target = (event.target as HTMLElement | null)?.closest<HTMLElement>("[data-path]");
    select(target?.dataset.path ?? null, true);
  });

  return {
    /** Depois de trocar de aba. */
    sync,
    /** A aba ativa veio do nó de tela no Explorador: abre no modo visual. */
    screenOpened() {
      const tab = dependencies.activeTab();
      if (!tab || !isDesignTab(tab)) return;
      if (viewOf(tab) === "text") views.set(tab.id, "design");
      sync();
    },
    /** Mudanças no editor: redesenha o texto alterado e segue o cursor. */
    editorChanged(docChanged: boolean, selectionSet: boolean) {
      const tab = dependencies.activeTab();
      if (!tab || !isDesignTab(tab) || viewOf(tab) === "text") return;
      if (docChanged) scheduleRender();
      else if (selectionSet && tree) {
        const path = pathAtOffset(tree, dependencies.cursorOffset());
        select(path ? pathKey(path) : null, false);
      }
    },
    toggleCode,
    /** Esquece a vista e a seleção de uma aba fechada. */
    forget(tabId: number) {
      views.delete(tabId);
      selections.delete(tabId);
    },
  };
}
