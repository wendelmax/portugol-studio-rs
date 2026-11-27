/**
 * Interações do canvas (ADR 0018, Fase 2): soltar controles da caixa de
 * ferramentas, arrastar para mover, alças para redimensionar e teclado
 * (setas, Delete, Esc). Nada aqui escreve texto: tudo vira {@link DesignEdit}
 * e passa por `edit_design` no núcleo, que reescreve o `.design`.
 */
import type { DesignEdit, DesignNode } from "../api.ts";
import { attributeOf, isComponentTag, layoutOf, nodeAt, pathFromKey, pathKey } from "./canvas-model.ts";
import { dockInsertIndex, elementCount, planInsert, planInsertComponent, planMove, type DropPlan, type DropTarget } from "./drop-edits.ts";
import { dockSide, freePosition, gridCell, parseTracks, resizeRect, snap, stackIndex, type Handle, type Point, type Rect } from "./placement.ts";

/** Tipo MIME do arrastar e soltar a partir da caixa de ferramentas. */
export const TOOL_MIME = "application/x-ferro-design-tool";

const CONTAINER_SELECTOR = ".design-window, .design-panel, .design-stack, .design-grid, .design-dock";
const DRAG_THRESHOLD = 4;
const ALL_HANDLES: Handle[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
const GROW_HANDLES: Handle[] = ["e", "se", "s"];

export interface CanvasInteractionHost {
  viewport: HTMLElement;
  frame: () => HTMLElement | null;
  tree: () => DesignNode | null;
  selected: () => string | null;
  select: (path: string | null, moveCursor: boolean) => void;
  apply: (plan: DropPlan | { edits: DesignEdit[]; select: number[] | null }) => Promise<boolean>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
}

function contentOrigin(element: HTMLElement): Point {
  const rect = element.getBoundingClientRect();
  const style = getComputedStyle(element);
  return {
    x: rect.left + element.clientLeft + (Number.parseFloat(style.paddingLeft) || 0),
    y: rect.top + element.clientTop + (Number.parseFloat(style.paddingTop) || 0),
  };
}

function toRect(rect: DOMRect): Rect {
  return { x: rect.left, y: rect.top, width: rect.width, height: rect.height };
}

/** Contêiner sob o ponteiro, ignorando o controle arrastado e o que há dentro dele. */
function containerAt(client: Point, excluded: number[] | null): HTMLElement | null {
  for (const element of document.elementsFromPoint(client.x, client.y)) {
    const container = (element as HTMLElement).closest<HTMLElement>(CONTAINER_SELECTOR);
    if (!container || container.dataset.path === undefined) continue;
    const path = pathFromKey(container.dataset.path);
    if (excluded && excluded.length <= path.length && excluded.every((v, i) => path[i] === v)) continue;
    return container;
  }
  return null;
}

/** Destino de um soltar no ponto `client`. `grab` é onde o controle foi pego. */
export function dropTargetAt(tree: DesignNode, client: Point, grab: Point, snapping: boolean, excluded: number[] | null): DropTarget | null {
  const container = containerAt(client, excluded);
  if (!container) return null;
  const parent = pathFromKey(container.dataset.path ?? "");
  const node = nodeAt(tree, parent);
  if (!node) return null;
  const origin = contentOrigin(container);
  const local = { x: client.x - origin.x, y: client.y - origin.y };
  const style = getComputedStyle(container);
  const count = elementCount(tree, parent);
  switch (layoutOf(node.tag)) {
    case "free": {
      const { x, y } = freePosition(local, grab, snapping);
      return { kind: "free", parent, index: count, x, y };
    }
    case "stack": {
      const children = [...container.querySelectorAll<HTMLElement>(":scope > [data-path]")]
        .filter((child) => !excluded || child.dataset.path !== pathKey(excluded))
        .map((child) => {
          const childPath = pathFromKey(child.dataset.path ?? "");
          return { index: childPath[childPath.length - 1], rect: toRect(child.getBoundingClientRect()) };
        });
      const index = stackIndex(client, children, style.flexDirection.startsWith("row"), count);
      return { kind: "stack", parent, index };
    }
    case "grid": {
      const cell = gridCell(
        local,
        parseTracks(style.gridTemplateRows),
        parseTracks(style.gridTemplateColumns),
        Number.parseFloat(style.rowGap) || 0,
        Number.parseFloat(style.columnGap) || 0,
      );
      return { kind: "grid", parent, index: count, ...cell };
    }
    case "dock":
      return { kind: "dock", parent, index: dockInsertIndex(node), side: dockSide(client, toRect(container.getBoundingClientRect())) };
    default:
      return null;
  }
}

/** Primeira posição livre para um controle inserido pelo teclado: em cascata a partir de (16, 16). */
function cascadePosition(tree: DesignNode, parent: number[]): Point {
  const node = nodeAt(tree, parent);
  const taken = new Set((node?.children ?? []).map((child) => `${attributeOf(child, "X")},${attributeOf(child, "Y")}`));
  let offset = 16;
  while (taken.has(`${offset},${offset}`)) offset += 16;
  return { x: offset, y: offset };
}

/** Destino de um controle inserido sem arrastar (clique ou Enter na caixa): o contêiner selecionado, ou a janela. */
export function keyboardTarget(tree: DesignNode, selected: string | null): DropTarget | null {
  let parent = selected === null ? [] : pathFromKey(selected);
  let node = nodeAt(tree, parent);
  while (node && layoutOf(node.tag) === "none" && parent.length > 0) {
    parent = parent.slice(0, -1);
    node = nodeAt(tree, parent);
  }
  if (!node) return null;
  const count = node.children.length;
  switch (layoutOf(node.tag)) {
    case "free":
      return { kind: "free", parent, index: count, ...cascadePosition(tree, parent) };
    case "stack":
      return { kind: "stack", parent, index: count };
    case "grid":
      return { kind: "grid", parent, index: count, row: 0, column: 0 };
    case "dock":
      return { kind: "dock", parent, index: dockInsertIndex(node), side: "Cima" };
    default:
      return null;
  }
}

/** Plano para inserir `tag` em `target` (componentes vão sempre para a janela). */
export function insertPlan(tree: DesignNode, tag: string, target: DropTarget | null): DropPlan | null {
  if (isComponentTag(tag)) return planInsertComponent(tag, tree);
  return target ? planInsert(tag, target) : null;
}

export function mountCanvasInteractions(host: CanvasInteractionHost) {
  const { viewport } = host;
  const overlay = document.createElement("div");
  overlay.className = "design-overlay";
  overlay.hidden = true;
  const handles = new Map<Handle, HTMLElement>();
  for (const handle of ALL_HANDLES) {
    const element = document.createElement("div");
    element.className = `design-handle design-handle-${handle}`;
    element.dataset.handle = handle;
    element.setAttribute("aria-hidden", "true");
    overlay.append(element);
    handles.set(handle, element);
  }
  let suppressClick = false;

  const selectedElement = (): HTMLElement | null => {
    const frame = host.frame();
    const path = host.selected();
    if (!frame || path === null) return null;
    if (path === "") return frame.querySelector<HTMLElement>(".design-window");
    return frame.querySelector<HTMLElement>(`[data-path="${CSS.escape(path)}"]`);
  };

  function parentLayout(path: number[]) {
    const tree = host.tree();
    if (!tree || path.length === 0) return "none";
    const parent = nodeAt(tree, path.slice(0, -1));
    return parent ? layoutOf(parent.tag) : "none";
  }

  /** Reposiciona as alças sobre o controle selecionado. */
  function placeOverlay() {
    const element = selectedElement();
    const path = host.selected();
    if (!element || path === null || element.classList.contains("design-component")) {
      overlay.hidden = true;
      return;
    }
    if (!overlay.isConnected) viewport.append(overlay);
    const rect = element.getBoundingClientRect();
    const base = viewport.getBoundingClientRect();
    overlay.style.left = `${rect.left - base.left + viewport.scrollLeft}px`;
    overlay.style.top = `${rect.top - base.top + viewport.scrollTop}px`;
    overlay.style.width = `${rect.width}px`;
    overlay.style.height = `${rect.height}px`;
    const free = parentLayout(pathFromKey(path)) === "free";
    const visible = free ? ALL_HANDLES : GROW_HANDLES;
    for (const [handle, node] of handles) node.hidden = !visible.includes(handle);
    overlay.hidden = false;
  }

  // Soltar a partir da caixa de ferramentas.
  viewport.addEventListener("dragover", (event) => {
    if (!event.dataTransfer?.types.includes(TOOL_MIME)) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = "copy";
  });
  viewport.addEventListener("drop", (event) => {
    const tag = event.dataTransfer?.getData(TOOL_MIME);
    const tree = host.tree();
    if (!tag || !tree) return;
    event.preventDefault();
    const target = dropTargetAt(tree, { x: event.clientX, y: event.clientY }, { x: 0, y: 0 }, !event.altKey, null);
    const plan = insertPlan(tree, tag, target);
    if (plan) void host.apply(plan);
    else host.setStatus("Solte o controle dentro da janela, de um Painel, Pilha, Grade ou Encaixe.", "error");
  });

  // Arrastar para mover.
  viewport.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) return;
    const target = event.target as HTMLElement;
    const handle = target.closest<HTMLElement>("[data-handle]");
    if (handle) {
      startResize(event, handle.dataset.handle as Handle);
      return;
    }
    const element = target.closest<HTMLElement>(".design-el[data-path]");
    if (!element || element.dataset.path === "" || element.classList.contains("design-component")) return;
    const path = pathFromKey(element.dataset.path ?? "");
    const start = { x: event.clientX, y: event.clientY };
    const rect = element.getBoundingClientRect();
    const grab = { x: start.x - rect.left, y: start.y - rect.top };
    let dragging = false;

    const move = (moveEvent: PointerEvent) => {
      const dx = moveEvent.clientX - start.x;
      const dy = moveEvent.clientY - start.y;
      if (!dragging && Math.hypot(dx, dy) < DRAG_THRESHOLD) return;
      if (!dragging) {
        dragging = true;
        element.classList.add("is-dragging");
        element.style.pointerEvents = "none";
        overlay.hidden = true;
      }
      element.style.transform = `translate(${dx}px, ${dy}px)`;
    };
    const up = (upEvent: PointerEvent) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      if (!dragging) return;
      suppressClick = true;
      const tree = host.tree();
      const drop = tree ? dropTargetAt(tree, { x: upEvent.clientX, y: upEvent.clientY }, grab, !upEvent.altKey, path) : null;
      element.style.transform = "";
      element.style.pointerEvents = "";
      element.classList.remove("is-dragging");
      const plan = drop ? planMove(path, drop) : null;
      if (plan) void host.apply(plan);
      else {
        host.setStatus("Não dá para soltar o controle aí.", "error");
        placeOverlay();
      }
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  });

  function startResize(event: PointerEvent, handle: Handle) {
    const element = selectedElement();
    const selected = host.selected();
    if (!element || selected === null) return;
    event.preventDefault();
    event.stopPropagation();
    const path = pathFromKey(selected);
    const free = parentLayout(path) === "free";
    const startRect: Rect = {
      x: free ? element.offsetLeft : 0,
      y: free ? element.offsetTop : 0,
      width: element.offsetWidth,
      height: element.offsetHeight,
    };
    const start = { x: event.clientX, y: event.clientY };
    let current = startRect;
    const move = (moveEvent: PointerEvent) => {
      current = resizeRect(startRect, handle, { x: moveEvent.clientX - start.x, y: moveEvent.clientY - start.y }, !moveEvent.altKey);
      element.style.width = `${current.width}px`;
      element.style.height = `${current.height}px`;
      if (free) {
        element.style.left = `${current.x}px`;
        element.style.top = `${current.y}px`;
      }
      placeOverlay();
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      suppressClick = true;
      if (current === startRect) return;
      const edits: DesignEdit[] = [
        { kind: "setAttribute", path, name: "Largura", value: String(current.width) },
        { kind: "setAttribute", path, name: "Altura", value: String(current.height) },
      ];
      if (free && (current.x !== startRect.x || current.y !== startRect.y)) {
        edits.push(
          { kind: "setAttribute", path, name: "X", value: String(current.x) },
          { kind: "setAttribute", path, name: "Y", value: String(current.y) },
        );
      }
      void host.apply({ edits, select: path });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  // O clique que encerra um arrasto não deve mudar a seleção.
  viewport.addEventListener("click", (event) => {
    if (!suppressClick) return;
    suppressClick = false;
    event.stopPropagation();
  }, true);

  // Teclado: setas movem (Shift: um passo da grade), Delete apaga, Esc sobe para o contêiner.
  viewport.addEventListener("keydown", (event) => {
    const selected = host.selected();
    const tree = host.tree();
    if (selected === null || !tree) return;
    const path = pathFromKey(selected);
    if (event.key === "Escape" && path.length > 0) {
      event.preventDefault();
      host.select(pathKey(path.slice(0, -1)), true);
      return;
    }
    if ((event.key === "Delete" || event.key === "Backspace") && path.length > 0) {
      event.preventDefault();
      void host.apply({ edits: [{ kind: "remove", path }], select: path.length > 1 ? path.slice(0, -1) : null });
      return;
    }
    const arrows: Record<string, Point> = { ArrowLeft: { x: -1, y: 0 }, ArrowRight: { x: 1, y: 0 }, ArrowUp: { x: 0, y: -1 }, ArrowDown: { x: 0, y: 1 } };
    const arrow = arrows[event.key];
    if (!arrow || parentLayout(path) !== "free") return;
    event.preventDefault();
    const node = nodeAt(tree, path);
    if (!node) return;
    const step = event.shiftKey ? 8 : 1;
    const x = Number(attributeOf(node, "X") ?? 0) + arrow.x * step;
    const y = Number(attributeOf(node, "Y") ?? 0) + arrow.y * step;
    const edits: DesignEdit[] = [];
    if (arrow.x) edits.push({ kind: "setAttribute", path, name: "X", value: String(Math.max(0, event.shiftKey ? snap(x) : x)) });
    if (arrow.y) edits.push({ kind: "setAttribute", path, name: "Y", value: String(Math.max(0, event.shiftKey ? snap(y) : y)) });
    void host.apply({ edits, select: path });
  });

  viewport.addEventListener("scroll", () => placeOverlay());

  return {
    /** Depois de redesenhar o canvas ou mudar a seleção. */
    refresh: placeOverlay,
  };
}
