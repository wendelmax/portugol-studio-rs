import type { SourceLanguage } from "./document-language";

/** Portugol-RS executa na VM; arquivos Rust exigem `rustc`. */
export function canRunSource(language: SourceLanguage, rustcAvailable: boolean | null): boolean {
  if (language === "portugol-rs") return true;
  return language === "rust" && rustcAvailable === true;
}

/** Execução nativa do Rust gerado a partir de Portugol-RS, opcional. */
export function canRunAsRust(language: SourceLanguage, rustcAvailable: boolean | null): boolean {
  return language === "portugol-rs" && rustcAvailable === true;
}

export function canDebugSource(language: SourceLanguage): boolean {
  return language === "portugol-rs";
}

export function canFormatSource(language: SourceLanguage, rustfmtAvailable: boolean | null): boolean {
  if (language === "portugol-rs" || language === "markup") return true;
  return language === "rust" && rustfmtAvailable === true;
}
