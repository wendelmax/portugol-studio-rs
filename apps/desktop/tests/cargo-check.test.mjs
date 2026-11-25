import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { ideSource } from "./support/ide-source.mjs";
import { isPaletteCommandAvailable } from "../src/command-palette.ts";

test("cargo check is explicit, workspace-scoped, and mapped to the active editor", () => {
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();
  const shell = readFileSync(new URL("../src/run/execution-controller-shell.ts", import.meta.url), "utf8");
  const commandShell = readFileSync(new URL("../src/commands/ide-command-shell.ts", import.meta.url), "utf8");
  const shortcuts = readFileSync(new URL("../src/commands/ide-shortcuts.ts", import.meta.url), "utf8");
  const controller = readFileSync(new URL("../src/run/cargo-project.ts", import.meta.url), "utf8");
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const service = readFileSync(new URL("../../../crates/portugol-service/src/cargo_project.rs", import.meta.url), "utf8");

  assert.match(commandShell, /isPaletteCommandAvailable\(command, dependencies\.paletteContext\(\)\)/);
  assert.match(shortcuts, /command: "Analisar projeto Cargo", matches: \(\) => false, action: actions\.checkCargoProject/);
  assert.match(shell, /createCargoProjectController\([\s\S]*hasCargoConsent: \(\) => hasCargoExecutionConsent\(sessionStorage\)/);
  assert.match(shell, /checkCargoProject: api\.checkCargoProject/);
  assert.match(controller, /rememberCargoConsent\(\)/);
  assert.match(controller, /Cargo encontrou diagnósticos no arquivo aberto; confira o Console para detalhes do projeto\./);
  const linter = main.match(/const rustSourceLinter = linter\([\s\S]*?\n\}\);/)?.[0] ?? "";
  assert.doesNotMatch(linter, /checkCargoProject/);
  assert.match(api, /invoke<CargoProjectCheckResult>\("check_cargo_project"/);
  assert.match(tauri, /workspace::cargo_project_and_source[\s\S]*check_cargo_project_with_cancel/);
  assert.match(service, /\.arg\("check"\)[\s\S]*\.arg\("--message-format=json"\)[\s\S]*\.arg\("--manifest-path"\)/);
});

test("Cargo analysis is offered only for Rust files inside a Cargo project while idle", () => {
  const base = {
    language: "rust", cargoProject: true, cargoAvailable: true, canRunAsRust: false,
    runActive: false, paused: false, porsRunPending: false,
  };
  for (const command of ["Analisar projeto Cargo", "Testar projeto Cargo"]) {
    assert.equal(isPaletteCommandAvailable(command, base), true);
    assert.equal(isPaletteCommandAvailable(command, { ...base, cargoProject: false }), false);
    assert.equal(isPaletteCommandAvailable(command, { ...base, cargoAvailable: false }), false);
    assert.equal(isPaletteCommandAvailable(command, { ...base, runActive: true }), false);
    assert.equal(isPaletteCommandAvailable(command, { ...base, porsRunPending: true }), false);
    assert.equal(isPaletteCommandAvailable(command, { ...base, language: "portugol-rs" }), false);
    assert.equal(isPaletteCommandAvailable(command, { ...base, language: null }), false);
  }
});
