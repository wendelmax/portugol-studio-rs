import test from "node:test";
import assert from "node:assert/strict";
import {
  exerciseTestPanelState,
  exerciseTestStatus,
  isCurrentExerciseTestRequest,
} from "../src/exercise-test-panel.ts";

const exercise = { id: 3, statement: "Some dois números", cases: 2 };

test("exercise tests stay empty for regular source files", () => {
  const state = exerciseTestPanelState(undefined, {}, undefined);

  assert.deepEqual(state, { kind: "empty" });
  assert.equal(exerciseTestStatus(state), "Nenhum teste configurado para este arquivo.");
});

test("exercise tests report available cases without claiming they ran", () => {
  const state = exerciseTestPanelState(exercise, {}, undefined);

  assert.deepEqual(state, { kind: "ready", cases: 2 });
  assert.equal(exerciseTestStatus(state), "2 casos configurados para este exercício.");
});

test("exercise with no configured cases does not offer execution", () => {
  const state = exerciseTestPanelState({ ...exercise, cases: 0 }, {}, undefined);

  assert.deepEqual(state, { kind: "no-cases" });
  assert.equal(exerciseTestStatus(state), "Este exercício não possui casos de execução configurados.");
});

test("exercise test result is shown only for the exact document revision", () => {
  const source = {};
  const staleSource = {};
  const run = { source, status: "completed", result: { total: 2, passed: 1, failures: ["Caso 2. saída diferente"] } };

  assert.deepEqual(exerciseTestPanelState(exercise, source, run), { kind: "completed", result: run.result });
  assert.deepEqual(exerciseTestPanelState(exercise, staleSource, run), { kind: "ready", cases: 2 });
  assert.equal(exerciseTestStatus({ kind: "completed", result: run.result }), "1 de 2 casos aprovados.");
});

test("pending test response is discarded when its tab, document, or request changes", () => {
  const document = {};
  const request = { tabId: 7, document, generation: 1 };

  assert.equal(isCurrentExerciseTestRequest(request, request, true), true);
  assert.equal(isCurrentExerciseTestRequest(request, request, false), false);
  assert.equal(isCurrentExerciseTestRequest(request, { ...request, tabId: 8 }, true), false);
  assert.equal(isCurrentExerciseTestRequest(request, { ...request, document: {} }, true), false);
  assert.equal(isCurrentExerciseTestRequest(request, { ...request, generation: 2 }, true), false);
});

test("running and failed case evaluation have explicit status messages", () => {
  assert.equal(exerciseTestStatus({ kind: "running", cases: 2 }), "Executando 2 casos desta revisão…");
  assert.equal(exerciseTestStatus({ kind: "error", message: "Falha no corretor." }), "Falha no corretor.");
  assert.equal(exerciseTestStatus({
    kind: "completed",
    result: { total: 2, passed: 2, failures: [], completed: true },
  }), "2 de 2 casos aprovados. Exercício concluído nesta revisão.");
  assert.equal(
    exerciseTestStatus({ kind: "ready", cases: 4 }, "portugol-rs"),
    "4 casos configurados para este exercício (.pors).",
  );
  assert.equal(
    exerciseTestStatus({
      kind: "completed",
      result: { total: 4, passed: 4, failures: [], completed: true },
    }, "portugol-rs"),
    "4 de 4 casos aprovados (.pors). Exercício concluído nesta revisão.",
  );
});
