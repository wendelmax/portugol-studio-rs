import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { ideSource } from "./support/ide-source.mjs";

test("encaminha projetos Cargo por comandos Tauri tipados e consentimento explícito", () => {
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();
  const rustExecution = readFileSync(new URL("../src/run/rust-execution.ts", import.meta.url), "utf8");
  const languageCapabilities = readFileSync(new URL("../src/state/language-capabilities.ts", import.meta.url), "utf8");
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const workspace = readFileSync(new URL("../src-tauri/src/workspace.rs", import.meta.url), "utf8");
  const runner = readFileSync(new URL("../../../crates/portugol-service/src/cargo_project.rs", import.meta.url), "utf8");

  assert.match(api, /runCargoProject =[\s\S]*target: CargoRunTarget \| null[\s\S]*args: string\[\][\s\S]*target, args/);
  assert.match(api, /cargoProjectRunTargets = \(id: number, workspaceId: string, relativeSource: string\)/);
  assert.match(main, /args\.disabled = controls\.arguments\.disabled/);
  assert.match(languageCapabilities, /disabled: pors/);
  assert.match(main, /parseArguments: parseProcessArguments/);
  assert.match(main, /runCargoProject: api\.runCargoProject/);
  assert.match(main, /cargoRunTargets: api\.cargoProjectRunTargets/);
  assert.match(main, /if \(tab\.workspaceId && tab\.workspacePath && tab\.cargoProject === undefined\)/);
  assert.match(main, /Cargo pode acessar a rede e executar scripts de build/);
  assert.match(rustExecution, /candidate\.workspaceId === tab\.workspaceId[\s\S]*dependencies\.isDirty\(candidate\)/);
  assert.match(tauri, /async fn run_cargo_project[\s\S]*args: Vec<String>/);
  assert.match(tauri, /run_cargo_project_target_with_args_and_cancel/);
  assert.match(tauri, /async fn cargo_project_run_targets[\s\S]*cargo_run_targets_with_cancel/);
  assert.match(workspace, /cargo_project_for_source[\s\S]*checked_file/);
  assert.match(runner, /Command::new\("cargo"\)[\s\S]*\.arg\("build"\)[\s\S]*\.arg\("--manifest-path"\)/);
  assert.match(runner, /configure_process_tree\(&mut run\)/);
  assert.doesNotMatch(runner, /Command::new\(args|sh -c/);
});
