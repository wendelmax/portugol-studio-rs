import test from "node:test";
import assert from "node:assert/strict";
import { canDebugSource, canFormatSource, canRunSource } from "../src/runtime-capabilities.ts";

test("classic Portugol uses its existing runtime and debugger", () => {
  assert.equal(canRunSource("portugol", null), true);
  assert.equal(canDebugSource("portugol"), true);
  assert.equal(canFormatSource("portugol", null), true);
});

test("Portugol-RS can run only when rustc is available and stays non-debuggable", () => {
  assert.equal(canRunSource("portugol-rs", null), false);
  assert.equal(canRunSource("portugol-rs", false), false);
  assert.equal(canRunSource("portugol-rs", true), true);
  assert.equal(canDebugSource("portugol-rs"), false);
  assert.equal(canFormatSource("portugol-rs", true), false);
});

test("independent Rust files require rustc and remain non-debuggable", () => {
  assert.equal(canRunSource("rust", null), false);
  assert.equal(canRunSource("rust", false), false);
  assert.equal(canRunSource("rust", true), true);
  assert.equal(canDebugSource("rust"), false);
  assert.equal(canFormatSource("rust", null), false);
  assert.equal(canFormatSource("rust", false), false);
  assert.equal(canFormatSource("rust", true), true);
});
