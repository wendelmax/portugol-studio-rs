import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import {
  DEFAULT_EDITOR_PREFERENCES,
  EDITOR_TAB_SIZE_KEY,
  EDITOR_WRAP_KEY,
  editorPreferenceExtensions,
  loadEditorPreferences,
  normalizeTabSize,
  saveEditorPreferences,
} from "../src/editor/preferences.ts";

function memoryStorage(initial = {}) {
  const map = new Map(Object.entries(initial));
  return {
    getItem: (key) => map.get(key) ?? null,
    setItem: (key, value) => { map.set(key, String(value)); },
  };
}

test("editor preferences default safely and normalize invalid tab sizes", () => {
  assert.deepEqual(loadEditorPreferences(memoryStorage()), DEFAULT_EDITOR_PREFERENCES);
  assert.equal(normalizeTabSize("2"), 2);
  assert.equal(normalizeTabSize(8), 8);
  assert.equal(normalizeTabSize("3"), 4);
  assert.equal(normalizeTabSize("abc"), 4);
});

test("editor preferences persist and reload word wrap and tab size", () => {
  const storage = memoryStorage();
  saveEditorPreferences(storage, { wordWrap: true, tabSize: 2 });

  assert.equal(storage.getItem(EDITOR_WRAP_KEY), "true");
  assert.equal(storage.getItem(EDITOR_TAB_SIZE_KEY), "2");
  assert.deepEqual(loadEditorPreferences(storage), { wordWrap: true, tabSize: 2 });
});

test("editorPreferenceExtensions applies tabSize facet and optional lineWrapping", () => {
  const compact = EditorState.create({
    doc: "funcao inicio() {}",
    extensions: editorPreferenceExtensions({ wordWrap: false, tabSize: 2 }),
  });
  assert.equal(compact.facet(EditorState.tabSize), 2);

  const wrapped = editorPreferenceExtensions({ wordWrap: true, tabSize: 8 });
  const unwrapped = editorPreferenceExtensions({ wordWrap: false, tabSize: 8 });
  assert.equal(wrapped.length, unwrapped.length + 1);
});
