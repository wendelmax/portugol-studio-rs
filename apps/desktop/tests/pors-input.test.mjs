import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const read = (path) => readFile(new URL(path, import.meta.url), "utf8");

test("Portugol-RS routes leia through the console input IPC", async () => {
  const [main, api, backend] = await Promise.all([
    read("../src/main.ts"),
    read("../src/api.ts"),
    read("../../../apps/desktop/src-tauri/src/lib.rs"),
  ]);

  assert.match(main, /await runListeners;[\s\S]*api\.runPortugolRs\(porsRunId, source\)/);
  assert.match(main, /input\(id, expected\)[\s\S]*console-hint[\s\S]*inputForm\.hidden = false/);
  assert.match(main, /api\.sendInput\(runId, line\)/);
  assert.match(api, /registerPortugolRsRun[\s\S]*register_portugol_rs_run/);
  assert.match(backend, /run_portugol_rs_with_io[\s\S]*\.emit\([\s\n]*"run:input"/);
  assert.match(backend, /portugol_rs_input_senders[\s\S]*sender\.send\(line\)/);
});
