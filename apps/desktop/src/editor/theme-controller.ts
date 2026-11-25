import type { Compartment, Extension } from "@codemirror/state";
import type { EditorView } from "@codemirror/view";

export interface EditorThemeControllerDependencies {
  sourceEditor: Pick<EditorView, "dispatch">;
  sourceTheme: Compartment;
  generatedEditor: Pick<EditorView, "dispatch">;
  generatedTheme: Compartment;
  editorTheme: () => Extension;
  refreshHelpTheme: () => void;
  applyPageTheme: (choice: string) => void;
  onSystemThemeChange: (refresh: () => void) => void;
}

/** Mantém o tema sincronizado entre editores, ajuda e página. */
export function createEditorThemeController(dependencies: EditorThemeControllerDependencies) {
  function refreshEditorThemes() {
    dependencies.sourceEditor.dispatch({
      effects: dependencies.sourceTheme.reconfigure(dependencies.editorTheme()),
    });
    dependencies.generatedEditor.dispatch({
      effects: dependencies.generatedTheme.reconfigure(dependencies.editorTheme()),
    });
    dependencies.refreshHelpTheme();
  }

  function applyTheme(choice: string) {
    dependencies.applyPageTheme(choice);
    refreshEditorThemes();
  }

  dependencies.onSystemThemeChange(refreshEditorThemes);
  return { refreshEditorThemes, applyTheme };
}
