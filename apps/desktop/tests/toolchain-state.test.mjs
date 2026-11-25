import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { stripTypeScriptTypes } from "node:module";

async function loadToolchainState() {
  const source = await readFile(new URL("../src/shell/toolchain-state.ts", import.meta.url), "utf8");
  return import(`data:text/javascript;base64,${Buffer.from(stripTypeScriptTypes(source)).toString("base64")}`);
}

test("toolchain availability starts unknown and records independent checks", async () => {
  const { createToolchainState } = await loadToolchainState();
  const state = createToolchainState();

  assert.deepEqual(state.snapshot(), { rustc: null, cargo: null, rustfmt: null });

  state.setRustcAvailable(true);
  state.setCargoAvailable(false);
  state.setRustfmtAvailable(true);

  assert.deepEqual(state.snapshot(), { rustc: true, cargo: false, rustfmt: true });
});
