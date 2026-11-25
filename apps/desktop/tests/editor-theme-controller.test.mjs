import test from "node:test";
import assert from "node:assert/strict";
import { createEditorThemeController } from "../src/editor/theme-controller.ts";

function fixture() {
  const calls = [];
  let themeIndex = 0;
  let systemThemeChange;
  const controller = createEditorThemeController({
    sourceEditor: { dispatch: ({ effects }) => calls.push(`source:${effects}`) },
    sourceTheme: { reconfigure: (theme) => `source-theme:${theme}` },
    generatedEditor: { dispatch: ({ effects }) => calls.push(`generated:${effects}`) },
    generatedTheme: { reconfigure: (theme) => `generated-theme:${theme}` },
    editorTheme: () => `extension-${++themeIndex}`,
    refreshHelpTheme: () => calls.push("help"),
    applyPageTheme: (choice) => calls.push(`page:${choice}`),
    onSystemThemeChange: (listener) => { calls.push("subscribe"); systemThemeChange = listener; },
  });
  calls.length = 0;
  return { calls, controller, systemThemeChange };
}

test("theme changes refresh source editor, generated Rust editor, and help", () => {
  const { calls, systemThemeChange } = fixture();

  systemThemeChange();

  assert.deepEqual(calls, ["source:source-theme:extension-1", "generated:generated-theme:extension-2", "help"]);
});

test("selected page theme applies globally and refreshes all editor surfaces", () => {
  const { calls, controller } = fixture();

  controller.applyTheme("solarized-dark");

  assert.deepEqual(calls, [
    "page:solarized-dark",
    "source:source-theme:extension-1",
    "generated:generated-theme:extension-2",
    "help",
  ]);
});
