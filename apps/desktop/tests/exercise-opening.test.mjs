import assert from "node:assert/strict";
import { test } from "node:test";
import { createFerroExerciseOpener } from "../src/exercise-opening.ts";

test("opens a FERRO exercise by its stable catalog ID", async () => {
  const opened = [];
  const exercise = { id: "ferro-05-divisao-segura", title: "Divisão segura", statement: "Divida com segurança", cases: 3 };
  const open = createFerroExerciseOpener(async () => [exercise], (document) => opened.push(document));

  const result = await open(exercise.id);

  assert.equal(result, exercise);
  assert.deepEqual(opened, [{
    exercise: { id: 0, statement: exercise.statement, cases: exercise.cases },
    name: exercise.title,
    contents: undefined,
    ferro: exercise,
  }]);
});

test("does not open a document when a FERRO exercise is absent", async () => {
  let opened = false;
  const open = createFerroExerciseOpener(async () => [], () => { opened = true; });

  await assert.rejects(open("ferro-missing"), /não encontrado no catálogo/);
  assert.equal(opened, false);
});

test("preserves catalog errors and clears a rejected load for retry", async () => {
  let attempts = 0;
  const opened = [];
  const exercise = { id: "ferro-02-media", title: "Média", statement: "Leia e escreva", cases: 1 };
  const open = createFerroExerciseOpener(async () => {
    attempts += 1;
    if (attempts === 1) throw new Error("Catálogo indisponível");
    return [exercise];
  }, (document) => opened.push(document));

  await assert.rejects(open(exercise.id), /Catálogo indisponível/);
  await open(exercise.id);

  assert.equal(attempts, 2);
  assert.equal(opened.length, 1);
});
