import test from "node:test";
import assert from "node:assert/strict";
import { createRustExecutionController } from "../src/run/rust-execution.ts";

function fixture(overrides = {}) {
  const events = [];
  let document = { toString: () => "fn main() {}" };
  let tab = {
    id: 4, language: "rust", workspaceId: null, workspacePath: null,
    cargoProject: false, path: "main.rs", diagnostics: [],
  };
  let tabs = [tab];
  let runningId = null;
  let pending = false;
  let rustcAvailable = true;
  let cargoAvailable = true;
  const dependencies = {
    activeTab: () => tab,
    tabs: () => tabs,
    isBusy: () => runningId !== null || pending,
    rustcAvailable: () => rustcAvailable,
    cargoAvailable: () => cargoAvailable,
    isDirty: (candidate) => candidate.dirty === true,
    save: async () => true,
    argumentsText: () => "",
    parseArguments: () => [],
    currentDocument: () => document,
    isCurrent: (currentTab, currentDocument) => currentTab === tab && currentDocument === document,
    setRunId: (id) => { runningId = id; events.push(["runId", id]); },
    runId: () => runningId,
    setPending: (value) => { pending = value; events.push(["pending", value]); },
    setRunTabId: (id) => events.push(["runTab", id]),
    syncLanguageControls: (currentTab) => events.push(["sync", currentTab.id]),
    setRunning: (value) => events.push(["running", value]),
    hasConsent: () => true,
    askConsent: async () => true,
    rememberConsent: () => events.push(["remember"]),
    chooseCargoTarget: async () => null,
    registerRun: async () => 12,
    cargoRunTargets: async () => ({ targets: [], defaultTarget: null }),
    runCargoProject: async () => ({ status: "executed", stdout: "", stderr: "", exitCode: 0, phase: "runtime", reason: null }),
    runRustSource: async () => ({ status: "executed", stdout: "olá\n", stderr: "", exitCode: 0 }),
    clearRuntimeError: () => events.push(["clearRuntime"]),
    clearDebugLine: () => events.push(["clearDebug"]),
    setPaused: (value) => events.push(["paused", value]),
    clearPaused: () => events.push(["paused", null]),
    showConsole: () => events.push(["showConsole"]),
    clearConsole: () => events.push(["clearConsole"]),
    appendProcessOutput: (result) => events.push(["output", result.stdout]),
    appendConsole: (...args) => events.push(["console", ...args]),
    setStatus: (...args) => events.push(["status", ...args]),
    renderMessages: () => events.push(["renderMessages"]),
    setDiagnostics: (currentTab, diagnostics) => { currentTab.runtimeCompileDiagnostics = diagnostics; events.push(["diagnostics", diagnostics]); },
    toCodeMirror: (diagnostic) => diagnostic,
    runListeners: Promise.resolve(),
    hideInput: () => events.push(["hideInput"]),
    ...overrides.dependencies,
  };
  const controller = createRustExecutionController(dependencies);
  return {
    controller, dependencies, events,
    getTab: () => tab,
    getDocument: () => document,
    setTab: (next) => { tab = next; },
    setTabs: (next) => { tabs = next; },
    setDocument: (next) => { document = next; },
    setRustcAvailable: (value) => { rustcAvailable = value; },
    setCargoAvailable: (value) => { cargoAvailable = value; },
    getRunId: () => runningId,
  };
}

test("Rust execution requires the Rust toolchain", async () => {
  const { controller, events, setRustcAvailable } = fixture();
  setRustcAvailable(false);

  await controller.run();

  assert.equal(events.some(([name]) => name === "runId"), false);
  assert.equal(events.some(([name, message]) => name === "status" && message === "Instale a toolchain Rust para executar arquivos .rs."), true);
});

test("standalone Rust execution registers the run, reports output, and cleans up", async () => {
  const { controller, events, getRunId } = fixture();

  await controller.run();

  assert.equal(events.some(([name, value]) => name === "output" && value === "olá\n"), true, JSON.stringify(events));
  assert.equal(events.some(([name, message]) => name === "status" && message === "Programa Rust finalizado com rustc."), true);
  assert.equal(getRunId(), null);
  assert.deepEqual(events.filter(([name]) => name === "running").at(-1), ["running", false]);
});

test("Rust results are ignored after the source document changes", async () => {
  const { controller, dependencies, events, setDocument, getRunId } = fixture();
  dependencies.runRustSource = async () => {
    setDocument({ toString: () => "fn main() { println!(\"mudou\"); }" });
    return { status: "executed", stdout: "antigo", stderr: "", exitCode: 0 };
  };

  await controller.run();

  assert.equal(events.some(([name]) => name === "output"), false);
  assert.equal(getRunId(), null);
});

test("Cargo execution refuses to run while another Rust file is dirty", async () => {
  const { controller, events, getTab, setTab, setTabs } = fixture();
  const tab = { ...getTab(), workspaceId: "workspace", workspacePath: "src/main.rs", cargoProject: true };
  setTab(tab);
  setTabs([tab, { ...tab, id: 5, dirty: true }]);

  await controller.run();

  assert.equal(events.some(([name]) => name === "runId"), false);
  assert.equal(events.some(([name, message]) => name === "status" && message === "Salve os outros arquivos alterados deste projeto antes de executar Cargo."), true);
});

test("Cargo execution uses the selected workspace target and reports the process result", async () => {
  const { controller, dependencies, events, getTab, setTab } = fixture();
  const target = { packageName: "exemplo", targetName: "exemplo" };
  setTab({ ...getTab(), workspaceId: "workspace", workspacePath: "src/main.rs", cargoProject: true });
  dependencies.cargoRunTargets = async () => ({ targets: [target], defaultTarget: target });
  dependencies.runCargoProject = async (_id, workspaceId, workspacePath, selected, args) => {
    events.push(["cargoRun", workspaceId, workspacePath, selected, args]);
    return { status: "executed", stdout: "ok\n", stderr: "", exitCode: 0, phase: "runtime", reason: null };
  };

  await controller.run();

  assert.deepEqual(events.find(([name]) => name === "cargoRun"), ["cargoRun", "workspace", "src/main.rs", target, []]);
  assert.equal(events.some(([name, message]) => name === "status" && message === "Projeto Cargo finalizado."), true);
});
