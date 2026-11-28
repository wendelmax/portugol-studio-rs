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

test("numa tela, o botão da prévia abre o designer antes do painel do UI.*", async () => {
  const { readFile } = await import("node:fs/promises");
  const shell = await readFile(new URL("../src/ferro-ui-preview-shell.ts", import.meta.url), "utf8");
  const listener = shell.indexOf('toggle.addEventListener("click"');
  const panel = shell.indexOf("return mountFerroUiPreviewPanel(");
  assert.ok(listener > 0 && listener < panel, "o desvio para telas é registrado antes do painel");
  assert.match(shell, /event\.stopImmediatePropagation\(\);\n\s*void dependencies\.showScreen\?\.\(\);/);
});
