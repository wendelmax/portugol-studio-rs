export type LearningMode = "beginner" | "bridge" | "rust";

const STORAGE_KEY = "ferro.learningMode";

function workspaceStorageKey(preferenceKey: string): string {
  return `${STORAGE_KEY}.workspace.${encodeURIComponent(preferenceKey)}`;
}

type StorageReader = Pick<Storage, "getItem">;
type StorageWriter = Pick<Storage, "setItem">;

export function loadLearningMode(storage?: StorageReader, preferenceKey?: string): LearningMode {
  try {
    const source = storage ?? globalThis.localStorage;
    const workspaceValue = preferenceKey ? source.getItem(workspaceStorageKey(preferenceKey)) : null;
    const value = workspaceValue ?? source.getItem(STORAGE_KEY);
    return value === "bridge" || value === "rust" ? value : "beginner";
  } catch {
    return "beginner";
  }
}

export function saveLearningMode(mode: LearningMode, storage?: StorageWriter, preferenceKey?: string): boolean {
  try {
    const key = preferenceKey ? workspaceStorageKey(preferenceKey) : STORAGE_KEY;
    (storage ?? globalThis.localStorage).setItem(key, mode);
    return true;
  } catch {
    return false;
  }
}
