// Tema da IDE e do editor. "system" segue o sistema operacional; perfis de
// tema ("preset:..." ou personalizados) definem uma base clara ou escura e
// cores próprias. A escolha fica salva.

import type { Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { applyThemeColors, themeProfile } from "../themes";

export const THEME_KEY = "portugol.theme";
export const DEFAULT_THEME = "system";

const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

/** Tema salvo, ou o padrão quando não há escolha válida. */
export function themeChoice(): string {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === "light" || saved === "dark" || saved === "system" || (saved && themeProfile(saved))) return saved;
  } catch {
    // Sem armazenamento local: fica no padrão.
  }
  return DEFAULT_THEME;
}

let appliedTheme = themeChoice();

export function isDark(): boolean {
  const choice = themeProfile(appliedTheme)?.base ?? appliedTheme;
  return choice === "dark" || (choice === "system" && darkQuery.matches);
}

/** Cores de sintaxe: vêm das variáveis CSS do tema ativo. */
export const editorColors = syntaxHighlighting(HighlightStyle.define([
  { tag: tags.keyword, color: "var(--syntax-keyword)" },
  { tag: [tags.typeName, tags.namespace], color: "var(--syntax-type)" },
  { tag: tags.number, color: "var(--syntax-number)" },
  { tag: tags.bool, color: "var(--syntax-logical)" },
  { tag: tags.function(tags.variableName), color: "var(--syntax-function)" },
  { tag: tags.standard(tags.function(tags.variableName)), color: "var(--syntax-function)" },
  { tag: tags.string, color: "var(--syntax-string)" },
  { tag: tags.special(tags.string), color: "var(--syntax-interpolation)" },
  { tag: tags.comment, color: "var(--syntax-comment)" },
  { tag: tags.operator, color: "var(--syntax-operator)" },
]));

const darkEditor = EditorView.theme({}, { dark: true });

/** Extensão de tema dos editores CodeMirror para o tema atual. */
export const editorTheme = (): Extension => isDark() ? darkEditor : [];

/** Salva a escolha e aplica base e cores do tema à página. */
export function applyPageTheme(choice: string) {
  appliedTheme = choice;
  try {
    localStorage.setItem(THEME_KEY, choice);
  } catch {
    // Ignora: a escolha vale só para esta sessão.
  }
  if (choice === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = themeProfile(choice)?.base ?? choice;
  applyThemeColors(document.documentElement, themeProfile(choice)?.colors);
}

/** Avisa quando o sistema troca entre claro e escuro. */
export function onSystemThemeChange(listener: () => void) {
  darkQuery.addEventListener("change", listener);
}
