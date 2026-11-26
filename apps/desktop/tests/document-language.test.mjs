import test from "node:test";
import assert from "node:assert/strict";
import { detectSourceLanguage, ensureSourceExtension } from "../src/document-language.ts";

test("detects Portugol-RS from case-insensitive .pors paths", () => {
  assert.equal(detectSourceLanguage("/tmp/Aula álgebra.PORS"), "portugol-rs");
});

test("keeps Portugol-RS as the default for non-Rust paths", () => {
  assert.equal(detectSourceLanguage("/tmp/programa.pors"), "portugol-rs");
  assert.equal(detectSourceLanguage("/tmp/lib.rs"), "rust");
  assert.equal(detectSourceLanguage("/tmp/programa.txt"), "portugol-rs");
});

test("adds the correct source extension for Rust documents", () => {
  assert.equal(ensureSourceExtension("/tmp/lib", "rust"), "/tmp/lib.rs");
  assert.equal(ensureSourceExtension("/tmp/lib.rs", "rust"), "/tmp/lib.rs");
});

test("adds the active language extension without changing an explicit source suffix", () => {
  assert.equal(ensureSourceExtension("/tmp/Olá", "portugol-rs"), "/tmp/Olá.pors");
  assert.equal(ensureSourceExtension("/tmp/Olá.PORS", "portugol-rs"), "/tmp/Olá.PORS");
  assert.equal(ensureSourceExtension("/tmp/aula.rs", "portugol-rs"), "/tmp/aula.rs");
});
