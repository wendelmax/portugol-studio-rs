import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";

const moduleUrl = new URL("../src/learning-content.ts", import.meta.url);

test("Beginner, Bridge e Rust apresentam os conceitos próprios de cada nível", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  const beginner = conceptsForMode("beginner");
  const bridge = conceptsForMode("bridge");
  const rust = conceptsForMode("rust");

  assert.deepEqual(beginner.map(({ id }) => id), ["variables", "conditionals", "loops"]);
  assert.deepEqual(
    bridge.map(({ id }) => id),
    ["variables", "conditionals", "loops", "mutability", "ownership", "borrowing", "functions", "option-result", "collections", "records-enums"],
  );
  assert.deepEqual(rust.map(({ id }) => id), ["mutability", "ownership", "borrowing"]);
});

test("conceitos apontam para exercícios .pors existentes e Rust-only não aponta para exercício PORS", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  const concepts = new Map(conceptsForMode("bridge").map(({ id, practicePorsExerciseId }) => [id, practicePorsExerciseId]));

  assert.equal(concepts.get("variables"), "pors-12-saudacao-cadeia");
  assert.equal(concepts.get("conditionals"), "pors-11-par-ou-impar");
  assert.equal(concepts.get("loops"), "pors-03-soma-ate-n");
  assert.equal(concepts.get("functions"), "pors-04-funcao-fatorial");
  assert.equal(concepts.get("option-result"), "pors-05-divisao-segura");
  assert.equal(concepts.get("collections"), "pors-06-filtrar-positivos");
  assert.equal(concepts.get("records-enums"), "pors-13-registro-ponto");
  for (const concept of conceptsForMode("rust")) assert.equal(concept.practicePorsExerciseId, undefined, concept.id);
});

test("cada prompt de prática combina com o enunciado real do exercício .pors associado", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  const exercisesDir = new URL("../../../crates/portugol-service/assets/exercicios-pors/", import.meta.url);
  const expectedTerms = new Map([
    ["variables", /nome/i],
    ["conditionals", /par|ímpar/i],
    ["loops", /1 até N/i],
    ["functions", /função/i],
    ["option-result", /divi/i],
    ["collections", /positivos/i],
    ["records-enums", /Retangulo/i],
  ]);

  for (const concept of conceptsForMode("bridge")) {
    if (!concept.practicePorsExerciseId) continue;
    const file = `${concept.practicePorsExerciseId.slice("pors-".length)}.pors.json`;
    const definition = JSON.parse(readFileSync(new URL(file, exercisesDir), "utf8"));
    assert.equal(definition.id, concept.practicePorsExerciseId);
    const term = expectedTerms.get(concept.id);
    assert.ok(term?.test(concept.practice), `${concept.id}: o prompt de prática trata do mesmo conceito`);
    assert.ok(term?.test(definition.statement), `${concept.id}: o exercício .pors real trata do mesmo conceito`);
  }

  const exercisesTs = readFileSync(new URL("../src/exercises.ts", import.meta.url), "utf8");
  assert.ok(!exercisesTs.includes("Portugol clássico (.por)"), "diálogo de exercícios não deve listar grupo legado .por");
});

test("cada conceito em Beginner, Bridge e Rust tem explicação completa, sem alegar detecção automática", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  for (const mode of ["beginner", "bridge", "rust"]) {
    for (const concept of conceptsForMode(mode)) {
      assert.ok(concept.title.trim(), `${mode}/${concept.id}: título`);
      assert.ok(concept.definition.trim(), `${mode}/${concept.id}: definição`);
      assert.ok(concept.whatChanged.trim(), `${mode}/${concept.id}: o que mudou`);
      assert.ok(concept.whyItMatters.trim(), `${mode}/${concept.id}: por que importa`);
      assert.ok(concept.practice.trim(), `${mode}/${concept.id}: praticar`);
      assert.equal(concept.provenance, "manual-selection");
      if (mode !== "rust") assert.ok(concept.portugolExample.code.trim(), `${mode}/${concept.id}: exemplo Portugol`);
    }
  }
});

test("cartão Aprender apresenta seções nomeadas e renderiza o conteúdo do catálogo como texto", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const main = ideSource();
  for (const id of ["learning-what-changed", "learning-why-it-matters", "learning-practice"]) {
    assert.match(html, new RegExp(`id="${id}"`));
    assert.match(main, new RegExp(`\\$\\("${id}"\\)\\.textContent = concept\\.`));
  }
  assert.match(html, /<article class="learning-card">/);
  assert.match(html, /<h3 id="learning-title" aria-live="polite" aria-atomic="true">/);
  assert.match(html, /id="learning-practice-open"[^>]*hidden/);
  assert.match(html, /id="learning-practice-status"[^>]*role="status"/);
  assert.match(main, /exercises\.openById\(exerciseId\)/);
});

test("ownership explica move de String e identifica o trecho como exemplo editorial", async () => {
  const { findLearningConcept } = await import(moduleUrl.href);
  const concept = findLearningConcept("ownership");
  assert.ok(concept);
  assert.match(concept.definition, /mov|propriedade/i);
  assert.match(concept.rustExample.code, /String::from/);
  assert.equal(concept.rustExample.provenance, "editorial");
  assert.match(concept.rustExample.caption, /didático.*não é código gerado/i);
});

test("exemplo introdutório de borrowing usa a referência de string idiomática", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  const concept = conceptsForMode("rust").find(({ id }) => id === "borrowing");
  assert.ok(concept);
  assert.match(concept.rustExample.code, /fn tamanho\(texto: &str\)/);
});

test("mudança de modo mantém o conceito compatível ou volta ao primeiro disponível", async () => {
  const { resolveConceptForMode } = await import(moduleUrl.href);
  assert.equal(resolveConceptForMode("bridge", "ownership").id, "ownership");
  assert.equal(resolveConceptForMode("beginner", "ownership").id, "variables");
  assert.equal(resolveConceptForMode("rust", "borrowing").id, "borrowing");
});

test("inspector Aprender identifica seleção manual, deixa exemplos Rust inicialmente ocultos e sem progresso", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const main = ideSource();
  assert.match(html, /id="learning-inspector-slot"/);
  assert.match(html, /id="learning-panel"[^>]*role="region"/);
  assert.match(main, /\$\("learning-inspector-slot"\)\.append\(\$\("learning-panel"\)\)/);
  assert.match(html, /Conceito selecionado manualmente/);
  assert.match(html, /id="learning-rust-example" hidden/);
  assert.match(html, /Estudar um conceito não registra progresso/);
});

test("inspetor oferece pin acessível por aba sem confundir escolha manual com contexto detectado", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const main = ideSource();
  assert.match(html, /id="learning-pin"[^>]*aria-pressed="false"[^>]*disabled>Fixar tópico/);
  assert.match(main, /learningPins\?: LearningPins/);
  assert.match(main, /learningPinForMode\(tab\?\.learningPins, learningMode/);
  assert.match(main, /Tópico fixado — escolha manual nesta aba/);
  assert.match(main, /\$<HTMLButtonElement>\("learning-pin"\)\.onclick/);
  assert.match(main, /if \(tab && learningPinForMode\(tab\.learningPins, learningMode/);
});

test("todas as abas resolvem para o ID real de seu painel", async () => {
  const { panelContentId } = await import(new URL("../src/panel-tabs.ts", import.meta.url).href);
  assert.deepEqual(
    ["console", "messages", "debug", "learning"].map(panelContentId),
    ["console", "messages", "debug", "learning-panel"],
  );
});
