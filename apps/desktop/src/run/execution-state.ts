import type { DebugInfo } from "../api";
import type { PendingRunEvent } from "../run-event-router";

export interface ExecutionState {
  runId: number | null;
  runTabId: number | null;
  pendingEvents: PendingRunEvent[] | null;
  ferroRunPending: boolean;
  paused: DebugInfo | null;
}

/** Cria o estado compartilhado do ciclo de execução e depuração. */
export function createExecutionState(): ExecutionState {
  return {
    runId: null,
    runTabId: null,
    pendingEvents: null,
    ferroRunPending: false,
    paused: null,
  };
}

/** Indica se há execução ou início de execução em andamento. */
export function isExecutionBusy(state: ExecutionState): boolean {
  return state.runId !== null || state.pendingEvents !== null || state.ferroRunPending;
}
