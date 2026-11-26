import test from "node:test";
import assert from "node:assert/strict";
import { createRunEventsController } from "../src/run/run-events.ts";

function fixture() {
  const events = [];
  let runId = 8;
  let runTabId = 3;
  let activeTabId = 3;
  let pending = null;
  const dependencies = {
    runId: () => runId,
    setRunId: (value) => { runId = value; events.push(["runId", value]); },
    runTabId: () => runTabId,
    setRunTabId: (value) => { runTabId = value; events.push(["runTab", value]); },
    activeTabId: () => activeTabId,
    pendingEvents: () => pending,
    setPaused: (value) => events.push(["paused", value]),
    setRunning: (value) => events.push(["running", value]),
    setDebugLine: (value) => events.push(["debugLine", value]),
    setRuntimeError: (value) => events.push(["runtimeError", value]),
    appendConsole: (...args) => events.push(["console", ...args]),
    clearConsole: () => events.push(["clear"]),
    setInputHint: (text) => events.push(["hint", text]),
    showInput: () => events.push(["showInput"]),
    clearInput: () => events.push(["clearInput"]),
    focusInput: () => events.push(["focusInput"]),
    scrollConsole: () => events.push(["scroll"]),
    setStatus: (...args) => events.push(["status", ...args]),
    goTo: (offset) => events.push(["goTo", offset]),
  };
  return {
    controller: createRunEventsController(dependencies), events,
    setRunId: (value) => { runId = value; },
    setRunTabId: (value) => { runTabId = value; },
    setActiveTabId: (value) => { activeTabId = value; },
    setPending: (value) => { pending = value; },
  };
}

test("run events render only for their active run and queue events during startup", () => {
  const { controller, events, setPending, setRunId } = fixture();

  controller.handlers.output(8, "ativo");
  setRunId(null);
  controller.handlers.output(7, "perdido");
  const queued = [];
  setPending(queued);
  controller.handlers.output(9, "inicial");

  assert.deepEqual(events.filter(([name]) => name === "console"), [["console", "ativo"]]);
  assert.equal(queued.length, 1);
});

test("input event reveals the expected console prompt", () => {
  const { controller, events } = fixture();

  controller.handlers.input(8, "inteiro");

  assert.deepEqual(events, [
    ["hint", "leia (inteiro):"], ["showInput"], ["clearInput"], ["focusInput"], ["scroll"],
  ]);
});

test("runtime error completion clears run state and marks the running editor", () => {
  const { controller, events } = fixture();

  controller.handlers.finished(8, {
    status: "error", message: "falhou",
    start: { offset: 5, line: 2, column: 1 }, end: { offset: 8, line: 2, column: 4 }, trace: [],
  });

  assert.deepEqual(events.slice(0, 5), [
    ["runId", null], ["runTab", null], ["paused", null], ["debugLine", null], ["running", false],
  ]);
  assert.deepEqual(events.find(([name]) => name === "runtimeError"), ["runtimeError", { from: 5, to: 8 }]);
  assert.deepEqual(events.find(([name]) => name === "goTo"), ["goTo", 5]);
});

test("background runtime errors do not move the active editor cursor", () => {
  const { controller, events, setActiveTabId } = fixture();
  setActiveTabId(9);

  controller.handlers.finished(8, {
    status: "error", message: "falhou",
    start: { offset: 5, line: 2, column: 1 }, end: { offset: 8, line: 2, column: 4 }, trace: [],
  });

  assert.equal(events.some(([name]) => name === "runtimeError" || name === "goTo"), false);
});

test("runtime error preserves the caller frame (inicio) in the stack trace", () => {
  const { controller, events } = fixture();

  controller.handlers.finished(8, {
    status: "error",
    message: "divisão por zero",
    start: { offset: 12, line: 2, column: 5 },
    end: { offset: 18, line: 2, column: 11 },
    trace: [
      { function: "dividir", line: 2 },
      { function: "inicio", line: 6 },
    ],
  });

  const errorLog = events.find(([name, , kind]) => name === "console" && kind === "error")?.[1] ?? "";
  assert.match(errorLog, /chamado em inicio \(linha 6\)/);
});
