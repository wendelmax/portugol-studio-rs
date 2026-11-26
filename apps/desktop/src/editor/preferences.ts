// Preferências do editor (quebra de linha e recuo), salvas entre sessões.

import { EditorState, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { indentUnit } from "@codemirror/language";

export const TAB_SIZE_OPTIONS = [2, 4, 8] as const;
export type TabSizeOption = (typeof TAB_SIZE_OPTIONS)[number];

export interface EditorPreferences {
  wordWrap: boolean;
  tabSize: TabSizeOption;
}

export const DEFAULT_EDITOR_PREFERENCES: EditorPreferences = { wordWrap: false, tabSize: 4 };

export const EDITOR_WRAP_KEY = "portugol.editor.wordWrap";
export const EDITOR_TAB_SIZE_KEY = "portugol.editor.tabSize";

type ReadableStorage = Pick<Storage, "getItem">;
type WritableStorage = Pick<Storage, "setItem">;

export function normalizeTabSize(value: unknown): TabSizeOption {
  const parsed = typeof value === "number" ? value : Number.parseInt(String(value), 10);
  return (TAB_SIZE_OPTIONS as readonly number[]).includes(parsed)
    ? (parsed as TabSizeOption)
    : DEFAULT_EDITOR_PREFERENCES.tabSize;
}

export function loadEditorPreferences(storage: ReadableStorage | null | undefined): EditorPreferences {
  try {
    return {
      wordWrap: storage?.getItem(EDITOR_WRAP_KEY) === "true",
      tabSize: normalizeTabSize(storage?.getItem(EDITOR_TAB_SIZE_KEY)),
    };
  } catch {
    return { ...DEFAULT_EDITOR_PREFERENCES };
  }
}

export function saveEditorPreferences(storage: WritableStorage | null | undefined, preferences: EditorPreferences) {
  try {
    storage?.setItem(EDITOR_WRAP_KEY, String(preferences.wordWrap));
    storage?.setItem(EDITOR_TAB_SIZE_KEY, String(preferences.tabSize));
  } catch {
    // A preferência vale apenas nesta sessão.
  }
}

/** Extensões do CodeMirror derivadas das preferências do usuário. */
export function editorPreferenceExtensions(preferences: EditorPreferences): Extension[] {
  return [
    EditorState.tabSize.of(preferences.tabSize),
    indentUnit.of(" ".repeat(preferences.tabSize)),
    ...(preferences.wordWrap ? [EditorView.lineWrapping] : []),
  ];
}
