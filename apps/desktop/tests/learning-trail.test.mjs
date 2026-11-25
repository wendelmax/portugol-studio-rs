import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { test } from "node:test";

const moduleUrl = new URL("../src/learning/trail.ts", import.meta.url);

test("learning trail catalog is versioned and maps examples to PORS exercises", async () => {
  const { LEARNING_TRAIL_VERSION, LEARNING_TRAIL } = await import(moduleUrl.href);
  const exampleDirectory = new URL("../../../crates/portugol-service/assets/exemplos-pors/", import.meta.url);
  const exerciseDirectory = new URL("../../../crates/portugol-service/assets/exercicios-pors/", import.meta.url);
  const examples = new Set(await readdir(exampleDirectory));
  const exerciseFiles = new Set(await readdir(exerciseDirectory));

  assert.equal(LEARNING_TRAIL_VERSION, 1);
  assert.ok(LEARNING_TRAIL.length > 0);
  for (const stage of LEARNING_TRAIL) {
    assert.ok(stage.examplePaths.length > 0, `${stage.id} must include a PORS example`);
    assert.ok(stage.exercises.length > 0, `${stage.id} must include a PORS exercise`);
    for (const path of stage.examplePaths) assert.ok(examples.has(path), `Missing PORS example ${path}`);
    for (const exercise of stage.exercises) {
      const filename = `${exercise.id.slice("pors-".length)}.pors.json`;
      assert.ok(exerciseFiles.has(filename), `Missing PORS exercise ${filename}`);
      const definition = JSON.parse(await readFile(new URL(filename, exerciseDirectory), "utf8"));
      assert.equal(definition.id, exercise.id);
      assert.equal(definition.title, exercise.title);
    }
  }
});

test("an empty exercise catalog has an explicit empty state", async () => {
  const { learningTrailProgress } = await import(moduleUrl.href);
  const progress = learningTrailProgress([], () => false);

  assert.equal(progress.status, "empty");
  assert.equal(progress.totalStages, 0);
  assert.equal(progress.suggestedExercise, null);
  assert.match(progress.message, /ainda não tem exercícios/);
});

test("learning trail progress stays empty until every exercise passes and suggests a concrete exercise", async () => {
  const { LEARNING_TRAIL, learningTrailProgress } = await import(moduleUrl.href);
  const progress = learningTrailProgress(LEARNING_TRAIL, () => false);

  assert.equal(progress.status, "not-started");
  assert.equal(progress.completedStages, 0);
  assert.equal(progress.totalStages, LEARNING_TRAIL.length);
  assert.equal(progress.suggestedExercise.id, LEARNING_TRAIL[0].exercises[0].id);
  assert.match(progress.message, /Comece|Resolva/);
});

test("a stage completes only when all its exercises pass and next action names the next exercise", async () => {
  const { LEARNING_TRAIL, learningTrailProgress } = await import(moduleUrl.href);
  const firstStage = LEARNING_TRAIL[0];
  const passed = new Set(firstStage.exercises.slice(0, -1).map(({ id }) => `pors:${id}`));
  const partial = learningTrailProgress(LEARNING_TRAIL, (identity) => passed.has(identity));

  assert.equal(partial.completedStages, 0);
  assert.equal(partial.suggestedExercise.id, firstStage.exercises.at(-1).id);

  for (const { id } of firstStage.exercises) passed.add(`pors:${id}`);
  const next = learningTrailProgress(LEARNING_TRAIL, (identity) => passed.has(identity));
  assert.equal(next.completedStages, 1);
  assert.equal(next.status, "in-progress");
  assert.equal(next.suggestedExercise.id, LEARNING_TRAIL[1].exercises[0].id);
  assert.match(next.message, new RegExp(LEARNING_TRAIL[1].exercises[0].title));
});

test("learning trail reports a clear completed state after all catalog exercises pass", async () => {
  const { LEARNING_TRAIL, learningTrailProgress } = await import(moduleUrl.href);
  const all = new Set(LEARNING_TRAIL.flatMap(({ exercises }) => exercises.map(({ id }) => `pors:${id}`)));
  const progress = learningTrailProgress(LEARNING_TRAIL, (identity) => all.has(identity));

  assert.equal(progress.status, "completed");
  assert.equal(progress.completedStages, LEARNING_TRAIL.length);
  assert.equal(progress.suggestedExercise, null);
  assert.match(progress.message, /concluída/i);
});
