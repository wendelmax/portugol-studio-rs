import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createTabCloser } from "../src/state/tab-closer.ts";

function fixture({ activeId = 2, dirty = new Set(), confirm = true } = {}) {
  const tabs = [{ id: 1, path: "um.fer" }, { id: 2, path: "dois.fer" }, { id: 3, path: "tres.fer" }];
  const events = [];
  const dependencies = {
    tabs: () => tabs,
    activeId: () => activeId,
    homeId: 0,
    helpId: -1,
    setHelpOpen: (value) => events.push(["helpOpen", value]),
    isDirty: (tab) => dirty.has(tab.id),
    tabName: (tab) => tab.path,
    confirmDiscard: async (tab) => { events.push(["confirm", tab.id]); return confirm; },
    removeTab: (index) => { events.push(["remove", index]); tabs.splice(index, 1); },
    activate: (id) => { activeId = id; events.push(["activate", id]); },
    renderTabs: () => events.push(["render"]),
  };
  return { closer: createTabCloser(dependencies), tabs, events };
}

test("closing help selects the preceding file or returns to home", async () => {
  const { closer, events } = fixture({ activeId: -1 });

  assert.equal(await closer.close(-1), true);
  assert.deepEqual(events, [["helpOpen", false], ["activate", 3]]);
});

test("closing the active help tab returns home when no files are open", async () => {
  const { closer, tabs, events } = fixture({ activeId: -1 });
  tabs.splice(0);

  assert.equal(await closer.close(-1), true);
  assert.deepEqual(events, [["helpOpen", false], ["activate", 0]]);
});

test("closing an unknown tab succeeds without changing state", async () => {
  const { closer, events } = fixture();

  assert.equal(await closer.close(99), true);
  assert.deepEqual(events, []);
});

test("canceling an unsaved tab close leaves the tab open", async () => {
  const { closer, tabs, events } = fixture({ dirty: new Set([2]), confirm: false });

  assert.equal(await closer.close(2), false);
  assert.equal(tabs.some((tab) => tab.id === 2), true);
  assert.deepEqual(events, [["confirm", 2]]);
});

test("closing an active dirty tab removes it and activates its previous neighbor", async () => {
  const { closer, tabs, events } = fixture({ dirty: new Set([2]) });

  assert.equal(await closer.close(2), true);
  assert.deepEqual(tabs.map((tab) => tab.id), [1, 3]);
  assert.deepEqual(events, [["confirm", 2], ["remove", 1], ["activate", 1]]);
});

test("closing a background tab preserves the active tab and rerenders the strip", async () => {
  const { closer, tabs, events } = fixture({ activeId: 3 });

  assert.equal(await closer.close(1), true);
  assert.deepEqual(tabs.map((tab) => tab.id), [2, 3]);
  assert.deepEqual(events, [["remove", 0], ["render"]]);
});

test("IDE delegates close confirmation and mutation to the tab closer", async () => {
  const [main, shell] = await Promise.all([
    readFile(new URL("../src/main.ts", import.meta.url), "utf8"),
    readFile(new URL("../src/state/tabs-shell.ts", import.meta.url), "utf8"),
  ]);
  assert.match(main, /createTabsShell\(/);
  assert.match(shell, /createTabCloser\(/);
  assert.match(shell, /return tabCloser\.close\(id\)/);
});
