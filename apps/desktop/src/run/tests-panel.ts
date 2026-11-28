import type {
  CargoTestPanelState,
  CargoTestRun,
  ExerciseTestPanelState,
  ExerciseTestRequest,
  ExerciseTestRun,
} from "../exercise-test-panel";
import type { ExerciseDocument, ExerciseGrade } from "../exercises";
import type { Tab } from "../state/tab";

export interface TestsPanelView {
  clearResults: () => void;
  setTitle: (title: string) => void;
  setStatus: (status: string) => void;
  setRunHidden: (hidden: boolean) => void;
  setRunDisabled: (disabled: boolean) => void;
  setRunText: (text: string) => void;
  appendResult: (text: string) => void;
  setResultsHidden: (hidden: boolean) => void;
}

export interface TestsPanelDependencies {
  tabs: () => Tab[];
  activeTab: () => Tab | undefined;
  activeId: () => number;
  currentDocument: () => object;
  documentForTab: (tab: Tab) => object;
  cargoAvailable: () => boolean | null;
  runId: () => number | null;
  runPending: () => boolean;
  tabName: (tab: Tab) => string;
  evaluateExercise: (
    exercise: ExerciseDocument,
    source: string,
    isCurrent: () => boolean,
  ) => Promise<ExerciseGrade>;
  cargoTestPanelState: (currentDocument: object, run: CargoTestRun | undefined) => CargoTestPanelState;
  cargoTestStatus: (state: CargoTestPanelState) => string;
  exerciseTestPanelState: (
    exercise: ExerciseDocument["exercise"] | undefined,
    currentDocument: object,
    run: ExerciseTestRun | undefined,
  ) => ExerciseTestPanelState;
  exerciseTestStatus: (state: ExerciseTestPanelState, language?: "ferro-rust") => string;
  isCurrentExerciseTestRequest: (
    request: ExerciseTestRequest,
    current: ExerciseTestRequest,
    tabExists: boolean,
  ) => boolean;
  runCargoTests: () => Promise<void>;
  view: TestsPanelView;
}

/** Controla a exibição e a execução dos testes de exercícios e projetos Cargo. */
export function createTestsPanelController(dependencies: TestsPanelDependencies) {
  function render() {
    const tab = dependencies.activeTab();
    const view = dependencies.view;
    view.clearResults();

    if (!tab?.exercise && tab?.language === "rust" && tab.cargoProject === true) {
      const state = dependencies.cargoTestPanelState(dependencies.currentDocument(), tab.cargoTestRun);
      view.setTitle(`Testes Cargo — ${dependencies.tabName(tab)}`);
      view.setStatus(dependencies.cargoTestStatus(state));
      view.setRunHidden(false);
      view.setRunDisabled(
        state.kind === "running"
        || dependencies.cargoAvailable() !== true
        || dependencies.runId() !== null
        || dependencies.runPending(),
      );
      view.setRunText(state.kind === "running" ? "Executando…" : "Executar testes");
      if (state.kind === "completed") {
        for (const failure of state.result.failures) view.appendResult(failure);
      }
      view.setResultsHidden(state.kind !== "completed" || state.kind === "completed" && state.result.failures.length === 0);
      return;
    }

    const state = dependencies.exerciseTestPanelState(tab?.exercise?.exercise, dependencies.currentDocument(), tab?.exerciseTestRun);
    const language = tab?.exercise ? "ferro-rust" : undefined;
    view.setTitle(tab?.exercise ? `Testes — ${dependencies.tabName(tab)}` : "Testes do projeto");
    view.setStatus(dependencies.exerciseTestStatus(state, language));
    view.setRunHidden(state.kind === "empty" || state.kind === "no-cases");
    view.setRunDisabled(state.kind === "running");
    view.setRunText(state.kind === "running" ? "Executando…" : "Executar casos");
    let resultCount = 0;
    if (state.kind === "completed") {
      for (const failure of state.result.failures) {
        view.appendResult(failure);
        resultCount += 1;
      }
      for (const hint of state.result.hints ?? []) {
        view.appendResult(`Dica: ${hint}`);
        resultCount += 1;
      }
    }
    view.setResultsHidden(state.kind !== "completed" || resultCount === 0);
  }

  async function runExerciseTests(): Promise<void> {
    const tab = dependencies.activeTab();
    if (!tab?.exercise && tab?.language === "rust" && tab.cargoProject === true) {
      await dependencies.runCargoTests();
      return;
    }
    if (!tab?.exercise || tab.exercise.exercise.cases <= 0) return;
    const document = dependencies.currentDocument();
    const request: ExerciseTestRequest = {
      tabId: tab.id,
      document,
      generation: (tab.exerciseTestGeneration ?? 0) + 1,
    };
    tab.exerciseTestGeneration = request.generation;
    tab.exerciseTestRun = { source: document, status: "running" };
    render();
    const isCurrent = () => {
      const currentDocument = tab.id === dependencies.activeId()
        ? dependencies.currentDocument()
        : dependencies.documentForTab(tab);
      return dependencies.isCurrentExerciseTestRequest(request, {
        tabId: tab.id,
        document: currentDocument,
        generation: tab.exerciseTestGeneration ?? 0,
      }, dependencies.tabs().includes(tab));
    };
    try {
      const grade = await dependencies.evaluateExercise(tab.exercise, document.toString(), isCurrent);
      if (!isCurrent()) return;
      if (!grade.current) {
        tab.exerciseTestRun = {
          source: document,
          status: "error",
          message: "Esta avaliação foi substituída por outra. Execute os casos novamente.",
        };
        return;
      }
      tab.exerciseTestRun = {
        source: document,
        status: "completed",
        result: {
          total: grade.evaluation.total,
          passed: grade.evaluation.passed,
          failures: grade.evaluation.failures,
          hints: grade.evaluation.hints,
          completed: grade.progress.completed,
        },
      };
    } catch (message) {
      if (isCurrent()) {
        tab.exerciseTestRun = {
          source: document,
          status: "error",
          message: typeof message === "string" ? message : "Não foi possível executar os casos do exercício.",
        };
      }
    } finally {
      render();
    }
  }

  return { render, runExerciseTests };
}
