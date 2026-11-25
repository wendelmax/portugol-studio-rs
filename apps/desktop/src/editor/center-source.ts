export interface CenterSourceDependencies {
  storage: () => Pick<Storage, "getItem" | "setItem"> | null;
  hasActiveTab: () => boolean;
  centerSelection: () => void;
  setPressed: (pressed: boolean) => void;
  focusEditor: () => void;
}

const PREFERENCE_KEY = "portugol.centerSource";

/** Restores and updates the editor's center-source preference. */
export function createCenterSourceController(dependencies: CenterSourceDependencies) {
  let active = false;
  try {
    active = dependencies.storage()?.getItem(PREFERENCE_KEY) === "true";
  } catch {
    // A preferência permanece disponível durante esta sessão.
  }
  dependencies.setPressed(active);

  function toggle() {
    if (!dependencies.hasActiveTab()) return;
    active = !active;
    dependencies.setPressed(active);
    try {
      dependencies.storage()?.setItem(PREFERENCE_KEY, String(active));
    } catch {
      // A preferência permanece válida nesta sessão.
    }
    if (active) dependencies.centerSelection();
    dependencies.focusEditor();
  }

  return { isActive: () => active, toggle };
}
