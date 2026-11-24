import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { CompletionContext } from "@codemirror/autocomplete";
import { portugolRs } from "../src/portugol.ts";

test("PORS highlighting recognizes the implemented while and counted loops", () => {
  const source = "enquanto para pare continue";
  const state = EditorState.create({ doc: source, extensions: [portugolRs()] });
  const keywords = [];

  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "keyword") keywords.push(state.doc.sliceString(node.from, node.to));
    },
  });

  assert.deepEqual(keywords, ["enquanto", "para", "pare", "continue"]);
});

test("PORS highlights and completes return statements", async () => {
  const state = EditorState.create({ doc: "retorne", extensions: [portugolRs()] });
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

test("PORS autocomplete suggests supported loop controls without promising unsupported ones", async () => {
  const state = EditorState.create({ doc: "", extensions: [portugolRs()] });
  const [source] = state.languageDataAt("autocomplete", 0);
  const result = await source(new CompletionContext(state, 0, true));
  const labels = result.options.map(({ label }) => label);

  assert.ok(labels.includes("enquanto"));
  assert.ok(labels.includes("para"));
  assert.ok(labels.includes("pare"));
  assert.ok(labels.includes("continue"));
  assert.ok(!labels.includes("faca"));
});

test("PORS highlights leia and completes it as a supported function", async () => {
  const source = "funcao inicio() { seja mutavel idade: inteiro = 0; leia(idade); }";
  const state = EditorState.create({ doc: source, extensions: [portugolRs()] });
  const builtins = [];
  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "variableName.standard") builtins.push(state.doc.sliceString(node.from, node.to));
    },
  });
  const completionState = EditorState.create({ doc: "lei", extensions: [portugolRs()] });
  const [completion] = completionState.languageDataAt("autocomplete", 3);
  const result = await completion(new CompletionContext(completionState, 3, true));
  assert.deepEqual(builtins, ["leia"]);
  assert.ok(result.options.some(({ label, apply }) => label === "leia" && apply === "leia("));
});

test("PORS autocomplete offers a chained senao se conditional", async () => {
  const state = EditorState.create({ doc: "", extensions: [portugolRs()] });
  const [completion] = state.languageDataAt("autocomplete", 0);
  const result = await completion(new CompletionContext(state, 0, true));

  assert.ok(result.options.some(({ label }) => label === "se senao se"));
});

test("PORS highlighting recognizes each keyword in a chained senao se conditional", () => {
  const source = "se (verdadeiro) {} senao se (falso) {} senao {}";
  const state = EditorState.create({ doc: source, extensions: [portugolRs()] });
  const keywords = [];

  syntaxTree(state).iterate({
    enter(node) {
      if (node.name === "keyword") keywords.push(state.doc.sliceString(node.from, node.to));
    },
  });

  assert.deepEqual(keywords, ["se", "senao", "se", "senao"]);
});
