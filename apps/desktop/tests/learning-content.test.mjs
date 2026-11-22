import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const moduleUrl = new URL("../src/learning-content.ts", import.meta.url);

test("Beginner, Bridge e Rust apresentam os conceitos próprios de cada nível", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  const beginner = conceptsForMode("beginner");
  const bridge = conceptsForMode("bridge");
  const rust = conceptsForMode("rust");

  assert.deepEqual(beginner.map(({ id }) => id), ["variables", "conditionals", "loops"]);
  assert.deepEqual(bridge.map(({ id }) => id), ["variables", "conditionals", "loops", "mutability", "ownership", "borrowing"]);
  assert.deepEqual(rust.map(({ id }) => id), ["mutability", "ownership", "borrowing"]);
});

test("conceitos básicos apontam para exercícios existentes e Rust-only não aponta para exercício Portugol", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  const concepts = new Map(conceptsForMode("bridge").map(({ id, practiceExerciseId }) => [id, practiceExerciseId]));

  assert.equal(concepts.get("variables"), 2);
  assert.equal(concepts.get("conditionals"), 12);
  assert.equal(concepts.get("loops"), 24);
  for (const concept of conceptsForMode("rust")) assert.equal(concept.practiceExerciseId, undefined, concept.id);
});

test("cada prompt de prática combina com o enunciado real do exercício associado", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  const catalog = readFileSync(new URL("../../../crates/portugol-service/src/exercise_catalog.rs", import.meta.url), "utf8");
  const expectedTerms = new Map([
    ["variables", /nome/i],
    ["conditionals", /par|ímpar/i],
    ["loops", /ordem crescente/i],
  ]);

  for (const concept of conceptsForMode("beginner")) {
    const catalogLine = catalog.split("\n").find((line) => line.includes(`(${concept.practiceExerciseId},`));
    const entry = catalogLine?.match(/"([^"]+)"/);
    assert.ok(entry, `${concept.id}: ID listado no catálogo Rust`);
    const source = readFileSync(new URL(`../../../crates/portugol-service/assets/exercicios/${entry[1]}`, import.meta.url), "utf8");
    const statement = source.match(/<enunciado>([\s\S]*?)<\/enunciado>/)?.[1] ?? source;
    const term = expectedTerms.get(concept.id);
    assert.ok(term?.test(concept.practice), `${concept.id}: o prompt de prática trata do mesmo conceito`);
    assert.ok(term?.test(statement), `${concept.id}: o exercício real trata do mesmo conceito`);
  }
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
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
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
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(html, /id="learning-inspector-slot"/);
  assert.match(html, /id="learning-panel"[^>]*role="region"/);
  assert.match(main, /\$\("learning-inspector-slot"\)\.append\(\$\("learning-panel"\)\)/);
  assert.match(html, /Conceito selecionado manualmente/);
  assert.match(html, /id="learning-rust-example" hidden/);
  assert.match(html, /Estudar um conceito não registra progresso/);
});

test("inspetor oferece pin acessível por aba sem confundir escolha manual com contexto detectado", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
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
