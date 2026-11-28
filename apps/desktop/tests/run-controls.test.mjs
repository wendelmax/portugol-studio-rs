import test from "node:test";
import assert from "node:assert/strict";
import { createRunControls } from "../src/run/controls.ts";

function fixture(overrides = {}) {
  const state = { disabled: {}, inputHidden: false, debugVarsHidden: false, inspectorEmptyHidden: true, inspectorText: "" };
  const dependencies = {
    activeLanguage: () => "ferro-rust",
    canRun: () => true,
    canDebug: () => true,
    rustcAvailable: () => true,
    runId: () => 7,
    paused: () => null,
    setButtonDisabled: (id, disabled) => { state.disabled[id] = disabled; },
    setInputHidden: (hidden) => { state.inputHidden = hidden; },
    setDebugVarsHidden: (hidden) => { state.debugVarsHidden = hidden; },
    setInspectorEmptyHidden: (hidden) => { state.inspectorEmptyHidden = hidden; },
    setInspectorText: (text) => { state.inspectorText = text; },
    ...overrides,
  };
  return { controls: createRunControls(dependencies), state };
}

test("run controls lock run actions while execution is active", () => {
  const { controls, state } = fixture();

  controls.setRunning(true);

  assert.equal(state.disabled["btn-run"], true);
  assert.equal(state.disabled["btn-debug"], true);
  assert.equal(state.disabled["btn-stop"], false);
  assert.equal(state.disabled["btn-debug-pause"], false);
  assert.equal(state.disabled["btn-debug-continue"], true);
  assert.equal(state.inputHidden, false);
});

test("a paused run exposes debugger actions and displays variables", () => {
  const paused = { variables: [{ name: "idade" }] };
  const { controls, state } = fixture({ paused: () => paused });

  controls.setRunning(true);

  assert.equal(state.disabled["btn-run"], false);
  assert.equal(state.disabled["btn-debug-pause"], true);
  assert.equal(state.disabled["btn-debug-continue"], false);
  assert.equal(state.debugVarsHidden, false);
  assert.equal(state.inspectorEmptyHidden, true);
  assert.equal(state.inspectorText, "Nenhuma variável neste ponto.");
});

test("stopped run hides input and explains that the inspector needs a pause", () => {
  const { controls, state } = fixture({ runId: () => null });

  controls.setRunning(false);

  assert.equal(state.disabled["btn-stop"], true);
  assert.equal(state.inputHidden, true);
  assert.equal(state.debugVarsHidden, true);
  assert.equal(state.inspectorEmptyHidden, false);
  assert.equal(state.inspectorText, "Pause a execução para inspecionar as variáveis.");
});

test("native rustc execution disables debug pause even on a ferro-rust tab", () => {
  const { controls, state } = fixture({ canPause: () => false });

  controls.setRunning(true);

  assert.equal(state.disabled["btn-stop"], false);
  assert.equal(state.disabled["btn-debug-pause"], true);
});
