/**
 * Edições que o designer pede ao soltar um controle da caixa de ferramentas
 * ou ao arrastar um controle que já está na tela. Sem DOM: recebe o destino
 * já calculado ({@link DropTarget}) e a árvore do `.design`.
 */
import type { DesignEdit, DesignNode } from "../api.ts";
import { attributeOf, isVisualTag, nodeAt } from "./canvas-model.ts";
import { isWithin, pathAfterMove, type DockSide } from "./placement.ts";

export type DropTarget =
  | { kind: "free"; parent: number[]; index: number; x: number; y: number }
  | { kind: "stack"; parent: number[]; index: number }
  | { kind: "grid"; parent: number[]; index: number; row: number; column: number }
  | { kind: "dock"; parent: number[]; index: number; side: DockSide };

export type DropPlan = { edits: DesignEdit[]; select: number[] };

/** Atributos que posicionam o controle no destino. */
function placementAttributes(target: DropTarget): [string, string][] {
  switch (target.kind) {
    case "free":
      return [["X", String(target.x)], ["Y", String(target.y)]];
    case "grid":
      return [["Linha", String(target.row)], ["Coluna", String(target.column)]];
    case "dock":
      return [["Encaixar", target.side]];
    case "stack":
      return [];
  }
}

/** Soltar um controle novo da caixa de ferramentas. */
export function planInsert(tag: string, target: DropTarget): DropPlan {
  return {
    edits: [{ kind: "insert", parent: target.parent, index: target.index, tag, attributes: placementAttributes(target) }],
    select: [...target.parent, target.index],
  };
}

/** Componente sem aparência: sempre no fim da `Janela`. */
export function planInsertComponent(tag: string, root: DesignNode): DropPlan {
  const index = root.children.length;
  return { edits: [{ kind: "insert", parent: [], index, tag }], select: [index] };
}

const samePath = (a: readonly number[], b: readonly number[]) => a.length === b.length && a.every((v, i) => v === b[i]);

/**
 * Arrastar um controle que já está na tela. No mesmo contêiner só mudam os
 * atributos de posição (ou a ordem, na `Pilha`); em outro contêiner, o
 * controle é movido e ganha os atributos do destino. Devolve `null` se o
 * destino for o próprio controle ou algo dentro dele.
 */
export function planMove(path: readonly number[], target: DropTarget): DropPlan | null {
  if (path.length === 0 || isWithin(target.parent, path)) return null;
  const parent = path.slice(0, -1);
  if (samePath(parent, target.parent)) {
    if (target.kind === "stack") {
      const select = pathAfterMove(path, target.parent, target.index);
      return { edits: [{ kind: "move", path: [...path], parent: target.parent, index: target.index }], select };
    }
    return {
      edits: placementAttributes(target).map(([name, value]) => ({ kind: "setAttribute", path: [...path], name, value })),
      select: [...path],
    };
  }
  const moved = pathAfterMove(path, target.parent, target.index);
  return {
    edits: [
      { kind: "move", path: [...path], parent: target.parent, index: target.index },
      ...placementAttributes(target).map(([name, value]): DesignEdit => ({ kind: "setAttribute", path: moved, name, value })),
    ],
    select: moved,
  };
}

/**
 * Onde um controle entra no `Encaixe`: antes do último filho visual quando ele
 * preenche o centro (não tem `Encaixar`), para que o centro continue sendo o
 * último; senão, no fim.
 */
export function dockInsertIndex(dock: DesignNode): number {
  const visual = dock.children.filter((child) => isVisualTag(child.tag));
  const last = visual[visual.length - 1];
  if (last && attributeOf(last, "Encaixar") === undefined) return last.path[last.path.length - 1];
  return dock.children.length;
}

/** Número de elementos (de qualquer tipo) dentro do contêiner, para inserir no fim. */
export function elementCount(root: DesignNode, parent: readonly number[]): number {
  return nodeAt(root, parent)?.children.length ?? 0;
}
