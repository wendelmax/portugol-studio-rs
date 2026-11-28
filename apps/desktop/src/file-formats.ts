/**
 * Tabela única dos formatos de arquivo (ADR 0023, Emenda 2). A tabela vem de
 * `crates/ferro-formatos`, gerada em `file-formats.generated.ts`; aqui ficam
 * só as consultas. A IDE não compara extensões por conta própria.
 */
import { FILE_FORMATS, RETIRED_EXTENSIONS, type FileFormat, type FileKind } from "./file-formats.generated.ts";

export type { FileFormat, FileKind };
export { FILE_FORMATS };

export type FileMatch = {
  format: FileFormat;
  kind: FileKind;
  /** Extensão encontrada, sem o ponto. */
  extension: string;
  /** Extensão antiga, aceita só durante a transição. */
  legacy: boolean;
};

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

function hasSuffix(name: string, extension: string): boolean {
  return name.length > extension.length + 1 && name.toLowerCase().endsWith(`.${extension}`);
}

/**
 * Formato de um arquivo pelo nome ou caminho. Extensões mais longas ganham
 * (`exercicio.licao` é exercício); um nome que é só a extensão não conta.
 */
export function classifyFile(path: string, table: readonly FileFormat[] = FILE_FORMATS): FileMatch | null {
  const name = fileName(path);
  let best: FileMatch | null = null;
  for (const format of table) {
    const candidates: [string, boolean][] = [[format.extension, false], ...format.legacy.map((e): [string, boolean] => [e, true])];
    for (const [extension, legacy] of candidates) {
      if ((!best || extension.length > best.extension.length) && hasSuffix(name, extension)) {
        best = { format, kind: format.kind, extension, legacy };
      }
    }
  }
  return best;
}

export function fileKind(path: string): FileKind | null {
  return classifyFile(path)?.kind ?? null;
}

export function fileFormat(kind: FileKind): FileFormat {
  const format = FILE_FORMATS.find((candidate) => candidate.kind === kind);
  if (!format) throw new Error(`Formato desconhecido: ${kind}`);
  return format;
}

/** Extensão canônica do tipo, sem o ponto. */
export function fileExtension(kind: FileKind): string {
  return fileFormat(kind).extension;
}

/** Todas as extensões aceitas do tipo (canônica primeiro), sem o ponto. */
export function acceptedExtensions(kind: FileKind): string[] {
  const format = fileFormat(kind);
  return [format.extension, ...format.legacy];
}

/** Programas: criados pelo aluno, executados e salvos fora do workspace. */
export function isProgramKind(kind: FileKind | null): kind is "source" | "rust" {
  return kind === "source" || kind === "rust";
}

/** Marcação de projetos: o desenho da tela e o manifesto. */
export function isMarkupKind(kind: FileKind | null): kind is "design" | "manifest" {
  return kind === "design" || kind === "manifest";
}

/** Extensão legada (`.por`, `.pex`), retirada no ADR 0016. */
export function isRetiredFile(path: string): boolean {
  const name = fileName(path);
  return RETIRED_EXTENSIONS.some((extension) => hasSuffix(name, extension));
}

/** Nome sem a extensão reconhecida: `JanelaPrincipal.design` → `JanelaPrincipal`. */
export function stripFileExtension(name: string): string {
  const match = classifyFile(name);
  return match ? name.slice(0, name.length - match.extension.length - 1) : name;
}

/** Se o caminho usar uma extensão antiga, devolve o nome com a extensão canônica. */
export function canonicalFileName(path: string): string | null {
  const match = classifyFile(path);
  if (!match?.legacy) return null;
  return `${stripFileExtension(fileName(path))}.${match.format.extension}`;
}

/** Extensões canônicas dos tipos para mensagens: `".fer, .rs e .licao"`. */
export function describeExtensions(kinds: readonly FileKind[], conjunction: "e" | "ou" = "e"): string {
  const names = FILE_FORMATS.filter((format) => kinds.includes(format.kind)).map((format) => `.${format.extension}`);
  if (names.length <= 1) return names.join("");
  return `${names.slice(0, -1).join(", ")} ${conjunction} ${names[names.length - 1]}`;
}

export const ALL_FILE_KINDS: readonly FileKind[] = FILE_FORMATS.map((format) => format.kind);
