import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";

const moduleUrl = new URL("../src/learning-mode.ts", import.meta.url);

test("modo inicial é beginner e persiste beginner, bridge e rust", async () => {
  const { loadLearningMode, saveLearningMode } = await import(moduleUrl.href);
  const values = new Map();
  const storage = {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  };

  assert.equal(loadLearningMode(storage), "beginner");
  assert.equal(saveLearningMode("bridge", storage), true);
  assert.equal(loadLearningMode(storage), "bridge");
  assert.equal(saveLearningMode("rust", storage), true);
  assert.equal(loadLearningMode(storage), "rust");
});

test("valor desconhecido ou armazenamento indisponível retorna beginner sem lançar", async () => {
  const { loadLearningMode, saveLearningMode } = await import(moduleUrl.href);
  assert.equal(loadLearningMode({ getItem: () => "future-mode" }), "beginner");
  assert.equal(loadLearningMode({ getItem: () => { throw new Error("blocked"); } }), "beginner");
  assert.equal(saveLearningMode("bridge", { setItem: () => { throw new Error("blocked"); } }), false);
});

test("workspace modes persist independently and fall back to the global legacy preference", async () => {
  const { loadLearningMode, saveLearningMode } = await import(moduleUrl.href);
  const values = new Map([["portugol.learningMode", "bridge"]]);
  const storage = {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  };

  assert.equal(loadLearningMode(storage, "workspace-a"), "bridge");
  assert.equal(saveLearningMode("rust", storage, "workspace-a"), true);
  assert.equal(saveLearningMode("beginner", storage, "workspace-b"), true);
  assert.equal(loadLearningMode(storage, "workspace-a"), "rust");
  assert.equal(loadLearningMode(storage, "workspace-b"), "beginner");
  assert.equal(loadLearningMode(storage), "bridge");
  assert.equal(values.get("portugol.learningMode.workspace.workspace-a"), "rust");
});

test("workspace tabs carry their stable preference key and restore mode on activation", () => {
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();
  const modeShell = readFileSync(new URL("../src/learning/mode-shell.ts", import.meta.url), "utf8");
  const toolchainStatus = readFileSync(new URL("../src/shell/toolchain-status.ts", import.meta.url), "utf8");
  const modeController = readFileSync(new URL("../src/learning/mode-controller.ts", import.meta.url), "utf8");
  const workspace = readFileSync(new URL("../src-tauri/src/workspace.rs", import.meta.url), "utf8");

  assert.match(api, /interface WorkspaceListing \{[\s\S]*preferenceKey: string/);
  assert.match(workspace, /preference_key: String/);
  assert.match(workspace, /Sha256::digest\(canonical\.to_string_lossy\(\)\.as_bytes\(\)\)/);
  assert.match(main, /workspacePreferenceKey\?: string/);
  assert.match(main, /applyLearningMode\(loadLearningMode\(undefined, tab\.workspacePreferenceKey\), false\)/);
  assert.match(modeShell, /persist: \(mode, preferenceKey\) => saveLearningMode\(mode, undefined, preferenceKey\)/);
  assert.match(modeController, /dependencies\.persist\(availableMode, dependencies\.activeTab\(\)\?\.workspacePreferenceKey\)/);
  assert.match(main, /restoreLearningMode: \(\) => applyLearningMode\(loadLearningMode\(undefined, activeTab\(\)\?\.workspacePreferenceKey\), false\)/);
  assert.match(toolchainStatus, /dependencies\.restoreLearningMode\(\)/);
});

test("seletor associa a disponibilidade de Rust à toolchain e mantém instrução correta", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const main = ideSource();
  const toolchainStatus = readFileSync(new URL("../src/shell/toolchain-status.ts", import.meta.url), "utf8");
  assert.match(html, /id="learning-mode"/);
  assert.match(html, /for="learning-mode"/);
  assert.match(html, /value="beginner"/);
  assert.match(html, /value="bridge"/);
  assert.match(html, /value="rust"[^>]*disabled/);
  assert.match(html, /learning-mode-rust-help/);
  assert.match(toolchainStatus, /rustc\) para habilitar este nível/);
});

test("modo Rust usa editor e executor Rust existentes sem reescrever a fonte", () => {
  const main = ideSource();
  const toolchainStatus = readFileSync(new URL("../src/shell/toolchain-status.ts", import.meta.url), "utf8");
  const modeController = readFileSync(new URL("../src/learning/mode-controller.ts", import.meta.url), "utf8");
  const rustExecution = readFileSync(new URL("../src/run/rust-execution.ts", import.meta.url), "utf8");
  assert.match(main, /checkRust: api\.rustToolchainAvailable/);
  assert.match(main, /runRustSource: api\.runRustSource/);
  assert.match(rustExecution, /dependencies\.runRustSource\(id, source, rustArgs\)/);
  assert.match(toolchainStatus, /setRustOptionDisabled\(!available\)/);
  assert.match(main, /Nível Rust:[\s\S]{0,180}arquivo atual permanece intacto/);
  const applyMode = modeController.match(/return \(mode: LearningMode,[\s\S]*?\n  \};/)?.[0];
  assert.ok(applyMode, "a troca de modo precisa ter implementação identificável");
  assert.doesNotMatch(applyMode, /view\.dispatch|newTab\(/, "trocar modo não edita nem troca de documento");
});
