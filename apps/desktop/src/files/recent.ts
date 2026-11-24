// Lista de arquivos abertos recentemente, guardada no armazenamento local.

export const RECENT_KEY = "portugol.recent";
export const RECENT_LIMIT = 6;

type RecentStorage = Pick<Storage, "getItem" | "setItem">;

/** Lê a lista salva, ignorando valores inválidos. */
export function loadRecentFiles(storage: RecentStorage | null): string[] {
  try {
    const saved: unknown = JSON.parse(storage?.getItem(RECENT_KEY) ?? "[]");
    if (Array.isArray(saved)) {
      return saved.filter((path): path is string => typeof path === "string").slice(0, RECENT_LIMIT);
    }
  } catch {
    // A lista continua funcionando nesta sessão quando o armazenamento está indisponível.
  }
  return [];
}

/** Coloca `path` no topo, sem duplicar. */
export function withRecentFile(list: readonly string[], path: string): string[] {
  return [path, ...list.filter((entry) => entry !== path)].slice(0, RECENT_LIMIT);
}

/** Troca `oldPath` por `newPath` (arquivo renomeado), mantendo-o no topo. */
export function withRenamedRecentFile(list: readonly string[], oldPath: string, newPath: string): string[] {
  return [newPath, ...list.filter((path) => path !== oldPath && path !== newPath)].slice(0, RECENT_LIMIT);
}

export function withoutRecentFile(list: readonly string[], path: string): string[] {
  return list.filter((entry) => entry !== path);
}

/**
 * Estado da lista de recentes. Cada alteração é salva e notificada; se o
 * armazenamento falhar, a lista em memória continua valendo.
 */
export function createRecentFiles(storage: RecentStorage | null, onChange: (list: readonly string[]) => void) {
  let list = loadRecentFiles(storage);
  const update = (next: string[]) => {
    list = next;
    try {
      storage?.setItem(RECENT_KEY, JSON.stringify(list));
    } catch {
      // A lista em memória permanece disponível.
    }
    onChange(list);
  };
  return {
    list: (): readonly string[] => list,
    remember: (path: string) => update(withRecentFile(list, path)),
    rename: (oldPath: string, newPath: string) => update(withRenamedRecentFile(list, oldPath, newPath)),
    forget: (path: string) => update(withoutRecentFile(list, path)),
  };
}
