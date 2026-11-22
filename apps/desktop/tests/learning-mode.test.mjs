import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

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

test("seletor associa a disponibilidade de Rust à toolchain e mantém instrução correta", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(html, /id="learning-mode"/);
  assert.match(html, /for="learning-mode"/);
  assert.match(html, /value="beginner"/);
  assert.match(html, /value="bridge"/);
  assert.match(html, /value="rust"[^>]*disabled/);
  assert.match(html, /learning-mode-rust-help/);
  assert.match(main, /rustc\) para habilitar este nível/);
});

test("modo Rust usa editor e executor Rust existentes sem reescrever a fonte", () => {
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(main, /rustToolchainAvailable\(\)/);
  assert.match(main, /api\.runRustSource\(id, source, rustArgs\)/);
  assert.match(main, /rustOption\.disabled = !available/);
  assert.match(main, /Nível Rust:[\s\S]{0,180}arquivo atual permanece intacto/);
  const applyMode = main.match(/function applyLearningMode\([\s\S]*?\n\}/)?.[0];
  assert.ok(applyMode, "a troca de modo precisa ter implementação identificável");
  assert.doesNotMatch(applyMode, /view\.dispatch|newTab\(/, "trocar modo não edita nem troca de documento");
});
