/**
 * Modo visual das telas (ADR 0018, Fase 2): as vistas de uma aba `.design`.
 *
 * - *Design*: só a janela desenhada;
 * - *Dividido*: a janela em cima e o pseudo-XAML embaixo;
 * - *Marcação*: só o pseudo-XAML (é como o `.design` abre direto);
 * - *Código* (F7): abre o code-behind `.fer` da tela.
 *
 * Abrir o nó de tela no Explorador vai para *Design*. Nesta etapa o canvas só
 * desenha e seleciona; arrastar, redimensionar e o painel de propriedades vêm
 * nos próximos PRs.
 */
import type { DataField, DesignEdit, DesignNode, DesignSchema } from "../api.ts";
import { acceptedExtensions, fileExtension, fileKind, stripFileExtension } from "../file-formats.ts";
import type { Tab } from "../state/tab.ts";
import { buildCanvasModel, nodeAt, pathAtOffset, pathFromKey, pathKey, type DefaultSizes } from "./canvas-model.ts";
import { markSelection, renderCanvas } from "./canvas-dom.ts";
import type { CanvasInteractionHost } from "./interactions.ts";
import { insertPlan, keyboardTarget } from "./interactions.ts";
import { codeBehindTemplate, handlerName, placeHandler } from "./handlers.ts";
import { schemaFor } from "./properties-model.ts";
import { attributeOf } from "./canvas-model.ts";

export type DesignView = "design" | "split" | "text";

/** O que o canvas e o painel lateral recebem do shell. */
export type DesignerHost = CanvasInteractionHost & {
  schema: () => DesignSchema | null;
  sizes: () => DefaultSizes;
  dataFields: () => DataField[] | null;
};

