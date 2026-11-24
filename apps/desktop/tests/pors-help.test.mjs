import test from "node:test";
import assert from "node:assert/strict";
import {
  PORS_HELP_DEFAULT_TOPIC,
  PORS_HELP_LIBRARIES,
  PORS_HELP_TOPICS,
  filterPorsHelp,
} from "../src/pors-help.ts";

test("PORS help starts on a PORS topic and covers all 9 PORS libraries", () => {
  assert.ok(PORS_HELP_TOPICS.some((topic) => topic.id === PORS_HELP_DEFAULT_TOPIC));
  const libNames = PORS_HELP_LIBRARIES.map((lib) => lib.name);
  for (const expected of [
    "PORS: Matematica",
    "PORS: Texto",
    "PORS: Tipos",
    "PORS: Util",
    "PORS: Calendario",
    "PORS: Arquivos",
    "PORS: Json",
    "PORS: Internet",
    "PORS: UI",
  ]) {
    assert.ok(libNames.includes(expected), `esperava biblioteca ${expected} na ajuda PORS`);
  }
});

test("PORS help examples use .pors syntax without classic Portugol keywords", () => {
  for (const topic of PORS_HELP_TOPICS) {
    if (!topic.example) continue;
    assert.ok(topic.example.fileName.endsWith(".pors"));
    assert.ok(!topic.example.source.includes("programa {"));
    assert.ok(!topic.example.source.includes("inclua biblioteca"));
  }
});

test("filterPorsHelp matches accents, library functions and topics case-insensitively", () => {
  const byOpcao = filterPorsHelp("opcao");
  assert.ok(byOpcao.topics.some((t) => t.id === "pors:opcao-resultado"));

  const byDividir = filterPorsHelp("dividir");
  assert.ok(byDividir.libraries.some((l) => l.id === "pors-lib:Texto"));

  const byBissexto = filterPorsHelp("bissexto");
  assert.ok(byBissexto.libraries.some((l) => l.id === "pors-lib:Calendario"));

  const byJson = filterPorsHelp("minificar");
  assert.ok(byJson.libraries.some((l) => l.id === "pors-lib:Json"));

  const byArquivos = filterPorsHelp("ler_linhas");
  assert.ok(byArquivos.libraries.some((l) => l.id === "pors-lib:Arquivos"));

  const byInternet = filterPorsHelp("enviar_json");
  assert.ok(byInternet.libraries.some((l) => l.id === "pors-lib:Internet"));

  const empty = filterPorsHelp("nao-existe-xyz-123");
  assert.equal(empty.topics.length, 0);
  assert.equal(empty.libraries.length, 0);
});
