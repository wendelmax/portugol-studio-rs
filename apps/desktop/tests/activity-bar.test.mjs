import test from "node:test";
import assert from "node:assert/strict";
import { mountActivityBar } from "../src/shell/activity-bar.ts";

function fixture() {
  const events = [];
  const buttons = ["learn", "examples", "debug", "settings"].map((activity) => ({
    dataset: { activity },
    attributes: {},
    onclick: null,
    setAttribute(name, value) { this.attributes[name] = value; },
  }));
  mountActivityBar(buttons, {
    setInspectorView: (visible) => events.push(["inspector", visible]),
    showPanel: (panel) => events.push(["panel", panel]),
    activateHome: () => events.push(["home"]),
    showUtilities: (visible) => events.push(["utilities", visible]),
    openSettings: () => events.push(["settings"]),
  });
  return { events, buttons };
}

test("activity buttons synchronize pressed state and route to their current destinations", () => {
  const f = fixture();

  for (const activity of ["learn", "examples", "debug", "settings"]) {
    f.events.length = 0;
    const button = f.buttons.find((item) => item.dataset.activity === activity);
    button.onclick();

    assert.deepEqual(f.buttons.map((item) => item.attributes["aria-pressed"]),
      f.buttons.map((item) => String(item === button)));
    const expected = {
      learn: [["panel", "learning"]],
      examples: [["inspector", false], ["home"]],
      debug: [["inspector", false], ["utilities", true], ["panel", "debug"]],
      settings: [["inspector", false], ["settings"]],
    }[activity];
    assert.deepEqual(f.events, expected);
  }
});

test("activity destinations preserve the editor's view actions", () => {
  const f = fixture();
  const click = (activity) => f.buttons.find((item) => item.dataset.activity === activity).onclick();

  click("examples");
  click("debug");
  click("settings");

  assert.ok(f.events.some(([name]) => name === "home"));
  assert.ok(f.events.some(([name, value]) => name === "utilities" && value === true));
  assert.ok(f.events.some(([name, value]) => name === "panel" && value === "debug"));
  assert.ok(f.events.some(([name]) => name === "settings"));
});

test("clicking learn while already active toggles utilities closed and clears pressed state", () => {
  const events = [];
  let learningOpen = true;
  const buttons = ["learn", "examples", "debug", "settings"].map((activity) => ({
    dataset: { activity },
    attributes: {},
    onclick: null,
    setAttribute(name, value) { this.attributes[name] = value; },
  }));
  mountActivityBar(buttons, {
    setInspectorView: (visible) => events.push(["inspector", visible]),
    showPanel: (panel) => events.push(["panel", panel]),
    activateHome: () => events.push(["home"]),
    showUtilities: (visible) => { learningOpen = visible; events.push(["utilities", visible]); },
    openSettings: () => events.push(["settings"]),
    isLearningActive: () => learningOpen,
  });

  buttons[0].onclick();

  assert.equal(buttons[0].attributes["aria-pressed"], "false");
  assert.deepEqual(events, [["utilities", false]]);
});
