import test from "node:test";
import assert from "node:assert/strict";
import { canDebugSource, canFormatSource, canRunAsRust, canRunSource } from "../src/runtime-capabilities.ts";

test("Portugol-RS runs and debugs in the VM without rustc and runs generated Rust only when rustc is available", () => {
  assert.equal(canRunSource("portugol-rs", null), true);
  assert.equal(canRunSource("portugol-rs", false), true);
  assert.equal(canRunSource("portugol-rs", true), true);
  assert.equal(canRunAsRust("portugol-rs", null), false);
  assert.equal(canRunAsRust("portugol-rs", false), false);
  assert.equal(canRunAsRust("portugol-rs", true), true);
  assert.equal(canRunAsRust("rust", true), false);
  assert.equal(canDebugSource("portugol-rs"), true);
  assert.equal(canFormatSource("portugol-rs", null), true);
  assert.equal(canFormatSource("portugol-rs", false), true);
  assert.equal(canFormatSource("portugol-rs", true), true);
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

test("desktop markup is formatted but never run, debugged or exported to Rust", () => {
  assert.equal(canRunSource("markup", true), false);
  assert.equal(canRunAsRust("markup", true), false);
  assert.equal(canDebugSource("markup"), false);
  assert.equal(canFormatSource("markup", null), true);
});
