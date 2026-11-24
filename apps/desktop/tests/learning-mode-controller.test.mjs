import test from "node:test";
import assert from "node:assert/strict";
import { createLearningModeController } from "../src/learning/mode-controller.ts";

function fixture(overrides = {}) {
  const events = [];
  let mode = "beginner";
  let tab = { language: "portugol-rs", workspacePreferenceKey: "aula-1" };
  const controller = createLearningModeController({
    rustcAvailable: () => true,
    activeTab: () => tab,
    setMode: (next) => { mode = next; events.push(["mode", next]); },
    setSelectorValue: (next) => events.push(["selector", next]),
    persist: (next, preferenceKey) => events.push(["persist", next, preferenceKey]),
    panelVisible: () => false,
    renderPanel: () => events.push(["panel"]),
    setPreviewHidden: (hidden) => events.push(["preview-hidden", hidden]),
    renderMappingGutter: () => events.push(["mapping"]),
    setBridgePressed: (pressed) => events.push(["pressed", pressed]),
    schedulePreview: () => events.push(["schedule"]),
    cancelPreview: () => events.push(["cancel"]),
    ...overrides,
  });
  return { controller, events, getMode: () => mode, setTab: (next) => { tab = next; } };
}

test("Rust mode falls back to beginner until rustc is available", () => {
  const { controller, events, getMode } = fixture({ rustcAvailable: () => false });

  const selected = controller("rust", false);

  assert.equal(selected, "beginner");
  assert.equal(getMode(), "beginner");
  assert.deepEqual(events, [
    ["mode", "beginner"],
    ["selector", "beginner"],
    ["preview-hidden", true],
    ["mapping"],
    ["pressed", false],
    ["cancel"],
  ]);
});

test("bridge mode persists per workspace and schedules the preview for non-Rust tabs", () => {
  const { controller, events } = fixture({ panelVisible: () => true });

  const selected = controller("bridge");

  assert.equal(selected, "bridge");
  assert.deepEqual(events, [
    ["mode", "bridge"],
    ["selector", "bridge"],
    ["persist", "bridge", "aula-1"],
    ["panel"],
    ["preview-hidden", false],
    ["mapping"],
    ["pressed", true],
    ["schedule"],
  ]);
});

test("bridge preview is canceled for Rust tabs and non-bridge modes", () => {
  const { controller, events, setTab } = fixture();
  setTab({ language: "rust" });

  controller("bridge", false);
  controller("beginner", false);

  assert.deepEqual(events.filter(([name]) => ["preview-hidden", "pressed", "schedule", "cancel"].includes(name)), [
    ["preview-hidden", true],
    ["pressed", false],
    ["cancel"],
    ["preview-hidden", true],
    ["pressed", false],
    ["cancel"],
  ]);
  assert.equal(events.some(([name]) => name === "schedule"), false);
});
