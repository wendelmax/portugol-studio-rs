import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("cargo run resolves workspace binaries and requires an explicit target when needed", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const service = readFileSync(new URL("../../../crates/portugol-service/src/cargo_project.rs", import.meta.url), "utf8");

  assert.match(html, /id="cargo-target-dialog"[\s\S]*id="cargo-target-select"[\s\S]*value="run"/);
  assert.match(api, /cargoProjectRunTargets = \(id: number, workspaceId: string, relativeSource: string\)[\s\S]*invoke<CargoRunTargets>\("cargo_project_run_targets"/);
  assert.match(api, /runCargoProject[\s\S]*target: CargoRunTarget \| null[\s\S]*args: string\[\]/);
  assert.match(main, /cargoProjectRunTargets\(discoveryId, tab\.workspaceId, tab\.workspacePath\)[\s\S]*defaultTarget/);
  assert.match(main, /if \(tab\.cargoProject === true && cargoAvailable !== true\) \{[\s\S]*Instale Cargo/);
  assert.match(main, /chooseCargoTarget/);
  assert.match(main, /cargoTarget = target/);
  assert.match(tauri, /cargo_project_run_targets[\s\S]*rust_source_cancellations[\s\S]*cargo_run_targets_with_cancel/);
  assert.match(tauri, /async fn run_cargo_project[\s\S]*args: Vec<String>/);
  assert.match(tauri, /run_cargo_project_target_with_args_and_cancel/);
  assert.match(service, /\.arg\("--package"\)[\s\S]*\.arg\("--bin"\)[\s\S]*\.arg\("--"\)\.args\(args\)/);
});
