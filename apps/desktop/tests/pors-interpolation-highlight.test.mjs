import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { portugolRs } from "../src/portugol.ts";

function tokens(doc) {
  const state = EditorState.create({ doc, extensions: [portugolRs()] });
  const found = [];
  syntaxTree(state).iterate({
    enter(node) {
      const slice = state.doc.sliceString(node.from, node.to);
      if (["string", "interpolation", "variableName", "number", "operator"].includes(node.name)) {
        found.push([node.name, slice]);
      }
    },
  });
  return found;
}

test("PORS highlights interpolated parts with exact UTF-8 spans", () => {
  assert.deepEqual(tokens('"Olá, {nome}!"'), [
    ["string", '"Olá, '],
    ["interpolation", "{"],
    ["variableName", "nome"],
    ["interpolation", "}"],
    ["string", '!"'],
  ]);
});

test("PORS keeps escaped and malformed braces as literal string", () => {
  for (const doc of [String.raw`escreval("\{nome}");`, 'escreval("{}");', 'escreval("abri {");']) {
    assert.ok(tokens(doc).every(([name]) => name !== "interpolation"), doc);
  }
});
