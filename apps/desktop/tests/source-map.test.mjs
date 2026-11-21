import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { mappedPreviewSegments, sourceDiagnostic } from "../src/source-map-ui.ts";

const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
const ui = readFileSync(new URL("../src/source-map-ui.ts", import.meta.url), "utf8");

test("Rust export API exposes generated and source editor ranges", () => {
  assert.match(api, /interface SourceMapInfo[\s\S]*generatedStart: Position[\s\S]*generatedEnd: Position[\s\S]*sourceStart: Position \| null[\s\S]*sourceEnd: Position \| null[\s\S]*quality: "exact" \| "approximate" \| "generated_only"/);
  assert.match(api, /interface RustExport[\s\S]*sourceMap: SourceMapInfo\[\]/);
});

test("PORS run API distinguishes execution, compiler diagnostics and runtime limits", () => {
  assert.match(api, /status: "executed"[\s\S]*status: "compile_error"[\s\S]*diagnostics: PortugolRsCompileDiagnostic\[\][\s\S]*status: "runtime_limit"[\s\S]*reason: "time" \| "output" \| "string_memory"/);
});

test("mapped diagnostics and preview segments require source mappings and label approximate locations", () => {
  assert.match(ui, /if \(!diagnostic\.sourceStart \|\| !diagnostic\.sourceEnd\) return null/);
  assert.match(ui, /localização aproximada/);
  assert.match(ui, /mappedPreviewSegments/);
});

test("compile diagnostics navigate mapped locations and leave generated-only errors unmapped", () => {
  const sourceStart = { offset: 8, line: 2, column: 3 };
  const sourceEnd = { offset: 13, line: 2, column: 8 };
  const diagnostic = {
    severity: "error", code: "E0308", message: "tipos incompatíveis", rendered: "error E0308",
    generatedStart: { offset: 20, line: 2, column: 1 }, generatedEnd: { offset: 25, line: 2, column: 6 },
    sourceStart, sourceEnd, quality: "approximate",
  };
  assert.equal(sourceDiagnostic(diagnostic).start, sourceStart);
  assert.match(sourceDiagnostic(diagnostic).message, /aproximada/);
  assert.equal(sourceDiagnostic({ ...diagnostic, sourceStart: null, sourceEnd: null, quality: "generated_only" }), null);
});

test("Rust preview mappings use UTF-16 generated offsets and retain explicit source targets", () => {
  const segments = mappedPreviewSegments("fn 😀() {}", [{
    generatedStart: { offset: 3, line: 1, column: 4 }, generatedEnd: { offset: 7, line: 1, column: 8 },
    sourceStart: { offset: 4, line: 2, column: 1 }, sourceEnd: { offset: 8, line: 2, column: 5 }, quality: "exact",
  }]);
  assert.deepEqual(segments, [{ from: 3, to: 7, sourceOffset: 4, sourceLine: 2, quality: "exact" }]);
});

test("overlapping preview mappings prefer the narrowest source segment", () => {
  const position = (offset) => ({ offset, line: 1, column: offset + 1 });
  const segments = mappedPreviewSegments("0123456789", [
    { generatedStart: position(1), generatedEnd: position(9), sourceStart: position(10), sourceEnd: position(20), quality: "approximate" },
    { generatedStart: position(3), generatedEnd: position(6), sourceStart: position(30), sourceEnd: position(33), quality: "exact" },
  ]);
  assert.deepEqual(segments, [{ from: 3, to: 6, sourceOffset: 30, sourceLine: 1, quality: "exact" }]);
});

test("PORS run response is discarded when its tab or document revision changes", () => {
  assert.match(main, /activeTab\(\) !== tab \|\| view\.state\.doc !== documentState/);
  assert.match(main, /result\.status === "executed"/);
  assert.match(main, /result\.status === "runtime_limit"/);
  assert.match(main, /tempo máximo de 3 segundos/);
  assert.match(main, /limite de saída de 1 MiB/);
  assert.match(main, /limite de memória para cadeias de 16 MiB/);
  assert.match(main, /result\.diagnostics\.map\(sourceDiagnostic\)/);
  assert.match(main, /mapped\.onclick = \(\) => goTo\(segment\.sourceOffset\)/);
});
