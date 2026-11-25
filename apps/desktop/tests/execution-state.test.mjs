import test from "node:test";
import assert from "node:assert/strict";
import { createExecutionState, isExecutionBusy } from "../src/run/execution-state.ts";

test("execution state starts idle and derives pending/running busy state", () => {
  const state = createExecutionState();

  assert.deepEqual(state, {
    runId: null,
    runTabId: null,
    pendingEvents: null,
    porsRunPending: false,
    paused: null,
  });
  assert.equal(isExecutionBusy(state), false);

  state.porsRunPending = true;
  assert.equal(isExecutionBusy(state), true);
  state.porsRunPending = false;
  state.pendingEvents = [];
  assert.equal(isExecutionBusy(state), true);
  state.pendingEvents = null;
  state.runId = 12;
  assert.equal(isExecutionBusy(state), true);
  state.runId = null;
  assert.equal(isExecutionBusy(state), false);
});
