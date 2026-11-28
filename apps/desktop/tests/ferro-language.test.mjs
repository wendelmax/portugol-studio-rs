import assert from "node:assert/strict";
import test from "node:test";
import { FERRO_SYNTAX_ENTRIES, completeFerro, wordRangeAt } from "../../web-playground/ferro-language.mjs";

const labels = (source) => completeFerro(source, source.length).map((entry) => entry.label);

test("complete Ferro keywords, types and builtins by prefix", () => {
  assert.ok(labels("escr").includes("escreval"));
  assert.ok(labels("inte").includes("inteiro"));
  assert.ok(completeFerro("escr", 4).every((entry) => entry.insertText === entry.label));
});

test("wordRangeAt keeps Resultado:: as part of the same token", () => {
  const source = "Resultado::O";
  assert.deepEqual(wordRangeAt(source, source.length), {
    start: 0,
    end: 12,
    prefix: "Resultado::O",
  });
  assert.ok(labels(source).includes("Resultado::Ok"));
});

test("completions are suppressed inside comments and strings", () => {
  assert.deepEqual(labels("// escr"), []);
  const quoted = 'escreval("escr";';
  assert.deepEqual(completeFerro(quoted, quoted.indexOf('";')), []);
});

test("completions stay suppressed inside strings that span lines", () => {
  assert.deepEqual(labels('seja s = "abc\nescr'), []);
  assert.ok(labels('seja s = "a//b"; escr').includes("escreval"));
});

test("syntax reference only advertises syntax Ferro accepts", () => {
  const entriesByLabel = new Map(FERRO_SYNTAX_ENTRIES.map((entry) => [entry.label, entry]));
  for (const invalid of ["senao_se", "faca_enquanto", "nulo", "??", "??=", "?."]) {
    assert.equal(entriesByLabel.has(invalid), false, `${invalid} não é sintaxe Ferro`);
  }
  assert.ok(entriesByLabel.has("faca"));
  assert.match(entriesByLabel.get("senao").example, /senao se/);
  assert.match(entriesByLabel.get("Opcao::Nenhum").description, /ausência/);
});

test("syntax entries cover every reference group and carry examples", () => {
  const groups = new Set(FERRO_SYNTAX_ENTRIES.map((entry) => entry.group));
  for (const group of [
    "Estrutura",
    "Declarações",
    "Tipos",
    "Controle",
    "Dados",
    "Entrada e saída",
    "Valores",
    "Operadores",
  ]) {
    assert.ok(groups.has(group), `grupo ${group} ausente`);
  }
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Estrutura" && e.label === "funcao"));
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Controle" && e.label === "enquanto"));
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Entrada e saída" && e.label === "escreval"));
  assert.ok(FERRO_SYNTAX_ENTRIES.some((e) => e.group === "Operadores" && e.label === "?"));
  for (const entry of FERRO_SYNTAX_ENTRIES) {
    assert.ok(entry.signature.length > 0, entry.label);
    assert.ok(entry.description.length > 0, entry.label);
    assert.ok(entry.example.length > 0, entry.label);
    assert.equal(entry.insertText, entry.label);
  }
  assert.ok(completeFerro("T", 1).length <= 12);
});
