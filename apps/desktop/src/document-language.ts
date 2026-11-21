export type SourceLanguage = "portugol" | "portugol-rs" | "rust";

export function detectSourceLanguage(path: string): SourceLanguage {
  if (/\.rs$/i.test(path)) return "rust";
  return /\.pors$/i.test(path) ? "portugol-rs" : "portugol";
}

export function ensureSourceExtension(path: string, language: SourceLanguage): string {
  if (/\.(por|pors|rs)$/i.test(path)) return path;
  return `${path}.${language === "portugol-rs" ? "pors" : language === "rust" ? "rs" : "por"}`;
}
