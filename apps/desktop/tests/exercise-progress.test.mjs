import assert from "node:assert/strict";
import { test } from "node:test";

const moduleUrl = new URL("../src/exercise-progress.ts", import.meta.url);

function memoryStorage(initial = null) {
  let value = initial;
  return {
    getItem: () => value,
    setItem: (_key, next) => { value = next; },
    value: () => value,
  };
}

test("exercise identity distinguishes built-in entries from imported definitions", async () => {
  const { exerciseIdentity } = await import(moduleUrl.href);

  assert.equal(await exerciseIdentity(7), "builtin:7");
  assert.equal(await exerciseIdentity(7, "exercise contents"), await exerciseIdentity(7, "exercise contents"));
  assert.notEqual(await exerciseIdentity(7, "exercise contents"), await exerciseIdentity(8, "exercise contents"));
});

test("source revision is a SHA-256 digest of the exact source", async () => {
  const { sourceRevision } = await import(moduleUrl.href);

  assert.equal(
    await sourceRevision("abc"),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
  );
  assert.notEqual(await sourceRevision("abc"), await sourceRevision("abc "));
});

test("only a complete real checker result creates validated progress", async () => {
  const { createExerciseProgress } = await import(moduleUrl.href);
  const storage = memoryStorage();
  const progress = createExerciseProgress(storage);

  assert.equal((await progress.record("builtin:1", "programa {}", { total: 0, passed: 0, failures: [], hints: [] })).recorded, false);
  assert.equal((await progress.record("builtin:1", "programa {}", { total: 3, passed: 2, failures: ["Caso 3"], hints: [] })).completed, false);
  assert.equal(progress.hasPassed("builtin:1"), false);

  const result = await progress.record("builtin:1", "programa { funcao inicio() {} }", { total: 3, passed: 3, failures: [], hints: [] });
  assert.equal(result.recorded, true);
  assert.equal(result.completed, true);
  assert.match(result.revision, /^[a-f0-9]{64}$/);
  assert.equal(progress.hasPassed("builtin:1"), true);
  assert.equal(JSON.parse(storage.value()).version, 1);
  assert.equal(storage.value().includes("programa {"), false);
});

test("completion remains historical while every result retains its tested revision", async () => {
  const { createExerciseProgress } = await import(moduleUrl.href);
  const progress = createExerciseProgress(memoryStorage());
  const passed = { total: 1, passed: 1, failures: [], hints: [] };
  const failed = { total: 1, passed: 0, failures: ["Caso 1"], hints: [] };

  const first = await progress.record("builtin:2", "source one", passed);
  const second = await progress.record("builtin:2", "source two", failed);

  assert.equal(progress.hasPassed("builtin:2"), true);
  assert.notEqual(first.revision, second.revision);
  assert.equal(progress.read().exercises["builtin:2"].latest.status, "failed");
  assert.equal(progress.read().exercises["builtin:2"].completedRevision, first.revision);
});

test("invalid storage is ignored and unavailable storage does not reject grading", async () => {
  const { createExerciseProgress } = await import(moduleUrl.href);
  const malformed = memoryStorage("{not-json");
  const progress = createExerciseProgress(malformed);
  assert.equal(progress.hasPassed("builtin:1"), false);
  assert.equal((await progress.record("builtin:1", "source", { total: 1, passed: 1, failures: [], hints: [] })).recorded, true);

  const unavailable = createExerciseProgress({ getItem: () => { throw new Error("blocked"); }, setItem: () => { throw new Error("blocked"); } });
  assert.equal(unavailable.hasPassed("builtin:1"), false);
  assert.equal((await unavailable.record("builtin:1", "source", { total: 1, passed: 1, failures: [], hints: [] })).recorded, false);

  const noCrypto = createExerciseProgress(memoryStorage(), null);
  const noCryptoResult = await noCrypto.record("builtin:1", "source", { total: 1, passed: 1, failures: [], hints: [] });
  assert.equal(noCryptoResult.recorded, false);
  assert.equal(noCryptoResult.completed, true);
  const noIdentity = await noCrypto.record(null, "source", { total: 1, passed: 1, failures: [], hints: [] });
  assert.equal(noIdentity.recorded, false);
  assert.equal(noIdentity.completed, true);
});

