import type { ExerciseDocument } from "../exercises";
import type { WorkspaceExerciseRef } from "../state/exercise-tab";
import { mountWorkspaceExplorer, type WorkspaceExplorerHost } from "../workspace/explorer";
import { createFileIO, type FileIODependencies } from "./io";
import { createRecentFiles } from "./recent";
import { createRecentFilesView, type RecentFilesViewDependencies } from "./recent-view";

type FileStateDependencies = Pick<FileIODependencies,
  "tabs" | "replaceTabs" | "activeTab" | "activate" | "isDirty"> & {
  newWorkspaceTab: () => void;
  newFileTab: FileIODependencies["newTab"];
};

type WorkspaceDependencies = Pick<WorkspaceExplorerHost,
  "activeId" | "homeId" | "openWorkspaceSource" | "renderTabs" | "setExplorerVisible" | "cargoProjectDetected" | "screenOpened"> & {
  openExercise: (document: ExerciseDocument, workspaceExercise?: WorkspaceExerciseRef) => void;
};

export interface WorkspaceFilesShellDependencies {
  state: FileStateDependencies;
  workspace: WorkspaceDependencies;
  files: Omit<FileIODependencies,
    "tabs" | "replaceTabs" | "activeTab" | "activate" | "newTab" | "isDirty" | "rememberRecent">;
  recentView: Omit<RecentFilesViewDependencies, "openFile" | "setStatus">;
}

function localStorageOrNull(): Storage | null {
  try {
    return localStorage;
  } catch {
    return null;
  }
}

/** Composes file I/O, arquivos recentes e explorador com dependências cíclicas resolvidas localmente. */
export function createWorkspaceFilesShell(dependencies: WorkspaceFilesShellDependencies) {
  let fileIO!: ReturnType<typeof createFileIO>;
  const recentFilesView = createRecentFilesView({
    ...dependencies.recentView,
    openFile: (path) => fileIO.openFile(path),
    setStatus: dependencies.files.setStatus,
  });
  const recentFiles = createRecentFiles(localStorageOrNull(), (list) => recentFilesView.render(list));
  fileIO = createFileIO({
    ...dependencies.files,
    tabs: dependencies.state.tabs,
    replaceTabs: dependencies.state.replaceTabs,
    activeTab: dependencies.state.activeTab,
    activate: dependencies.state.activate,
    newTab: dependencies.state.newFileTab,
    isDirty: dependencies.state.isDirty,
    rememberRecent: (path) => recentFiles.remember(path),
  });
  const workspaceExplorer = mountWorkspaceExplorer({
    tabs: dependencies.state.tabs,
    replaceTabs: dependencies.state.replaceTabs,
    activeTab: dependencies.state.activeTab,
    activeId: dependencies.workspace.activeId,
    homeId: dependencies.workspace.homeId,
    activate: dependencies.state.activate,
    newTab: dependencies.state.newWorkspaceTab,
    openWorkspaceSource: dependencies.workspace.openWorkspaceSource,
    openExercise: dependencies.workspace.openExercise,
    isDirty: dependencies.state.isDirty,
    save: () => fileIO.save(),
    renderTabs: dependencies.workspace.renderTabs,
    setExplorerVisible: dependencies.workspace.setExplorerVisible,
    cargoProjectDetected: dependencies.workspace.cargoProjectDetected,
    screenOpened: () => dependencies.workspace.screenOpened?.(),
    recentFiles: {
      remember: (path) => recentFiles.remember(path),
      rename: (oldPath, newPath) => recentFiles.rename(oldPath, newPath),
      forget: (path) => recentFiles.forget(path),
    },
  });

  workspaceExplorer.render();
  void workspaceExplorer.restore();
  return { fileIO, recentFiles, recentFilesView, workspaceExplorer };
}
