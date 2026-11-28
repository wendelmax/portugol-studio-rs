import { Compartment, EditorState, type Extension } from "@codemirror/state";
import {
  EditorView, highlightActiveLine, highlightActiveLineGutter, highlightSpecialChars,
  drawSelection, keymap, lineNumbers, type KeyBinding, type ViewUpdate,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching, foldGutter, foldKeymap, indentOnInput } from "@codemirror/language";
import { highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { lintGutter } from "@codemirror/lint";
import type { SourceLanguage } from "../document-language";

export interface EditorExtensionDependencies {
  sourceLanguage: Compartment;
  ferroTools: Compartment;
  theme: Compartment;
  /** Compartimento opcional para quebra de linha e recuo (preferências do usuário). */
  preferences?: Compartment;
  editorPreferences?: () => Extension;
  ferroExtensions: () => Extension[];
  rustExtensions: () => Extension[];
  /** Ferramentas da marcação dos projetos desktop (.design e .ferproj). */
  markupExtensions?: () => Extension[];
  ferroRustLanguage: () => Extension;
  rustLanguage: () => Extension;
  markupLanguage?: () => Extension;
  editorColors: Extension;
  editorTheme: () => Extension;
  findDialog: Extension;
  editorSearchKeys: KeyBinding[];
  onEditorClick: (event: Event, editor: EditorView) => boolean;
  onGoToDeclaration: () => boolean;
  onUpdate: (update: ViewUpdate) => void;
}

/** Monta as extensões comuns e específicas da linguagem do documento. */
export function createEditorExtensionFactory(dependencies: EditorExtensionDependencies) {
  return function extensions(language: SourceLanguage): Extension[] {
    const isRust = language === "rust";
    const isMarkup = language === "markup";
    const tools = isRust
      ? dependencies.rustExtensions()
      : isMarkup
        ? dependencies.markupExtensions?.() ?? []
        : dependencies.ferroExtensions();
    const languageExtension = isRust
      ? dependencies.rustLanguage()
      : isMarkup
        ? dependencies.markupLanguage?.() ?? []
        : dependencies.ferroRustLanguage();

    return [
      lineNumbers(),
      highlightActiveLineGutter(),
      highlightSpecialChars(),
      history(),
      drawSelection(),
      indentOnInput(),
      bracketMatching(),
      closeBrackets(),
      autocompletion(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      dependencies.findDialog,
      EditorView.contentAttributes.of({ "aria-label": "Editor de código" }),
      dependencies.preferences && dependencies.editorPreferences
        ? dependencies.preferences.of(dependencies.editorPreferences())
        : EditorState.tabSize.of(4),
      dependencies.sourceLanguage.of(languageExtension),
      dependencies.ferroTools.of(tools),
      foldGutter({ markerDOM: (open) => {
        const marker = document.createElement("span");
        marker.textContent = open ? "−" : "+";
        marker.title = open ? "Recolher bloco" : "Expandir bloco";
        marker.setAttribute("aria-label", marker.title);
        return marker;
      }}),
      dependencies.editorColors,
      dependencies.theme.of(dependencies.editorTheme()),
      lintGutter(),
      EditorView.domEventHandlers({ click: dependencies.onEditorClick }),
      keymap.of([
        { key: "F12", run: dependencies.onGoToDeclaration },
        ...closeBracketsKeymap, ...defaultKeymap, ...dependencies.editorSearchKeys,
        ...historyKeymap, ...completionKeymap, ...foldKeymap, indentWithTab,
      ]),
      EditorView.updateListener.of(dependencies.onUpdate),
    ];
  };
}
