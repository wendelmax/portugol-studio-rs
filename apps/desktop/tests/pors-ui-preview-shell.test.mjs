import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("PORS visual preview shell owns DOM and api.ts adapters", async () => {
  const shell = await readFile(new URL("../src/pors-ui-preview-shell.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(shell, /mountPorsUiPreviewPanel\(/);
  assert.match(shell, /api\.porsUiStart/);
  assert.match(shell, /api\.porsUiDispatch/);
  assert.match(shell, /api\.porsUiCancel/);
  assert.match(shell, /api\.porsUiClose/);
  assert.match(main, /mountPorsUiPreviewShell\(/);
  assert.doesNotMatch(main, /mountPorsUiPreviewPanel\(/);
});
