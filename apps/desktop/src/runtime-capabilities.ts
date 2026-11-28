import type { SourceLanguage } from "./document-language";

/** Ferro executa na VM; arquivos Rust exigem `rustc`. */
export function canRunSource(language: SourceLanguage, rustcAvailable: boolean | null): boolean {
  if (language === "ferro-rust") return true;
  return language === "rust" && rustcAvailable === true;
}

/** Execução nativa do Rust gerado a partir de Ferro, opcional. */
export function canRunAsRust(language: SourceLanguage, rustcAvailable: boolean | null): boolean {
  return language === "ferro-rust" && rustcAvailable === true;
}

export function canDebugSource(language: SourceLanguage): boolean {
  return language === "ferro-rust";
}

export function canFormatSource(language: SourceLanguage, rustfmtAvailable: boolean | null): boolean {
  if (language === "ferro-rust" || language === "markup") return true;
  return language === "rust" && rustfmtAvailable === true;
}
