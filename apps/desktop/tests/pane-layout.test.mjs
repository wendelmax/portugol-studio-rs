import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { DEFAULT_PANE_LAYOUT, loadPaneLayout, resizePaneForKey, savePaneLayout } from "../src/pane-layout.ts";

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

test("saves a normalized layout and reports storage failure", () => {
  const target = storage();
  assert.equal(savePaneLayout(target, { explorer: 0.25, bottom: -1, utilities: 0.4 }), true);
  assert.deepEqual(JSON.parse(target.value()), { explorer: 0.25, bottom: 0.12, utilities: 0.4 });
  assert.equal(savePaneLayout({ setItem() { throw new Error("storage blocked"); } }, DEFAULT_PANE_LAYOUT), false);
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

test("pane separators expose accessible keyboard-resizable controls", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  for (const id of ["workspace-explorer-splitter", "splitter", "utilities-splitter"]) {
    const markup = html.match(new RegExp(`<div id="${id}"[^>]*>`))?.[0] ?? "";
    assert.match(markup, /role="separator"/);
    assert.match(markup, /tabindex="0"/);
    assert.match(markup, /aria-valuemin=/);
    assert.match(markup, /aria-valuemax=/);
    assert.match(markup, /aria-valuenow=/);
  }
});
