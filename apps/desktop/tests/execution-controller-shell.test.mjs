import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("execution controller wiring lives in a dedicated shell", async () => {
  const shell = await readFile(new URL("../src/run/execution-controller-shell.ts", import.meta.url), "utf8");
  const runShell = await readFile(new URL("../src/run/run-shell.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(shell, /createRustExecutionController\(/);
  assert.match(shell, /createPortugolRsExecutionController\(/);
  assert.match(shell, /createClassicExecutionController\(/);
  assert.match(shell, /createCargoProjectController\(/);
  assert.match(shell, /createRunDispatcher\(/);
  assert.match(main, /createRunShell\(/);
  assert.match(runShell, /createExecutionControllerShell\(/);
  assert.doesNotMatch(main, /createRustExecutionController\(/);
  assert.doesNotMatch(main, /createPortugolRsExecutionController\(/);
  assert.doesNotMatch(main, /createClassicExecutionController\(/);
  assert.doesNotMatch(main, /createCargoProjectController\(/);
  assert.doesNotMatch(main, /createRunDispatcher\(/);
});
