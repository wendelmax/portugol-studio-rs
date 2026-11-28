import { Compartment } from "@codemirror/state";
import { linter } from "@codemirror/lint";
import type { Inspection, SymbolInfo } from "../api";
import type { Tab } from "../state/tab";
import { createLatestOnlyRustSourceChecker } from "../rust-source-check";
import { createCenterSourceController } from "./center-source";
import { createBreakpointGutter, toCodeMirror } from "./decorations";
import { createFerroLintSourceCheck } from "./ferro-linter";
import { createRustLintSourceCheck } from "./rust-linter";

export interface EditorSetupDependencies {
  storage: () => Pick<Storage, "getItem" | "setItem"> | null;
  hasActiveTab: () => boolean;
  centerSelection: () => void;
  setCenterPressed: (pressed: boolean) => void;
  focusEditor: () => void;
  syncBreakpoints: (state: import("@codemirror/state").EditorState) => void;
  bugIcon: () => HTMLElement;
  activeTab: () => Tab | undefined;
  /** Inspeção de código-fonte; a aba diz se é o code-behind de uma tela. */
  inspectFerroRust: (source: string, tab: Tab | undefined) => Promise<Inspection>;
  /** Inspeção de .design e .ferproj; o tipo vem da extensão do caminho. */
  inspectMarkup: (source: string, path: string, tab: Tab | undefined) => Promise<Inspection>;
  renderMessages: () => void;
  showOutline: (symbols: SymbolInfo[]) => void;
  updateLibraryButtons: (symbols: SymbolInfo[]) => void;
  isRustcAvailable: () => boolean;
  checkRustSource: (source: string) => Promise<import("../api").RustSourceCompileDiagnostic[]>;
}

/** Monta o estado e as ferramentas compartilhadas pelos editores da IDE. */
export function createEditorSetup(dependencies: EditorSetupDependencies) {
  const theme = new Compartment();
  const preferences = new Compartment();
  const sourceLanguage = new Compartment();
  const ferroTools = new Compartment();
  const generatedRustTheme = new Compartment();

  const centerSource = createCenterSourceController({
    storage: dependencies.storage,
    hasActiveTab: dependencies.hasActiveTab,
    centerSelection: dependencies.centerSelection,
    setPressed: dependencies.setCenterPressed,
    focusEditor: dependencies.focusEditor,
  });

  const breakpointGutter = createBreakpointGutter(
    dependencies.syncBreakpoints,
    dependencies.bugIcon,
  );

  const ferroLinter = linter(createFerroLintSourceCheck({
    activeTab: dependencies.activeTab,
    inspectFerroRust: dependencies.inspectFerroRust,
    renderMessages: dependencies.renderMessages,
    showOutline: dependencies.showOutline,
    updateLibraryButtons: dependencies.updateLibraryButtons,
    toCodeMirror,
  }), { delay: 350 });

  const markupLinter = linter(createFerroLintSourceCheck({
    activeTab: dependencies.activeTab,
    language: "markup",
    inspectFerroRust: (source, tab) =>
      dependencies.inspectMarkup(source, tab?.workspacePath ?? tab?.path ?? "tela.design", tab),
    renderMessages: dependencies.renderMessages,
    showOutline: dependencies.showOutline,
    updateLibraryButtons: dependencies.updateLibraryButtons,
    toCodeMirror,
  }), { delay: 350 });

  const latestRustSourceCheck = createLatestOnlyRustSourceChecker(dependencies.checkRustSource);
  const rustSourceLinter = linter(createRustLintSourceCheck({
    activeTab: dependencies.activeTab,
    isRustcAvailable: dependencies.isRustcAvailable,
    checkSource: latestRustSourceCheck,
    renderMessages: dependencies.renderMessages,
    toCodeMirror,
  }), { delay: 500 });

  return {
    theme,
    preferences,
    sourceLanguage,
    ferroTools,
    generatedRustTheme,
    centerSource,
    breakpointGutter,
    ferroLinter,
    markupLinter,
    rustSourceLinter,
  };
}
