/**
 * Regras de posicionamento do designer, sem DOM: onde um controle solto ou
 * arrastado vai parar em cada tipo de contêiner, e como as alças mudam o
 * tamanho. A parte que lê o DOM fica em `interactions.ts`.
 */

/** Passo da grade de alinhamento (pixels lógicos). */
export const SNAP = 8;
/** Tamanho mínimo de um controle ao redimensionar. */
export const MIN_SIZE = 8;

export type Point = { x: number; y: number };
export type Rect = { x: number; y: number; width: number; height: number };
export type Handle = "n" | "s" | "e" | "w" | "ne" | "nw" | "se" | "sw";
export type DockSide = "Cima" | "Baixo" | "Esquerda" | "Direita";

export function snap(value: number, enabled = true): number {
  return enabled ? Math.round(value / SNAP) * SNAP : Math.round(value);
}

/**
 * Posição X/Y num contêiner livre. `point` é relativo à área de conteúdo do
 * contêiner (já sem borda e preenchimento) e `grab` é onde o controle foi
 * pego, relativo ao canto dele.
 */
export function freePosition(point: Point, grab: Point = { x: 0, y: 0 }, snapping = true): Point {
  return {
    x: Math.max(0, snap(point.x - grab.x, snapping)),
    y: Math.max(0, snap(point.y - grab.y, snapping)),
  };
}

/**
 * Posição de inserção numa `Pilha`: antes do primeiro filho cujo meio fica
 * depois do ponteiro. `children` traz os filhos visuais na ordem do arquivo,
 * com a posição de cada um entre os elementos do contêiner.
 */
export function stackIndex(
  point: Point,
  children: readonly { index: number; rect: Rect }[],
  horizontal: boolean,
  elementCount: number,
): number {
  for (const child of children) {
    const middle = horizontal ? child.rect.x + child.rect.width / 2 : child.rect.y + child.rect.height / 2;
    if ((horizontal ? point.x : point.y) < middle) return child.index;
  }
  return elementCount;
}

/** Trilhas resolvidas pelo navegador (`"40px 120.5px"`) em números. */
export function parseTracks(value: string): number[] {
  return value
    .split(/\s+/)
    .map((part) => Number.parseFloat(part))
    .filter((size) => Number.isFinite(size));
}

function trackAt(offset: number, tracks: readonly number[], gap: number): number {
  let edge = 0;
  for (let index = 0; index < tracks.length; index++) {
    edge += tracks[index];
    if (offset < edge + gap / 2) return index;
    edge += gap;
  }
  return Math.max(0, tracks.length - 1);
}

/** Linha e coluna da `Grade` sob o ponteiro (relativo à área de conteúdo). */
export function gridCell(
  point: Point,
  rows: readonly number[],
  columns: readonly number[],
  rowGap: number,
  columnGap: number,
): { row: number; column: number } {
  return { row: trackAt(point.y, rows, rowGap), column: trackAt(point.x, columns, columnGap) };
}

/** Borda do `Encaixe` mais perto do ponteiro. */
export function dockSide(point: Point, rect: Rect): DockSide {
  const distances: [DockSide, number][] = [
    ["Cima", point.y - rect.y],
    ["Baixo", rect.y + rect.height - point.y],
    ["Esquerda", point.x - rect.x],
    ["Direita", rect.x + rect.width - point.x],
  ];
  distances.sort((a, b) => a[1] - b[1]);
  return distances[0][0];
}

/** Novo retângulo ao arrastar uma alça por `delta`, com snap e tamanho mínimo. */
export function resizeRect(rect: Rect, handle: Handle, delta: Point, snapping = true): Rect {
  let { x, y, width, height } = rect;
  const right = x + width;
  const bottom = y + height;
  if (handle.includes("e")) width = snap(right + delta.x, snapping) - x;
  if (handle.includes("s")) height = snap(bottom + delta.y, snapping) - y;
  if (handle.includes("w")) {
    x = Math.min(snap(x + delta.x, snapping), right - MIN_SIZE);
    width = right - x;
  }
  if (handle.includes("n")) {
    y = Math.min(snap(y + delta.y, snapping), bottom - MIN_SIZE);
    height = bottom - y;
  }
  return { x, y, width: Math.max(MIN_SIZE, width), height: Math.max(MIN_SIZE, height) };
}

/**
 * Caminho do elemento depois de `move` (mesma regra do núcleo: `parent` e
 * `index` lidos antes da mudança).
 */
export function pathAfterMove(path: readonly number[], parent: readonly number[], index: number): number[] {
  const last = path[path.length - 1];
  const sourceParent = path.slice(0, -1);
  const target = [...parent];
  let position = index;
  const sameParent = sourceParent.length === target.length && sourceParent.every((value, i) => value === target[i]);
  if (sameParent) {
    if (last < position) position -= 1;
  } else if (
    target.length > sourceParent.length
    && sourceParent.every((value, i) => value === target[i])
    && target[sourceParent.length] > last
  ) {
    target[sourceParent.length] -= 1;
  }
  return [...target, position];
}

/** Se `inner` está dentro de `outer` (ou é ele). */
export function isWithin(inner: readonly number[], outer: readonly number[]): boolean {
  return inner.length >= outer.length && outer.every((value, index) => inner[index] === value);
}
