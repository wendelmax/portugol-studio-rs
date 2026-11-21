import test from "node:test";
import assert from "node:assert/strict";
import { detectSourceLanguage, ensureSourceExtension } from "../src/document-language.ts";

test("detects Portugol-RS from case-insensitive .pors paths", () => {
  assert.equal(detectSourceLanguage("/tmp/Aula álgebra.PORS"), "portugol-rs");
});

test("keeps classic Portugol as the default for other paths", () => {
  assert.equal(detectSourceLanguage("/tmp/programa.por"), "portugol");
  assert.equal(detectSourceLanguage("/tmp/lib.rs"), "rust");
  assert.equal(detectSourceLanguage("/tmp/programa.txt"), "portugol");
});

test("adds the correct source extension for Rust documents", () => {
  assert.equal(ensureSourceExtension("/tmp/lib", "rust"), "/tmp/lib.rs");
  assert.equal(ensureSourceExtension("/tmp/lib.rs", "rust"), "/tmp/lib.rs");
});

test("adds the active language extension without changing an explicit source suffix", () => {
  assert.equal(ensureSourceExtension("/tmp/Olá", "portugol-rs"), "/tmp/Olá.pors");
  assert.equal(ensureSourceExtension("/tmp/Olá.PORS", "portugol"), "/tmp/Olá.PORS");
  assert.equal(ensureSourceExtension("/tmp/aula", "portugol"), "/tmp/aula.por");
});
