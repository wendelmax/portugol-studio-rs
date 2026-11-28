import test from "node:test";
import assert from "node:assert/strict";
import { createCenterSourceController } from "../src/editor/center-source.ts";

function fixture({ stored = null, storageError = false, hasTab = true } = {}) {
  const calls = [];
  let tabOpen = hasTab;
  const values = new Map(stored === null ? [] : [["ferro.centerSource", stored]]);
  const storage = {
    getItem(key) { if (storageError) throw new Error("storage unavailable"); return values.get(key) ?? null; },
    setItem(key, value) { if (storageError) throw new Error("storage unavailable"); values.set(key, value); calls.push(["save", key, value]); },
  };
  const controller = createCenterSourceController({
    storage: () => storage,
    hasActiveTab: () => tabOpen,
    centerSelection: () => calls.push(["center"]),
    setPressed: (pressed) => calls.push(["pressed", pressed]),
    focusEditor: () => calls.push(["focus"]),
  });
  return { calls, controller, values, setHasTab: (value) => { tabOpen = value; } };
}

test("restores the saved center preference and sets button state", () => {
  const f = fixture({ stored: "true" });
  assert.equal(f.controller.isActive(), true);
  assert.deepEqual(f.calls, [["pressed", true]]);
});

test("does not toggle without an active document", () => {
  const f = fixture({ hasTab: false });
  f.controller.toggle();
  assert.deepEqual(f.calls, [["pressed", false]]);
});

test("centers when enabling and focuses after each toggle", () => {
  const f = fixture();
  f.controller.toggle();
  assert.equal(f.controller.isActive(), true);
  assert.deepEqual(f.calls.slice(1), [
    ["pressed", true], ["save", "ferro.centerSource", "true"], ["center"], ["focus"],
  ]);
  f.controller.toggle();
  assert.equal(f.controller.isActive(), false);
  assert.deepEqual(f.calls.slice(-3), [
    ["pressed", false], ["save", "ferro.centerSource", "false"], ["focus"],
  ]);
});

test("continues to work when storage cannot be read or written", () => {
  const f = fixture({ storageError: true });
  assert.equal(f.controller.isActive(), false);
  f.controller.toggle();
  assert.equal(f.controller.isActive(), true);
  assert.ok(f.calls.some(([name]) => name === "center"));
  assert.ok(f.calls.some(([name]) => name === "focus"));
});
