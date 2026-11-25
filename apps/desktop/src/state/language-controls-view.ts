import type { Tab } from "./tab";
import {
  deriveLanguageControls,
  LANGUAGE_BUTTON_IDS,
  type LanguageControlState,
} from "./language-capabilities.ts";

type Button = Pick<HTMLButtonElement, "title" | "disabled" | "dataset" | "setAttribute">;

export interface LanguageControlsViewDependencies {
  button: (id: string) => Button;
  argumentsInput: () => Pick<HTMLInputElement, "disabled" | "title">;
  formatButton: () => Button;
  libraryButtons: () => ReadonlyMap<string, Button>;
  state: () => Omit<LanguageControlState, "defaultTitles" | "libraryNames">;
}

/** Applies language-specific availability and descriptions to the shell controls. */
export function createLanguageControlsView(dependencies: LanguageControlsViewDependencies) {
  function sync(tab: Tab) {
    const defaultTitles: Partial<Record<(typeof LANGUAGE_BUTTON_IDS)[number], string>> = {};
    for (const id of LANGUAGE_BUTTON_IDS) {
      const button = dependencies.button(id);
      button.dataset.defaultTitle ??= button.title;
      defaultTitles[id] = button.dataset.defaultTitle;
    }

    const libraries = dependencies.libraryButtons();
    const controls = deriveLanguageControls(tab, {
      ...dependencies.state(),
      defaultTitles,
      libraryNames: [...libraries.keys()],
    });

    for (const id of LANGUAGE_BUTTON_IDS) {
      const button = dependencies.button(id);
      button.disabled = controls.buttons[id].disabled;
      button.title = controls.buttons[id].title;
      button.setAttribute("aria-description", controls.buttons[id].description);
    }

    const args = dependencies.argumentsInput();
    args.disabled = controls.arguments.disabled;
    args.title = controls.arguments.title;

    const format = dependencies.formatButton();
    format.disabled = controls.format.disabled;
    format.title = controls.format.title;

    for (const [name, button] of libraries) {
      button.disabled = controls.libraries[name].disabled;
      button.title = controls.libraries[name].title;
    }

    for (const id of ["editor-definition", "editor-rename"]) {
      const button = dependencies.button(id);
      button.disabled = controls.symbolActions.disabled;
      button.setAttribute("aria-description", controls.symbolActions.description);
    }
  }

  return { sync };
}
