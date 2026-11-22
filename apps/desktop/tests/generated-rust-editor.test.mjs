import assert from "node:assert/strict";
import { test } from "node:test";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import {
  createGeneratedRustEditorState,
  generatedRustDecorations,
  generatedRustMappings,
  mappedSourceOffsetAt,
  mappedSourceOffsetAtSelection,
} from "../src/generated-rust-editor.ts";

test("generated Rust editor state is read-only and preserves pure Rust source", () => {
  const state = createGeneratedRustEditorState("fn main() { println!(\"oi\"); }");

  assert.equal(state.doc.toString(), 'fn main() { println!("oi"); }');
  assert.equal(state.facet(EditorState.readOnly), true);
  assert.equal(state.facet(EditorView.editable), false);
  assert.ok(state.facet(EditorView.contentAttributes).some((attributes) => attributes["aria-keyshortcuts"] === "F12"));
  assert.ok(state.languageDataAt("commentTokens", 0).some(({ line }) => line === "//"));
});

test("generated Rust mappings decorate valid code ranges with source offsets", () => {
  const state = createGeneratedRustEditorState("let idade = 16;");
  const updated = state.update({
    effects: generatedRustMappings.of([
      { from: 4, to: 9, sourceOffset: 22, sourceLine: 3, generatedLine: 1, quality: "exact" },
      { from: -4, to: 2, sourceOffset: -1, sourceLine: 0, generatedLine: 0, quality: "exact" },
      { from: 5, to: 30, sourceOffset: 99, sourceLine: 4, generatedLine: 1, quality: "approximate" },
    ]),
  }).state;
  assert.deepEqual(updated.field(generatedRustDecorations), [
    { from: 4, to: 9, sourceOffset: 22, sourceLine: 3, generatedLine: 1, quality: "exact" },
  ]);
  const changed = updated.update({ changes: { from: 0, to: updated.doc.length, insert: "fn main() {}" } }).state;
  assert.deepEqual(changed.field(generatedRustDecorations), [], "segmentos antigos não sobrevivem à troca do documento");
});

test("mapped Rust positions navigate only inside a valid mapped range", () => {
  const mappings = [
    { from: 4, to: 9, sourceOffset: 22, sourceLine: 3, generatedLine: 1, quality: "exact" },
  ];

  assert.equal(mappedSourceOffsetAt(mappings, 4), 22);
  assert.equal(mappedSourceOffsetAt(mappings, 8), 22);
  assert.equal(mappedSourceOffsetAt(mappings, 9), null);
  assert.equal(mappedSourceOffsetAt(mappings, 2), null);
});

test("F12 resolves the source offset for the generated editor selection", () => {
  const state = createGeneratedRustEditorState("let idade = 16;");
  const mapped = state.update({
    effects: generatedRustMappings.of([
      { from: 4, to: 9, sourceOffset: 22, sourceLine: 3, generatedLine: 1, quality: "exact" },
    ]),
    selection: { anchor: 6 },
  }).state;
  const unmapped = mapped.update({ selection: { anchor: 12 } }).state;

  assert.equal(mappedSourceOffsetAtSelection(mapped), 22);
  assert.equal(mappedSourceOffsetAtSelection(unmapped), null);
});
