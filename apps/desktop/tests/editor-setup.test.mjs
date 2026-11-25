import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const setupUrl = new URL("../src/editor/setup.ts", import.meta.url);
const mainUrl = new URL("../src/main.ts", import.meta.url);

test("editor setup owns shared compartments, linters, centering, and breakpoint gutter", async () => {
  const setup = await readFile(setupUrl, "utf8");
  const main = await readFile(mainUrl, "utf8");

  assert.match(setup, /export function createEditorSetup/);
  assert.match(setup, /new Compartment\(\)/);
  assert.match(setup, /createCenterSourceController/);
  assert.match(setup, /createBreakpointGutter/);
  assert.match(setup, /createPortugolLintSourceCheck/);
  assert.match(setup, /createRustLintSourceCheck/);
  assert.match(main, /createEditorSetup\(/);
  assert.doesNotMatch(main, /const portugolLinter = linter\(/);
  assert.doesNotMatch(main, /const rustSourceLinter = linter\(/);
});
