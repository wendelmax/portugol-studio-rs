export type SourceLanguage = "portugol" | "portugol-rs";

export function detectSourceLanguage(path: string): SourceLanguage {
  return /\.pors$/i.test(path) ? "portugol-rs" : "portugol";
}

export function ensureSourceExtension(path: string, language: SourceLanguage): string {
  if (/\.(por|pors)$/i.test(path)) return path;
  return `${path}.${language === "portugol-rs" ? "pors" : "por"}`;
}
