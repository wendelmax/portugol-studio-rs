export type WorkspaceEntry =
  | { kind: "folder"; name: string; path: string; entries: WorkspaceEntry[] }
  | { kind: "file"; name: string; path: string };

export type WorkspaceFileEntry = Extract<WorkspaceEntry, { kind: "file" }>;

export function collectWorkspaceFiles(entries: readonly WorkspaceEntry[]): WorkspaceFileEntry[] {
  const files: WorkspaceFileEntry[] = [];
  for (const entry of entries) {
    if (entry.kind === "file") files.push(entry);
    else files.push(...collectWorkspaceFiles(entry.entries));
  }
  return files;
}

export function flattenWorkspaceEntries(
  entries: WorkspaceEntry[],
  expanded: ReadonlySet<string> = new Set(),
): { entry: WorkspaceEntry; depth: number; position: number; setSize: number }[] {
  const rows: { entry: WorkspaceEntry; depth: number; position: number; setSize: number }[] = [];
  const visit = (children: WorkspaceEntry[], depth: number) => {
    children.forEach((entry, index) => {
      rows.push({ entry, depth, position: index + 1, setSize: children.length });
      if (entry.kind === "folder" && expanded.has(entry.path)) visit(entry.entries, depth + 1);
    });
  };
  visit(entries, 0);
  return rows;
}

export function workspaceFileKind(path: string): "portugol-rs" | "rust" | "exercise" | "markup" | null {
  const fileName = path.split(/[\\/]/).pop()?.toLowerCase() ?? "";
  if (fileName.endsWith(".pors.json")) return "exercise";
  const extension = fileName.split(".").pop();
  if (extension === "pors") return "portugol-rs";
  if (extension === "rs") return "rust";
  if (extension === "design" || extension === "porsproj") return "markup";
  return null;
}

/** Marcador curto do tipo de arquivo no Explorador. */
export function workspaceFileMarker(path: string): string {
  const kind = workspaceFileKind(path);
  if (kind === "rust") return "R";
  if (kind === "exercise") return "EX";
  if (kind === "markup") return /\.porsproj$/i.test(path) ? "PJ" : "TL";
  return "RS";
}

export function workspaceSourceFileName(name: string, language: "pors" | "rs"): string {
  const trimmed = name.trim();
  if (!trimmed || trimmed === "." || trimmed === ".." || /[\\/]/.test(trimmed)) {
    throw new Error("Informe um nome de arquivo sem pastas.");
  }
  if (trimmed.endsWith(".")) {
    throw new Error("O nome do arquivo não pode terminar com ponto.");
  }

  const dot = trimmed.lastIndexOf(".");
  if (dot >= 0) {
    const extension = trimmed.slice(dot + 1).toLowerCase();
    if (extension !== language) {
      throw new Error(`A extensão do nome deve ser .${language}.`);
    }
    return trimmed;
  }
  return `${trimmed}.${language}`;
}

export function workspaceSubfolderPath(raw: string): string {
  const trimmed = raw.trim();
  if (!trimmed) return "";
  if (trimmed.includes("\\")) {
    throw new Error("Use '/' para separar subpastas.");
  }
  if (trimmed.startsWith("/") || /^[a-zA-Z]:/.test(trimmed)) {
    throw new Error("Informe uma subpasta relativa dentro do projeto.");
  }
  const withoutTrailing = trimmed.replace(/\/+$/, "");
  if (!withoutTrailing) {
    throw new Error("Informe uma subpasta relativa dentro do projeto.");
  }
  const segments = withoutTrailing.split("/");
  for (const segment of segments) {
    if (!segment || segment === "." || segment === "..") {
      throw new Error("Informe uma subpasta relativa válida dentro do projeto.");
    }
  }
  return segments.join("/");
}

export function workspaceParentFolders(relativePath: string): string[] {
  const segments = relativePath.split("/").filter(Boolean);
  const parents: string[] = [];
  for (let i = 1; i < segments.length; i++) {
    parents.push(segments.slice(0, i).join("/"));
  }
  return parents;
}

export function workspaceRenameFileName(name: string, currentPath: string): string {
  const currentName = currentPath.split(/[\\/]/).pop() ?? currentPath;
  if (name === currentName) return currentName;
  const trimmed = name.trim();
  if (!trimmed || trimmed === "." || trimmed === ".." || /[\\/]/.test(trimmed)) {
    throw new Error("Informe um nome de arquivo sem pastas.");
  }
  if (trimmed.endsWith(".")) {
    throw new Error("O nome do arquivo não pode terminar com ponto.");
  }

  const lowerCurrent = currentName.toLowerCase();
  const extension = lowerCurrent.endsWith(".pors.json")
    ? "pors.json"
    : lowerCurrent.endsWith(".pors")
      ? "pors"
      : lowerCurrent.endsWith(".rs")
        ? "rs"
        : "";
  if (!extension) {
    throw new Error("A extensão deste arquivo não pode ser alterada no Explorador.");
  }

  const lowerTrimmed = trimmed.toLowerCase();
  if (extension === "pors.json") {
    if (lowerTrimmed.endsWith(".pors.json")) return trimmed;
    if (trimmed.includes(".")) {
      throw new Error("A extensão deve continuar sendo .pors.json.");
    }
    return `${trimmed}.pors.json`;
  }

  const dot = trimmed.lastIndexOf(".");
  if (dot >= 0) {
    if (trimmed.slice(dot + 1).toLowerCase() !== extension) {
      throw new Error(`A extensão deve continuar sendo .${extension}.`);
    }
    return trimmed;
  }
  return `${trimmed}.${extension}`;
}
