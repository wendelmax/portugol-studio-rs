import test from "node:test";
import assert from "node:assert/strict";
import { createRustLintSourceCheck } from "../src/editor/rust-linter.ts";

const position = (offset) => ({ offset, line: 1, column: offset + 1 });
const diagnostic = (overrides = {}) => ({ severity: "error", code: "E0382", message: "use of moved value", rendered: "value was moved here", start: position(2), end: position(3), ...overrides });
function fixture() {
  const calls = [];
  let selected = { language: "rust", cargoProject: false, diagnostics: [], runtimeCompileDiagnostics: [] };
  let available = true;
  let checkSource = async (source) => ({ source, diagnostics: [diagnostic()] });
  const createCheck = () => createRustLintSourceCheck({ activeTab: () => selected, isRustcAvailable: () => available, checkSource, renderMessages: () => calls.push("render"), toCodeMirror: (info, length) => ({ info, length }) });
  return { calls, createCheck, setSelected: (tab) => { selected = tab; }, selected: () => selected, setAvailable: (value) => { available = value; }, setCheckSource: (check) => { checkSource = check; } };
}
const document = (text) => ({ length: text.length, toString: () => text });
const editor = (text) => ({ state: { doc: document(text) } });

test("clears Rust diagnostics when rustc is unavailable or a Cargo project owns checks", async () => {
  const f = fixture();
  const check = f.createCheck();
  f.setAvailable(false);
  assert.deepEqual(await check(editor("fn main() {}")), []);
  assert.deepEqual(f.selected().diagnostics, []);
  assert.deepEqual(f.calls, ["render"]);
  f.setAvailable(true);
  f.setSelected({ ...f.selected(), diagnostics: [diagnostic()], cargoProject: true });
  assert.deepEqual(await check(editor("fn main() {}")), []);
  assert.deepEqual(f.selected().diagnostics, []);
  assert.deepEqual(f.calls, ["render", "render"]);
});

test("ignores compiler responses for a replaced tab or document", async () => {
  const f = fixture();
  let resolveCheck;
  f.setCheckSource((source) => new Promise((resolve) => { resolveCheck = () => resolve({ source, diagnostics: [diagnostic()] }); }));
  const view = editor("old");
  const pending = f.createCheck()(view);
  view.state.doc = document("new");
  resolveCheck();
  assert.deepEqual(await pending, []);
  assert.deepEqual(f.calls, []);

  f.setCheckSource((source) => new Promise((resolve) => { resolveCheck = () => resolve({ source, diagnostics: [diagnostic()] }); }));
  const staleTabResult = f.createCheck()(editor("old tab"));
  f.setSelected({ language: "rust", cargoProject: false, diagnostics: [], runtimeCompileDiagnostics: [] });
  resolveCheck();
  assert.deepEqual(await staleTabResult, []);
  assert.deepEqual(f.calls, []);
});

test("converts compiler errors and warnings while ignoring other severities", async () => {
  const f = fixture();
  f.setCheckSource(async (source) => ({ source, diagnostics: [diagnostic(), diagnostic({ severity: "warning", code: null, rendered: null }), diagnostic({ severity: "note" })] }));
  const result = await f.createCheck()(editor("abcd"));
  assert.equal(result.length, 2);
  assert.equal(result[0].info.code, "E0382");
  assert.equal(result[1].info.code, "rustc");
  assert.equal(result[0].info.help, "value was moved here");
  assert.equal(result[0].length, 4);
  assert.equal(f.selected().diagnostics.length, 2);
  assert.deepEqual(f.calls, ["render"]);
});

test("turns source-check failures into visible warnings and preserves runtime diagnostics", async () => {
  const f = fixture();
  const runtime = { severity: "error", code: "runtime", message: "runtime failure", help: null, start: position(0), end: position(0) };
  f.setSelected({ ...f.selected(), runtimeCompileDiagnostics: [runtime] });
  f.setCheckSource(async (source) => ({ source, diagnostics: [], error: "rustc failed" }));
  const result = await f.createCheck()(editor("fn main() {}"));
  assert.equal(result.length, 2);
  assert.equal(result[0].info.severity, "warning");
  assert.equal(result[0].info.code, "rustc.check");
  assert.match(result[0].info.message, /rustc failed/);
  assert.equal(result[1].info.code, "runtime");
});
