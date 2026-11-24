import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { ideSource } from "./support/ide-source.mjs";

test("cargo check is explicit, workspace-scoped, and mapped to the active editor", () => {
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const service = readFileSync(new URL("../../../crates/portugol-service/src/cargo_project.rs", import.meta.url), "utf8");

  assert.match(main, /if \(command === "Analisar projeto Cargo"\) return tab\?\.language === "rust" && tab\.cargoProject === true && cargoAvailable === true/);
  assert.match(main, /command: "Analisar projeto Cargo", matches: \(\) => false, action: checkCargoProject/);
  assert.match(main, /hasCargoExecutionConsent\(sessionStorage\)[\s\S]*rememberCargoExecutionConsent\(sessionStorage, true\)/);
  assert.match(main, /api\.checkCargoProject\(registeredId, tab\.workspaceId, tab\.workspacePath\)/);
  assert.match(main, /Cargo encontrou diagnósticos no arquivo aberto; confira o Console para detalhes do projeto\./);
  const linter = main.match(/const rustSourceLinter = linter\([\s\S]*?\n\}\);/)?.[0] ?? "";
  assert.doesNotMatch(linter, /checkCargoProject/);
  assert.match(api, /invoke<CargoProjectCheckResult>\("check_cargo_project"/);
  assert.match(tauri, /workspace::cargo_project_and_source[\s\S]*check_cargo_project_with_cancel/);
  assert.match(service, /\.arg\("check"\)[\s\S]*\.arg\("--message-format=json"\)[\s\S]*\.arg\("--manifest-path"\)/);
});
