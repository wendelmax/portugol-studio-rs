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

function fixture({ includeLibrary } = {}) {
  const events = [];
  const libraryButtons = [];
  const snippetButtons = [];
  const tabButtons = new Map([["libraries", fakeButton()], ["models", fakeButton()]]);
  const panels = new Map([["libraries", { hidden: false }], ["models", { hidden: true }]]);
  const tab = { id: 1, language: "portugol" };
  let activeTab = tab;
  let documentState = { toString: () => "programa {}" };
  let running = false;
  const snippets = [
    { label: "para", detail: "laço", apply: () => {} },
    { label: "programa", detail: "estrutura" },
  ];
  const controller = createUtilityInsertions({
    libraries: ["Matematica"],
    snippets,
    createButton: fakeButton,
    appendLibraryButton: (button) => libraryButtons.push(button),
    appendSnippetButton: (button) => snippetButtons.push(button),
    activeTab: () => activeTab,
    isRunning: (candidate) => running && candidate === activeTab,
    currentDocument: () => documentState,
    includeLibrary: includeLibrary ?? (async (source, name) => { events.push(["include", source, name]); return { from: 0, to: 0, insert: "inclua biblioteca Matematica\n" }; }),
    isCurrent: (candidate, captured) => candidate === activeTab && captured === documentState,
    applyLibraryChange: (change) => events.push(["change", change]),
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
    setDocument: (value) => { documentState = value; },
    setRunning: (value) => { running = value; },
  };
}

test("library insertion applies only to the captured active document", async () => {
  const f = fixture();
  const button = f.libraryButtons[0];

  await button.onclick();

  assert.deepEqual(f.events, [
    ["include", "programa {}", "Matematica"],
    ["change", { from: 0, to: 0, insert: "inclua biblioteca Matematica\n" }],
    ["focus"],
  ]);
  assert.equal(button.disabled, true);
});

test("library insertion ignores stale results and reports current-tab failures", async () => {
  let resolveInclude;
  const f = fixture({ includeLibrary: () => new Promise((resolve) => { resolveInclude = resolve; }) });
  const button = f.libraryButtons[0];
  const pending = button.onclick();
  f.setDocument({ toString: () => "mudou" });
  resolveInclude({ from: 0, to: 0, insert: "" });

  await pending;

  assert.equal(f.events.some(([kind]) => kind === "change"), false);
  assert.equal(f.events.some(([kind]) => kind === "focus"), false);

  const failed = fixture({ includeLibrary: async () => { throw "Falha ao incluir"; } });
  await failed.libraryButtons[0].onclick();
  assert.deepEqual(failed.events, [["status", "Falha ao incluir", "error"]]);
});

test("library insertion enforces running and PORS guards", async () => {
  const f = fixture();
  f.setRunning(true);
  await f.libraryButtons[0].onclick();
  assert.equal(f.events.length, 0);

  f.setRunning(false);
  f.setTab({ id: 2, language: "portugol-rs" });
  await f.libraryButtons[0].onclick();

  assert.deepEqual(f.events, [["status", "Bibliotecas clássicas ainda não estão disponíveis no Portugol-RS.", undefined]]);
});

test("snippet buttons retain language guards, insertion, and new-program action", () => {
  const f = fixture();
  const [loopButton, programButton] = f.snippetButtons;

  f.setTab({ id: 2, language: "rust" });
  loopButton.onclick();
  assert.deepEqual(f.events, []);

  f.setTab({ id: 1, language: "portugol" });
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
