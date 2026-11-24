import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { breakpointLines, breakpoints, toCodeMirror, toggleBreakpoint } from "../src/editor/decorations.ts";

const stateWith = (doc) => EditorState.create({ doc, extensions: [breakpoints] });
const lineStart = (state, line) => state.doc.line(line).from;

test("breakpoints toggle on and off and are reported as sorted line numbers", () => {
  let state = stateWith("a\nb\nc\nd\n");
  state = state.update({ effects: [toggleBreakpoint.of(lineStart(state, 3)), toggleBreakpoint.of(lineStart(state, 1))] }).state;
  assert.deepEqual(breakpointLines(state), [1, 3]);
  state = state.update({ effects: toggleBreakpoint.of(lineStart(state, 3)) }).state;
  assert.deepEqual(breakpointLines(state), [1]);
});

test("breakpoints follow the code when lines are inserted above them", () => {
  let state = stateWith("a\nb\nc\n");
  state = state.update({ effects: toggleBreakpoint.of(lineStart(state, 2)) }).state;
  state = state.update({ changes: { from: 0, insert: "novo\noutro\n" } }).state;
  assert.deepEqual(breakpointLines(state), [4]);
});

test("diagnostics are clamped to the document and keep the help text", () => {
  const info = {
    severity: "error",
    code: "pors.type",
    message: "Tipos incompatíveis",
    help: "Converta o valor",
    start: { offset: 4, line: 1, column: 5 },
    end: { offset: 4, line: 1, column: 5 },
  };
  assert.deepEqual(toCodeMirror(info, 10), {
    from: 4,
    to: 5,
    severity: "error",
    source: "pors.type",
    message: "Tipos incompatíveis\nAjuda: Converta o valor",
  });
  const beyond = toCodeMirror({ ...info, help: null, start: { ...info.start, offset: 50 }, end: { ...info.end, offset: 60 } }, 10);
  assert.equal(beyond.from, 10);
  assert.equal(beyond.to, 10);
  assert.equal(beyond.message, "Tipos incompatíveis");
});
