import type { SourceLanguage } from "./document-language";

export type SemanticSymbolLanguage = Extract<SourceLanguage, "portugol" | "portugol-rs">;

export function semanticSymbolLanguage(language: SourceLanguage): SemanticSymbolLanguage | null {
  return language === "rust" ? null : language;
}
