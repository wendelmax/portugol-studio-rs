import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { createTabFactory, isTabDirty, tabName } from "../src/state/tabs.ts";

test("tab factory assigns unique ids, detects language, and activates each created tab", () => {
  const tabs = [];
  const activations = [];
  const factory = createTabFactory({
    extensions: () => [],
    addTab: (tab) => tabs.push(tab),
    activate: (id) => activations.push(id),
  });

  const first = factory.create({ text: "funcao inicio() {}", path: "aula.pors" });
  const second = factory.create({ text: "fn main() {}", path: "main.rs" });

  assert.deepEqual([first.id, second.id], [1, 2]);
  assert.equal(first.language, "portugol-rs");
  assert.equal(second.language, "rust");
  assert.deepEqual(tabs, [first, second]);
  assert.deepEqual(activations, [1, 2]);
  assert.ok(first.state instanceof EditorState);
  assert.equal(first.state.doc.toString(), "funcao inicio() {}");
});

test("tab factory keeps workspace, exercise, and explicit language metadata", () => {
  const factory = createTabFactory({ extensions: () => [], addTab() {}, activate() {} });
  const exercise = { id: "EX-1" };
  const tab = factory.create({
    text: "fn main() {}", path: null, exampleName: "exemplo.rs", exercise,
    explicitLanguage: "rust", workspaceFile: { id: "workspace", path: "src/main.rs", preferenceKey: "ws" },
  });

  assert.equal(tab.language, "rust");
  assert.equal(tab.exampleName, "exemplo.rs");
  assert.equal(tab.exercise, exercise);
  assert.equal(tab.workspaceId, "workspace");
  assert.equal(tab.workspacePath, "src/main.rs");
  assert.equal(tab.workspacePreferenceKey, "ws");
  assert.deepEqual(tab.diagnostics, []);
  assert.deepEqual(tab.symbols, []);
});

test("tab utilities derive names and dirty state from the active buffer", () => {
  const tab = { id: 1, path: "src/aula.por", saved: "salvo", state: { doc: { toString: () => "rascunho" } } };

  assert.equal(tabName(tab), "aula.por");
  assert.equal(tabName({ id: 2, path: null, exampleName: "Exemplo" }), "Exemplo");
  assert.equal(tabName({ id: 3, path: null }), "Sem título");
  assert.equal(isTabDirty(tab, 1, "editado"), true);
  assert.equal(isTabDirty(tab, 1, "salvo"), false);
  assert.equal(isTabDirty(tab, 2, "ignorado"), true);
});

test("IDE delegates tab creation and metadata helpers to the state module", async () => {
  const { readFile } = await import("node:fs/promises");
  const [main, shell] = await Promise.all([
    readFile(new URL("../src/main.ts", import.meta.url), "utf8"),
    readFile(new URL("../src/state/tabs-shell.ts", import.meta.url), "utf8"),
  ]);
  assert.match(main, /createTabsShell\(/);
  assert.match(shell, /createTabFactory\(/);
  assert.match(shell, /tabFactory\.create\(/);
  assert.match(shell, /isTabDirty\(/);
});
