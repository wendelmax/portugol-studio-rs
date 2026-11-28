import test from "node:test";
import assert from "node:assert/strict";

import { createEditorNavigationBindings } from "../src/editor/navigation-bindings.ts";

function fixture(language = "ferro", offset = 42, cursor = 84) {
  const calls = [];
  const bindings = createEditorNavigationBindings({
    activeLanguage: () => language,
    selectionHead: () => cursor,
    goToDeclarationAt: async (...args) => { calls.push(args); },
  });
  const editor = { posAtCoords: () => offset };
  return { bindings, calls, editor };
}

test("primary Ctrl/Cmd clicks navigate quietly to the clicked source position", () => {
  for (const modifier of ["ctrlKey", "metaKey"]) {
    const { bindings, calls, editor } = fixture();
    const event = { button: 0, clientX: 8, clientY: 9, [modifier]: true };

    assert.equal(bindings.onEditorClick(event, editor), false);
    assert.deepEqual(calls, [[42, true]]);
  }
});

test("editor clicks ignore Rust, non-primary buttons, missing modifiers, and absent positions", () => {
  const rust = fixture("rust");
  const ferro = fixture();
  const absent = fixture("ferro", null);

  assert.equal(rust.bindings.onEditorClick({ button: 0, ctrlKey: true }, rust.editor), false);
  assert.equal(ferro.bindings.onEditorClick({ button: 1, ctrlKey: true }, ferro.editor), false);
  assert.equal(ferro.bindings.onEditorClick({ button: 0 }, ferro.editor), false);
  assert.equal(absent.bindings.onEditorClick({ button: 0, metaKey: true }, absent.editor), false);
  assert.deepEqual(rust.calls, []);
  assert.deepEqual(ferro.calls, []);
  assert.deepEqual(absent.calls, []);
});

test("F12 navigates from the current selection head", () => {
  const { bindings, calls } = fixture();

  assert.equal(bindings.onGoToDeclaration(), true);
  assert.deepEqual(calls, [[84]]);
});
