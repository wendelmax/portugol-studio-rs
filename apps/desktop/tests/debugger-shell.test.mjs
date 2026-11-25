import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("debugger shell composes command and pause controllers through injected state", async () => {
  const shell = await readFile(new URL("../src/run/debugger-shell.ts", import.meta.url), "utf8");
  const presentation = await readFile(new URL("../src/run/execution-presentation.ts", import.meta.url), "utf8");
  const runShell = await readFile(new URL("../src/run/run-shell.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(shell, /createDebuggerCommands\(dependencies\)/);
  assert.match(shell, /createPausePresentation\(/);
  assert.match(shell, /syncBreakpoints: debuggerCommands\.syncBreakpoints/);
  assert.match(shell, /setPaused: pausePresentation\.setPaused/);
  assert.match(presentation, /createDebuggerShell\(/);
  assert.match(presentation, /createRunControls\(/);
  assert.match(main, /createRunShell\(/);
  assert.match(runShell, /createExecutionPresentationShell\(/);
  assert.doesNotMatch(main, /createDebuggerShell\(/);
  assert.doesNotMatch(main, /createRunControls\(/);
  assert.doesNotMatch(main, /const debuggerCommands = createDebuggerCommands\(/);
  assert.doesNotMatch(main, /const pausePresentation = createPausePresentation\(/);
});
