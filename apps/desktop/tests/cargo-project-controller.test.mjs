import test from "node:test";
import assert from "node:assert/strict";
import { createCargoProjectController } from "../src/run/cargo-project.ts";

function fixture(overrides = {}) {
  const events = [];
  let document = { text: "fn main() {}" };
  let active = {
    id: 7,
    path: "src/main.rs",
    language: "rust",
    workspaceId: "workspace-1",
    workspacePath: "src/main.rs",
    cargoProject: true,
    state: { doc: document },
  };
  let tabs = [active];
  let cargoAvailable = true;
  let busy = false;
  let runningId = null;
  const dependencies = {
    tabs: () => tabs,
    activeTab: () => active,
    isBusy: () => busy,
    cargoAvailable: () => cargoAvailable,
    isDirty: (tab) => tab.dirty === true,
    save: async () => { events.push(["save"]); return true; },
    currentDocument: () => document,
    isCurrent: (tab, source) => active === tab && document === source,
    setPending: (value) => events.push(["pending", value]),
    setRunTabId: (id) => events.push(["runTab", id]),
    runId: () => runningId,
    setRunId: (id) => { runningId = id; events.push(["runId", id]); },
    syncLanguageControls: (tab) => events.push(["sync", tab.id]),
    setRunning: (value) => events.push(["running", value]),
    hasCargoConsent: () => true,
    askCargoConsent: async (operation) => { events.push(["consent", operation]); return true; },
    rememberCargoConsent: () => events.push(["rememberConsent"]),
    registerRun: async () => 42,
    checkCargoProject: async () => ({ status: "checked", diagnostics: [], stderr: "", reason: null }),
    testCargoProject: async () => ({
      status: "passed", passed: 2, failed: 0, ignored: 1, failures: [], stdout: "ok\n", stderr: "", exitCode: 0, reason: null,
    }),
    clearRuntimeError: () => events.push(["clearRuntimeError"]),
    clearConsole: () => events.push(["clearConsole"]),
    showPanel: (name) => events.push(["panel", name]),
    setStatus: (...args) => events.push(["status", ...args]),
    setDiagnostics: (tab, diagnostics) => { tab.diagnostics = diagnostics; events.push(["diagnostics", diagnostics]); },
    appendConsole: (...args) => events.push(["console", ...args]),
    renderExerciseTests: () => events.push(["renderTests"]),
    ...overrides.dependencies,
  };
  const controller = createCargoProjectController(dependencies);
  return {
    controller,
    dependencies,
    events,
    tab: active,
    getDocument: () => document,
    setDocument: (next) => { document = next; },
    setActive: (next) => { active = next; },
    setTabs: (next) => { tabs = next; },
    setBusy: (value) => { busy = value; },
    setCargoAvailable: (value) => { cargoAvailable = value; },
    getRunId: () => runningId,
  };
}

test("Cargo check refuses a workspace with another dirty Rust tab", async () => {
  const { controller, events, tab, setTabs } = fixture();
  setTabs([tab, { ...tab, id: 8, dirty: true }]);

  await controller.check();

  assert.deepEqual(events, [["status", "Salve os outros arquivos Rust alterados deste workspace antes da análise Cargo.", "error"]]);
});

test("Cargo check maps diagnostics and always clears run state", async () => {
  const { controller, dependencies, events, tab, getRunId } = fixture();
  dependencies.checkCargoProject = async () => ({
    status: "checked",
    diagnostics: [{ severity: "warning", code: "unused", message: "não usado", rendered: "aviso", start: { offset: 0, line: 1, column: 1 }, end: { offset: 1, line: 1, column: 2 } }],
    stderr: "aviso do cargo",
    reason: null,
  });

  await controller.check();

  assert.deepEqual(tab.diagnostics, [{
    severity: "warning", code: "unused", message: "não usado", help: "aviso", start: { offset: 0, line: 1, column: 1 }, end: { offset: 1, line: 1, column: 2 },
  }]);
  assert.deepEqual(events.find(([name]) => name === "console"), ["console", "aviso do cargo", undefined]);
  assert.equal(events.some(([name, message]) => name === "status" && message === "Análise concluída: 1 aviso(s) no arquivo aberto."), true);
  assert.equal(getRunId(), null);
  assert.deepEqual(events.filter(([name]) => name === "running").at(-1), ["running", false]);
});

test("Cargo tests store completed results and render the tests panel", async () => {
  const { controller, events, tab, getRunId } = fixture();

  await controller.runTests();

  assert.deepEqual(tab.cargoTestRun, {
    source: tab.state.doc,
    status: "completed",
    result: { status: "passed", passed: 2, failed: 0, ignored: 1, failures: [], reason: null },
  });
  assert.equal(events.some(([name, panel]) => name === "panel" && panel === "tests"), true);
  assert.equal(events.some(([name, message]) => name === "status" && message === "Cargo test aprovado (2 teste(s))."), true);
  assert.equal(events.filter(([name]) => name === "renderTests").length, 3);
  assert.equal(getRunId(), null);
});

test("results from an old document do not update the active Cargo tab", async () => {
  const { controller, dependencies, events, tab, setDocument } = fixture();
  dependencies.checkCargoProject = async () => {
    setDocument({ text: "mudou" });
    return { status: "checked", diagnostics: [], stderr: "tardio", reason: null };
  };

  await controller.check();

  assert.equal(tab.diagnostics, undefined);
  assert.equal(events.some(([name]) => name === "diagnostics"), false);
  assert.equal(events.some(([name, message]) => name === "status" && message === "Analisando projeto com cargo check…"), true);
});
