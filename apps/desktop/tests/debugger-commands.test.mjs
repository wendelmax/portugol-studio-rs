import test from "node:test";
import assert from "node:assert/strict";
import { createDebuggerCommands } from "../src/run/debugger-commands.ts";

function fixture(overrides = {}) {
  const events = [];
  let id = 42;
  let runTab = 7;
  let activeTab = 7;
  let paused = { position: { line: 2 }, variables: [] };
  const dependencies = {
    runId: () => id,
    runTabId: () => runTab,
    activeTabId: () => activeTab,
    breakpointLines: () => [2, 5],
    setBreakpoints: async (...args) => events.push(["breakpoints", ...args]),
    setDebugLine: (line) => events.push(["debugLine", line]),
    paused: () => paused,
    setPaused: (value) => { paused = value; events.push(["paused", value]); },
    setRunning: (value) => events.push(["running", value]),
    setStatus: (...args) => events.push(["status", ...args]),
    runLanguage: () => "portugol",
    debugContinue: async (runId) => events.push(["continue", runId]),
    stop: async () => true,
    ...overrides,
  };
  return {
    commands: createDebuggerCommands(dependencies), events,
    setRunId: (value) => { id = value; },
    setRunTab: (value) => { runTab = value; },
    setActiveTab: (value) => { activeTab = value; },
    getPaused: () => paused,
  };
}

test("breakpoint updates go only to the active tab's run", async () => {
  const { commands, events, setActiveTab } = fixture();

  const state = { doc: { toString: () => "programa {}" } };
  await commands.syncBreakpoints(state);
  setActiveTab(8);
  await commands.syncBreakpoints(state);

  assert.deepEqual(events, [["breakpoints", 42, "programa {}", [2, 5]]]);
});

test("debug command resumes a paused run and ignores a finished run", async () => {
  const { commands, events, setRunId } = fixture();

  await commands.debugCommand(async (id) => events.push(["step", id]));
  setRunId(null);
  await commands.debugCommand(async (id) => events.push(["step", id]));

  assert.equal(events.some(([name, value]) => name === "debugLine" && value === null), true);
  assert.equal(events.some(([name, value]) => name === "paused" && value === null), true);
  assert.equal(events.some(([name, id]) => name === "step" && id === 42), true);
  assert.equal(events.filter(([name]) => name === "step").length, 1);
});

test("stop reports completion only when the same run is still active", async () => {
  const { commands, events, setRunId, setActiveTab } = fixture({ stop: async () => true });

  await commands.stopProgram();
  setRunId(42);
  setActiveTab(8);
  await commands.stopProgram();

  assert.equal(events.filter(([name, message]) => name === "status" && message === "Interrupção solicitada…").length, 1);
});
