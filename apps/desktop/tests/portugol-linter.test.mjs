import test from "node:test";
import assert from "node:assert/strict";
import { createPortugolLintSourceCheck } from "../src/editor/portugol-linter.ts";

const position = (offset) => ({ offset, line: 1, column: offset + 1 });
const diagnostic = (code = "parser") => ({ severity: "error", code, message: code, help: null, start: position(0), end: position(1) });
function fixture() {
  const calls = [];
  const tab = { language: "portugol-rs", diagnostics: [], symbols: [], runtimeCompileDiagnostics: [] };
  let active = tab;
  let porsInspection = async (source) => ({ diagnostics: [diagnostic(source)], symbols: [{ name: source }] });
  const createCheck = () => createPortugolLintSourceCheck({
    activeTab: () => active,
    inspectPortugolRs: (source) => porsInspection(source),
    renderMessages: () => calls.push("messages"),
    showOutline: (symbols) => calls.push(["outline", symbols]),
    updateLibraryButtons: (symbols) => calls.push(["libraries", symbols]),
    toCodeMirror: (item, length) => ({ item, length }),
  });
  return {
    calls, tab, createCheck,
    setActive: (next) => { active = next; },
    setInspectPortugolRs: (fn) => { porsInspection = fn; },
  };
}
const document = (text) => ({ length: text.length, toString: () => text });
const editor = (text) => ({ state: { doc: document(text) } });

test("routes PORS documents to the PORS inspection service", async () => {
  const f = fixture();
  const check = f.createCheck();
  assert.equal((await check(editor("pors")))[0].item.code, "pors");
  assert.deepEqual(f.tab.symbols, [{ name: "pors" }]);
  assert.deepEqual(f.tab.diagnosticsAnalysis, { source: "pors", status: "complete" });
});

test("marks the current source unavailable when language inspection fails", async () => {
  const f = fixture();
  f.setInspectPortugolRs(async () => { throw new Error("inspector offline"); });

  assert.deepEqual(await f.createCheck()(editor("fonte")), []);
  assert.deepEqual(f.tab.diagnosticsAnalysis, { source: "fonte", status: "unavailable" });
  assert.ok(f.calls.includes("messages"));
});

test("clears diagnostics, symbols, outline, and library controls for Rust tabs", async () => {
  const f = fixture();
  f.tab.language = "rust";
  f.tab.diagnostics = [diagnostic()];
  f.tab.symbols = [{ name: "old" }];
  assert.deepEqual(await f.createCheck()(editor("fn main() {}")), []);
  assert.deepEqual(f.tab.diagnostics, []);
  assert.deepEqual(f.tab.symbols, []);
  assert.deepEqual(f.calls, ["messages", ["outline", []], ["libraries", []]]);
});

test("aggregates runtime diagnostics and converts against the document length", async () => {
  const f = fixture();
  f.tab.runtimeCompileDiagnostics = [diagnostic("runtime")];
  const result = await f.createCheck()(editor("abcdef"));
  assert.deepEqual(result.map(({ item }) => item.code), ["abcdef", "runtime"]);
  assert.deepEqual(result.map(({ length }) => length), [6, 6]);
});

test("does not update UI for an inspection result from a replaced document", async () => {
  const f = fixture();
  let resolveInspection;
  f.setInspectPortugolRs((source) => new Promise((resolve) => { resolveInspection = () => resolve({ diagnostics: [diagnostic(source)], symbols: [{ name: source }] }); }));
  const view = editor("old");
  const pending = f.createCheck()(view);
  view.state.doc = document("new");
  resolveInspection();
  const result = await pending;
  assert.deepEqual(f.calls, []);
  assert.deepEqual(f.tab.diagnostics, []);
  assert.deepEqual(result.map(({ item }) => item.code), ["old"]);
});

test("a markup linter inspects only markup tabs and receives the tab for its path", async () => {
  const markupTab = { language: "markup", workspacePath: "App/JanelaPrincipal.design", diagnostics: [], symbols: [], runtimeCompileDiagnostics: [] };
  const porsTab = { language: "portugol-rs", diagnostics: [], symbols: [], runtimeCompileDiagnostics: [] };
  let active = markupTab;
  const seen = [];
  const check = createPortugolLintSourceCheck({
    activeTab: () => active,
    language: "markup",
    inspectPortugolRs: async (source, tab) => {
      seen.push([source, tab.workspacePath]);
      return { diagnostics: [diagnostic("design.unknown-element")], symbols: [] };
    },
    renderMessages: () => {},
    showOutline: () => {},
    updateLibraryButtons: () => {},
    toCodeMirror: (item, length) => ({ item, length }),
  });

  assert.equal((await check(editor("<Janela />")))[0].item.code, "design.unknown-element");
  assert.deepEqual(seen, [["<Janela />", "App/JanelaPrincipal.design"]]);

  active = porsTab;
  assert.deepEqual(await check(editor("funcao inicio() {}")), []);
  assert.equal(seen.length, 1);
});

test("the PORS linter leaves markup tabs alone", async () => {
  const f = fixture();
  const markupTab = { language: "markup", diagnostics: [], symbols: [], runtimeCompileDiagnostics: [] };
  f.setActive(markupTab);
  let called = false;
  f.setInspectPortugolRs(async () => { called = true; return { diagnostics: [], symbols: [] }; });
  assert.deepEqual(await f.createCheck()(editor("<Janela />")), []);
  assert.equal(called, false);
});
