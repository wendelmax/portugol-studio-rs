export type WorkspaceExportStatus = "selection" | "cancelled";

export function workspaceExportStatus(
  selectedCount: number,
  state: WorkspaceExportStatus = "selection",
): string {
  if (state === "cancelled") return "Exportação cancelada. Nenhum arquivo foi copiado.";
  if (selectedCount === 0) return "Selecione ao menos um arquivo.";
  return `${selectedCount} ${selectedCount === 1 ? "arquivo selecionado" : "arquivos selecionados"}.`;
}
