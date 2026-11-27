import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  PORS_HELP_DEFAULT_TOPIC,
  PORS_HELP_LIBRARIES,
  PORS_HELP_TOPICS,
  filterPorsHelp,
} from "../src/pors-help.ts";

test("PORS help starts on a PORS topic and covers all 13 PORS libraries grouped into standard and visual", () => {
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
    "PORS: Graficos",
    "PORS: Teclado",
    "PORS: Mouse",
    "PORS: Sons",
  ]) {
    assert.ok(libNames.includes(expected), `esperava biblioteca ${expected} na ajuda PORS`);
  }

  const standardLibs = PORS_HELP_LIBRARIES.filter((lib) => lib.group === "standard");
  const visualLibs = PORS_HELP_LIBRARIES.filter((lib) => lib.group === "visual");
  assert.equal(standardLibs.length, 8);
  assert.equal(visualLibs.length, 5);
});

test("PORS help covers loops and typed functions alongside core topics", () => {
  const topicIds = PORS_HELP_TOPICS.map((topic) => topic.id);
  for (const id of [
    "pors:visao-geral",
    "pors:condicionais",
    "pors:lacos",
    "pors:funcoes",
    "pors:opcao-resultado",
    "pors:colecoes-tipos-compostos",
    "pors:ui-mvu",
  ]) {
    assert.ok(topicIds.includes(id), `esperava tópico ${id} na ajuda PORS`);
  }
});

test("help.ts organizes navigation around PORS without classic library menu", () => {
  const helpSource = readFileSync(new URL("../src/help.ts", import.meta.url), "utf8");
  assert.ok(!helpSource.includes("help-assets/bibliotecas.json"));
  assert.ok(!helpSource.includes("Bibliotecas clássicas"));
  assert.ok(helpSource.includes("Primeiros passos (PORS)"));
  assert.ok(helpSource.includes("Bibliotecas padrão (PORS)"));
  assert.ok(helpSource.includes("UI e Multimídia (PORS)"));
  assert.ok(helpSource.includes("Referência da sintaxe (.fer)"));
});

test("Ferro help examples use .fer syntax without classic Portugol keywords", () => {
  for (const topic of PORS_HELP_TOPICS) {
    if (!topic.example) continue;
    assert.ok(topic.example.fileName.endsWith(".fer"));
    assert.ok(!topic.example.source.includes("programa {"));
    assert.ok(!topic.example.source.includes("inclua biblioteca"));
  }
});

test("PORS help explains chained senao se conditionals with an example", () => {
  const topic = PORS_HELP_TOPICS.find((item) => item.id === "pors:condicionais");
  assert.ok(topic);
  assert.ok(topic.paragraphs.some((paragraph) => paragraph.includes("senao se")));
  assert.ok(topic.example?.source.includes("senao se"));
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

  const byGraficos = filterPorsHelp("desenhar_poligono");
  assert.ok(byGraficos.libraries.some((l) => l.id === "pors-lib:Graficos"));

  const byTeclado = filterPorsHelp("ler_tecla");
  assert.ok(byTeclado.libraries.some((l) => l.id === "pors-lib:Teclado"));

  const byMouse = filterPorsHelp("posicao_x");
  assert.ok(byMouse.libraries.some((l) => l.id === "pors-lib:Mouse"));

  const bySons = filterPorsHelp("reproduzir_som");
  assert.ok(bySons.libraries.some((l) => l.id === "pors-lib:Sons"));

  const empty = filterPorsHelp("nao-existe-xyz-123");
  assert.equal(empty.topics.length, 0);
  assert.equal(empty.libraries.length, 0);
});
