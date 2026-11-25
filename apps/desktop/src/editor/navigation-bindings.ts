export interface EditorNavigationBindingsDependencies {
  activeLanguage: () => string | undefined;
  selectionHead: () => number;
  goToDeclarationAt: (offset: number, quiet?: boolean) => Promise<void>;
}

interface EditorAtCoordinates {
  posAtCoords: (coordinates: { x: number; y: number }) => number | null;
}

/** Liga o clique com modificador e F12 à navegação de declarações. */
export function createEditorNavigationBindings(dependencies: EditorNavigationBindingsDependencies) {
  function onEditorClick(event: Event, editor: EditorAtCoordinates): boolean {
    const mouse = event as MouseEvent;
    if (dependencies.activeLanguage() === "rust") return false;
    if (mouse.button !== 0 || (!mouse.ctrlKey && !mouse.metaKey)) return false;
    const offset = editor.posAtCoords({ x: mouse.clientX, y: mouse.clientY });
    if (offset === null) return false;
    void dependencies.goToDeclarationAt(offset, true);
    return false;
  }

  function onGoToDeclaration(): boolean {
    void dependencies.goToDeclarationAt(dependencies.selectionHead());
    return true;
  }

  return { onEditorClick, onGoToDeclaration };
}
