import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const moduleUrl = new URL("../src/learning-mode.ts", import.meta.url);

test("modo inicial é beginner e aceita preferência bridge persistida", async () => {
  const { loadLearningMode, saveLearningMode } = await import(moduleUrl.href);
  const values = new Map();
  const storage = {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  };

  assert.equal(loadLearningMode(storage), "beginner");
  assert.equal(saveLearningMode("bridge", storage), true);
  assert.equal(loadLearningMode(storage), "bridge");
});

test("valor inválido ou armazenamento indisponível retorna beginner sem lançar", async () => {
  const { loadLearningMode, saveLearningMode } = await import(moduleUrl.href);
  assert.equal(loadLearningMode({ getItem: () => "rust" }), "beginner");
  assert.equal(loadLearningMode({ getItem: () => { throw new Error("blocked"); } }), "beginner");
  assert.equal(saveLearningMode("bridge", { setItem: () => { throw new Error("blocked"); } }), false);
});

test("seletor é rotulado e comunica que Rust ainda não está disponível", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="learning-mode"/);
  assert.match(html, /for="learning-mode"/);
  assert.match(html, /value="beginner"/);
  assert.match(html, /value="bridge"/);
  assert.match(html, /value="rust"[^>]*disabled/);
  assert.match(html, /Rust indisponível/);
});
