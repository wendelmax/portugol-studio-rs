export type SourceLanguage = "portugol-rs" | "rust";

export function detectSourceLanguage(path: string): SourceLanguage {
  return /\.rs$/i.test(path) ? "rust" : "portugol-rs";
}

export function ensureSourceExtension(path: string, language: SourceLanguage): string {
  if (/\.(pors|rs)$/i.test(path)) return path;
  return `${path}.${language === "rust" ? "rs" : "pors"}`;
}
