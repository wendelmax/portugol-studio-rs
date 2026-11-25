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

test("opens a PORS exercise by its stable catalog ID", async () => {
  const { createPorsExerciseOpener } = await import("../src/exercise-opening.ts");
  const opened = [];
  const exercise = { id: "pors-05-divisao-segura", title: "Divisão segura", statement: "Divida com segurança", cases: 3 };
  const open = createPorsExerciseOpener(async () => [exercise], (document) => opened.push(document));

  const result = await open(exercise.id);

  assert.equal(result, exercise);
  assert.deepEqual(opened, [{
    exercise: { id: 0, statement: exercise.statement, cases: exercise.cases },
    name: exercise.title,
    contents: undefined,
    pors: exercise,
  }]);
});

test("does not open a document when a PORS exercise is absent", async () => {
  const { createPorsExerciseOpener } = await import("../src/exercise-opening.ts");
  let opened = false;
  const open = createPorsExerciseOpener(async () => [], () => { opened = true; });

  await assert.rejects(open("pors-missing"), /não encontrado no catálogo/);
  assert.equal(opened, false);
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
