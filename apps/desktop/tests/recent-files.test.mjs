import test from "node:test";
import assert from "node:assert/strict";
import {
  createRecentFiles,
  loadRecentFiles,
  RECENT_KEY,
  RECENT_LIMIT,
  withRecentFile,
  withRenamedRecentFile,
  withoutRecentFile,
} from "../src/files/recent.ts";

function memoryStorage(initial = {}) {
  const data = new Map(Object.entries(initial));
  return {
    getItem: (key) => (data.has(key) ? data.get(key) : null),
    setItem: (key, value) => data.set(key, value),
    data,
  };
}

test("loading ignores invalid entries and respects the limit", () => {
  const paths = Array.from({ length: 9 }, (_, index) => `/p/${index}.fer`);
  const storage = memoryStorage({ [RECENT_KEY]: JSON.stringify([...paths, 42, null]) });
  assert.deepEqual(loadRecentFiles(storage), paths.slice(0, RECENT_LIMIT));
  assert.deepEqual(loadRecentFiles(memoryStorage({ [RECENT_KEY]: "{inválido" })), []);
  assert.deepEqual(loadRecentFiles(memoryStorage({ [RECENT_KEY]: "{}" })), []);
  assert.deepEqual(loadRecentFiles(null), []);
});

test("remembering moves the file to the top without duplicates", () => {
  assert.deepEqual(withRecentFile(["/a", "/b", "/c"], "/b"), ["/b", "/a", "/c"]);
  const full = ["/1", "/2", "/3", "/4", "/5", "/6"];
  assert.deepEqual(withRecentFile(full, "/novo"), ["/novo", "/1", "/2", "/3", "/4", "/5"]);
});

test("renaming replaces the old path and keeps it on top", () => {
  assert.deepEqual(withRenamedRecentFile(["/a", "/b", "/c"], "/c", "/d"), ["/d", "/a", "/b"]);
  assert.deepEqual(withRenamedRecentFile(["/a", "/d"], "/a", "/d"), ["/d"]);
});

test("forgetting removes only the given path", () => {
  assert.deepEqual(withoutRecentFile(["/a", "/b"], "/a"), ["/b"]);
  assert.deepEqual(withoutRecentFile(["/a"], "/x"), ["/a"]);
});

test("the store saves and notifies every change", () => {
  const storage = memoryStorage();
  const notified = [];
  const recent = createRecentFiles(storage, (list) => notified.push([...list]));
  recent.remember("/a.fer");
  recent.remember("/b.fer");
  recent.rename("/a.fer", "/c.fer");
  recent.forget("/b.fer");
  assert.deepEqual(recent.list(), ["/c.fer"]);
  assert.deepEqual(JSON.parse(storage.data.get(RECENT_KEY)), ["/c.fer"]);
  assert.deepEqual(notified, [["/a.fer"], ["/b.fer", "/a.fer"], ["/c.fer", "/b.fer"], ["/c.fer"]]);
});

test("the store keeps working in memory when saving fails", () => {
  const storage = {
    getItem: () => null,
    setItem: () => {
      throw new Error("cota excedida");
    },
  };
  const recent = createRecentFiles(storage, () => {});
  recent.remember("/a.fer");
  assert.deepEqual(recent.list(), ["/a.fer"]);
});
