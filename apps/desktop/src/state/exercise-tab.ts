import type { Tab } from "./tab";
import type { SourceLanguage } from "../document-language";
import type { ExerciseDocument } from "../exercises";

export interface WorkspaceExerciseRef {
  id: string;
  path: string;
  preferenceKey: string;
  exercise: true;
}

export interface ExerciseTabDependencies {
  tabs: () => Tab[];
  replaceTabs: (tabs: Tab[]) => void;
  activeTab: () => Tab | undefined;
  isDirty: (tab: Tab) => boolean;
  template: string;
  newTab: (
    text: string,
    path: string | null,
    exampleName?: string,
    examplePath?: string,
    exercise?: ExerciseDocument,
    explicitLanguage?: SourceLanguage,
  ) => Tab;
  activate: (id: number) => void;
  selectWorkspace: (id: string, path: string) => void;
}

/** Creates or reactivates the editor tab for a loaded exercise. */
export function createExerciseTabOpener(dependencies: ExerciseTabDependencies) {
  function open(exerciseDocument: ExerciseDocument, workspaceExercise?: WorkspaceExerciseRef) {
    if (workspaceExercise) {
      const existing = dependencies.tabs().find((tab) =>
        tab.workspaceId === workspaceExercise.id && tab.workspaceExercisePath === workspaceExercise.path,
      );
      if (existing) {
        dependencies.activate(existing.id);
        return;
      }
    }

    const template = exerciseDocument.pors?.starter_code ?? dependencies.template;
    const source = `/*\n${exerciseDocument.exercise.statement.replaceAll("*/", "* /")}\n*/\n\n${template}`;
    const current = dependencies.activeTab();
    const tabs = dependencies.tabs();
    if (current && !current.path && !current.exampleName && !dependencies.isDirty(current) && tabs.length === 1) {
      dependencies.replaceTabs([]);
    }

    const tab = dependencies.newTab(
      source,
      null,
      exerciseDocument.name,
      undefined,
      exerciseDocument,
      "portugol-rs",
    );
    if (!workspaceExercise) return;

    tab.workspaceId = workspaceExercise.id;
    tab.workspaceExercisePath = workspaceExercise.path;
    tab.workspacePreferenceKey = workspaceExercise.preferenceKey;
    dependencies.selectWorkspace(workspaceExercise.id, workspaceExercise.path);
    dependencies.activate(tab.id);
  }

  return { open };
}
