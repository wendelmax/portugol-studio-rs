import assert from "node:assert/strict";
import { test } from "node:test";
import { createExerciseOpener } from "../src/exercise-opening.ts";

test("opens a built-in exercise by its explicit catalog ID", async () => {
  const opened = [];
  const exercise = { id: 12, statement: "Verifique se o número é par", cases: 3 };
  const open = createExerciseOpener(async () => [exercise], (document) => opened.push(document));

  const result = await open(12);

  assert.equal(result, exercise);
  assert.deepEqual(opened, [{ exercise, name: "Exercício 12" }]);
});

test("does not open a document when the requested exercise is absent", async () => {
  let opened = false;
  const open = createExerciseOpener(async () => [], () => { opened = true; });

  await assert.rejects(open(999), /Exercício 999 não encontrado/);
  assert.equal(opened, false);
});

test("preserves catalog errors and clears a rejected load for retry", async () => {
  let attempts = 0;
  const opened = [];
  const exercise = { id: 2, statement: "Leia e escreva", cases: 1 };
  const open = createExerciseOpener(async () => {
    attempts += 1;
    if (attempts === 1) throw new Error("Catálogo indisponível");
    return [exercise];
  }, (document) => opened.push(document));

  await assert.rejects(open(2), /Catálogo indisponível/);
  await open(2);

  assert.equal(attempts, 2);
  assert.equal(opened.length, 1);
});