export interface DesignerShellDependencies {
  area: HTMLElement;
  toolbar: HTMLElement;
  surface: HTMLElement;
  /** Onde a janela desenhada é montada, ao lado da caixa de ferramentas. */
  viewport: HTMLElement;
  status: HTMLElement;
  buttons: Record<DesignView | "code", HTMLButtonElement> & { undo?: HTMLButtonElement; redo?: HTMLButtonElement };
  activeTab: () => Tab | undefined;
  source: () => string;
  cursorOffset: () => number;
  selectInEditor: (from: number) => void;
  designTree: (source: string) => Promise<DesignNode | null>;
  designSchema: () => Promise<DesignSchema>;
  /** Aplica edições estruturais e devolve o novo texto (rejeita com a mensagem). */
  editDesign: (source: string, edits: DesignEdit[]) => Promise<string>;
  /** Troca o texto do editor pelo novo `.design`. */
  replaceSource: (source: string) => void;
  /** Arrastar, redimensionar e teclado no canvas; opcional para testes sem DOM. */
  mountInteractions?: (host: DesignerHost) => { refresh: () => void };
  /** Campos do registro de `Dados` da tela, lidos do code-behind (para os dados de exemplo). */
  dataFields?: (tab: Tab, designSource: string) => Promise<DataField[]>;
  /** Desfazer e refazer do editor: o designer só muda o texto, então o histórico é um só. */
  undo?: () => boolean;
  redo?: () => boolean;
  /** Desenho do canvas; o padrão monta o DOM com `canvas-dom.ts`. Substituível em testes sem DOM. */
  canvas?: {
    paint: (tree: DesignNode, sizes: DefaultSizes, selected: string | null) => void;
    highlight: (selected: string | null) => void;
  };
  hasFile: (workspaceId: string | undefined, path: string) => boolean;
  openFile: (workspaceId: string | undefined, path: string) => Promise<boolean>;
  /** Cria um arquivo vazio no workspace e o abre numa aba (duplo clique numa tela sem código). */
  createFile?: (workspaceId: string | undefined, path: string) => Promise<boolean>;
  /** Insere texto no editor ativo (`insert` nulo só move o cursor) e o foca. */
  editActive?: (insert: { from: number; insert: string } | null, cursor: number) => void;
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

/** Caminho canônico do code-behind de um `.design`: `App/JanelaPrincipal.design` → `App/JanelaPrincipal.fer`. */
export function codeBehindPath(designPath: string): string {
  return codeBehindCandidates(designPath)[0];
}

/**
 * Candidatos de code-behind baseados nas extensões aceitas para código-fonte.
 */
export function codeBehindCandidates(designPath: string): string[] {
  const slash = designPath.lastIndexOf("/");
  const folder = slash >= 0 ? designPath.slice(0, slash + 1) : "";
  const stem = stripFileExtension(designPath.slice(slash + 1));
  return acceptedExtensions("source").map((extension) => `${folder}${stem}.${extension}`);
}

/** Caminho do `.design` de um code-behind, se o arquivo for código-fonte. */
export function designPathForCode(codePath: string): string | null {
  if (fileKind(codePath) !== "source") return null;
  const slash = codePath.lastIndexOf("/");
  const folder = slash >= 0 ? codePath.slice(0, slash + 1) : "";
  return `${folder}${stripFileExtension(codePath.slice(slash + 1))}.${fileExtension("design")}`;
}

/**
 * Menor troca que leva `previous` a `next`: o designer reescreve o arquivo
 * inteiro, mas o editor recebe só o trecho que mudou (preserva rolagem e
 * deixa o desfazer do editor com passos pequenos).
 */
export function textChange(previous: string, next: string): { from: number; to: number; insert: string } | null {
  if (previous === next) return null;
  let start = 0;
  const limit = Math.min(previous.length, next.length);
  while (start < limit && previous.charCodeAt(start) === next.charCodeAt(start)) start++;
  let endPrevious = previous.length;
  let endNext = next.length;
  while (endPrevious > start && endNext > start && previous.charCodeAt(endPrevious - 1) === next.charCodeAt(endNext - 1)) {
    endPrevious--;
    endNext--;
  }
  return { from: start, to: endPrevious, insert: next.slice(start, endNext) };
}

export function mountDesignerShell(dependencies: DesignerShellDependencies) {
  const setTimer = dependencies.setTimeout ?? ((callback, delay) => setTimeout(callback, delay));
  const clearTimer = dependencies.clearTimeout ?? ((handle) => clearTimeout(handle as number));
  const views = new Map<number, DesignView>();
  const selections = new Map<number, string | null>();
  let tree: DesignNode | null = null;
  let sizes: DefaultSizes = {};
  let schema: DesignSchema | null = null;
  let dataFields: DataField[] | null = null;
  let timer: unknown = null;
  let renderGeneration = 0;
  let interactions: { refresh: () => void } | null = null;
  const currentFrame = () => dependencies.viewport.querySelector<HTMLElement>(".design-frame");
  const canvas = dependencies.canvas ?? {
    paint(next: DesignNode, defaults: DefaultSizes, selected: string | null) {
      const frame = renderCanvas(buildCanvasModel(next, defaults));
      dependencies.viewport.replaceChildren(frame);
      markSelection(frame, selected);
    },
    highlight(selected: string | null) {
      const frame = currentFrame();
      if (frame) markSelection(frame, selected);
    },
  };

  void dependencies.designSchema().then((loaded) => {
    schema = loaded;
    sizes = Object.fromEntries(
      loaded.elements.map((element) => [element.name, [element.defaultWidth, element.defaultHeight] as const]),
    );
    refresh();
  }).catch(() => undefined);

  const viewOf = (tab: Tab) => views.get(tab.id) ?? "text";

  /** Code-behind que existe no workspace, em qualquer extensão aceita. */
  function existingCodeBehind(tab: Tab): string | null {
    if (!tab.workspaceId || !tab.workspacePath) return null;
    return codeBehindCandidates(tab.workspacePath).find((path) => dependencies.hasFile(tab.workspaceId, path)) ?? null;
  }

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
    const code = existingCodeBehind(tab);
    dependencies.buttons.code.disabled = code === null;
    dependencies.buttons.code.title = code !== null
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
    let selected = selections.get(tab.id) ?? null;
    if (selected !== null && !nodeAt(next, pathFromKey(selected))) selected = null;
    selections.set(tab.id, selected);
    canvas.paint(next, sizes, selected);
    interactions?.refresh();
    if (dependencies.dataFields) {
      const source = dependencies.source();
      void dependencies.dataFields(tab, source).then((fields) => {
        if (generation !== renderGeneration || dependencies.activeTab() !== tab) return;
        dataFields = fields;
        interactions?.refresh();
      }).catch(() => undefined);
    }
  }

