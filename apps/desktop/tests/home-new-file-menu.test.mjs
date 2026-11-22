import test from "node:test";
import assert from "node:assert/strict";
import { mountHomeNewFileMenu } from "../src/home-new-file-menu.ts";

function setup(languages = ["portugol-rs", "rust"]) {
  const callbacks = new Map();
  const buttons = languages.map((language) => ({
    dataset: { homeNewLanguage: language },
    addEventListener(type, callback) { callbacks.set(`${language}:${type}`, callback); },
  }));
  const summary = { focused: false, focus() { this.focused = true; } };
  const menu = {
    open: true,
    handlers: new Map(),
    querySelector(selector) {
      assert.equal(selector, "summary");
      return summary;
    },
    querySelectorAll(selector) {
      assert.equal(selector, "[data-home-new-language]");
      return buttons;
    },
    addEventListener(type, callback) { this.handlers.set(type, callback); },
  };
  return { callbacks, menu, summary, buttons };
}

test("home new file menu selects Portugol-RS and closes", () => {
  const state = setup();
  let selected = "";
  mountHomeNewFileMenu(state.menu, {
    "portugol-rs": () => { selected = "portugol-rs"; },
    rust: () => { selected = "rust"; },
  });

  state.callbacks.get("portugol-rs:click")();

  assert.equal(selected, "portugol-rs");
  assert.equal(state.menu.open, false);
});

test("home new file menu selects Rust and closes", () => {
  const state = setup();
  let selected = "";
  mountHomeNewFileMenu(state.menu, {
    "portugol-rs": () => { selected = "portugol-rs"; },
    rust: () => { selected = "rust"; },
  });

  state.callbacks.get("rust:click")();

  assert.equal(selected, "rust");
  assert.equal(state.menu.open, false);
});

test("home new file menu Escape closes and restores focus", () => {
  const state = setup();
  mountHomeNewFileMenu(state.menu, { "portugol-rs": () => {}, rust: () => {} });
  let prevented = false;

  state.menu.handlers.get("keydown")({ key: "Escape", preventDefault() { prevented = true; } });

  assert.equal(prevented, true);
  assert.equal(state.menu.open, false);
  assert.equal(state.summary.focused, true);
});

test("home new file menu ignores unknown language values", () => {
  const state = setup(["toString"]);
  let invoked = false;
  mountHomeNewFileMenu(state.menu, { "portugol-rs": () => { invoked = true; }, rust: () => { invoked = true; } });

  state.callbacks.get("toString:click")();

  assert.equal(invoked, false);
  assert.equal(state.menu.open, true);
});
