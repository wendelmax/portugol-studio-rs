import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("symbol editor actions compose rename and navigation outside main", async () => {
  const actions = await readFile(new URL("../src/editor/symbol-editor-actions.ts", import.meta.url), "utf8");
  const shell = await readFile(new URL("../src/editor/actions-shell.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(actions, /createSymbolRenamer\(/);
  assert.match(actions, /createSymbolNavigator\(/);
  assert.match(actions, /renameAt:.*openAt/s);
  assert.match(actions, /goToDeclarationAt:.*goToDeclarationAt/s);
  assert.match(shell, /createSymbolEditorActions\(/);
  assert.match(shell, /createEditorFormatter\(/);
  assert.match(main, /createEditorActionsShell\(/);
  assert.doesNotMatch(main, /createSymbolEditorActions\(/);
  assert.doesNotMatch(main, /createEditorFormatter\(/);
  assert.doesNotMatch(main, /createSymbolRenamer\(/);
  assert.doesNotMatch(main, /createSymbolNavigator\(/);
});
