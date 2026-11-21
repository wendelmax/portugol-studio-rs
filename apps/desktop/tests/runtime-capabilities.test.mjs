import test from "node:test";
import assert from "node:assert/strict";
import { canDebugSource, canRunSource } from "../src/runtime-capabilities.ts";

test("classic Portugol uses its existing runtime and debugger", () => {
  assert.equal(canRunSource("portugol", null), true);
  assert.equal(canDebugSource("portugol"), true);
});

test("Portugol-RS can run only when rustc is available and stays non-debuggable", () => {
  assert.equal(canRunSource("portugol-rs", null), false);
  assert.equal(canRunSource("portugol-rs", false), false);
  assert.equal(canRunSource("portugol-rs", true), true);
  assert.equal(canDebugSource("portugol-rs"), false);
});

test("independent Rust files remain unsupported by the IDE runtime", () => {
  assert.equal(canRunSource("rust", true), false);
  assert.equal(canDebugSource("rust"), false);
});
