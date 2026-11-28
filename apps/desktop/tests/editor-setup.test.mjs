import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const setupUrl = new URL("../src/editor/setup.ts", import.meta.url);
const mainUrl = new URL("../src/main.ts", import.meta.url);
const shellUrl = new URL("../src/editor/editor-shell.ts", import.meta.url);

test("editor setup owns shared compartments, linters, centering, and breakpoint gutter", async () => {
  const setup = await readFile(setupUrl, "utf8");
  const main = await readFile(mainUrl, "utf8");
  const shell = await readFile(shellUrl, "utf8");

  assert.match(setup, /export function createEditorSetup/);
  assert.match(setup, /new Compartment\(\)/);
  assert.match(setup, /createCenterSourceController/);
  assert.match(setup, /createBreakpointGutter/);
  assert.match(setup, /createFerroLintSourceCheck/);
  assert.match(setup, /createRustLintSourceCheck/);
  assert.match(main, /createEditorShell\(/);
  assert.doesNotMatch(main, /createEditorSetup\(|new EditorView\(/);
  assert.match(shell, /createEditorSetup\(/);
  assert.match(shell, /createEditorUpdateController\(/);
  assert.match(shell, /createEditorExtensionFactory\(/);
  assert.match(shell, /createEditorThemeController\(/);
  assert.doesNotMatch(main, /const ferroLinter = linter\(/);
  assert.doesNotMatch(main, /const rustSourceLinter = linter\(/);
});
