export type PanelName = "console" | "messages" | "debug" | "learning";

export function panelContentId(name: PanelName): string {
  return name === "learning" ? "learning-panel" : name;
}
