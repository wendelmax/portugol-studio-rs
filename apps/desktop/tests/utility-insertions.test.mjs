import test from "node:test";
import assert from "node:assert/strict";
import { createUtilityInsertions } from "../src/editor/utility-insertions.ts";

function fakeButton() {
  return {
    textContent: "",
    title: "",
    disabled: false,
    attributes: {},
    onclick: null,
    setAttribute(name, value) { this.attributes[name] = value; },
  };
}

function fixture({ insertText } = {}) {
  const events = [];
  const libraryButtons = [];
  const snippetButtons = [];
  const tabButtons = new Map([["libraries", fakeButton()], ["models", fakeButton()]]);
  const panels = new Map([["libraries", { hidden: false }], ["models", { hidden: true }]]);
  const tab = { id: 1, language: "portugol-rs" };
  let activeTab = tab;
  let running = false;
  const snippets = [
    { label: "para", detail: "laço", apply: () => {} },
    { label: "programa", detail: "estrutura" },
  ];
  const controller = createUtilityInsertions({
    porsLibraries: ["UI", "Json", "Matematica"],
    porsSnippets: snippets,
    createButton: fakeButton,
    clearLibraryButtons: () => { libraryButtons.length = 0; },
    clearSnippetButtons: () => { snippetButtons.length = 0; },
    appendLibraryButton: (button) => libraryButtons.push(button),
    appendSnippetButton: (button) => snippetButtons.push(button),
    activeTab: () => activeTab,
    isRunning: (candidate) => running && candidate === activeTab,
    insertText,
    focusEditor: () => events.push(["focus"]),
    setStatus: (message, kind) => events.push(["status", message, kind]),
    createProgramTab: () => events.push(["newProgram"]),
    selection: () => ({ from: 4, to: 8 }),
    applySnippet: (snippet, from, to) => events.push(["snippet", snippet.label, from, to]),
    utilityTabButton: (name) => tabButtons.get(name),
    utilityPanel: (name) => panels.get(name),
  });
  controller.mount();
  return {
    controller, events, libraryButtons, snippetButtons, tabButtons, panels,
    setTab: (value) => { activeTab = value; },
    setRunning: (value) => { running = value; },
  };
}

test("module buttons insert PORS module prefix and focus editor", () => {
  const inserted = [];
  const f = fixture({ insertText: (text) => inserted.push(text) });
  f.controller.updateLibraryButtons([], "portugol-rs");

  assert.deepEqual(f.libraryButtons.map((b) => b.textContent), ["UI", "Json", "Matematica"]);
  assert.equal(f.libraryButtons[0].disabled, false);

  f.libraryButtons[1].onclick();

  assert.deepEqual(inserted, ["Json."]);
  assert.deepEqual(f.events, [
    ["focus"],
    ["status", "Módulo PORS inserido: Json. (use Ctrl+Espaço para completar).", "ok"],
  ]);
});

test("module buttons enforce running and Rust guards and fall back to status hint without insertText", () => {
  const f = fixture();
  f.setRunning(true);
  f.libraryButtons[2].onclick();
  assert.equal(f.events.length, 0);

  f.setRunning(false);
  f.setTab({ id: 2, language: "rust" });
  f.controller.updateLibraryButtons([], "rust");
  assert.equal(f.libraryButtons[0].disabled, true);
  f.libraryButtons[2].onclick();
  assert.equal(f.events.length, 0);

  f.setTab({ id: 1, language: "portugol-rs" });
  f.controller.updateLibraryButtons([], "portugol-rs");
  f.libraryButtons[2].onclick();
  assert.deepEqual(f.events, [["status", "Use Matematica.funcao(...) diretamente em Ferro.", undefined]]);
});

test("snippet buttons retain language guards, insertion, and new-program action", () => {
  const f = fixture();
  const [loopButton, programButton] = f.snippetButtons;

  f.setTab({ id: 2, language: "rust" });
  loopButton.onclick();
  assert.deepEqual(f.events, []);

  f.setTab({ id: 1, language: "portugol-rs" });
  loopButton.onclick();
  programButton.onclick();

  assert.deepEqual(f.events, [["snippet", "para", 4, 8], ["focus"], ["newProgram"]]);
});

test("utility tab selection updates the selected tab and panel visibility", () => {
  const f = fixture();

  f.tabButtons.get("libraries").onclick();

  assert.equal(f.tabButtons.get("libraries").attributes["aria-selected"], "true");
  assert.equal(f.tabButtons.get("models").attributes["aria-selected"], "false");
  assert.equal(f.panels.get("libraries").hidden, false);
  assert.equal(f.panels.get("models").hidden, true);

  f.tabButtons.get("models").onclick();
  assert.equal(f.panels.get("libraries").hidden, true);
  assert.equal(f.panels.get("models").hidden, false);
});
