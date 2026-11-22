import assert from "node:assert/strict";
import { test } from "node:test";
import { learningPinForMode, setLearningPin, toggleLearningPin } from "../src/learning-pins.ts";

test("pinning a topic records the selected concept only for the requested learning mode", () => {
  const pins = { beginner: "variables" };
  const next = setLearningPin(pins, "bridge", "ownership");

  assert.deepEqual(next, { beginner: "variables", bridge: "ownership" });
  assert.deepEqual(pins, { beginner: "variables" });
});

test("choosing another concept while pinned replaces only that mode's topic", () => {
  assert.deepEqual(setLearningPin({ bridge: "ownership", rust: "borrowing" }, "bridge", "mutability"), {
    bridge: "mutability",
    rust: "borrowing",
  });
});

test("toggling the active concept off removes that mode pin and keeps other modes", () => {
  assert.deepEqual(toggleLearningPin({ bridge: "ownership", rust: "borrowing" }, "bridge", "ownership"), {
    rust: "borrowing",
  });
});

test("toggling a different concept replaces the existing pin for its mode", () => {
  assert.deepEqual(toggleLearningPin({ bridge: "ownership" }, "bridge", "mutability"), {
    bridge: "mutability",
  });
});

test("a pin is active only when its concept exists in the current mode catalog", () => {
  const pins = { beginner: "ownership", bridge: "ownership", rust: "borrowing" };
  assert.equal(learningPinForMode(pins, "beginner", ["variables", "conditionals", "loops"]), undefined);
  assert.equal(learningPinForMode(pins, "bridge", ["variables", "ownership"]), "ownership");
  assert.equal(learningPinForMode(pins, "rust", ["mutability", "borrowing"]), "borrowing");
});
