import type { Exercise } from "./api";

export interface ExerciseTestResult {
  total: number;
  passed: number;
  failures: string[];
  completed?: boolean;
}

export type ExerciseTestRun =
  | { source: object; status: "running" }
  | { source: object; status: "completed"; result: ExerciseTestResult }
  | { source: object; status: "error"; message: string };

export type ExerciseTestPanelState =
  | { kind: "empty" }
  | { kind: "no-cases" }
  | { kind: "ready"; cases: number }
  | { kind: "running"; cases: number }
  | { kind: "completed"; result: ExerciseTestResult }
  | { kind: "error"; message: string };

export function exerciseTestPanelState(
  exercise: Exercise | undefined,
  currentDocument: object,
  run: ExerciseTestRun | undefined,
): ExerciseTestPanelState {
  if (!exercise) return { kind: "empty" };
  if (exercise.cases <= 0) return { kind: "no-cases" };
  if (!run || run.source !== currentDocument) return { kind: "ready", cases: exercise.cases };
  if (run.status === "running") return { kind: "running", cases: exercise.cases };
  if (run.status === "completed") return { kind: "completed", result: run.result };
  return { kind: "error", message: run.message };
}

export interface ExerciseTestRequest {
  tabId: number;
  document: object;
  generation: number;
}

export function isCurrentExerciseTestRequest(
  request: ExerciseTestRequest,
  current: ExerciseTestRequest,
  tabExists: boolean,
): boolean {
  return tabExists
    && request.tabId === current.tabId
    && request.document === current.document
    && request.generation === current.generation;
}

export function exerciseTestStatus(state: ExerciseTestPanelState): string {
  switch (state.kind) {
    case "empty": return "Nenhum teste configurado para este arquivo.";
    case "no-cases": return "Este exercício não possui casos de execução configurados.";
    case "ready": return `${state.cases} casos configurados para este exercício.`;
    case "running": return `Executando ${state.cases} casos desta revisão…`;
    case "completed": return state.result.completed
      ? `${state.result.passed} de ${state.result.total} casos aprovados. Exercício concluído nesta revisão.`
      : `${state.result.passed} de ${state.result.total} casos aprovados.`;
    case "error": return state.message;
  }
}
