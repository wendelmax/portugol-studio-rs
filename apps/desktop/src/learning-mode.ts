export type LearningMode = "beginner" | "bridge";

const STORAGE_KEY = "portugol.learningMode";

type StorageReader = Pick<Storage, "getItem">;
type StorageWriter = Pick<Storage, "setItem">;

export function loadLearningMode(storage?: StorageReader): LearningMode {
  try {
    const value = (storage ?? globalThis.localStorage).getItem(STORAGE_KEY);
    return value === "bridge" ? "bridge" : "beginner";
  } catch {
    return "beginner";
  }
}

export function saveLearningMode(mode: LearningMode, storage?: StorageWriter): boolean {
  try {
    (storage ?? globalThis.localStorage).setItem(STORAGE_KEY, mode);
    return true;
  } catch {
    return false;
  }
}
