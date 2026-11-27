// Gerado por crates/portugol-formatos. Não edite à mão: rode
// `UPDATE_FILE_FORMATS=1 cargo test -p portugol-formatos`.

export type FileKind = "source" | "rust" | "lesson" | "design" | "manifest";

export type FileFormat = {
  kind: FileKind;
  extension: string;
  legacy: readonly string[];
  label: string;
  mime: string;
};

export const FILE_FORMATS: readonly FileFormat[] = [
  { kind: "source", extension: "pors", legacy: [], label: "Portugol-RS", mime: "text/x-portugol-rs" },
  { kind: "rust", extension: "rs", legacy: [], label: "Rust", mime: "text/x-rust" },
  { kind: "lesson", extension: "pors.json", legacy: [], label: "Exercício Portugol-RS", mime: "application/json" },
  { kind: "design", extension: "design", legacy: [], label: "Tela (pseudo-XAML)", mime: "application/xml" },
  { kind: "manifest", extension: "ferproj", legacy: [], label: "Projeto desktop PORS", mime: "application/xml" },
];

export const RETIRED_EXTENSIONS: readonly string[] = ["por", "pex"];
