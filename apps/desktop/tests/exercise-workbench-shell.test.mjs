import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("exercise and test panel composition lives outside main", async () => {
  const shell = await readFile(new URL("../src/learning/exercise-workbench.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(shell, /createExerciseTabOpener\(/);
  assert.match(shell, /mountExercises\(/);
  assert.match(shell, /createTestsPanelController\(/);
  assert.match(main, /createExerciseWorkbenchShell\(/);
  assert.doesNotMatch(main, /createExerciseTabOpener\(/);
  assert.doesNotMatch(main, /createTestsPanelController\(/);
  assert.doesNotMatch(main, /mountExercises\(/);
});
