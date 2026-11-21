export interface WorkspaceRenameTab {
  path?: string | null;
  workspacePath?: string;
  workspaceExercisePath?: string;
  exampleName?: string;
}

export type WorkspaceRenamePreparation = "ready" | "cancelled" | "save-failed";

export async function prepareWorkspaceRename<T extends WorkspaceRenameTab>(options: {
  tabs: readonly T[];
  sourcePath: string;
  isDirty: (tab: T) => boolean;
  confirmSave: () => Promise<boolean>;
  saveTab: (tab: T) => Promise<boolean>;
}): Promise<WorkspaceRenamePreparation> {
  const dirtySourceTabs = options.tabs.filter((tab) =>
    tab.workspacePath === options.sourcePath && options.isDirty(tab),
  );
  if (dirtySourceTabs.length === 0) return "ready";
  if (!await options.confirmSave()) return "cancelled";

  for (const tab of dirtySourceTabs) {
    if (!await options.saveTab(tab)) return "save-failed";
  }
  return "ready";
}

export function applyWorkspaceRenameToTabs<T extends WorkspaceRenameTab>(
  tabs: readonly T[],
  sourcePath: string,
  renamedFile: { path: string; relativePath: string },
): string[] {
  const previousAbsolutePaths: string[] = [];
  for (const tab of tabs) {
    if (tab.workspacePath === sourcePath) {
      if (tab.path) previousAbsolutePaths.push(tab.path);
      tab.path = renamedFile.path;
      tab.workspacePath = renamedFile.relativePath;
    }
    if (tab.workspaceExercisePath === sourcePath) {
      tab.workspaceExercisePath = renamedFile.relativePath;
      tab.exampleName = renamedFile.relativePath.split(/[\\/]/).pop();
    }
  }
  return previousAbsolutePaths;
}
