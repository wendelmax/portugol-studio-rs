import test from "node:test";
import assert from "node:assert/strict";
import { hasRustExecutionConsent, rememberRustExecutionConsent, RUST_EXECUTION_CONSENT_KEY } from "../src/rust-execution-consent.ts";

function memoryStorage() {
  const values = new Map();
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  };
}

test("Rust execution consent is remembered for the current session only", () => {
  const storage = memoryStorage();
  assert.equal(hasRustExecutionConsent(storage), false);
  rememberRustExecutionConsent(storage, true);
  assert.equal(storage.getItem(RUST_EXECUTION_CONSENT_KEY), "accepted");
  assert.equal(hasRustExecutionConsent(storage), true);
});

test("declining Rust execution does not store consent", () => {
  const storage = memoryStorage();
  rememberRustExecutionConsent(storage, false);
  assert.equal(hasRustExecutionConsent(storage), false);
});
