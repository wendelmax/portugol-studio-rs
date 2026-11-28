import type { SourceLanguage } from "./document-language";

export type SemanticSymbolLanguage = Extract<SourceLanguage, "ferro-rust">;

export function semanticSymbolLanguage(language: SourceLanguage): SemanticSymbolLanguage | null {
  return language === "ferro-rust" ? language : null;
}
