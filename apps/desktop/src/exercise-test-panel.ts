import type { Exercise } from "./api";

export interface ExerciseTestResult {
  total: number;
  passed: number;
  failures: string[];
  hints?: string[];
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

export function exerciseTestStatus(
  state: ExerciseTestPanelState,
  language?: "portugol" | "portugol-rs",
): string {
  const tag = language === "portugol-rs" ? " (.pors)" : "";
  switch (state.kind) {
    case "empty": return "Nenhum teste configurado para este arquivo.";
    case "no-cases": return "Este exercício não possui casos de execução configurados.";
    case "ready": return `${state.cases} casos configurados para este exercício${tag}.`;
    case "running": return `Executando ${state.cases} casos desta revisão${tag}…`;
    case "completed": return state.result.completed
      ? `${state.result.passed} de ${state.result.total} casos aprovados${tag}. Exercício concluído nesta revisão.`
      : `${state.result.passed} de ${state.result.total} casos aprovados${tag}.`;
    case "error": return state.message;
  }
}

export interface CargoTestSummary {
  status: "passed" | "failed" | "build_error" | "test_limit" | "cancelled";
  passed: number;
  failed: number;
  ignored: number;
  failures: string[];
  reason?: "time" | "output" | "memory" | "cancelled" | null;
}

export type CargoTestRun =
  | { source: object; status: "running" }
  | { source: object; status: "completed"; result: CargoTestSummary }
  | { source: object; status: "error"; message: string };

export type CargoTestPanelState =
  | { kind: "ready" }
  | { kind: "running" }
  | { kind: "completed"; result: CargoTestSummary }
  | { kind: "error"; message: string };

export function cargoTestPanelState(
  currentDocument: object,
  run: CargoTestRun | undefined,
): CargoTestPanelState {
  if (!run || run.source !== currentDocument) return { kind: "ready" };
  if (run.status === "running") return { kind: "running" };
  if (run.status === "completed") return { kind: "completed", result: run.result };
  return { kind: "error", message: run.message };
}

export function cargoTestStatus(state: CargoTestPanelState): string {
  switch (state.kind) {
    case "ready": return "Projeto Cargo pronto para executar testes (`cargo test`).";
    case "running": return "Executando `cargo test` para esta revisão…";
    case "error": return state.message;
    case "completed": {
      const { status, passed, failed, ignored, reason } = state.result;
      if (status === "cancelled") return "Execução de `cargo test` cancelada.";
      if (status === "test_limit") {
        return `Execução de \`cargo test\` interrompida (${reason === "time" ? "tempo limite" : "limite de saída"}).`;
      }
      if (status === "build_error") {
        return "Falha de compilação ao executar `cargo test`; confira o Console.";
      }
      const extra = ignored > 0 ? `, ${ignored} ignorado(s)` : "";
      return status === "passed"
        ? `Cargo test aprovado: ${passed} teste(s) aprovado(s)${extra}.`
        : `Cargo test falhou: ${passed} aprovado(s), ${failed} falhou(aram)${extra}.`;
    }
  }
}
