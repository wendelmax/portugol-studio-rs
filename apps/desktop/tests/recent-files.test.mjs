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
  const paths = Array.from({ length: 9 }, (_, index) => `/p/${index}.pors`);
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
  recent.remember("/a.pors");
  recent.remember("/b.pors");
  recent.rename("/a.pors", "/c.pors");
  recent.forget("/b.pors");
  assert.deepEqual(recent.list(), ["/c.pors"]);
  assert.deepEqual(JSON.parse(storage.data.get(RECENT_KEY)), ["/c.pors"]);
  assert.deepEqual(notified, [["/a.pors"], ["/b.pors", "/a.pors"], ["/c.pors", "/b.pors"], ["/c.pors"]]);
});

test("the store keeps working in memory when saving fails", () => {
  const storage = {
    getItem: () => null,
    setItem: () => {
      throw new Error("cota excedida");
    },
  };
  const recent = createRecentFiles(storage, () => {});
  recent.remember("/a.pors");
  assert.deepEqual(recent.list(), ["/a.pors"]);
});
