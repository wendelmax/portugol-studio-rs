import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("encaminha projetos Cargo por comandos Tauri tipados e consentimento explícito", () => {
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const workspace = readFileSync(new URL("../src-tauri/src/workspace.rs", import.meta.url), "utf8");
  const runner = readFileSync(new URL("../../../crates/portugol-service/src/cargo_project.rs", import.meta.url), "utf8");

  assert.match(api, /runCargoProject = \(id: number, workspaceId: string, relativeSource: string\)/);
  assert.match(main, /api\.runCargoProject\(id, tab\.workspaceId, tab\.workspacePath\)/);
  assert.match(main, /if \(tab\.workspaceId && tab\.workspacePath && tab\.cargoProject === undefined\)/);
  assert.match(main, /Cargo pode acessar a rede e executar scripts de build/);
  assert.match(main, /candidate\.workspaceId === tab\.workspaceId[\s\S]*isDirty\(candidate\)/);
  assert.match(tauri, /workspace::cargo_project_for_source[\s\S]*run_cargo_project_with_cancel/);
  assert.match(workspace, /cargo_project_for_source[\s\S]*checked_file/);
  assert.match(runner, /Command::new\("cargo"\)[\s\S]*\.arg\("build"\)[\s\S]*\.arg\("--manifest-path"\)/);
  assert.match(runner, /configure_process_tree\(&mut run\)/);
  assert.doesNotMatch(runner, /Command::new\(args|sh -c/);
});
