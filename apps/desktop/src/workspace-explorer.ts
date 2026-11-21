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

export function workspaceFileKind(path: string): "portugol" | "portugol-rs" | "rust" | "exercise" | null {
  const extension = path.split(/[\\/]/).pop()?.split(".").pop()?.toLowerCase();
  if (extension === "por") return "portugol";
  if (extension === "pors") return "portugol-rs";
  if (extension === "rs") return "rust";
  if (extension === "pex") return "exercise";
  return null;
}

export function workspaceSourceFileName(name: string, language: "por" | "pors" | "rs"): string {
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

  const currentDot = currentName.lastIndexOf(".");
  const extension = currentDot >= 0 ? currentName.slice(currentDot + 1).toLowerCase() : "";
  if (extension !== "por" && extension !== "pors" && extension !== "rs" && extension !== "pex") {
    throw new Error("A extensão deste arquivo não pode ser alterada no Explorador.");
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
