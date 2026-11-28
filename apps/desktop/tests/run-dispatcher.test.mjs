import test from "node:test";
import assert from "node:assert/strict";
import { createRunDispatcher } from "../src/run/dispatcher.ts";

function fixture() {
  const calls = [];
  const document = { id: "document" };
  const editorState = { id: "editor-state" };
  let tab = { language: "portugol-rs" };
  let busy = false;
  let paused = false;
  const dispatcher = createRunDispatcher({
    isBusy: () => busy,
    activeTab: () => tab,
    currentDocument: () => document,
    editorState: () => editorState,
    argumentsText: () => "arg1 arg2",
    breakpointLines: (state) => { calls.push(["breakpoints", state]); return [3, 8]; },
    rustExecution: { run: async (pause) => calls.push(["rust", pause]) },
    portugolRsExecution: { run: async (...args) => calls.push(["pors", ...args]) },
    isPaused: () => paused,
    debugContinue: async () => calls.push(["continue"]),
    debugStepOver: async () => calls.push(["step"]),
    stop: async () => calls.push(["stop"]),
    screenExecution: {
      handles: (current) => current.screen === true,
      run: async (current) => calls.push(["tela", current.language]),
    },
  });
  return {
    calls, document, editorState, dispatcher,
    setTab: (value) => { tab = value; },
    setBusy: (value) => { busy = value; },
    setPaused: (value) => { paused = value; },
  };
}

test("busy and no-tab guards prevent starting a program", async () => {
  const f = fixture();
  f.setBusy(true);
  await f.dispatcher.runProgram();
  f.setBusy(false);
  f.setTab(undefined);
  await f.dispatcher.runProgram();
  assert.deepEqual(f.calls, []);
});

test("dispatches Rust and PORS programs with their existing controller arguments", async () => {
  const f = fixture();
  f.setTab({ language: "rust" });
  await f.dispatcher.runProgram(true, false);
  f.setTab({ language: "portugol-rs" });
  await f.dispatcher.runProgram(true, true);
  assert.deepEqual(f.calls, [
    ["rust", true],
    ["breakpoints", f.editorState], ["pors", { language: "portugol-rs" }, f.document, [3, 8], true, true],
  ]);
});

test("continues or steps when paused, otherwise starts execution", async () => {
  const f = fixture();
  await f.dispatcher.runOrContinue();
  await f.dispatcher.stepOrStart();
  f.setPaused(true);
  await f.dispatcher.runOrContinue();
  await f.dispatcher.stepOrStart();
  assert.deepEqual(f.calls, [
    ["breakpoints", f.editorState], ["pors", { language: "portugol-rs" }, f.document, [3, 8], false, false],
    ["breakpoints", f.editorState], ["pors", { language: "portugol-rs" }, f.document, [3, 8], true, false],
    ["continue"], ["step"],
  ]);
});

test("delegates stop to the existing debugger controller", async () => {
  const f = fixture();
  await f.dispatcher.stopProgram();
  assert.deepEqual(f.calls, [["stop"]]);
});

test("markup tabs never start a PORS or Rust run", async () => {
  const f = fixture();
  f.setTab({ language: "markup" });
  await f.dispatcher.runProgram();
  await f.dispatcher.runProgram(true);
  assert.deepEqual(f.calls, []);
});

test("telas rodam numa janela própria, a partir do .design ou do código", async () => {
  const f = fixture();
  f.setTab({ language: "markup", screen: true });
  await f.dispatcher.runProgram();
  f.setTab({ language: "portugol-rs", screen: true });
  await f.dispatcher.runProgram();
  assert.deepEqual(f.calls, [["tela", "markup"], ["tela", "portugol-rs"]]);
});
