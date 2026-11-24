import { EditorState, type Extension } from "@codemirror/state";
import type { Tab } from "./tab";
import type { SourceLanguage } from "../document-language";
import { detectSourceLanguage } from "../document-language.ts";
import type { ExerciseDocument } from "../exercises";

export const PORS_TEMPLATE = "funcao inicio() {\n    seja mutavel idade: inteiro = 16;\n    idade += 1;\n    escreva(idade);\n}\n";

export interface NewTabOptions {
  text?: string;
  path?: string | null;
  exampleName?: string;
  examplePath?: string;
  exercise?: ExerciseDocument;
  explicitLanguage?: SourceLanguage;
  workspaceFile?: { id: string; path: string; preferenceKey: string };
}

export function createTabFactory(dependencies: {
  extensions: (language: SourceLanguage) => Extension[];
  addTab: (tab: Tab) => void;
  activate: (id: number) => void;
}) {
  let nextId = 1;

  function create(options: NewTabOptions = {}): Tab {
    const {
      text = PORS_TEMPLATE,
      path = null,
      exampleName,
      examplePath,
      exercise,
      explicitLanguage,
      workspaceFile,
    } = options;
    const language = explicitLanguage ?? detectSourceLanguage(path ?? exampleName ?? "programa.pors");
    const tab: Tab = {
      id: nextId++,
      path,
      language,
      exampleName,
      examplePath,
      exercise,
      workspaceId: workspaceFile?.id,
      workspacePath: workspaceFile?.path,
      workspacePreferenceKey: workspaceFile?.preferenceKey,
      state: EditorState.create({ doc: text, extensions: dependencies.extensions(language) }),
      saved: text,
      diagnostics: [],
      symbols: [],
    };
    dependencies.addTab(tab);
    dependencies.activate(tab.id);
    return tab;
  }

  return { create };
}

export function tabName(tab: Pick<Tab, "path" | "exampleName">): string {
  if (!tab.path) return tab.exampleName ?? "Sem título";
  return tab.path.split(/[\\/]/).pop() ?? tab.path;
}

export function isTabDirty(
  tab: Pick<Tab, "id" | "saved" | "state">,
  activeId: number,
  activeText: string,
): boolean {
  const text = tab.id === activeId ? activeText : tab.state.doc.toString();
  return text !== tab.saved;
}
