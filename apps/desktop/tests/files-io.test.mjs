import test from "node:test";
import assert from "node:assert/strict";
import { createFileIO } from "../src/files/io.ts";
import { detectSourceLanguage, ensureSourceExtension } from "../src/document-language.ts";

function makeTab(overrides = {}) {
  return {
    id: 1,
    path: null,
    language: "portugol-rs",
    saved: "",
    diagnostics: [],
    symbols: [],
    state: {},
    ...overrides,
  };
}

function fixture(overrides = {}) {
  const events = [];
  let tabs = overrides.initialTabs ?? [];
  let selectedTab = overrides.selectedTab;
  const dependencies = {
    tabs: () => tabs,
    replaceTabs: (next) => { tabs = next; },
    activeTab: () => selectedTab,
    activate: (id) => events.push(["activate", id]),
    newTab: (text, path) => {
      const tab = makeTab({ id: tabs.length + 1, path, saved: text });
      tabs = [...tabs, tab];
      selectedTab = tab;
      events.push(["newTab", text, path]);
    },
    isDirty: (tab) => tab.saved !== (tab.currentText ?? tab.saved),
    readFile: async (path) => { events.push(["readFile", path]); return "arquivo"; },
    openDialog: async () => null,
    saveDialog: async () => null,
    writeFile: async (path, text) => events.push(["writeFile", path, text]),
    writeWorkspaceFile: async (id, path, text) => events.push(["writeWorkspaceFile", id, path, text]),
    currentText: () => "conteúdo atual",
    detectSourceLanguage,
    ensureSourceExtension,
    rememberRecent: (path) => events.push(["recent", path]),
    renderTabs: () => events.push(["renderTabs"]),
    setStatus: (...args) => events.push(["status", ...args]),
    afterSave: (...args) => events.push(["afterSave", ...args]),
    ...overrides.dependencies,
  };
  return { io: createFileIO(dependencies), dependencies, events, getTabs: () => tabs };
}

test("opening an existing path activates its tab without reading it again", async () => {
  const existing = makeTab({ id: 8, path: "/aula.pors" });
  const { io, events } = fixture({ initialTabs: [existing], selectedTab: existing });

  await io.openFile("/aula.pors");

  assert.deepEqual(events, [["activate", 8], ["recent", "/aula.pors"]]);
});

test("opening a file replaces the only pristine untitled tab", async () => {
  const blank = makeTab({ id: 2, path: null, saved: "" });
  const { io, events, getTabs } = fixture({ initialTabs: [blank], selectedTab: blank });

  await io.openFile("/aula.pors");

  assert.equal(getTabs().some((tab) => tab.id === 2), false);
  assert.deepEqual(events, [
    ["readFile", "/aula.pors"],
    ["newTab", "arquivo", "/aula.pors"],
    ["recent", "/aula.pors"],
  ]);
});

test("canceling the open dialog does not change tabs or status", async () => {
  const tab = makeTab({ id: 3 });
  const { io, events, dependencies } = fixture({ initialTabs: [tab], selectedTab: tab });
  dependencies.openDialog = async () => null;

  await io.openFiles();

  assert.deepEqual(events, []);
});

test("saving as Portugol-RS adds .pors and reports the language change after writing", async () => {
  const tab = makeTab({ id: 4, path: null, language: "portugol-rs", saved: "antigo" });
  const { io, events, dependencies } = fixture({ selectedTab: tab });
  dependencies.saveDialog = async (options) => {
    assert.equal(options.defaultPath, "programa.pors");
    return "/tmp/aula";
  };

  const saved = await io.save();

  assert.equal(saved, true);
  assert.equal(tab.path, "/tmp/aula.pors");
  assert.equal(tab.language, "portugol-rs");
  assert.deepEqual(events.map(([name]) => name), ["writeFile", "afterSave", "recent", "renderTabs", "status"]);
  assert.equal(events[0][2], "conteúdo atual");
  assert.equal(events[1][1], tab);
  assert.equal(events[1][2], "portugol-rs");
  assert.equal(events[1][3], false);
});

test("saving a workspace tab uses its workspace path", async () => {
  const tab = makeTab({ id: 5, path: "src/main.pors", workspaceId: "workspace", workspacePath: "src/main.pors" });
  const { io, events } = fixture({ selectedTab: tab });

  const saved = await io.save();

  assert.equal(saved, true);
  assert.deepEqual(events[0], ["writeWorkspaceFile", "workspace", "src/main.pors", "conteúdo atual"]);
  assert.equal(tab.workspaceId, "workspace");
});

test("failed writes keep the tab unchanged and show the error", async () => {
  const tab = makeTab({ id: 6, path: "/old.pors", language: "portugol-rs", saved: "antigo" });
  const { io, events, dependencies } = fixture({ selectedTab: tab });
  dependencies.writeFile = async () => { throw new Error("disco cheio"); };

  const saved = await io.save();

  assert.equal(saved, false);
  assert.equal(tab.path, "/old.pors");
  assert.equal(tab.saved, "antigo");
  assert.deepEqual(events, [["status", "Error: disco cheio", "error"]]);
});
