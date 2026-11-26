import type { Tab } from "../state/tab";
import type { ExerciseDocument } from "../exercises";
import type { WorkspaceExerciseRef } from "../state/exercise-tab";
import { mountExercises } from "../exercises";
import { createExerciseTabOpener } from "../state/exercise-tab";
import { cargoTestPanelState, cargoTestStatus, exerciseTestPanelState, exerciseTestStatus, isCurrentExerciseTestRequest } from "../exercise-test-panel";
import { createTestsPanelController } from "../run/tests-panel";
import type { SourceLanguage } from "../document-language";
import { $ } from "../dom";

export interface ExerciseWorkbenchDependencies {
  tabs: () => Tab[];
  replaceTabs: (tabs: Tab[]) => void;
  activeTab: () => Tab | undefined;
  activeId: () => number;
  isDirty: (tab: Tab) => boolean;
  template: string;
  newTab: (text: string, path: string | null, exampleName?: string, examplePath?: string, exercise?: ExerciseDocument, explicitLanguage?: SourceLanguage) => Tab;
  activate: (id: number) => void;
  selectWorkspace: (id: string, path: string) => void;
  currentDocument: () => { toString(): string };
  documentForTab: (tab: Tab) => { toString(): string };
  cargoAvailable: () => boolean | null;
  runId: () => number | null;
  runPending: () => boolean;
  tabName: (tab: Tab) => string;
  runCargoTests: () => Promise<void>;
}

/** Compõe abertura, avaliação e painel de testes dos exercícios da IDE. */
export function createExerciseWorkbenchShell(dependencies: ExerciseWorkbenchDependencies) {
  const exerciseTabOpener = createExerciseTabOpener({
    tabs: dependencies.tabs,
    replaceTabs: dependencies.replaceTabs,
    activeTab: dependencies.activeTab,
    isDirty: dependencies.isDirty,
    template: dependencies.template,
    newTab: dependencies.newTab,
    activate: dependencies.activate,
    selectWorkspace: dependencies.selectWorkspace,
  });
  const exercises = mountExercises((exercise) => exerciseTabOpener.open(exercise));
  const testsPanel = createTestsPanelController({
    tabs: dependencies.tabs,
    activeTab: dependencies.activeTab,
    activeId: dependencies.activeId,
    currentDocument: dependencies.currentDocument,
    documentForTab: dependencies.documentForTab,
    cargoAvailable: dependencies.cargoAvailable,
    runId: dependencies.runId,
    runPending: dependencies.runPending,
    tabName: dependencies.tabName,
    evaluateExercise: (exercise, source, isCurrent) => exercises.evaluate(exercise, source, isCurrent),
    cargoTestPanelState,
    cargoTestStatus,
    exerciseTestPanelState,
    exerciseTestStatus,
    isCurrentExerciseTestRequest,
    runCargoTests: dependencies.runCargoTests,
    view: {
      clearResults: () => $<HTMLUListElement>("tests-results").replaceChildren(),
      setTitle: (title) => { $("tests-title").textContent = title; },
      setStatus: (status) => {
        const element = $("tests-status");
        if (element.textContent !== status) element.textContent = status;
      },
      setRunHidden: (hidden) => { $<HTMLButtonElement>("tests-run").hidden = hidden; },
      setRunDisabled: (disabled) => { $<HTMLButtonElement>("tests-run").disabled = disabled; },
      setRunText: (text) => { $<HTMLButtonElement>("tests-run").textContent = text; },
      appendResult: (text) => {
        const item = document.createElement("li");
        item.textContent = text;
        $<HTMLUListElement>("tests-results").append(item);
      },
      setResultsHidden: (hidden) => { $<HTMLUListElement>("tests-results").hidden = hidden; },
    },
  });

  testsPanel.render();
  $("home-exercises").onclick = exercises.browse;
  $("tests-run").onclick = () => { void testsPanel.runExerciseTests(); };

  function gradeActiveExercise() {
    const tab = dependencies.activeTab();
    const exercise = tab?.exercise;
    if (!tab || !exercise) return;
    const documentState = dependencies.currentDocument();
    const isCurrent = () => dependencies.tabs().includes(tab)
      && (tab.id === dependencies.activeId() ? dependencies.currentDocument() : tab.state.doc) === documentState;
    void exercises.grade(exercise, documentState.toString(), isCurrent);
  }

  return {
    openExercise: (exercise: ExerciseDocument, workspaceExercise?: WorkspaceExerciseRef) =>
      exerciseTabOpener.open(exercise, workspaceExercise),
    openPorsById: (exerciseId: string) => exercises.openPorsById(exerciseId),
    renderTests: () => testsPanel.render(),
    runTests: () => testsPanel.runExerciseTests(),
    gradeActiveExercise,
  };
}
