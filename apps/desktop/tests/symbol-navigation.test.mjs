import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { createSymbolNavigator } from "../src/editor/symbol-navigation.ts";

function fixture(language = "portugol", overrides = {}) {
  const events = [];
  let active = { id: 1, language };
  let state = EditorState.create({ doc: "funcao inicio() {}" });
  const view = {
    get state() { return state; },
    dispatch(spec) {
      events.push(["dispatch", spec]);
      state = state.update(spec).state;
    },
    focus() { events.push(["focus"]); },
  };
  const dependencies = {
    activeTab: () => active,
    view,
    resolvePortugol: async () => ({ start: { offset: 4 }, end: { offset: 11 } }),
    resolvePortugolRs: async () => ({ start: { offset: 5 }, end: { offset: 8 } }),
    setStatus: (message, kind) => events.push(["status", message, kind]),
    ...overrides,
  };
  return {
    navigator: createSymbolNavigator(dependencies), dependencies, events, view,
    setActive: (tab) => { active = tab; },
    setState: (next) => { state = next; },
  };
}

test("resolves Portugol and PORS symbols with their own service and selects the declaration", async () => {
  for (const [language, resolverName, expected] of [
    ["portugol", "resolvePortugol", { anchor: 4, head: 11 }],
    ["portugol-rs", "resolvePortugolRs", { anchor: 5, head: 8 }],
  ]) {
    const calls = [];
    const { navigator, dependencies, events, view } = fixture(language, {
      resolvePortugol: async (...args) => { calls.push(["portugol", ...args]); return { start: { offset: 4 }, end: { offset: 11 } }; },
      resolvePortugolRs: async (...args) => { calls.push(["pors", ...args]); return { start: { offset: 5 }, end: { offset: 8 } }; },
    });

    await navigator.goToDeclarationAt(2);

    assert.equal(calls.length, 1);
    assert.equal(calls[0][0], resolverName === "resolvePortugol" ? "portugol" : "pors");
    assert.deepEqual({ from: view.state.selection.main.from, to: view.state.selection.main.to }, { from: expected.anchor, to: expected.head });
    assert.deepEqual(events.map(([name]) => name), ["dispatch", "focus"]);
    assert.equal(dependencies.activeTab().language, language);
  }
});

test("Rust navigation reports that symbol navigation is unavailable", async () => {
  const { navigator, events } = fixture("rust");

  await navigator.goToDeclarationAt(0);

  assert.deepEqual(events, [["status", "Navegação de símbolos Rust ainda não está configurada.", undefined]]);
});

test("ignores a result when the active document changes while resolution is pending", async () => {
  let resolve;
  const { navigator, events, setState } = fixture("portugol", {
    resolvePortugol: () => new Promise((done) => { resolve = done; }),
  });
  const pending = navigator.goToDeclarationAt(0);
  setState(EditorState.create({ doc: "outro documento" }));
  resolve({ start: { offset: 0 }, end: { offset: 3 } });
  await pending;

  assert.deepEqual(events, []);
});

test("quiet navigation failures do not display a status message", async () => {
  const { navigator, events } = fixture("portugol-rs", {
    resolvePortugolRs: async () => { throw "símbolo não encontrado"; },
  });

  await navigator.goToDeclarationAt(0, true);

  assert.deepEqual(events, []);
});
