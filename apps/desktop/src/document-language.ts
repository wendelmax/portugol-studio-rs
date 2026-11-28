import { classifyFile, fileExtension, isMarkupKind } from "./file-formats.ts";

/**
 * Linguagem de um documento aberto no editor. `markup` cobre a marcação dos
 * projetos desktop FERRO (ADR 0018): o desenho de uma tela (`.design`) e o
 * manifesto (`.ferproj`). Marcação não executa, não depura e não tem prévia
 * Rust; só tem realce, diagnósticos, estrutura e formatação.
 */
export type SourceLanguage = "ferro-rust" | "rust" | "markup";

export function detectSourceLanguage(path: string): SourceLanguage {
  const kind = classifyFile(path)?.kind ?? null;
  if (kind === "rust") return "rust";
  if (isMarkupKind(kind)) return "markup";
  return "ferro-rust";
}

export function isMarkupPath(path: string): boolean {
  return isMarkupKind(classifyFile(path)?.kind ?? null);
}

export function ensureSourceExtension(path: string, language: SourceLanguage): string {
  const kind = classifyFile(path)?.kind;
  if (kind === "source" || kind === "rust" || isMarkupKind(kind ?? null)) return path;
  const extension = fileExtension(language === "rust" ? "rust" : language === "markup" ? "design" : "source");
  return `${path}.${extension}`;
}
