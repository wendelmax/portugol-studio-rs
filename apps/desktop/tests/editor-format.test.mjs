import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { createEditorFormatter } from "../src/editor/format.ts";

function fixture(language = "portugol", overrides = {}) {
  const events = [];
  let active = { id: 1, language };
  let state = EditorState.create({ doc: "codigo" });
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
    isRunningTab: () => false,
    rustfmtAvailable: () => true,
    view,
    formatRust: async () => "formatado",
    formatPortugolRs: async () => [{ from: 0, to: 6, insert: "formatado" }],
    formatPortugol: async () => [{ from: 0, to: 6, insert: "formatado" }],
    setStatus: (message, kind) => events.push(["status", message, kind]),
    ...overrides,
  };
  return {
    formatter: createEditorFormatter(dependencies), dependencies, events, view,
    setActive: (tab) => { active = tab; },
    setState: (next) => { state = next; },
  };
}

test("uses the formatter for each source language and updates the editor", async () => {
  for (const [language, expected] of [["portugol", "classic"], ["portugol-rs", "pors"], ["rust", "rust"]]) {
    const calls = [];
    const { formatter, dependencies, view, events } = fixture(language, {
      formatRust: async (source) => { calls.push(["rust", source]); return "formatado"; },
      formatPortugolRs: async (source) => { calls.push(["pors", source]); return [{ from: 0, to: 6, insert: "formatado" }]; },
      formatPortugol: async (source) => { calls.push(["classic", source]); return [{ from: 0, to: 6, insert: "formatado" }]; },
    });

    await formatter.formatCode();

    assert.deepEqual(calls, [[expected, "codigo"]]);
    assert.equal(view.state.doc.toString(), "formatado");
    assert.deepEqual(events.map(([name]) => name), ["dispatch", "status", "focus"]);
    assert.equal(dependencies.activeTab().language, language);
  }
});

test("reports when the source is already formatted without dispatching an edit", async () => {
  const { formatter, events } = fixture("rust", { formatRust: async () => "codigo" });

  await formatter.formatCode();

  assert.deepEqual(events, [["status", "O código já está formatado.", "ok"], ["focus"]]);
});

test("does not format a running tab and explains when rustfmt is unavailable", async () => {
  const running = fixture("portugol", { isRunningTab: () => true });
  await running.formatter.formatCode();
  assert.deepEqual(running.events, []);

  const noRustfmt = fixture("rust", { rustfmtAvailable: () => false });
  await noRustfmt.formatter.formatCode();
  assert.deepEqual(noRustfmt.events, [["status", "Instale o componente rustfmt com `rustup component add rustfmt`.", "error"]]);
});

test("ignores formatting results after the active document changes", async () => {
  let resolve;
  const { formatter, events, setState } = fixture("portugol", {
    formatPortugol: () => new Promise((done) => { resolve = done; }),
  });
  const pending = formatter.formatCode();
  setState(EditorState.create({ doc: "outro documento" }));
  resolve([{ from: 0, to: 6, insert: "inadequado" }]);
  await pending;

  assert.deepEqual(events, []);
});
