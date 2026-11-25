import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("tab activation shell composes activation adapters outside main", async () => {
  const shell = await readFile(new URL("../src/state/tab-activation-shell.ts", import.meta.url), "utf8");
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");

  assert.match(shell, /createTabActivator\(/);
  assert.match(main, /createTabActivationShell\(/);
  assert.doesNotMatch(main, /const tabActivator = createTabActivator\(/);
});
