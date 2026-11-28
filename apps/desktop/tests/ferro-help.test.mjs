import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  FERRO_HELP_DEFAULT_TOPIC,
  FERRO_HELP_LIBRARIES,
  FERRO_HELP_TOPICS,
  filterFerroHelp,
} from "../src/ferro-help.ts";

test("FERRO help starts on a FERRO topic and covers all 16 FERRO libraries grouped into standard and visual", () => {
  assert.ok(FERRO_HELP_TOPICS.some((topic) => topic.id === FERRO_HELP_DEFAULT_TOPIC));
  const libNames = FERRO_HELP_LIBRARIES.map((lib) => lib.name);
  for (const expected of [
    "Matematica",
    "Texto",
    "Tipos",
    "Util",
    "Calendario",
    "Arquivos",
    "Json",
    "Internet",
    "Graficos",
    "Teclado",
    "Mouse",
    "Sons",
    "Dialogo",
    "Telas",
  ]) {
    assert.ok(libNames.includes(expected), `esperava biblioteca ${expected} na ajuda FERRO`);
  }

  const standardLibs = FERRO_HELP_LIBRARIES.filter((lib) => lib.group === "standard");
  const visualLibs = FERRO_HELP_LIBRARIES.filter((lib) => lib.group === "visual");
  assert.equal(standardLibs.length, 8);
  assert.equal(visualLibs.length, 8);
  assert.ok(!libNames.includes("UI"), "UI.* saiu da linguagem");
});

test("FERRO help covers loops and typed functions alongside core topics", () => {
  const topicIds = FERRO_HELP_TOPICS.map((topic) => topic.id);
  for (const id of [
    "ferro:visao-geral",
    "ferro:condicionais",
    "ferro:lacos",
    "ferro:funcoes",
    "ferro:opcao-resultado",
    "ferro:colecoes-tipos-compostos",
    "ferro:ui-mvu",
    "ferro:telas",
  ]) {
    assert.ok(topicIds.includes(id), `esperava tópico ${id} na ajuda FERRO`);
  }
});

test("help.ts organizes navigation around FERRO without classic library menu", () => {
  const helpSource = readFileSync(new URL("../src/help.ts", import.meta.url), "utf8");
  assert.ok(!helpSource.includes("help-assets/bibliotecas.json"));
  assert.ok(!helpSource.includes("Bibliotecas clássicas"));
  assert.ok(helpSource.includes("Primeiros passos com Ferro"));
  assert.ok(helpSource.includes("Bibliotecas padrão"));
  assert.ok(helpSource.includes("UI e Multimídia"));
  assert.ok(helpSource.includes("Referência da sintaxe (.fer)"));
});

test("Ferro help examples use .fer syntax without classic Ferro keywords", () => {
  for (const topic of FERRO_HELP_TOPICS) {
    if (!topic.example) continue;
    assert.ok(topic.example.fileName.endsWith(".fer"));
    assert.ok(!topic.example.source.includes("programa {"));
    assert.ok(!topic.example.source.includes("inclua biblioteca"));
  }
});

test("FERRO help explains chained senao se conditionals with an example", () => {
  const topic = FERRO_HELP_TOPICS.find((item) => item.id === "ferro:condicionais");
  assert.ok(topic);
  assert.ok(topic.paragraphs.some((paragraph) => paragraph.includes("senao se")));
  assert.ok(topic.example?.source.includes("senao se"));
});

test("filterFerroHelp matches accents, library functions and topics case-insensitively", () => {
  const byOpcao = filterFerroHelp("opcao");
  assert.ok(byOpcao.topics.some((t) => t.id === "ferro:opcao-resultado"));

  const byDividir = filterFerroHelp("dividir");
  assert.ok(byDividir.libraries.some((l) => l.id === "ferro-lib:Texto"));

  const byBissexto = filterFerroHelp("bissexto");
  assert.ok(byBissexto.libraries.some((l) => l.id === "ferro-lib:Calendario"));

  const byJson = filterFerroHelp("minificar");
  assert.ok(byJson.libraries.some((l) => l.id === "ferro-lib:Json"));

  const byArquivos = filterFerroHelp("ler_linhas");
  assert.ok(byArquivos.libraries.some((l) => l.id === "ferro-lib:Arquivos"));

  const byInternet = filterFerroHelp("enviar_json");
  assert.ok(byInternet.libraries.some((l) => l.id === "ferro-lib:Internet"));

  const byGraficos = filterFerroHelp("desenhar_poligono");
  assert.ok(byGraficos.libraries.some((l) => l.id === "ferro-lib:Graficos"));

  const byTeclado = filterFerroHelp("ler_tecla");
  assert.ok(byTeclado.libraries.some((l) => l.id === "ferro-lib:Teclado"));

  const byMouse = filterFerroHelp("posicao_x");
  assert.ok(byMouse.libraries.some((l) => l.id === "ferro-lib:Mouse"));

  const bySons = filterFerroHelp("reproduzir_som");
  assert.ok(bySons.libraries.some((l) => l.id === "ferro-lib:Sons"));

  const empty = filterFerroHelp("nao-existe-xyz-123");
  assert.equal(empty.topics.length, 0);
  assert.equal(empty.libraries.length, 0);
});
