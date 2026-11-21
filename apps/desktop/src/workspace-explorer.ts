export type WorkspaceEntry =
  | { kind: "folder"; name: string; path: string; entries: WorkspaceEntry[] }
  | { kind: "file"; name: string; path: string };

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

export function workspaceFileKind(path: string): "portugol" | "portugol-rs" | "exercise" | null {
  const extension = path.split(/[\\/]/).pop()?.split(".").pop()?.toLowerCase();
  if (extension === "por") return "portugol";
  if (extension === "pors") return "portugol-rs";
  if (extension === "pex") return "exercise";
  return null;
}