test("grading records only the source passed to the checker and infrastructure rejection records nothing", async () => {
  const { createExerciseProgress, createValidatedExerciseGrader } = await import(moduleUrl.href);
  const progress = createExerciseProgress(memoryStorage());
  const grader = createValidatedExerciseGrader(
    async (source) => ({ total: 1, passed: source === "valid" ? 1 : 0, failures: source === "valid" ? [] : ["Caso 1"], hints: [] }),
    progress,
    "builtin:9",
  );

  const failed = await grader("invalid");
  assert.equal(failed.result.completed, false);
  const passed = await grader("valid");
  assert.equal(passed.result.completed, true);
  assert.equal(progress.read().exercises["builtin:9"].latest.revision, passed.result.revision);

  const unavailableGrader = createValidatedExerciseGrader(async () => { throw new Error("checker unavailable"); }, progress, "builtin:10");
  await assert.rejects(unavailableGrader("source"), /checker unavailable/);
  assert.equal(progress.hasPassed("builtin:10"), false);
});

test("catalog helper labels completion as historical", async () => {
  const { exerciseOptionLabel } = await import(moduleUrl.href);
  assert.equal(exerciseOptionLabel("Exercício 4", true), "Exercício 4 — concluído");
  assert.equal(exerciseOptionLabel("Exercício 4", false), "Exercício 4");
});

test("a superseded asynchronous evaluation cannot replace the latest attempt", async () => {
  const { createExerciseProgress, createValidatedExerciseGrader } = await import(moduleUrl.href);
  const progress = createExerciseProgress(memoryStorage());
  let current = false;
  const grader = createValidatedExerciseGrader(
    async () => ({ total: 1, passed: 1, failures: [], hints: [] }),
    progress,
    "builtin:11",
    () => current,
  );

  const result = await grader("stale source");
  assert.equal(result.result.recorded, false);
  assert.equal(progress.hasPassed("builtin:11"), false);
});

test("an evaluation superseded while hashing cannot persist completion", async () => {
  const { createExerciseProgress, createValidatedExerciseGrader } = await import(moduleUrl.href);
  const storage = memoryStorage();
  let finishDigest;
  let digestStarted;
  const started = new Promise((resolve) => { digestStarted = resolve; });
  const gate = new Promise((resolve) => { finishDigest = resolve; });
  const cryptoApi = {
    subtle: {
      digest: async (algorithm, source) => {
        digestStarted();
        await gate;
        return globalThis.crypto.subtle.digest(algorithm, source);
      },
    },
  };
  const progress = createExerciseProgress(storage, cryptoApi);
  let current = true;
  const grader = createValidatedExerciseGrader(
    async () => ({ total: 1, passed: 1, failures: [], hints: [] }),
    progress,
    "builtin:13",
    () => current,
  );

  const pending = grader("stale after hashing begins");
  await started;
  current = false;
  finishDigest();
  const result = await pending;

  assert.equal(result.result.recorded, false);
  assert.equal(progress.hasPassed("builtin:13"), false);
  assert.equal(progress.read().exercises["builtin:13"], undefined);
});

test("storage from a future schema version does not expose stale completion", async () => {
  const { createExerciseProgress } = await import(moduleUrl.href);
  const future = memoryStorage(JSON.stringify({ version: 2, exercises: { "builtin:12": { completedRevision: "a".repeat(64) } } }));
  assert.equal(createExerciseProgress(future).hasPassed("builtin:12"), false);
});

test("exercise feedback distinguishes complete, partial, static-only, and unsaved results", async () => {
  const { exerciseResultSummary, exerciseRevisionStatus } = await import(moduleUrl.href);

  assert.equal(exerciseResultSummary({ total: 2, passed: 2, hints: [] }, { completed: true }), "2 de 2 casos aprovados. Exercício concluído nesta revisão.");
  assert.equal(exerciseResultSummary({ total: 2, passed: 1, hints: [] }, { completed: false }), "1 de 2 casos aprovados. Ainda faltam casos para concluir.");
  assert.equal(exerciseResultSummary({ total: 0, passed: 0, hints: ["dica"] }, { completed: false }), "Análise estática concluída.");
  assert.equal(exerciseRevisionStatus({ revision: "a".repeat(64), recorded: false }), `Revisão avaliada: ${"a".repeat(12)}. Não foi possível salvar o progresso neste dispositivo.`);
  assert.equal(exerciseRevisionStatus({ revision: null, recorded: false }), "Sem casos de teste avaliados; esta análise não gera conclusão de exercício.");
});
