import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createTabCollection } from "../src/state/tab-collection.ts";

test("starts on the home id and has no active document", () => {
  const collection = createTabCollection(0);

  assert.equal(collection.activeId(), 0);
  assert.equal(collection.activeTab(), undefined);
  assert.deepEqual(collection.tabs(), []);
});

test("adds, activates, removes, and replaces tabs through explicit operations", () => {
  const collection = createTabCollection(0);
  const first = { id: 1 };
  const second = { id: 2 };
  const restored = { id: 3 };

  collection.add(first);
  collection.add(second);
  collection.setActiveId(2);
  assert.equal(collection.activeTab(), second);

  collection.removeAt(1);
  assert.deepEqual(collection.tabs(), [first]);
  collection.replace([restored]);
  collection.setActiveId(3);
  assert.equal(collection.activeTab(), restored);
});

test("tab list reads are snapshots that cannot mutate the collection", () => {
  const collection = createTabCollection(0);
  collection.add({ id: 1 });
  const snapshot = collection.tabs();

  snapshot.pop();

  assert.equal(collection.tabs().length, 1);
});

test("main delegates shared tab collection state to the store", async () => {
  const [main, shell] = await Promise.all([
    readFile(new URL("../src/main.ts", import.meta.url), "utf8"),
    readFile(new URL("../src/state/tabs-shell.ts", import.meta.url), "utf8"),
  ]);

  assert.match(main, /createTabsShell\(/);
  assert.match(shell, /createTabCollection\(dependencies\.homeId\)/);
  assert.doesNotMatch(main, /let tabs: Tab\[\]/);
  assert.doesNotMatch(main, /let activeId = homeId/);
  assert.match(main, /setActiveId: tabCollection\.setActiveId/);
  assert.match(main, /tabCollection\.replace\(next\)/);
});
