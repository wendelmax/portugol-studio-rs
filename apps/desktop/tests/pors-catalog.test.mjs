import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  buildPorsExampleCatalog,
  buildPorsExampleCatalogFolder,
  formatPorsExerciseOptionValue,
  formatPorsExerciseOutcome,
  parsePorsExerciseOptionValue,
  PORS_EXAMPLE_PREFIX,
} from "../src/pors-catalog.ts";

test("constrói árvore aninhada para categorias com separador e mantém a tela inicial somente em PORS", () => {
  const tree = buildPorsExampleCatalog([
    {
      id: "classic-bib-mat-arredondar",
      category: "Bibliotecas / Matematica",
      title: "Arredondar",
      file_name: "arredondar.pors",
      description: "Exemplo de arredondamento",
      is_visual: false,
      license: "LGPL-3.0-only",
      source: "funcao inicio() {}",
    },
    {
      id: "classic-bib-mat-raiz",
      category: "Bibliotecas / Matematica",
      title: "Raiz",
      file_name: "raiz.pors",
      description: "Exemplo de raiz",
      is_visual: false,
      license: "LGPL-3.0-only",
      source: "funcao inicio() {}",
    },
  ]);

  assert.equal(tree.length, 1);
  assert.equal(tree[0].kind, "folder");
  assert.equal(tree[0].name, "Bibliotecas");
  assert.equal(tree[0].entries.length, 1);
  assert.equal(tree[0].entries[0].kind, "folder");
  assert.equal(tree[0].entries[0].name, "Matematica");
  assert.equal(tree[0].entries[0].entries.length, 2);

  const examplesTs = readFileSync(new URL("../src/examples.ts", import.meta.url), "utf8");
  assert.match(examplesTs, /api\.listPorsExamples\(\)/);
  assert.doesNotMatch(examplesTs, /api\.exampleCatalog\(/);
  assert.doesNotMatch(examplesTs, /api\.openExample\(/);
  assert.doesNotMatch(examplesTs, /exemplo\.por/);
});

test("agrupa exemplos Portugol-RS por categoria dentro da pasta raiz", () => {
  assert.equal(buildPorsExampleCatalogFolder([]), null);

  const folder = buildPorsExampleCatalogFolder([
    {
      id: "pors-01",
      category: "Fundamentos",
      title: "Olá",
      file_name: "01-ola.pors",
      description: "Exemplo inicial",
      is_visual: false,
      license: "LGPL-3.0-only",
      source: "funcao inicio() {}",
    },
    {
      id: "pors-02",
      category: "UI Visual",
      title: "Contador",
      file_name: "02-contador.pors",
      description: "App MVU",
      is_visual: true,
      license: "LGPL-3.0-only",
      source: "funcao inicio() {}",
    },
  ]);

  assert.equal(folder?.kind, "folder");
  assert.equal(folder?.name, "Portugol-RS (.pors)");
  assert.equal(folder?.entries.length, 2);
  assert.equal(folder?.entries[0].kind, "folder");
  assert.equal(folder?.entries[0].name, "Fundamentos");
  assert.equal(folder?.entries[0].entries[0].file, `${PORS_EXAMPLE_PREFIX}pors-01`);
});

test("codifica e decodifica chaves de exercícios .pors sem colidir com IDs numéricos", () => {
  const key = formatPorsExerciseOptionValue("pors-01-dobro");
  assert.equal(key, "pors:pors-01-dobro");
  assert.equal(parsePorsExerciseOptionValue(key), "pors-01-dobro");
  assert.equal(parsePorsExerciseOptionValue("12"), null);
});

test("converte resultados de avaliação .pors em falhas e dicas legíveis", () => {
  const passed = formatPorsExerciseOutcome({
    schema_version: 1,
    exercise_id: "pors-01-dobro",
    status: "passed",
    total: 2,
    passed: 2,
    cases: [
      { name: "positivo", passed: true, expected_output: "42", actual_output: "42", error: null },
      { name: "zero", passed: true, expected_output: "0", actual_output: "0", error: null },
    ],
    diagnostics: [],
    hints: [],
  });
  assert.deepEqual(passed, { total: 2, passed: 2, failures: [], hints: [] });

  const failed = formatPorsExerciseOutcome({
    schema_version: 1,
    exercise_id: "pors-01-dobro",
    status: "failed",
    total: 2,
    passed: 1,
    cases: [
      { name: "positivo", passed: false, expected_output: "42", actual_output: "21", error: null },
      { name: "zero", passed: true, expected_output: "0", actual_output: "0", error: null },
    ],
    diagnostics: [],
    hints: ["Multiplique por 2"],
  });
  assert.equal(failed.passed, 1);
  assert.equal(failed.failures.length, 1);
  assert.match(failed.failures[0], /positivo.*42.*21/);
  assert.deepEqual(failed.hints, ["Multiplique por 2"]);

  const compileErr = formatPorsExerciseOutcome({
    schema_version: 1,
    exercise_id: "pors-01-dobro",
    status: "compile_error",
    total: 2,
    passed: 0,
    cases: [],
    diagnostics: ["exercicio.pors:1:1 erro: Nome não declarado"],
    hints: [],
  });
  assert.deepEqual(compileErr.failures, ["exercicio.pors:1:1 erro: Nome não declarado"]);

  const limitErr = formatPorsExerciseOutcome({
    schema_version: 1,
    exercise_id: "pors-01-dobro",
    status: "limit_exceeded",
    total: 1,
    passed: 0,
    cases: [
      {
        name: "positivo",
        passed: false,
        expected_output: "42",
        actual_output: "",
        error: "Limite de passos excedido",
      },
    ],
    diagnostics: [],
    hints: ["Verifique o laço"],
  });
  assert.match(limitErr.failures[0], /Limite de passos excedido/);
});
