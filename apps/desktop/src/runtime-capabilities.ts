import type { SourceLanguage } from "./document-language";

export function canRunSource(language: SourceLanguage, rustcAvailable: boolean | null): boolean {
  if (language === "portugol") return true;
  return language === "portugol-rs" && rustcAvailable === true;
}

export function canDebugSource(language: SourceLanguage): boolean {
  return language === "portugol";
}
