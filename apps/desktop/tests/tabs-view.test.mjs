import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createTabsView } from "../src/state/tabs-view.ts";

class FakeElement {
  children = [];
  attributes = new Map();
  className = "";
  textContent = "";
  title = "";
  onclick = null;
  onauxclick = null;

  constructor(tagName) { this.tagName = tagName; }
  append(...children) {
    this.children.push(...children);
    this.textContent += children.filter((child) => typeof child === "string").join("");
  }
  prepend(...children) { this.children.unshift(...children); }
  replaceChildren(...children) { this.children = children; }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  getAttribute(name) { return this.attributes.get(name) ?? null; }
}

function fixture() {
  const doc = {
    title: "",
    createElement: (tagName) => new FakeElement(tagName),
  };
  const container = new FakeElement("nav");
  const events = [];
  const tabs = [
    { id: 1, path: "/tmp/aula.por", dirty: true },
    { id: 2, path: null, exampleName: "exemplo.pors", dirty: false },
  ];
  let activeId = 1;
  let helpOpen = true;
  const view = createTabsView({
    container,
    document: doc,
    tabs: () => tabs,
    activeId: () => activeId,
    homeId: 0,
    helpId: -1,
    helpOpen: () => helpOpen,
    tabName: (tab) => tab.path?.split("/").at(-1) ?? tab.exampleName,
    isDirty: (tab) => tab.dirty,
    icon: () => new FakeElement("span"),
    activate: (id) => { activeId = id; events.push(["activate", id]); },
    newTab: () => events.push(["new"]),
    closeTab: (id) => events.push(["close", id]),
  });
  return { doc, container, events, tabs, view, setActiveId: (id) => { activeId = id; } };
}

test("tab strip renders home, help, file tabs, and new-file action", () => {
  const { container, view } = fixture();

  view.render();

  assert.deepEqual(container.children.map((element) => element.className), [
    "tab home-tab", "file-tab tab", "file-tab tab active", "file-tab tab", "tab add-tab",
  ]);
  assert.equal(container.children[1].children[0].textContent, "Ajuda");
  assert.equal(container.children[2].children[0].textContent, "aula.por");
  assert.equal(container.children[2].children[0].className, "dirty");
  assert.equal(container.children[3].children[0].textContent, "exemplo.pors");
  assert.equal(container.children[0].textContent, "Início");
  assert.equal(container.children[2].title, "/tmp/aula.por");
  assert.equal(container.children[3].title, "Arquivo ainda não salvo");
  assert.equal(container.children.at(-1).textContent, "+");
});

test("tab strip routes navigation, closing, and new-file actions to the shell", () => {
  const { container, events, view } = fixture();
  view.render();

  container.children[0].onclick();
  container.children[1].children[0].onclick();
  container.children[1].children[1].onclick({ stopPropagation() {} });
  container.children[2].children[1].onclick({ stopPropagation() {} });
  container.children.at(-1).onclick();

  assert.deepEqual(events, [["activate", 0], ["activate", -1], ["close", -1], ["close", 1], ["new"]]);
});

test("tab strip updates the document title for file, home, and help", () => {
  const { doc, view, setActiveId } = fixture();
  view.render();
  assert.equal(doc.title, "aula.por — Portugol Studio");

  setActiveId(0);
  view.render();
  assert.equal(doc.title, "Portugol Studio");

  setActiveId(-1);
  view.render();
  assert.equal(doc.title, "Ajuda — Portugol Studio");
});

test("IDE delegates tab strip rendering to the tested view module", async () => {
  const [main, shell] = await Promise.all([
    readFile(new URL("../src/main.ts", import.meta.url), "utf8"),
    readFile(new URL("../src/state/tabs-shell.ts", import.meta.url), "utf8"),
  ]);
  assert.match(main, /createTabsShell\(/);
  assert.match(shell, /createTabsView\(/);
  assert.match(shell, /tabsView\.render\(\)/);
});
