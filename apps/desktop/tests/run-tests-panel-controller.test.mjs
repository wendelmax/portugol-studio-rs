import test from "node:test";
import assert from "node:assert/strict";
import { createTestsPanelController } from "../src/run/tests-panel.ts";
import {
  cargoTestPanelState,
  cargoTestStatus,
  exerciseTestPanelState,
  exerciseTestStatus,
  isCurrentExerciseTestRequest,
} from "../src/exercise-test-panel.ts";

function fixture(overrides = {}) {
  const document = { text: "funcao inicio() {}" };
  const tab = {
    id: 1,
    path: "basico.pors",
    language: "portugol-rs",
    exercise: { name: "basico.pors.json", exercise: { cases: 2 }, pors: true },
    exerciseTestGeneration: 0,
    saved: document.text,
    state: { doc: document },
  };
  const view = {
    title: "",
    status: "",
    runHidden: false,
    runDisabled: false,
    runText: "",
    results: [],
    resultsHidden: true,
    clearResults() { this.results = []; },
    setTitle(value) { this.title = value; },
    setStatus(value) { this.status = value; },
    setRunHidden(value) { this.runHidden = value; },
    setRunDisabled(value) { this.runDisabled = value; },
    setRunText(value) { this.runText = value; },
    appendResult(value) { this.results.push(value); },
    setResultsHidden(value) { this.resultsHidden = value; },
  };
  let active = tab;
  let currentDocument = document;
  let generation = 0;
  let cargoAvailable = false;
  let runId = null;
  let pending = false;
  let evaluate = async () => ({
    current: true,
    evaluation: { total: 2, passed: 1, failures: ["Caso 2 falhou"], hints: ["Confira a condição"] },
    progress: { completed: true },
  });
  const controller = createTestsPanelController({
    tabs: () => active ? [active] : [],
    activeTab: () => active,
    activeId: () => 1,
    currentDocument: () => currentDocument,
    documentForTab: (current) => current.state.doc,
    cargoAvailable: () => cargoAvailable,
    runId: () => runId,
    runPending: () => pending,
    tabName: (current) => current.path,
    evaluateExercise: (...args) => evaluate(...args),
    cargoTestPanelState,
    cargoTestStatus,
    exerciseTestPanelState,
    exerciseTestStatus,
    isCurrentExerciseTestRequest,
    runCargoTests: async () => {},
    view,
    ...overrides.dependencies,
  });
  return {
    controller,
    tab,
    view,
    setActive: (value) => { active = value; },
    setCurrentDocument: (value) => { currentDocument = value; },
    setGeneration: (value) => { generation = value; tab.exerciseTestGeneration = value; },
    getGeneration: () => generation,
    setCargoAvailable: (value) => { cargoAvailable = value; },
    setRunId: (value) => { runId = value; },
    setPending: (value) => { pending = value; },
    setEvaluate: (value) => { evaluate = value; },
  };
}

test("exercise test panel renders current case count and labels", () => {
  const { controller, view } = fixture();

  controller.render();

  assert.equal(view.title, "Testes — basico.pors");
  assert.equal(view.status, "2 casos configurados para este exercício (.pors).");
  assert.equal(view.runHidden, false);
  assert.equal(view.runDisabled, false);
  assert.equal(view.runText, "Executar casos");
  assert.equal(view.resultsHidden, true);
});

test("running exercise cases records the result and hints for the current document", async () => {
  const { controller, tab, view } = fixture();

  await controller.runExerciseTests();

  assert.equal(tab.exerciseTestRun.status, "completed");
  assert.deepEqual(tab.exerciseTestRun.result, {
    total: 2,
    passed: 1,
    failures: ["Caso 2 falhou"],
    hints: ["Confira a condição"],
    completed: true,
  });
  assert.equal(view.status, "1 de 2 casos aprovados (.pors). Exercício concluído nesta revisão.");
  assert.deepEqual(view.results, ["Caso 2 falhou", "Dica: Confira a condição"]);
  assert.equal(view.resultsHidden, false);
});

test("a result from a superseded document does not replace the current document's test state", async () => {
  const { controller, tab, view, setEvaluate, setCurrentDocument } = fixture();
  setEvaluate(async (_exercise, _source, isCurrent) => {
    setCurrentDocument({ text: "novo documento" });
    assert.equal(isCurrent(), false);
    return { current: false, evaluation: { total: 2, passed: 0, failures: [] }, progress: { completed: false } };
  });

  await controller.runExerciseTests();

  assert.equal(tab.exerciseTestRun.status, "running");
  assert.equal(view.status, "2 casos configurados para este exercício (.pors).");
});

test("Cargo test button stays disabled without Cargo or while a run is active", () => {
  const { controller, view, setActive, setCargoAvailable, setRunId } = fixture();
  setActive({ id: 1, path: "Cargo.toml", language: "rust", cargoProject: true, cargoTestRun: undefined, state: { doc: {} } });
  controller.render();
  assert.equal(view.runDisabled, true);

  setCargoAvailable(true);
  setRunId(9);
  controller.render();
  assert.equal(view.runDisabled, true);
});
