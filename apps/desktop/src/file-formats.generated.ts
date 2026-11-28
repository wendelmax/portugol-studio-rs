// Gerado por crates/ferro-formatos. Não edite à mão: rode
// `UPDATE_FILE_FORMATS=1 cargo test -p ferro-formatos`.

export type FileKind = "source" | "rust" | "lesson" | "design" | "manifest";

export type FileFormat = {
  kind: FileKind;
  extension: string;
  legacy: readonly string[];
  label: string;
  mime: string;
};

export const FILE_FORMATS: readonly FileFormat[] = [
  { kind: "source", extension: "fer", legacy: [], label: "Ferro", mime: "text/x-ferro" },
  { kind: "rust", extension: "rs", legacy: [], label: "Rust", mime: "text/x-rust" },
  { kind: "lesson", extension: "licao", legacy: [], label: "Lição Ferro", mime: "application/json" },
  { kind: "design", extension: "design", legacy: [], label: "Tela (pseudo-XAML)", mime: "application/xml" },
  { kind: "manifest", extension: "ferproj", legacy: [], label: "Projeto Ferro", mime: "application/xml" },
];

export const RETIRED_EXTENSIONS: readonly string[] = ["por", "pex"];
