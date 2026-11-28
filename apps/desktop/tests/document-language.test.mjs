import test from "node:test";
import assert from "node:assert/strict";
import { detectSourceLanguage, ensureSourceExtension } from "../src/document-language.ts";

test("detects Ferro from case-insensitive .fer paths and rejects removed .fer", () => {
  assert.equal(detectSourceLanguage("/tmp/Aula álgebra.FER"), "ferro-rust");
  assert.equal(ensureSourceExtension("/tmp/Aula.fer", "ferro-rust"), "/tmp/Aula.fer");
});

test("keeps Ferro as the default for non-Rust paths", () => {
  assert.equal(detectSourceLanguage("/tmp/programa.fer"), "ferro-rust");
  assert.equal(detectSourceLanguage("/tmp/lib.rs"), "rust");
  assert.equal(detectSourceLanguage("/tmp/programa.txt"), "ferro-rust");
});

test("adds the correct source extension for Rust documents", () => {
  assert.equal(ensureSourceExtension("/tmp/lib", "rust"), "/tmp/lib.rs");
  assert.equal(ensureSourceExtension("/tmp/lib.rs", "rust"), "/tmp/lib.rs");
});

test("adds the active language extension without changing an explicit source suffix", () => {
  assert.equal(ensureSourceExtension("/tmp/Olá", "ferro-rust"), "/tmp/Olá.fer");
  assert.equal(ensureSourceExtension("/tmp/Olá.FER", "ferro-rust"), "/tmp/Olá.FER");
  assert.equal(ensureSourceExtension("/tmp/aula.rs", "ferro-rust"), "/tmp/aula.rs");
});

test("detects desktop project markup and keeps its extension", () => {
  assert.equal(detectSourceLanguage("/tmp/App/JanelaPrincipal.design"), "markup");
  assert.equal(detectSourceLanguage("/tmp/App/meu-app.FERPROJ"), "markup");
  assert.equal(ensureSourceExtension("/tmp/App/Tela", "markup"), "/tmp/App/Tela.design");
  assert.equal(ensureSourceExtension("/tmp/App/meu-app.ferproj", "markup"), "/tmp/App/meu-app.ferproj");
  assert.equal(ensureSourceExtension("/tmp/App/Tela.design", "ferro-rust"), "/tmp/App/Tela.design");
});
