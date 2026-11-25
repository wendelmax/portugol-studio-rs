import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("main composes the execution subsystem through the run shell", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const shell = await readFile(new URL("../src/run/run-shell.ts", import.meta.url), "utf8");

  assert.match(main, /createRunShell\(/);
  assert.doesNotMatch(main, /createExecutionPresentationShell\(|createExecutionControllerShell\(|mountRunEventsShell\(|mountRunToolbar\(/);
  assert.match(shell, /createExecutionState\(/);
  assert.match(shell, /createExecutionPresentationShell\(/);
  assert.match(shell, /createExecutionControllerShell\(/);
  assert.match(shell, /mountRunEventsShell\(/);
  assert.match(shell, /mountRunToolbar\(/);
  assert.doesNotMatch(shell, /\binvoke\s*\(/);
});
