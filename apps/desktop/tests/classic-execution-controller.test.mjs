import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createClassicExecutionController } from "../src/run/classic-execution.ts";

function fixture(overrides = {}) {
  const events = [];
  const tab = { id: 4, path: "aula.por", diagnostics: [] };
  const document = { toString: () => "programa {}" };
  let runId = null;
  let pendingEvents = null;
  const dependencies = {
    parseArguments: (value) => value.trim() ? value.trim().split(" ") : [],
    runListeners: () => Promise.resolve(),
    run: async (...args) => { events.push(["run", ...args]); return 12; },
    setRuntimeError: () => events.push(["clearRuntime"]),
    setDebugLine: () => events.push(["clearDebug"]),
    clearPaused: () => events.push(["paused", null]),
    showConsole: () => events.push(["showConsole"]),
    clearConsole: () => events.push(["clearConsole"]),
    setPendingEvents: (value) => { pendingEvents = value; events.push(["pendingEvents", value === null ? null : "events"]); },
    setRunTabId: (value) => events.push(["runTab", value]),
    setRunId: (value) => { runId = value; events.push(["runId", value]); },
    setRunButtonsDisabled: (value) => events.push(["buttonsDisabled", value]),
    setRunning: (value) => events.push(["running", value]),
    setStatus: (...args) => events.push(["status", ...args]),
    replayEvents: (queued, id) => events.push(["replay", queued.length, id]),
    setDiagnostics: (candidate, diagnostics) => { candidate.diagnostics = diagnostics; events.push(["diagnostics", diagnostics]); },
    renderMessages: () => events.push(["renderMessages"]),
    showMessages: () => events.push(["showMessages"]),
    ...overrides,
  };
  return {
    controller: createClassicExecutionController(dependencies), dependencies, events, tab, document,
    getRunId: () => runId,
    getPendingEvents: () => pendingEvents,
  };
}

test("classic execution starts with arguments, breakpoints and queued event replay", async () => {
  const { controller, events, tab, document, getPendingEvents } = fixture();

  await controller.run(tab, document, "5 abc", [2, 8], true);

  assert.deepEqual(events.find(([name]) => name === "run"), ["run", "programa {}", ["5", "abc"], "aula.por", [2, 8], true]);
  assert.deepEqual(events.find(([name]) => name === "replay"), ["replay", 0, 12]);
  assert.equal(getPendingEvents(), null);
});

test("classic compile diagnostics are assigned to their tab and shown", async () => {
  const diagnostics = [{ severity: "error", message: "erro" }];
  const { controller, events, tab } = fixture({ run: async () => { throw diagnostics; } });

  await controller.run(tab, { toString: () => "inválido" }, "", [], false);

  assert.equal(tab.diagnostics, diagnostics);
  assert.equal(events.some(([name]) => name === "renderMessages"), true);
  assert.equal(events.some(([name]) => name === "showMessages"), true);
  assert.deepEqual(events.find(([name]) => name === "status"), ["status", "O programa contém erros e não pode ser executado.", "error"]);
  assert.equal(events.some(([name, value]) => name === "pendingEvents" && value === null), true);
});

test("listener and backend errors restore classic run controls", async () => {
  const { controller, events, getRunId } = fixture({ runListeners: async () => { throw new Error("listener indisponível"); } });

  await controller.run({ id: 4, diagnostics: [] }, { toString: () => "programa {}" }, "", [], false);

  assert.equal(getRunId(), null);
  assert.deepEqual(events.find(([name]) => name === "status"), ["status", "Error: listener indisponível", "error"]);
  assert.equal(events.some(([name, value]) => name === "runTab" && value === null), true);
  assert.equal(events.some(([name, value]) => name === "running" && value === false), true);
  assert.deepEqual(events.at(-1), ["pendingEvents", null]);
});

test("IDE delegates classic execution through the controller and api boundary", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const shell = await readFile(new URL("../src/run/execution-controller-shell.ts", import.meta.url), "utf8");
  const runShell = await readFile(new URL("../src/run/run-shell.ts", import.meta.url), "utf8");
  const dispatcher = await readFile(new URL("../src/run/dispatcher.ts", import.meta.url), "utf8");

  assert.match(shell, /createClassicExecutionController\(/);
  assert.match(shell, /run: api\.run/);
  assert.match(shell, /createRunDispatcher\(/);
  assert.match(main, /createRunShell\(/);
  assert.match(runShell, /createExecutionControllerShell\(/);
  assert.match(dispatcher, /classicExecution\.run\(tab, document, dependencies\.argumentsText\(\), breakpoints, pauseOnStart\)/);
});
