import test from "node:test";
import assert from "node:assert/strict";
import { debugMappingText } from "../src/run/debug-view.ts";

const info = {
  position: { offset: 0, line: 1, column: 1 },
  stack: [], variables: [],
  mapping: { hirNode: "Read", hirStart: { offset: 20, line: 2, column: 3 },
             hirEnd: { offset: 26, line: 2, column: 9 }, bytecodeOffset: 7 },
};
const links = [{ sourceLine: 2, generatedLine: 9, sourceOffset: 18, generatedOffset: 300, quality: "exact" }];

test("debugMappingText shows HIR node, bytecode offset and generated Rust line", () => {
  assert.equal(debugMappingText(info, links), "HIR Read@20..26 · bytecode 7 · Rust linha 9");
});

test("debugMappingText does not invent a Rust line without a mapped segment", () => {
  assert.equal(debugMappingText(info, [{ ...links[0], sourceOffset: 40, generatedLine: 99 }]),
    "HIR Read@20..26 · bytecode 7 · Rust linha —");
  assert.equal(debugMappingText({ ...info, mapping: null }, links), null);
});
