import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { CompletionContext } from "@codemirror/autocomplete";
import { portugolRs } from "../src/portugol.ts";

test("PORS highlighting recognizes the implemented while and counted loops", () => {
  const source = "enquanto para";
  const state = EditorState.create({ doc: source, extensions: [portugolRs()] });
  const keywords = [];

  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "keyword") keywords.push(state.doc.sliceString(node.from, node.to));
    },
  });

  assert.deepEqual(keywords, ["enquanto", "para"]);
});

test("PORS autocomplete suggests supported loops without promising unsupported ones", async () => {
  const state = EditorState.create({ doc: "", extensions: [portugolRs()] });
  const [source] = state.languageDataAt("autocomplete", 0);
  const result = await source(new CompletionContext(state, 0, true));
  const labels = result.options.map(({ label }) => label);

  assert.ok(labels.includes("enquanto"));
  assert.ok(labels.includes("para"));
  assert.ok(!labels.includes("faca"));
  assert.ok(!labels.includes("pare"));
});
