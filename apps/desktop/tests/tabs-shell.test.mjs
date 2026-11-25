import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { readFile } from "node:fs/promises";
import { createTabsShell } from "../src/state/tabs-shell.ts";

class FakeElement {
  children = [];
  attributes = new Map();
  className = "";
  textContent = "";
  title = "";
  onclick = null;
  onauxclick = null;
  constructor(tagName) { this.tagName = tagName; }
  append(...children) { this.children.push(...children); }
  prepend(...children) { this.children.unshift(...children); }
  replaceChildren(...children) { this.children = children; }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
}

function fixture() {
  const document = { title: "", createElement: (tag) => new FakeElement(tag) };
  const container = new FakeElement("nav");
  const activations = [];
  const prompts = [];
  let currentText = "";
  let editorState = EditorState.create({ doc: "" });
  let helpOpen = false;
  let allowDiscard = false;
  let shell;
  shell = createTabsShell({
    homeId: 0,
    helpId: -1,
    document,
    container,
    icon: () => new FakeElement("span"),
    extensions: () => [],
    activate: (id) => { activations.push(id); shell.tabCollection.setActiveId(id); },
    helpOpen: () => helpOpen,
    setHelpOpen: (value) => { helpOpen = value; },
    currentDocumentText: () => currentText,
    editorState: () => editorState,
    setCursorText: (text) => { prompts.push(["cursor", text]); },
    confirmDiscard: async (tab) => { prompts.push(["confirm", tab.id]); return allowDiscard; },
  });
  return {
    shell,
    container,
    document,
    activations,
    prompts,
    setCurrentText: (text) => { currentText = text; },
    setEditorState: (state) => { editorState = state; },
    setHelpOpen: (value) => { helpOpen = value; },
    setAllowDiscard: (value) => { allowDiscard = value; },
  };
}

test("tab shell creates and activates tabs and checks dirty state against the active editor", () => {
  const f = fixture();
  const tab = f.shell.newTab("salvo", "aula.pors");
  f.setCurrentText("salvo");

  assert.equal(tab.id, 1);
  assert.equal(f.shell.activeTab(), tab);
  assert.equal(f.shell.isDirty(tab), false);
  assert.deepEqual(f.activations, [1]);
  f.setCurrentText("alterado");
  assert.equal(f.shell.isDirty(tab), true);
});

test("tab shell renders cursor and tab strip, and closes dirty tabs only after confirmation", async () => {
  const f = fixture();
  const tab = f.shell.newTab("um\ndois", "aula.pors");
  f.setCurrentText("um\ndois");
  f.setEditorState(EditorState.create({ doc: "um\ndois", selection: { anchor: 5 } }));
  f.shell.renderCursor();
  f.shell.renderTabs();

  assert.deepEqual(f.prompts[0], ["cursor", "Linha 2, coluna 3"]);
  assert.equal(f.document.title, "aula.pors — Portugol Studio");
  assert.equal(f.container.children[1].children[0].textContent, "aula.pors");

  f.setCurrentText("alterado");
  assert.equal(await f.shell.closeTab(tab.id), false);
  assert.equal(f.shell.tabCollection.tabs().length, 1);
  f.setAllowDiscard(true);
  assert.equal(await f.shell.closeTab(tab.id), true);
  assert.equal(f.shell.tabCollection.tabs().length, 0);
  assert.deepEqual(f.activations, [1, 0]);
});

test("main delegates tab shell lifecycle to its composition module", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(main, /createTabsShell\(/);
  assert.match(main, /const \{ tabCollection \} = tabsShell/);
  assert.match(main, /function activeTab\(\) \{ return tabsShell\.activeTab\(\); \}/);
});
