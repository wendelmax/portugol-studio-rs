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
