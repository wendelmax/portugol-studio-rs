import test from "node:test";
import assert from "node:assert/strict";
import { createPanelNavigation } from "../src/layout/panel-navigation.ts";

function fixture() {
  const calls = [];
  const navigation = createPanelNavigation({
    bottomPanels: ["console", "tests", "messages", "debug"],
    inspectorElements: ["utilities-insert", "outline", "variable-inspector"],
    setInspectorElementHidden: (id, hidden) => calls.push(["inspector", id, hidden]),
    showUtilities: (visible) => calls.push(["utilities", visible]),
    renderLearningPanel: () => calls.push(["learning"]),
    setTabState: (name, active, selected) => calls.push(["tab", name, active, selected]),
    setPanelContentHidden: (name, hidden) => calls.push(["content", name, hidden]),
    setConsoleToolsHidden: (hidden) => calls.push(["consoleTools", hidden]),
    renderExerciseTests: () => calls.push(["tests"]),
  });
  return { calls, navigation };
}

test("learning view switches inspector, expands utilities, and renders learning", () => {
  const f = fixture();
  f.navigation.showPanel("learning");
  assert.deepEqual(f.calls, [
    ["inspector", "learning-panel", false],
    ["inspector", "utilities-insert", true], ["inspector", "outline", true], ["inspector", "variable-inspector", true],
    ["utilities", true], ["learning"],
  ]);
});

test("selects a bottom panel with matching active and ARIA states", () => {
  const f = fixture();
  f.navigation.showPanel("messages");
  assert.deepEqual(f.calls.filter(([name]) => name === "tab"), [
    ["tab", "console", false, false], ["tab", "tests", false, false],
    ["tab", "messages", true, true], ["tab", "debug", false, false],
  ]);
  assert.deepEqual(f.calls.filter(([name]) => name === "content"), [
    ["content", "console", true], ["content", "tests", true],
    ["content", "messages", false], ["content", "debug", true],
  ]);
});

test("shows console tools only for the console and renders tests on the tests panel", () => {
  const f = fixture();
  f.navigation.showPanel("console");
  assert.deepEqual(f.calls.at(-1), ["consoleTools", false]);
  f.navigation.showPanel("tests");
  assert.deepEqual(f.calls.slice(-2), [["consoleTools", true], ["tests"]]);
});

test("switching bottom panels preserves the right learning inspector without hiding it", () => {
  const f = fixture();
  f.navigation.showPanel("learning");
  f.calls.length = 0;

  f.navigation.showPanel("console");

  assert.equal(f.calls.some(([name]) => name === "inspector"), false);
});
