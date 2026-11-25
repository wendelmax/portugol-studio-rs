import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { EditorState, Compartment } from "@codemirror/state";
import { createEditorExtensionFactory } from "../src/editor/extensions.ts";

function fixture() {
  const calls = [];
  const marker = (name) => {
    calls.push(name);
    return EditorState.tabSize.of(name === "rust" ? 8 : name === "pors" ? 2 : 4);
  };
  const dependencies = {
    sourceLanguage: new Compartment(),
    portugolTools: new Compartment(),
    theme: new Compartment(),
    portugolExtensions: () => [marker("portugolTools")],
    rustExtensions: () => [marker("rustTools")],
    portugolLanguage: () => marker("portugol"),
    portugolRsLanguage: () => marker("pors"),
    rustLanguage: () => marker("rust"),
    editorColors: EditorState.tabSize.of(4),
    editorTheme: () => EditorState.tabSize.of(4),
    findDialog: EditorState.tabSize.of(4),
    editorSearchKeys: [],
    onEditorClick: () => false,
    onGoToDeclaration: () => true,
    onUpdate: () => {},
  };
  return { createExtensions: createEditorExtensionFactory(dependencies), calls };
}

test("selects language and tooling extensions for classic Portugol, PORS, and Rust", () => {
  const { createExtensions, calls } = fixture();

  const classic = createExtensions("portugol");
  const pors = createExtensions("portugol-rs");
  const rust = createExtensions("rust");

  assert.ok(classic.length > 10);
  assert.ok(pors.length > 10);
  assert.ok(rust.length > 10);
  assert.deepEqual(calls, ["portugolTools", "portugol", "portugolTools", "pors", "rustTools", "rust"]);
});

test("routes mouse, F12, and editor updates through injected shell handlers", async () => {
  const module = await readFile(new URL("../src/editor/extensions.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(module, /EditorView\.domEventHandlers\([\s\S]*dependencies\.onEditorClick/);
  assert.match(module, /key: "F12", run: dependencies\.onGoToDeclaration/);
  assert.match(module, /EditorView\.updateListener\.of\(dependencies\.onUpdate\)/);
  assert.match(main, /createEditorExtensionFactory\(/);
  assert.match(main, /onEditorClick: editorNavigation\.onEditorClick/);
  assert.match(main, /onGoToDeclaration: editorNavigation\.onGoToDeclaration/);
  assert.match(main, /onUpdate: editorUpdateHandler/);
});
