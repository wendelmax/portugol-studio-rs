export type PanelName = "console" | "messages" | "debug" | "learning" | "tests";

export function panelContentId(name: PanelName): string {
  return name === "learning" ? "learning-panel" : name;
}

export function panelNameForKey(
  names: readonly PanelName[],
  current: PanelName,
  key: string,
): PanelName | null {
  const index = names.indexOf(current);
  if (index < 0 || names.length === 0) return null;
  if (key === "Home") return names[0];
  if (key === "End") return names[names.length - 1];
  if (key === "ArrowRight") return names[(index + 1) % names.length];
  if (key === "ArrowLeft") return names[(index - 1 + names.length) % names.length];
  return null;
}
