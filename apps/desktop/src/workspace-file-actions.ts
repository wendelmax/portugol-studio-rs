export interface WorkspaceRenameTab {
  path?: string | null;
  workspaceId?: string;
  workspacePath?: string;
  workspaceExercisePath?: string;
  exampleName?: string;
}

export type WorkspaceRenamePreparation = "ready" | "cancelled" | "save-failed";
export type WorkspaceFileDeleteChoice = "cancel" | "save" | "delete" | "discard";

export type WorkspaceFileDeleteResult<T, R> =
  | { status: "cancelled"; remainingTabs: readonly T[] }
  | { status: "save-failed"; remainingTabs: readonly T[] }
  | { status: "needs-decision"; remainingTabs: readonly T[] }
  | { status: "saved"; remainingTabs: readonly T[] }
  | { status: "deleted"; remainingTabs: T[]; deletedTabs: T[]; deletion: R };

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

export async function deleteWorkspaceFileFlow<T extends WorkspaceRenameTab, R>(options: {
  tabs: readonly T[];
  workspaceId: string;
  filePath: string;
  choice: WorkspaceFileDeleteChoice;
  isDirty: (tab: T) => boolean;
  saveTab: (tab: T) => Promise<boolean>;
  deleteFile: () => Promise<R>;
}): Promise<WorkspaceFileDeleteResult<T, R>> {
  if (options.choice === "cancel") {
    return { status: "cancelled", remainingTabs: options.tabs };
  }

  const affectedTabs = options.tabs.filter((tab) =>
    tab.workspaceId === options.workspaceId &&
    (tab.workspacePath === options.filePath || tab.workspaceExercisePath === options.filePath),
  );
  const dirtyTabs = affectedTabs.filter(options.isDirty);

  if (options.choice === "save") {
    for (const tab of dirtyTabs) {
      if (!await options.saveTab(tab)) {
        return { status: "save-failed", remainingTabs: options.tabs };
      }
    }
    return { status: "saved", remainingTabs: options.tabs };
  }

  if (options.choice === "delete" && dirtyTabs.length > 0) {
    return { status: "needs-decision", remainingTabs: options.tabs };
  }

  const deletion = await options.deleteFile();
  return {
    status: "deleted",
    remainingTabs: options.tabs.filter((tab) => !affectedTabs.includes(tab)),
    deletedTabs: affectedTabs,
    deletion,
  };
}
