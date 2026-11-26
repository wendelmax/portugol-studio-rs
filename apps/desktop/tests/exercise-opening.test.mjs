import assert from "node:assert/strict";
import { test } from "node:test";
import { createPorsExerciseOpener } from "../src/exercise-opening.ts";

test("opens a PORS exercise by its stable catalog ID", async () => {
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
  let opened = false;
  const open = createPorsExerciseOpener(async () => [], () => { opened = true; });

  await assert.rejects(open("pors-missing"), /não encontrado no catálogo/);
  assert.equal(opened, false);
});

test("preserves catalog errors and clears a rejected load for retry", async () => {
  let attempts = 0;
  const opened = [];
  const exercise = { id: "pors-02-media", title: "Média", statement: "Leia e escreva", cases: 1 };
  const open = createPorsExerciseOpener(async () => {
    attempts += 1;
    if (attempts === 1) throw new Error("Catálogo indisponível");
    return [exercise];
  }, (document) => opened.push(document));

  await assert.rejects(open(exercise.id), /Catálogo indisponível/);
  await open(exercise.id);

  assert.equal(attempts, 2);
  assert.equal(opened.length, 1);
});
