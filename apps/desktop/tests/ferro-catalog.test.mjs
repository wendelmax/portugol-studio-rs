import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  buildFerroExampleCatalog,
  buildFerroExampleCatalogFolder,
  formatFerroExerciseOptionValue,
  formatFerroExerciseOutcome,
  parseFerroExerciseOptionValue,
  FERRO_EXAMPLE_PREFIX,
} from "../src/ferro-catalog.ts";

test("constrói árvore aninhada para categorias com separador e mantém a tela inicial somente em FERRO", () => {
  const tree = buildFerroExampleCatalog([
    {
      id: "classic-bib-mat-arredondar",
      category: "Bibliotecas / Matematica",
      title: "Arredondar",
      file_name: "arredondar.fer",
      description: "Exemplo de arredondamento",
      is_visual: false,
      license: "LGPL-3.0-only",
      source: "funcao inicio() {}",
    },
    {
      id: "classic-bib-mat-raiz",
      category: "Bibliotecas / Matematica",
      title: "Raiz",
      file_name: "raiz.fer",
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
  assert.match(examplesTs, /api\.listFerroExamples\(\)/);
  assert.doesNotMatch(examplesTs, /api\.exampleCatalog\(/);
  assert.doesNotMatch(examplesTs, /api\.openExample\(/);
  assert.doesNotMatch(examplesTs, /exemplo\.por/);
});

test("agrupa exemplos Ferro por categoria dentro da pasta raiz", () => {
  assert.equal(buildFerroExampleCatalogFolder([]), null);

  const folder = buildFerroExampleCatalogFolder([
    {
      id: "ferro-01",
      category: "Fundamentos",
      title: "Olá",
      file_name: "01-ola.fer",
      description: "Exemplo inicial",
      is_visual: false,
      license: "LGPL-3.0-only",
      source: "funcao inicio() {}",
    },
    {
      id: "ferro-02",
      category: "UI Visual",
      title: "Contador",
      file_name: "02-contador.fer",
      description: "App MVU",
      is_visual: true,
      license: "LGPL-3.0-only",
      source: "funcao inicio() {}",
    },
  ]);

  assert.equal(folder?.kind, "folder");
  assert.equal(folder?.name, "Ferro (.fer)");
  assert.equal(folder?.entries.length, 2);
  assert.equal(folder?.entries[0].kind, "folder");
  assert.equal(folder?.entries[0].name, "Fundamentos");
  assert.equal(folder?.entries[0].entries[0].file, `${FERRO_EXAMPLE_PREFIX}ferro-01`);
});

test("codifica e decodifica chaves de exercícios .fer sem colidir com IDs numéricos", () => {
  const key = formatFerroExerciseOptionValue("ferro-01-dobro");
  assert.equal(key, "ferro:ferro-01-dobro");
  assert.equal(parseFerroExerciseOptionValue(key), "ferro-01-dobro");
  assert.equal(parseFerroExerciseOptionValue("12"), null);
});

test("converte resultados de avaliação .fer em falhas e dicas legíveis", () => {
  const passed = formatFerroExerciseOutcome({
    schema_version: 1,
    exercise_id: "ferro-01-dobro",
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

  const failed = formatFerroExerciseOutcome({
    schema_version: 1,
    exercise_id: "ferro-01-dobro",
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

  const compileErr = formatFerroExerciseOutcome({
    schema_version: 1,
    exercise_id: "ferro-01-dobro",
    status: "compile_error",
    total: 2,
    passed: 0,
    cases: [],
    diagnostics: ["exercicio.fer:1:1 erro: Nome não declarado"],
    hints: [],
  });
  assert.deepEqual(compileErr.failures, ["exercicio.fer:1:1 erro: Nome não declarado"]);

  const limitErr = formatFerroExerciseOutcome({
    schema_version: 1,
    exercise_id: "ferro-01-dobro",
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

test("projetos desktop de exemplo viram a pasta Projetos desktop", async () => {
  const { buildDesktopExampleFolder, DESKTOP_EXAMPLE_PREFIX } = await import("../src/ferro-catalog.ts");
  assert.equal(buildDesktopExampleFolder([]), null);
  const folder = buildDesktopExampleFolder([{ id: "contador", name: "Contador", description: "Soma 1." }]);
  assert.equal(folder.name, "Projetos desktop");
  assert.equal(folder.entries[0].file, `${DESKTOP_EXAMPLE_PREFIX}contador`);
  assert.equal(folder.entries[0].name, "Contador");
  assert.match(folder.entries[0].description, /Cria um projeto desktop/);
});
