import test from "node:test";
import assert from "node:assert/strict";
import { createPortugolRsExecutionController } from "../src/run/portugol-rs-execution.ts";

function fixture(overrides = {}) {
  const events = [];
  const tab = { id: 3, language: "portugol-rs", path: "exemplo.pors", diagnostics: [] };
  const document = { toString: () => "funcao inicio() {}" };
  let currentTab = tab;
  let currentDocument = document;
  let runId = null;
  let pending = false;
  let pendingEvents = null;
  let rustcAvailable = true;
  const dependencies = {
    rustcAvailable: () => rustcAvailable,
    isCurrent: (candidate, source) => currentTab === candidate && currentDocument === source,
    setRunId: (id) => { runId = id; events.push(["runId", id]); },
    runId: () => runId,
    setPending: (value) => { pending = value; events.push(["pending", value]); },
    setRunTabId: (id) => events.push(["runTab", id]),
    setPendingEvents: (value) => { pendingEvents = value; events.push(["pendingEvents", value === null ? null : "events"]); },
    setRunning: (value) => events.push(["running", value]),
    clearRuntimeError: () => events.push(["clearRuntime"]),
    clearDebugLine: () => events.push(["clearDebug"]),
    clearPaused: () => events.push(["paused", null]),
    showConsole: () => events.push(["showConsole"]),
    clearConsole: () => events.push(["clearConsole"]),
    hideInput: () => events.push(["hideInput"]),
    runListeners: () => Promise.resolve(),
    startVm: async (source, lines, pause, path) => { events.push(["vm", source, lines, pause, path]); return 20; },
    registerRun: async () => 21,
    runGeneratedRust: async () => ({ status: "executed", stdout: "rodou\n", stderr: "", exitCode: 0 }),
    setStatus: (...args) => events.push(["status", ...args]),
    appendProcessOutput: (result) => events.push(["output", result.stdout]),
    appendConsole: (...args) => events.push(["console", ...args]),
    renderMessages: () => events.push(["renderMessages"]),
    setDiagnostics: (candidate, diagnostics) => { candidate.runtimeCompileDiagnostics = diagnostics; events.push(["diagnostics", diagnostics]); },
    setVmDiagnostics: (candidate, diagnostics) => { candidate.diagnostics = diagnostics; events.push(["vmDiagnostics", diagnostics]); },
    showMessages: () => events.push(["showMessages"]),
    sourceDiagnostic: (diagnostic) => diagnostic.sourceStart ? ({ ...diagnostic, severity: "error", code: diagnostic.code ?? "rustc", help: diagnostic.rendered }) : null,
    toCodeMirror: (diagnostic) => diagnostic,
    replayEvents: (queued, id) => events.push(["replay", queued.length, id]),
    ...overrides.dependencies,
  };
  const controller = createPortugolRsExecutionController(dependencies);
  return {
    controller, dependencies, events, tab, document,
    setCurrent: (nextTab, nextDocument = document) => { currentTab = nextTab; currentDocument = nextDocument; },
    setRustcAvailable: (value) => { rustcAvailable = value; },
    getRunId: () => runId,
    getPending: () => pending,
    getPendingEvents: () => pendingEvents,
  };
}

test("Ferro VM starts with active breakpoints and replays early events", async () => {
  const { controller, events, tab, document, getPendingEvents } = fixture();

  await controller.run(tab, document, [2, 4], true, false);

  assert.deepEqual(events.find(([name]) => name === "vm"), ["vm", "funcao inicio() {}", [2, 4], true, "exemplo.pors"]);
  assert.deepEqual(events.find(([name]) => name === "replay"), ["replay", 0, 20]);
  assert.equal(getPendingEvents(), null);
});

test("VM compile diagnostics are displayed as a Ferro debug error", async () => {
  const diagnostics = [{ severity: "error", code: "PORS001", message: "erro", help: null, start: {}, end: {} }];
  const { controller, dependencies, events, tab, document } = fixture({
    dependencies: { startVm: async () => { throw diagnostics; } },
  });

  await controller.run(tab, document, [], true, false);

  assert.equal(tab.diagnostics, diagnostics);
  assert.equal(events.some(([name, message]) => name === "status" && message === "O Ferro contém erros e não pode ser depurado."), true);
  assert.equal(events.some(([name, panel]) => name === "showMessages"), true);
});

test("generated Rust diagnostics map back to PORS and discard generated-only entries", async () => {
  const mapped = { sourceStart: { offset: 2, line: 1, column: 3 }, sourceEnd: { offset: 4, line: 1, column: 5 }, rendered: "detail", code: "E0001", message: "erro" };
  const { controller, dependencies, events, tab, document } = fixture({
    dependencies: {
      runGeneratedRust: async () => ({
        status: "compile_error",
        diagnostics: [mapped, { ...mapped, sourceStart: null }],
        stderr: "erro rustc",
      }),
    },
  });

  await controller.run(tab, document, [], false, true);

  assert.deepEqual(tab.runtimeCompileDiagnostics, [{ ...mapped, severity: "error", help: "detail" }]);
  assert.equal(events.some(([name, message]) => name === "status" && message === "rustc encontrou erros no Rust gerado. Selecione um diagnóstico para voltar à origem."), true);
  assert.equal(events.some(([name]) => name === "showMessages"), true);
});

test("stale generated-Rust results are ignored and pending run state is cleared", async () => {
  let stale = false;
  const { controller, events, tab, document, getRunId, getPending } = fixture({
    dependencies: {
      isCurrent: () => !stale,
      runGeneratedRust: async () => {
        stale = true;
        return { status: "executed", stdout: "antigo", stderr: "", exitCode: 0 };
      },
    },
  });

  await controller.run(tab, document, [], false, true);

  assert.equal(events.some(([name]) => name === "output"), false);
  assert.equal(getRunId(), null);
  assert.equal(getPending(), false);
});
