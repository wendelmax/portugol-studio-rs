import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const read = (path) => readFile(new URL(path, import.meta.url), "utf8");

test("Ferro routes leia through the console input IPC", async () => {
  const [main, shell, toolbar, api, backend, execution, runEvents] = await Promise.all([
    read("../src/main.ts"),
    read("../src/run/execution-controller-shell.ts"),
    read("../src/run/toolbar.ts"),
    read("../src/api.ts"),
    read("../../../apps/desktop/src-tauri/src/lib.rs"),
    read("../src/run/portugol-rs-execution.ts"),
    read("../src/run/run-events.ts"),
  ]);

  assert.match(shell, /runGeneratedRust: api\.runPortugolRs/);
  assert.match(execution, /await dependencies\.runListeners[\s\S]*dependencies\.runGeneratedRust\(registeredId, source\)/);
  assert.match(runEvents, /input\(id, expected\)[\s\S]*dependencies\.setInputHint[\s\S]*dependencies\.showInput/);
  assert.match(main, /sendInput: api\.sendInput/);
  assert.match(toolbar, /dependencies\.sendInput\(id, line\)/);
  assert.match(api, /registerPortugolRsRun[\s\S]*register_portugol_rs_run/);
  assert.match(backend, /run_portugol_rs_with_io[\s\S]*\.emit\([\s\n]*"run:input"/);
  assert.match(backend, /portugol_rs_input_senders[\s\S]*sender\.send\(line\)/);
});
