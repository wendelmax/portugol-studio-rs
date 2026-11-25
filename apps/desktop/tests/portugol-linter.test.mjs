import test from "node:test";
import assert from "node:assert/strict";
import { createPortugolLintSourceCheck } from "../src/editor/portugol-linter.ts";

const position = (offset) => ({ offset, line: 1, column: offset + 1 });
const diagnostic = (code = "parser") => ({ severity: "error", code, message: code, help: null, start: position(0), end: position(1) });
function fixture() {
  const calls = [];
  const tab = { language: "portugol", diagnostics: [], symbols: [], runtimeCompileDiagnostics: [] };
  let active = tab;
  let inspection = async (source) => ({ diagnostics: [diagnostic(source)], symbols: [{ name: source }] });
  let porsInspection = async (source) => ({ diagnostics: [diagnostic(source)], symbols: [{ name: source }] });
  const createCheck = () => createPortugolLintSourceCheck({
    activeTab: () => active,
    inspectPortugol: (source) => inspection(source),
    inspectPortugolRs: (source) => porsInspection(source),
    renderMessages: () => calls.push("messages"),
    showOutline: (symbols) => calls.push(["outline", symbols]),
    updateLibraryButtons: (symbols) => calls.push(["libraries", symbols]),
    toCodeMirror: (item, length) => ({ item, length }),
  });
  return {
    calls, tab, createCheck,
    setActive: (next) => { active = next; },
    setInspectPortugol: (fn) => { inspection = fn; },
    setInspectPortugolRs: (fn) => { porsInspection = fn; },
  };
}
const document = (text) => ({ length: text.length, toString: () => text });
const editor = (text) => ({ state: { doc: document(text) } });

test("routes Portugol and PORS documents to their existing inspection services", async () => {
  const f = fixture();
  f.tab.language = "portugol";
  const check = f.createCheck();
  assert.equal((await check(editor("classic")))[0].item.code, "classic");
  f.tab.language = "portugol-rs";
  assert.equal((await check(editor("pors")))[0].item.code, "pors");
  assert.deepEqual(f.tab.symbols, [{ name: "pors" }]);
  assert.deepEqual(f.tab.diagnosticsAnalysis, { source: "pors", status: "complete" });
});

test("marks the current source unavailable when language inspection fails", async () => {
  const f = fixture();
  f.setInspectPortugol(async () => { throw new Error("inspector offline"); });

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
  f.setInspectPortugol((source) => new Promise((resolve) => { resolveInspection = () => resolve({ diagnostics: [diagnostic(source)], symbols: [{ name: source }] }); }));
  const view = editor("old");
  const pending = f.createCheck()(view);
  view.state.doc = document("new");
  resolveInspection();
  const result = await pending;
  assert.deepEqual(f.calls, []);
  assert.deepEqual(f.tab.diagnostics, []);
  assert.deepEqual(result.map(({ item }) => item.code), ["old"]);
});
