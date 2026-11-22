export const PANE_LAYOUT_KEY = "portugol.pane-layout.v1";

function workspaceStorageKey(preferenceKey: string): string {
  return `${PANE_LAYOUT_KEY}.workspace.${encodeURIComponent(preferenceKey)}`;
}

export const DEFAULT_PANE_LAYOUT = {
  explorer: 0.16,
  bottom: 0.25,
  utilities: 0.23,
} as const;

export type PaneLayout = { explorer: number; bottom: number; utilities: number };

const limits = {
  explorer: [0.12, 0.42],
  bottom: [0.12, 0.65],
  utilities: [0.15, 0.55],
} as const;

function ratio(value: unknown, fallback: number, minimum: number, maximum: number): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return fallback;
  return Math.min(maximum, Math.max(minimum, value));
}

export function normalizePaneLayout(value: Partial<Record<keyof PaneLayout, unknown>>): PaneLayout {
  return {
    explorer: ratio(value.explorer, DEFAULT_PANE_LAYOUT.explorer, ...limits.explorer),
    bottom: ratio(value.bottom, DEFAULT_PANE_LAYOUT.bottom, ...limits.bottom),
    utilities: ratio(value.utilities, DEFAULT_PANE_LAYOUT.utilities, ...limits.utilities),
  };
}

function parsePaneLayout(encoded: string | null): PaneLayout | null {
  if (!encoded) return null;
  try {
    const parsed: unknown = JSON.parse(encoded);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return null;
    return normalizePaneLayout(parsed as Partial<Record<keyof PaneLayout, unknown>>);
  } catch {
    return null;
  }
}

export function loadPaneLayout(storage: Pick<Storage, "getItem">, preferenceKey?: string): PaneLayout {
  try {
    const workspace = preferenceKey
      ? parsePaneLayout(storage.getItem(workspaceStorageKey(preferenceKey)))
      : null;
    return workspace ?? parsePaneLayout(storage.getItem(PANE_LAYOUT_KEY)) ?? { ...DEFAULT_PANE_LAYOUT };
  } catch {
    return { ...DEFAULT_PANE_LAYOUT };
  }
}

export function savePaneLayout(storage: Pick<Storage, "setItem">, layout: PaneLayout, preferenceKey?: string): boolean {
  try {
    const key = preferenceKey ? workspaceStorageKey(preferenceKey) : PANE_LAYOUT_KEY;
    storage.setItem(key, JSON.stringify(normalizePaneLayout(layout)));
    return true;
  } catch {
    return false;
  }
}

export function resizePaneForKey(
  key: string,
  orientation: "horizontal" | "vertical",
  direction: 1 | -1,
  current: number,
  bounds: { minimum: number; maximum: number },
): number | null {
  if (key === "Home") return bounds.minimum;
  if (key === "End") return bounds.maximum;
  const increment = orientation === "horizontal"
    ? key === "ArrowUp"
    : key === (direction === 1 ? "ArrowRight" : "ArrowLeft");
  const decrement = orientation === "horizontal"
    ? key === "ArrowDown"
    : key === (direction === 1 ? "ArrowLeft" : "ArrowRight");
  if (increment) return clampPaneSize(current + 24, bounds.minimum, bounds.maximum);
  if (decrement) return clampPaneSize(current - 24, bounds.minimum, bounds.maximum);
  return null;
}

export function calculateSidePaneMinimums(
  available: number,
  fixedWidth: number,
  explorerSeparator: number,
  utilitiesSeparator: number,
  showExplorer: boolean,
  showUtilities: boolean,
): { explorer: number; utilities: number } {
  const remaining = Math.max(0, available - fixedWidth
    - (showExplorer ? explorerSeparator : 0)
    - (showUtilities ? utilitiesSeparator : 0));
  if (showExplorer && showUtilities) {
    const scale = Math.min(1, remaining / (180 + 210));
    return { explorer: 180 * scale, utilities: 210 * scale };
  }
  if (showExplorer) return { explorer: Math.min(180, remaining), utilities: 0 };
  if (showUtilities) return { explorer: 0, utilities: Math.min(210, remaining) };
  return { explorer: 0, utilities: 0 };
}

export function clampPaneSize(value: number, minimum: number, maximum: number): number {
  const ceiling = Math.max(minimum, maximum);
  return Math.min(ceiling, Math.max(minimum, value));
}
