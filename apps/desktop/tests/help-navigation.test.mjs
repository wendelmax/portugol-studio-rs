import test from "node:test";
import assert from "node:assert/strict";
import {
  buildSyntaxTopicTree,
  createHelpHistory,
  parseInlineCodeRuns,
  resolveHelpTopicForEditor,
  tokenizePorsCode,
} from "../src/help-navigation.ts";

test("help history supports back, forward, deduplication, and truncating forward stack on new visit", () => {
  const history = createHelpHistory("pors:visao-geral");
  assert.equal(history.current(), "pors:visao-geral");
  assert.equal(history.canBack(), false);
  assert.equal(history.canForward(), false);

  history.visit("pors:visao-geral");
  assert.equal(history.canBack(), false);

  history.visit("pors-lib:Texto");
  history.visit("pors-lib:Texto.dividir");
  assert.equal(history.canBack(), true);
  assert.equal(history.canForward(), false);

  assert.equal(history.back(), "pors-lib:Texto");
  assert.equal(history.canForward(), true);
  assert.equal(history.forward(), "pors-lib:Texto.dividir");
  assert.equal(history.canForward(), false);

  assert.equal(history.back(), "pors-lib:Texto");
  history.visit("pors:ponte-rust");
  assert.equal(history.canForward(), false);
  assert.equal(history.current(), "pors:ponte-rust");
});

test("buildSyntaxTopicTree orders language sections pedagogically with nested directories", () => {
  const tree = buildSyntaxTopicTree([
    { id: "topicos/linguagem_portugol/bibliotecas/index.html", title: "Bibliotecas" },
    { id: "topicos/linguagem_portugol/index.html", title: "Linguagem Portugol" },
    { id: "topicos/linguagem_portugol/tipos/index.html", title: "Tipos de Dados" },
    { id: "topicos/linguagem_portugol/tipos/real.html", title: "Tipo Real" },
    { id: "topicos/linguagem_portugol/tipos/inteiro.html", title: "Tipo Inteiro" },
    { id: "topicos/linguagem_portugol/declaracoes/index.html", title: "Declarações" },
    { id: "topicos/linguagem_portugol/declaracoes/variavel.html", title: "Declaração de Variáveis" },
    { id: "topicos/linguagem_portugol/declaracoes/constante.html", title: "Declaração de Constante" },
  ]);

  assert.deepEqual(tree.map((node) => node.title), [
    "Linguagem Portugol",
    "Tipos de Dados",
    "Declarações",
    "Bibliotecas",
  ]);
  assert.deepEqual(tree[1].children.map((node) => node.title), ["Tipo Inteiro", "Tipo Real"]);
  assert.deepEqual(tree[2].children.map((node) => node.title), ["Declaração de Variáveis", "Declaração de Constante"]);
});

test("resolveHelpTopicForEditor resolves modules, members, enums, keywords, and builtins under cursor", () => {
  const doc = `funcao inicio() {\n    seja texto: cadeia = Texto.dividir("a,b", ",")[0];\n    seja r: Opcao<inteiro> = Opcao::Algum(1);\n    escreva(texto, r);\n}`;

  assert.equal(resolveHelpTopicForEditor(doc, doc.indexOf("dividir") + 2), "pors-lib:Texto.dividir");
  assert.equal(resolveHelpTopicForEditor(doc, doc.indexOf("Texto.") + 2), "pors-lib:Texto");
  assert.equal(resolveHelpTopicForEditor(doc, doc.indexOf("Algum") + 2), "pors:opcao-resultado");
  assert.equal(resolveHelpTopicForEditor(doc, doc.indexOf("seja") + 1), "topicos/linguagem_portugol/declaracoes/variavel.html");
  assert.equal(resolveHelpTopicForEditor(doc, doc.indexOf("escreva") + 2), "topicos/linguagem_portugol/entrada_saida/escreva.html");
  assert.equal(resolveHelpTopicForEditor(doc, doc.indexOf("funcao") + 2), "pors:funcoes");
});

test("parseInlineCodeRuns formats backtick spans safely without losing text", () => {
  assert.deepEqual(parseInlineCodeRuns("Use `seja mutavel` e `Opcao<T>`."), [
    { code: false, text: "Use " },
    { code: true, text: "seja mutavel" },
    { code: false, text: " e " },
    { code: true, text: "Opcao<T>" },
    { code: false, text: "." },
  ]);
  assert.deepEqual(parseInlineCodeRuns("Texto sem crase"), [{ code: false, text: "Texto sem crase" }]);
});

test("tokenizePorsCode classifies PORS keywords, types, strings, numbers, comments, and functions", () => {
  const tokens = tokenizePorsCode('// oi\nseja x: inteiro = 10;\nescreva("ok");');
  assert.ok(tokens.some((t) => t.kind === "comment" && t.text === "// oi"));
  assert.ok(tokens.some((t) => t.kind === "keyword" && t.text === "seja"));
  assert.ok(tokens.some((t) => t.kind === "type" && t.text === "inteiro"));
  assert.ok(tokens.some((t) => t.kind === "number" && t.text === "10"));
  assert.ok(tokens.some((t) => t.kind === "function" && t.text === "escreva"));
  assert.ok(tokens.some((t) => t.kind === "string" && t.text === '"ok"'));
});
