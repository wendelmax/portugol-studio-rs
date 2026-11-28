import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { CompletionContext } from "@codemirror/autocomplete";
import { ferroRust } from "../src/ferro-language.ts";

test("FERRO highlighting recognizes the implemented while and counted loops", () => {
  const source = "enquanto para pare continue";
  const state = EditorState.create({ doc: source, extensions: [ferroRust()] });
  const keywords = [];

  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "keyword") keywords.push(state.doc.sliceString(node.from, node.to));
    },
  });

  assert.deepEqual(keywords, ["enquanto", "para", "pare", "continue"]);
});

test("FERRO highlights and completes return statements", async () => {
  const state = EditorState.create({ doc: "retorne", extensions: [ferroRust()] });
  const keywords = [];
  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "keyword") keywords.push(state.doc.sliceString(node.from, node.to));
    },
  });
  const [completion] = state.languageDataAt("autocomplete", 7);
  const result = await completion(new CompletionContext(state, 7, true));

  assert.deepEqual(keywords, ["retorne"]);
  assert.ok(result.options.some(({ label }) => label === "retorne"));
});

test("FERRO autocomplete suggests supported loop controls without promising unsupported ones", async () => {
  const state = EditorState.create({ doc: "", extensions: [ferroRust()] });
  const [source] = state.languageDataAt("autocomplete", 0);
  const result = await source(new CompletionContext(state, 0, true));
  const labels = result.options.map(({ label }) => label);

  assert.ok(labels.includes("enquanto"));
  assert.ok(labels.includes("para"));
  assert.ok(labels.includes("pare"));
  assert.ok(labels.includes("continue"));
  assert.ok(!labels.includes("faca"));
});

test("FERRO highlights leia and completes it as a supported function", async () => {
  const source = "funcao inicio() { seja mutavel idade: inteiro = 0; leia(idade); }";
  const state = EditorState.create({ doc: source, extensions: [ferroRust()] });
  const builtins = [];
  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "variableName.standard") builtins.push(state.doc.sliceString(node.from, node.to));
    },
  });
  const completionState = EditorState.create({ doc: "lei", extensions: [ferroRust()] });
  const [completion] = completionState.languageDataAt("autocomplete", 3);
  const result = await completion(new CompletionContext(completionState, 3, true));
  assert.deepEqual(builtins, ["leia"]);
  assert.ok(result.options.some(({ label, apply }) => label === "leia" && apply === "leia("));
});

test("FERRO autocomplete offers a chained senao se conditional", async () => {
  const state = EditorState.create({ doc: "", extensions: [ferroRust()] });
  const [completion] = state.languageDataAt("autocomplete", 0);
  const result = await completion(new CompletionContext(state, 0, true));

  assert.ok(result.options.some(({ label }) => label === "se senao se"));
});

test("FERRO highlighting recognizes each keyword in a chained senao se conditional", () => {
  const source = "se (verdadeiro) {} senao se (falso) {} senao {}";
  const state = EditorState.create({ doc: source, extensions: [ferroRust()] });
  const keywords = [];

  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "keyword") keywords.push(state.doc.sliceString(node.from, node.to));
    },
  });

  assert.deepEqual(keywords, ["se", "senao", "se", "senao"]);
});
