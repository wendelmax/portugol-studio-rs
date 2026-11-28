import test from "node:test";
import assert from "node:assert/strict";
import { createSymbolRenamer } from "../src/editor/symbol-rename.ts";

function fixture() {
  const calls = [];
  let tab = { id: 1, language: "portugol-rs" };
  let doc = { toString: () => "fonte" };
  let running = false;
  let otherDialogOpen = false;
  let targetRequest = async (...args) => { calls.push(["target", ...args]); return { name: "idade", occurrences: 2 }; };
  let renameRequest = async (...args) => { calls.push(["rename", ...args]); return [{ from: 0, to: 3, insert: "anos" }]; };
  const ui = {
    dialog: { open: false, showModal() { this.open = true; calls.push(["showDialog"]); }, close() { this.open = false; calls.push(["closeDialog"]); } },
    form: { onsubmit: null },
    input: { value: "", select: () => calls.push(["selectInput"]) },
    error: { hidden: false, textContent: "" },
    submit: { disabled: false },
    description: { textContent: "" },
    cancelButton: { onclick: null },
  };
  const renamer = createSymbolRenamer({
    activeTab: () => tab,
    currentDocument: () => doc,
    isRunning: () => running,
    isCurrent: (candidate, document) => candidate === tab && document === doc,
    hasOpenDialog: () => otherDialogOpen,
    semanticSymbolLanguage: (language) => language === "portugol-rs" ? "portugol-rs" : null,
    renameTargetPortugolRs: (source, offset) => targetRequest("portugol-rs", source, offset),
    renameSymbolPortugolRs: (source, offset, name) => renameRequest("portugol-rs", source, offset, name),
    applyChanges: (changes) => calls.push(["changes", changes]),
    focusEditor: () => calls.push(["focus"]),
    setStatus: (...args) => calls.push(["status", ...args]),
    ui,
  });
  return {
    calls, renamer, ui,
    setTab: (value) => { tab = value; },
    setDocument: (value) => { doc = value; },
    setRunning: (value) => { running = value; },
    setOtherDialogOpen: (value) => { otherDialogOpen = value; },
    setTargetRequest: (fn) => { targetRequest = fn; },
    setRenameRequest: (fn) => { renameRequest = fn; },
  };
}

test("ignores missing, Rust, or currently running tabs", async () => {
  const f = fixture();
  f.setTab(undefined);
  await f.renamer.openAt(0);
  f.setTab({ id: 2, language: "rust" });
  await f.renamer.openAt(0);
  f.setTab({ id: 1, language: "portugol-rs" });
  f.setRunning(true);
  await f.renamer.openAt(0);
  assert.deepEqual(f.calls, []);
});

test("routes target lookup through the Ferro service and displays the target", async () => {
  const f = fixture();
  f.setTab({ id: 1, language: "portugol-rs" });
  await f.renamer.openAt(4);
  assert.deepEqual(f.calls[0], ["target", "portugol-rs", "fonte", 4]);
  assert.equal(f.ui.description.textContent, "idade · 2 ocorrências");
  assert.equal(f.ui.input.value, "idade");
  assert.equal(f.ui.dialog.open, true);
});

test("ignores stale target responses and another open dialog", async () => {
  const f = fixture();
  let resolveTarget;
  f.setTargetRequest(() => new Promise((resolve) => { resolveTarget = resolve; }));
  const pending = f.renamer.openAt(3);
  f.setDocument({ toString: () => "new source" });
  resolveTarget({ name: "old", occurrences: 1 });
  await pending;
  assert.equal(f.ui.dialog.open, false);

  f.setTargetRequest(async () => ({ name: "idade", occurrences: 2 }));
  f.setOtherDialogOpen(true);
  await f.renamer.openAt(3);
  assert.equal(f.ui.dialog.open, false);
});

test("submits trimmed names, applies changes, closes the dialog, and focuses the editor", async () => {
  const f = fixture();
  await f.renamer.openAt(3);
  f.ui.input.value = "  anos  ";
  let prevented = false;
  await f.ui.form.onsubmit({ preventDefault: () => { prevented = true; } });
  assert.equal(prevented, true);
  assert.deepEqual(f.calls.find(([name]) => name === "rename"), ["rename", "portugol-rs", "fonte", 3, "anos"]);
  assert.deepEqual(f.calls.find(([name]) => name === "changes"), ["changes", [{ from: 0, to: 3, insert: "anos" }]]);
  assert.equal(f.ui.dialog.open, false);
  assert.equal(f.ui.submit.disabled, false);
  assert.ok(f.calls.some(([name]) => name === "focus"));
});

test("shows lookup and submission errors without leaking non-string values", async () => {
  const f = fixture();
  f.setTargetRequest(async () => { throw "lookup failed"; });
  await f.renamer.openAt(0);
  assert.deepEqual(f.calls.at(-1), ["status", "lookup failed", "error"]);

  f.setTargetRequest(async () => ({ name: "idade", occurrences: 1 }));
  await f.renamer.openAt(0);
  f.setRenameRequest(async () => { throw { unexpected: true }; });
  await f.ui.form.onsubmit({ preventDefault() {} });
  assert.equal(f.ui.error.textContent, "Não foi possível renomear o símbolo.");
  assert.equal(f.ui.error.hidden, false);
  assert.equal(f.ui.submit.disabled, false);
});
