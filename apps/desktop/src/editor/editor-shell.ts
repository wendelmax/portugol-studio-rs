import { EditorView } from "@codemirror/view";
import type { EditorSetupDependencies } from "./setup";
import { createEditorSetup } from "./setup";
import type { RustPreviewHost } from "../rust/preview";
import { mountRustPreview } from "../rust/preview";
import type { LearningPanelHost } from "../learning/panel";
import { mountLearningPanel } from "../learning/panel";
import type { EditorUpdateDependencies } from "./update-controller";
import { createEditorUpdateController } from "./update-controller";
import type { EditorNavigationBindingsDependencies } from "./navigation-bindings";
import { createEditorNavigationBindings } from "./navigation-bindings";
import type { EditorExtensionDependencies } from "./extensions";
import { createEditorExtensionFactory } from "./extensions";
import type { EditorThemeControllerDependencies } from "./theme-controller";
import { createEditorThemeController } from "./theme-controller";
import { createHelpLoader } from "../shell/help-loader";
import { editorColors, editorTheme } from "./theme";
import type { LearningMode } from "../learning-mode";
import type { Tab } from "../state/tab";

export interface EditorShellDependencies {
  parent: HTMLElement;
  setup: Omit<EditorSetupDependencies, "focusEditor">;
  activeTab: () => Tab | undefined;
  learningMode: () => LearningMode;
  preview: Omit<RustPreviewHost, "sourceView" | "editorExtensions">;
  help: Parameters<typeof createHelpLoader>[0];
  learningPanel: Omit<LearningPanelHost, "sourceView" | "activeTab" | "learningMode">;
  update: Omit<EditorUpdateDependencies, "activeTab" | "currentEditorState" | "learningMode" | "centerActive">;
  navigation: Omit<EditorNavigationBindingsDependencies, "activeLanguage" | "selectionHead">;
  extensions: Omit<EditorExtensionDependencies,
    "sourceLanguage" | "portugolTools" | "theme" | "preferences" | "editorColors" | "editorTheme" | "onEditorClick" | "onGoToDeclaration" | "onUpdate">;
  theme: Pick<EditorThemeControllerDependencies, "applyPageTheme" | "onSystemThemeChange">;
}

/** Compõe o editor-fonte, a prévia, a ajuda e os controladores do editor. */
export function createEditorShell(dependencies: EditorShellDependencies) {
  const view = new EditorView({ parent: dependencies.parent });
  const editorSetup = createEditorSetup({
    ...dependencies.setup,
    focusEditor: () => view.focus(),
  });
  const rustPreview = mountRustPreview({
    ...dependencies.preview,
    sourceView: view,
    editorExtensions: [editorColors, editorSetup.generatedRustTheme.of(editorTheme())],
  });
  const help = createHelpLoader(dependencies.help);
  const learningPanel = mountLearningPanel({
    ...dependencies.learningPanel,
    sourceView: view,
    activeTab: dependencies.activeTab,
    learningMode: dependencies.learningMode,
  });
  const editorUpdateHandler = createEditorUpdateController({
    ...dependencies.update,
    activeTab: dependencies.activeTab,
    currentEditorState: () => view.state,
    learningMode: dependencies.learningMode,
    centerActive: () => editorSetup.centerSource.isActive(),
  });
  const editorNavigation = createEditorNavigationBindings({
    ...dependencies.navigation,
    activeLanguage: () => dependencies.activeTab()?.language,
    selectionHead: () => view.state.selection.main.head,
  });
  const extensions = createEditorExtensionFactory({
    ...dependencies.extensions,
    sourceLanguage: editorSetup.sourceLanguage,
    portugolTools: editorSetup.portugolTools,
    theme: editorSetup.theme,
    preferences: editorSetup.preferences,
    editorColors,
    editorTheme,
    onEditorClick: editorNavigation.onEditorClick,
    onGoToDeclaration: editorNavigation.onGoToDeclaration,
    onUpdate: editorUpdateHandler,
  });
  const editorThemeController = createEditorThemeController({
    sourceEditor: view,
    sourceTheme: editorSetup.theme,
    generatedEditor: rustPreview.generatedEditor,
    generatedTheme: editorSetup.generatedRustTheme,
    editorTheme,
    refreshHelpTheme: () => help.refreshTheme(),
    ...dependencies.theme,
  });

  return {
    ...editorSetup,
    view,
    rustPreview,
    generatedRustEditor: rustPreview.generatedEditor,
    help,
    openHelp: help.open,
    learningPanel,
    extensions,
    applyTheme: editorThemeController.applyTheme,
  };
}
