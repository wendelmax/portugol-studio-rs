import assert from "node:assert/strict";
import { test } from "node:test";
import { generatedRustDetachIsCurrent } from "../src/generated-rust-detach.ts";

test("allows detaching only when tab, document, artifact, and preview state still match", () => {
  const tab = { id: 7 };
  const document = { text: "escreva(1)" };
  const artifact = { code: "fn main() {}" };
  const snapshot = { tab, document, artifact };

  assert.equal(generatedRustDetachIsCurrent(snapshot, {
    tab, document, artifact, pending: false, stale: false,
  }), true);
});

test("rejects a confirmation when the active tab changed", () => {
  const snapshot = { tab: { id: 7 }, document: {}, artifact: {} };
  assert.equal(generatedRustDetachIsCurrent(snapshot, {
    ...snapshot, tab: { id: 8 }, pending: false, stale: false,
  }), false);
});

test("rejects a confirmation when the source document or artifact changed", () => {
  const snapshot = { tab: {}, document: {}, artifact: {} };
  assert.equal(generatedRustDetachIsCurrent(snapshot, {
    ...snapshot, document: {}, pending: false, stale: false,
  }), false);
  assert.equal(generatedRustDetachIsCurrent(snapshot, {
    ...snapshot, artifact: {}, pending: false, stale: false,
  }), false);
});

test("rejects a confirmation while generation is pending or stale", () => {
  const snapshot = { tab: {}, document: {}, artifact: {} };
  assert.equal(generatedRustDetachIsCurrent(snapshot, {
    ...snapshot, pending: true, stale: false,
  }), false);
  assert.equal(generatedRustDetachIsCurrent(snapshot, {
    ...snapshot, pending: false, stale: true,
  }), false);
});
