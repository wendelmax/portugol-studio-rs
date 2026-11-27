/**
 * Linguagem de um documento aberto no editor. `markup` cobre a marcação dos
 * projetos desktop PORS (ADR 0018): o desenho de uma tela (`.design`) e o
 * manifesto (`.ferproj`). Marcação não executa, não depura e não tem prévia
 * Rust; só tem realce, diagnósticos, estrutura e formatação.
 */
export type SourceLanguage = "portugol-rs" | "rust" | "markup";

const MARKUP_EXTENSION = /\.(design|ferproj)$/i;

export function detectSourceLanguage(path: string): SourceLanguage {
  if (/\.rs$/i.test(path)) return "rust";
  if (MARKUP_EXTENSION.test(path)) return "markup";
  return "portugol-rs";
}

export function isMarkupPath(path: string): boolean {
  return MARKUP_EXTENSION.test(path);
}

export function ensureSourceExtension(path: string, language: SourceLanguage): string {
  if (/\.(pors|rs)$/i.test(path) || MARKUP_EXTENSION.test(path)) return path;
  const extension = language === "rust" ? "rs" : language === "markup" ? "design" : "pors";
  return `${path}.${extension}`;
}
