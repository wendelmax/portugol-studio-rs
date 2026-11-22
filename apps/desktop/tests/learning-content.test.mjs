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

test("cada cartão tem definição e exemplo Portugol, sem alegar detecção automática", async () => {
  const { conceptsForMode } = await import(moduleUrl.href);
  for (const concept of conceptsForMode("bridge")) {
    assert.ok(concept.title.trim(), `${concept.id}: título`);
    assert.ok(concept.definition.trim(), `${concept.id}: definição`);
    assert.ok(concept.portugolExample.code.trim(), `${concept.id}: exemplo Portugol`);
    assert.equal(concept.provenance, "manual-selection");
  }
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

test("todas as abas resolvem para o ID real de seu painel", async () => {
  const { panelContentId } = await import(new URL("../src/panel-tabs.ts", import.meta.url).href);
  assert.deepEqual(
    ["console", "messages", "debug", "learning"].map(panelContentId),
    ["console", "messages", "debug", "learning-panel"],
  );
});
