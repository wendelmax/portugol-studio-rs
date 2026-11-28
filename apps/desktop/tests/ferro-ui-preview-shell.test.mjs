import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("FERRO visual preview shell owns DOM and api.ts adapters", async () => {
  const shell = await readFile(new URL("../src/ferro-ui-preview-shell.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(shell, /mountFerroUiPreviewPanel\(/);
  assert.match(shell, /api\.ferroUiStart/);
  assert.match(shell, /api\.ferroUiDispatch/);
  assert.match(shell, /api\.ferroUiCancel/);
  assert.match(shell, /api\.ferroUiClose/);
  assert.match(main, /mountFerroUiPreviewShell\(/);
  assert.doesNotMatch(main, /mountFerroUiPreviewPanel\(/);
});
