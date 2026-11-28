import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { ferroRust } from "../src/ferro-language.ts";

function tokens(doc) {
  const state = EditorState.create({ doc, extensions: [ferroRust()] });
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

test("FERRO highlights interpolated parts with exact UTF-8 spans", () => {
  assert.deepEqual(tokens('"Olá, {nome}!"'), [
    ["string", '"Olá, '],
    ["interpolation", "{"],
    ["variableName", "nome"],
    ["interpolation", "}"],
    ["string", '!"'],
  ]);
});

test("FERRO keeps escaped and malformed braces as literal string", () => {
  for (const doc of [String.raw`escreval("\{nome}");`, 'escreval("{}");', 'escreval("abri {");']) {
    assert.ok(tokens(doc).every(([name]) => name !== "interpolation"), doc);
  }
});

test("FERRO treats the first unescaped brace as interpolation even across nested braces and lines", () => {
  for (const [doc, expected] of [
    ['escreval("{a{b}");', ["a"]],
    ['escreval("{\n nome \n}");', ["nome"]],
  ]) {
    const found = tokens(doc);
    assert.deepEqual(
      found.filter(([name]) => name === "variableName").map(([, text]) => text.trim()),
      expected,
      doc,
    );
    assert.ok(found.some(([name]) => name === "interpolation"), doc);
  }
});
