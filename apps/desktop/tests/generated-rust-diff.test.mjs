import assert from "node:assert/strict";
import { test } from "node:test";
import { diffGeneratedRust, generatedRustDiffSummary } from "../src/generated-rust-diff.ts";

test("diff keeps unchanged lines as context and reports a replacement", () => {
  assert.deepEqual(diffGeneratedRust("fn main() {\n    old();\n}", "fn main() {\n    new();\n}"), {
    rows: [
      { kind: "context", oldLine: 1, newLine: 1, text: "fn main() {" },
      { kind: "removed", oldLine: 2, newLine: null, text: "    old();" },
      { kind: "added", oldLine: null, newLine: 2, text: "    new();" },
      { kind: "context", oldLine: 3, newLine: 3, text: "}" },
    ],
    added: 1,
    removed: 1,
    coarse: false,
    truncated: false,
  });
});

test("diff identifies inserted and removed lines with their source line numbers", () => {
  const inserted = diffGeneratedRust("a\nc", "a\nb\nc");
  assert.deepEqual(inserted.rows, [
    { kind: "context", oldLine: 1, newLine: 1, text: "a" },
    { kind: "added", oldLine: null, newLine: 2, text: "b" },
    { kind: "context", oldLine: 2, newLine: 3, text: "c" },
  ]);

  const removed = diffGeneratedRust("a\nb\nc", "a\nc");
  assert.deepEqual(removed.rows, [
    { kind: "context", oldLine: 1, newLine: 1, text: "a" },
    { kind: "removed", oldLine: 2, newLine: null, text: "b" },
    { kind: "context", oldLine: 3, newLine: 2, text: "c" },
  ]);
});

test("diff treats an empty document as one addressable line and identical input as context", () => {
  assert.deepEqual(diffGeneratedRust("", ""), {
    rows: [{ kind: "context", oldLine: 1, newLine: 1, text: "" }],
    added: 0,
    removed: 0,
    coarse: false,
    truncated: false,
  });
});

test("diff bounds its LCS allocation and labels the aggregated fallback", () => {
  const before = Array(1_000).fill("old").join("\n");
  const after = Array(1_000).fill("new").join("\n");
  const diff = diffGeneratedRust(before, after);

  assert.equal(diff.coarse, true);
  assert.equal(diff.removed, 1_000);
  assert.equal(diff.added, 1_000);
  assert.equal(diff.rows.length, 2_000);
  assert.equal(diff.truncated, false);
  assert.deepEqual(diff.rows[0], { kind: "removed", oldLine: 1, newLine: null, text: "old" });
  assert.deepEqual(diff.rows.at(-1), { kind: "added", oldLine: null, newLine: 1_000, text: "new" });
});

test("diff summary uses natural Portuguese singular and reports coarse or truncated output", () => {
  assert.equal(generatedRustDiffSummary(diffGeneratedRust("a", "a\nb")), "1 linha adicionada, 0 linhas removidas.");

  const large = Array(1_000).fill("x").join("\n");
  const coarse = diffGeneratedRust(large, Array(1_000).fill("different").join("\n"));
  assert.match(generatedRustDiffSummary(coarse), /Trecho agrupado/);

  const ordered = Array.from({ length: 1_000 }, (_, index) => `linha ${index}`);
  const reordered = [...ordered.slice(1), ordered[0]];
  const aggregated = diffGeneratedRust(ordered.join("\n"), reordered.join("\n"));
  const aggregateSummary = generatedRustDiffSummary(aggregated);
  assert.match(aggregateSummary, /pode incluir linhas sem alteração/);
  assert.doesNotMatch(aggregateSummary, /1000 linhas adicionadas, 1000 linhas removidas/);

  const manyLines = Array(6_000).fill("x").join("\n");
  const truncated = diffGeneratedRust(manyLines, "different\n" + Array(6_000).fill("y").join("\n"));
  assert.match(generatedRustDiffSummary(truncated), /Exibição limitada às primeiras linhas/);
});
