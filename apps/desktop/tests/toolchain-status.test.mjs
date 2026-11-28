import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createToolchainStatusController } from "../src/shell/toolchain-status.ts";

test("main delegates toolchain status composition to its shell", () => {
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  const shell = readFileSync(new URL("../src/shell/toolchain-status-shell.ts", import.meta.url), "utf8");

  assert.match(main, /mountToolchainStatusShell\(/);
  assert.doesNotMatch(main, /createToolchainStatusController\(/);
  assert.match(shell, /createToolchainStatusController\(/);
  assert.match(shell, /api\.rustToolchainAvailable/);
  assert.match(shell, /learning-mode-rust-option/);
});

function fixture({ rustResult = true, cargoResult = true, rustfmtResult = true, rustError, cargoError, rustfmtError } = {}) {
  const calls = [];
  let tab = { id: 1, language: "rust", workspacePreferenceKey: "course" };
  const resolve = (value, error) => error ? Promise.reject(error) : Promise.resolve(value);
  const controller = createToolchainStatusController({
    checkRust: () => resolve(rustResult, rustError),
    checkCargo: () => resolve(cargoResult, cargoError),
    checkRustfmt: () => resolve(rustfmtResult, rustfmtError),
    setRustcAvailable: (value) => calls.push(["rustc", value]),
    setCargoAvailable: (value) => calls.push(["cargo", value]),
    setRustfmtAvailable: (value) => calls.push(["rustfmt", value]),
    setRustOptionDisabled: (disabled) => calls.push(["rustDisabled", disabled]),
    setRustHelpText: (text) => calls.push(["rustHelp", text]),
    restoreLearningMode: () => calls.push(["learningMode"]),
    activeTab: () => tab,
    syncLanguageControls: (current) => calls.push(["sync", current]),
    forceRustLint: () => calls.push(["forceLint"]),
  });
  return { calls, controller, setTab: (value) => { tab = value; } };
}

test("enables Rust and refreshes learning and lint state for an active Rust tab", async () => {
  const f = fixture();
  await f.controller.initialize();
  assert.deepEqual(f.calls, [
    ["rustc", true], ["rustDisabled", false],
    ["rustHelp", "Toolchain rustc disponível. Novo Rust cria um arquivo .rs; trocar de nível não altera o arquivo aberto."],
    ["learningMode"], ["sync", { id: 1, language: "rust", workspacePreferenceKey: "course" }], ["forceLint"],
    ["cargo", true], ["sync", { id: 1, language: "rust", workspacePreferenceKey: "course" }],
    ["rustfmt", true], ["sync", { id: 1, language: "rust", workspacePreferenceKey: "course" }],
  ]);
});

test("disables Rust and skips lint refresh for a non-Rust active tab", async () => {
  const f = fixture({ rustResult: false });
  f.setTab({ id: 2, language: "ferro" });
  await f.controller.initialize();
  assert.ok(f.calls.some(([name, value]) => name === "rustDisabled" && value === true));
  assert.ok(f.calls.some(([name, text]) => name === "rustHelp" && text.startsWith("Instale a toolchain Rust")));
  assert.equal(f.calls.some(([name]) => name === "forceLint"), false);
});

test("reports a failed Rust check and keeps the editor state usable", async () => {
  const f = fixture({ rustError: new Error("not installed") });
  await f.controller.initialize();
  assert.ok(f.calls.some(([name, value]) => name === "rustc" && value === false));
  assert.ok(f.calls.some(([name, text]) => name === "rustHelp" && text.startsWith("Não foi possível verificar rustc")));
  assert.ok(f.calls.some(([name]) => name === "learningMode"));
  assert.ok(f.calls.some(([name]) => name === "forceLint"));
});

test("records Cargo and rustfmt failures and still refreshes the active language controls", async () => {
  const f = fixture({ cargoError: new Error("cargo missing"), rustfmtError: new Error("rustfmt missing") });
  f.setTab({ id: 3, language: "ferro-rust" });
  await f.controller.initialize();
  assert.ok(f.calls.some(([name, value]) => name === "cargo" && value === false));
  assert.ok(f.calls.some(([name, value]) => name === "rustfmt" && value === false));
  assert.equal(f.calls.filter(([name]) => name === "sync").length, 3);
  assert.equal(f.calls.some(([name]) => name === "forceLint"), false);
});