  /** Aplica edições do designer, troca o texto e redesenha já, sem esperar o atraso da digitação. */
  async function apply(plan: { edits: DesignEdit[]; select: number[] | null }): Promise<boolean> {
    const tab = dependencies.activeTab();
    if (!tab || !isDesignTab(tab) || plan.edits.length === 0) return false;
    const source = dependencies.source();
    try {
      const next = await dependencies.editDesign(source, plan.edits);
      if (dependencies.activeTab() !== tab || dependencies.source() !== source) return false;
      selections.set(tab.id, plan.select ? pathKey(plan.select) : null);
      if (next !== source) dependencies.replaceSource(next);
      refresh();
      return true;
    } catch (error) {
      dependencies.setStatus(String(error).replace(/^Error: /, ""), "error");
      refresh();
      return false;
    }
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
    canvas.highlight(path);
    interactions?.refresh();
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
      const code = existingCodeBehind(tab);
      if (code === null) {
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
    dataFields = null;
    dependencies.viewport.replaceChildren();
    if (tab && isDesignTab(tab) && viewOf(tab) !== "text") refresh();
  }

  dependencies.buttons.design.addEventListener("click", () => setView("design"));
  dependencies.buttons.split.addEventListener("click", () => setView("split"));
  dependencies.buttons.text.addEventListener("click", () => setView("text"));
  dependencies.buttons.code.addEventListener("click", () => void toggleCode());

  interactions = dependencies.mountInteractions?.({
    viewport: dependencies.viewport,
    frame: currentFrame,
    tree: () => tree,
    selected: () => {
      const tab = dependencies.activeTab();
      return tab ? (selections.get(tab.id) ?? null) : null;
    },
    select: (path, moveCursor) => select(path, moveCursor),
    apply,
    setStatus: dependencies.setStatus,
    schema: () => schema,
    sizes: () => sizes,
    dataFields: () => dataFields,
  }) ?? null;

  function history(direction: "undo" | "redo") {
    const run = direction === "undo" ? dependencies.undo : dependencies.redo;
    if (run?.()) refresh();
  }
  dependencies.buttons.undo?.addEventListener("click", () => history("undo"));
  dependencies.buttons.redo?.addEventListener("click", () => history("redo"));
  dependencies.viewport.addEventListener("keydown", (event) => {
    if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
    const key = event.key.toLowerCase();
    if (key === "z" && !event.shiftKey) {
      event.preventDefault();
      history("undo");
    } else if (key === "y" || (key === "z" && event.shiftKey)) {
      event.preventDefault();
      history("redo");
    }
  });

  /**
   * Duplo clique num controle: liga o evento padrão a uma função (a que já
   * estiver ligada, ou `btnOk_clicar`) e leva ao code-behind, criando o
   * arquivo e a função quando faltam.
   */
  async function createHandler(path: string): Promise<boolean> {
    const tab = dependencies.activeTab();
    if (!tab || !isDesignTab(tab) || !tree || !schema || !tab.workspaceId || !tab.workspacePath) return false;
    const node = nodeAt(tree, pathFromKey(path));
    const info = node && schemaFor(schema, node.tag);
    if (!node || !info) return false;
    const event = info.defaultEvent;
    if (!event) {
      dependencies.setStatus(`${info.label} não tem evento padrão; ligue um evento na aba Eventos.`);
      return false;
    }
    const window = attributeOf(tree, "Nome") ?? "Janela";
    const controlName = attributeOf(node, "Nome") ?? (path === "" ? window : node.tag.charAt(0).toLowerCase() + node.tag.slice(1));
    const handler = handlerName(attributeOf(node, event), controlName, event);
    if (attributeOf(node, event) !== handler) {
      const applied = await apply({ edits: [{ kind: "setAttribute", path: pathFromKey(path), name: event, value: handler }], select: pathFromKey(path) });
      if (!applied) return false;
    }
    const workspaceId = tab.workspaceId;
    const existing = existingCodeBehind(tab);
    let created = false;
    if (existing) {
      if (!(await dependencies.openFile(workspaceId, existing))) return false;
    } else {
      if (!dependencies.createFile || !(await dependencies.createFile(workspaceId, codeBehindPath(tab.workspacePath)))) return false;
      created = true;
    }
    let code = dependencies.source();
    if (created && code === "") {
      const template = codeBehindTemplate(window, attributeOf(tree, "Dados") ?? null);
      dependencies.editActive?.({ from: 0, insert: template }, template.length);
      code = template;
    }
    const plan = placeHandler(code, handler, window);
    dependencies.editActive?.(plan.insert === null ? null : { from: plan.from, insert: plan.insert }, plan.cursor);
    dependencies.setStatus(plan.insert === null ? `Função ${handler}` : `Função ${handler} criada no código da tela`, "ok");
    return true;
  }

  dependencies.viewport.addEventListener("dblclick", (event) => {
    const target = (event.target as HTMLElement | null)?.closest<HTMLElement>("[data-path]");
    if (target?.dataset.path !== undefined) void createHandler(target.dataset.path);
  });

  dependencies.viewport.addEventListener("click", (event) => {
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
    createHandler,
    /** Clique ou Enter num item da caixa de ferramentas: insere no contêiner selecionado. */
    async insertTool(tag: string): Promise<boolean> {
      const tab = dependencies.activeTab();
      if (!tab || !isDesignTab(tab) || !tree) return false;
      const plan = insertPlan(tree, tag, keyboardTarget(tree, selections.get(tab.id) ?? null));
      return plan ? apply(plan) : false;
    },
    /** Esquece a vista e a seleção de uma aba fechada. */
    forget(tabId: number) {
      views.delete(tabId);
      selections.delete(tabId);
    },
  };
}
