import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { calculateSidePaneMinimums, DEFAULT_PANE_LAYOUT, loadPaneLayout, resizePaneForKey, savePaneLayout } from "../src/pane-layout.ts";

function storage(initial = null) {
  let value = initial;
  return {
    getItem() { return value; },
    setItem(_key, next) { value = next; },
    value() { return value; },
  };
}

test("restores valid pane ratios and clamps out-of-range values", () => {
  const source = storage(JSON.stringify({ explorer: 0.3, bottom: 0.99, utilities: 0.1 }));
  assert.deepEqual(loadPaneLayout(source), { explorer: 0.3, bottom: 0.65, utilities: 0.15 });
});

test("invalid or unavailable layout storage falls back to defaults", () => {
  assert.deepEqual(loadPaneLayout(storage("{bad json")), DEFAULT_PANE_LAYOUT);
  assert.deepEqual(loadPaneLayout({ getItem() { throw new Error("storage blocked"); } }), DEFAULT_PANE_LAYOUT);
});

test("default layout reserves about 232 px for the explorer at 1536 px", () => {
  assert.equal(DEFAULT_PANE_LAYOUT.explorer, 0.16);
});

test("saves a normalized layout and reports storage failure", () => {
  const target = storage();
  assert.equal(savePaneLayout(target, { explorer: 0.25, bottom: -1, utilities: 0.4 }), true);
  assert.deepEqual(JSON.parse(target.value()), { explorer: 0.25, bottom: 0.12, utilities: 0.4 });
  assert.equal(savePaneLayout({ setItem() { throw new Error("storage blocked"); } }, DEFAULT_PANE_LAYOUT), false);
});

test("stores and restores pane ratios independently per workspace", () => {
  const values = new Map();
  const target = {
    getItem(key) { return values.get(key) ?? null; },
    setItem(key, value) { values.set(key, value); },
  };
  const first = { explorer: 0.2, bottom: 0.3, utilities: 0.25 };
  const second = { explorer: 0.35, bottom: 0.4, utilities: 0.3 };

  assert.equal(savePaneLayout(target, first, "workspace-a"), true);
  assert.equal(savePaneLayout(target, second, "workspace-b"), true);
  assert.deepEqual(loadPaneLayout(target, "workspace-a"), first);
  assert.deepEqual(loadPaneLayout(target, "workspace-b"), second);
  assert.equal(values.get("portugol.pane-layout.v1.workspace.workspace-a"), JSON.stringify(first));
});

test("workspace without a valid layout inherits the global layout", () => {
  const values = new Map([
    ["portugol.pane-layout.v1", JSON.stringify({ explorer: 0.28, bottom: 0.32, utilities: 0.3 })],
    ["portugol.pane-layout.v1.workspace.workspace-b", "{bad json"],
  ]);
  const target = {
    getItem(key) { return values.get(key) ?? null; },
    setItem(key, value) { values.set(key, value); },
  };

  assert.deepEqual(loadPaneLayout(target, "workspace-a"), { explorer: 0.28, bottom: 0.32, utilities: 0.3 });
  assert.deepEqual(loadPaneLayout(target, "workspace-b"), { explorer: 0.28, bottom: 0.32, utilities: 0.3 });
});

test("standalone pane layout continues to read and write the legacy key", () => {
  const target = storage();
  const layout = { explorer: 0.24, bottom: 0.34, utilities: 0.29 };

  assert.equal(savePaneLayout(target, layout), true);
  assert.equal(target.value(), JSON.stringify(layout));
  assert.deepEqual(loadPaneLayout(target), layout);
});

test("maps pane separator keys to bounded size changes", () => {
  const bounds = { minimum: 100, maximum: 400 };
  assert.equal(resizePaneForKey("ArrowUp", "horizontal", -1, 220, bounds), 244);
  assert.equal(resizePaneForKey("ArrowRight", "vertical", 1, 220, bounds), 244);
  assert.equal(resizePaneForKey("ArrowLeft", "vertical", -1, 220, bounds), 244);
  assert.equal(resizePaneForKey("ArrowDown", "horizontal", -1, 220, bounds), 196);
  assert.equal(resizePaneForKey("Home", "horizontal", -1, 220, bounds), 100);
  assert.equal(resizePaneForKey("End", "horizontal", -1, 220, bounds), 400);
  assert.equal(resizePaneForKey("ArrowUp", "horizontal", -1, 110, bounds), 134);
  assert.equal(resizePaneForKey("ArrowUp", "horizontal", -1, 390, bounds), 400);
  assert.equal(resizePaneForKey("ArrowDown", "horizontal", -1, 110, bounds), 100);
  assert.equal(resizePaneForKey("Escape", "horizontal", -1, 220, bounds), null);
});

test("fits both side panes into the width left after toolbar, editor, and separators", () => {
  const minimums = calculateSidePaneMinimums(430, 155, 9, 15, true, true);
  assert.ok(minimums.explorer < 180);
  assert.ok(minimums.utilities < 210);
  assert.ok(Math.abs(430 - 155 - 9 - 15 - minimums.explorer - minimums.utilities) < 0.001);
  assert.deepEqual(calculateSidePaneMinimums(1_000, 155, 9, 15, true, true), { explorer: 180, utilities: 210 });
  assert.deepEqual(calculateSidePaneMinimums(430, 155, 9, 15, true, false), { explorer: 180, utilities: 0 });
});

test("pane separators expose accessible keyboard-resizable controls", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  for (const id of ["workspace-explorer-splitter", "splitter", "utilities-splitter"]) {
    const markup = html.match(new RegExp(`<div id="${id}"[^>]*>`))?.[0] ?? "";
    assert.match(markup, /role="separator"/);
    assert.match(markup, /tabindex="0"/);
    assert.match(markup, /aria-valuemin=/);
    assert.match(markup, /aria-valuemax=/);
    assert.match(markup, /aria-valuenow=/);
    assert.match(markup, /aria-controls=/);
  }
});
