import type { SourceLanguage } from "./document-language";

export function canRunSource(language: SourceLanguage, rustcAvailable: boolean | null): boolean {
  if (language === "portugol") return true;
  return (language === "portugol-rs" || language === "rust") && rustcAvailable === true;
}

export function canDebugSource(language: SourceLanguage): boolean {
  return language === "portugol" || language === "portugol-rs";
}

export function canFormatSource(language: SourceLanguage, rustfmtAvailable: boolean | null): boolean {
  if (language === "portugol") return true;
  return language === "rust" && rustfmtAvailable === true;
}
