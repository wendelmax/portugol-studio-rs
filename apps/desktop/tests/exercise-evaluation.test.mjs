import test from "node:test";
import assert from "node:assert/strict";
import { createExerciseEvaluationRunner, createExerciseProgress } from "../src/exercise-progress.ts";

function deferred() {
  let resolve;
  const promise = new Promise((done) => { resolve = done; });
  return { promise, resolve };
}

function memoryStorage() {
  const values = new Map();
  return {
    getItem: (key) => values.get(key) ?? null,
    setItem: (key, value) => values.set(key, value),
  };
}

const result = { total: 1, passed: 1, failures: [], hints: [] };

test("checks from different tabs remain valid when they resolve in reverse order", async () => {
  const first = deferred();
  const second = deferred();
  const checks = [first, second];
  const progress = createExerciseProgress(memoryStorage());
  const runner = createExerciseEvaluationRunner({
    check: () => checks.shift().promise,
    identify: async (id) => `exercise:${id}`,
    progress,
  });
  const a = { exercise: { id: 1 }, name: "A" };
  const b = { exercise: { id: 2 }, name: "B" };
  const aPromise = runner.evaluate(a, "a");
  const bPromise = runner.evaluate(b, "b");

  second.resolve(result);
  const bOutcome = await bPromise;
  first.resolve(result);
  const aOutcome = await aPromise;

  assert.equal(aOutcome.current, true);
  assert.equal(bOutcome.current, true);
  assert.equal(aOutcome.progress.completed, true);
  assert.equal(bOutcome.progress.completed, true);
  assert.equal(progress.hasPassed("exercise:1"), true);
  assert.equal(progress.hasPassed("exercise:2"), true);
});

test("closing or editing a tab while its check is pending prevents progress writes", async () => {
  const check = deferred();
  const storage = memoryStorage();
  const progress = createExerciseProgress(storage);
  const runner = createExerciseEvaluationRunner({
    check: () => check.promise,
    identify: async (id) => `exercise:${id}`,
    progress,
  });
  let current = true;
  const pending = runner.evaluate({ exercise: { id: 4 }, name: "D" }, "old revision", () => current);
  current = false;
  check.resolve(result);
  const outcome = await pending;

  assert.equal(outcome.current, false);
  assert.equal(outcome.progress.recorded, false);
  assert.equal(progress.hasPassed("exercise:4"), false);
  assert.equal(storage.getItem("ferro.exercise-progress.v1"), null);
});

test("newer evaluation of the same exercise prevents an older result from replacing progress", async () => {
  const older = deferred();
  const newer = deferred();
  const checks = [older, newer];
  const progress = createExerciseProgress(memoryStorage());
  const runner = createExerciseEvaluationRunner({
    check: () => checks.shift().promise,
    identify: async (id) => `exercise:${id}`,
    progress,
  });
  const document = { exercise: { id: 9 }, name: "Mesmo exercício" };
  const oldRun = runner.evaluate(document, "revisão antiga");
  const newRun = runner.evaluate(document, "revisão nova");

  newer.resolve(result);
  const newOutcome = await newRun;
  older.resolve(result);
  const oldOutcome = await oldRun;

  assert.equal(newOutcome.progress.completed, true);
  assert.equal(newOutcome.progress.recorded, true);
  assert.equal(oldOutcome.current, true);
  assert.equal(oldOutcome.progress.recorded, false);
  assert.equal(progress.read().exercises["exercise:9"].latest.revision, newOutcome.progress.revision);
});
