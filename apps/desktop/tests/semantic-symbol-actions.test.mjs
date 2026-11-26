import test from "node:test";
import assert from "node:assert/strict";
import { semanticSymbolLanguage } from "../src/semantic-symbol-actions.ts";

test("semantic symbol commands are available for Portugol-RS", () => {
  assert.equal(semanticSymbolLanguage("portugol-rs"), "portugol-rs");
});

test("semantic symbol commands stay unavailable for Rust source", () => {
  assert.equal(semanticSymbolLanguage("rust"), null);
});
