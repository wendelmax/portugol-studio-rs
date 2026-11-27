import test from "node:test";
import assert from "node:assert/strict";
import {
  anchorsValue,
  elementTreeRows,
  eventRows,
  handlerSuggestion,
  parentLayout,
  planWrap,
  propertyEdit,
  propertyRows,
} from "../src/designer/properties-model.ts";

function node(tag, path, attributes = {}, children = []) {
  return {
    tag,
    path,
    attributes: Object.entries(attributes).map(([name, value]) => ({ name, value })),
    text: null,
    from: 0,
    to: 0,
    children,
  };
}

const attr = (name, type, extra = {}) => ({
  name,
  description: `Descrição de ${name}`,
  type,
  min: null,
  max: null,
  choices: [],
  required: false,
  binding: "none",
  placement: "any",
  ...extra,
});

const VISUAL = [
  attr("Nome", "name"),
  attr("X", "integer", { placement: "free", min: -10000, max: 10000 }),
  attr("Y", "integer", { placement: "free" }),
  attr("Ancora", "anchors", { placement: "free", choices: ["Cima", "Baixo", "Esquerda", "Direita"] }),
  attr("Largura", "integer"),
  attr("Alinhamento", "choice", { placement: "layout", choices: ["Inicio", "Centro", "Fim", "Esticar"] }),
  attr("Linha", "integer", { placement: "grid" }),
  attr("Encaixar", "choice", { placement: "dock", choices: ["Cima", "Baixo"] }),
  attr("Habilitado", "bool", { binding: "oneWay" }),
];

const SCHEMA = {
  window: {
    name: "Janela",
    attributes: [attr("Nome", "name", { required: true }), attr("Formato", "integer"), attr("Titulo", "text", { required: true, binding: "oneWay" }), attr("AoAbrir", "event")],
  },
  elements: [
    { name: "Botao", category: "control", attributes: [...VISUAL, attr("Texto", "text", { binding: "oneWay" }), attr("AoClicar", "event")] },
    { name: "Pilha", category: "container", attributes: [...VISUAL] },
    { name: "Grade", category: "container", attributes: [...VISUAL] },
    { name: "Temporizador", category: "component", attributes: [attr("Nome", "name"), attr("Intervalo", "integer"), attr("AoDisparar", "event")] },
  ],
};

const TREE = node("Janela", [], { Nome: "J", Titulo: "T" }, [
  node("Botao", [0], { Nome: "btnSomar", X: "16", Y: "8", Habilitado: "{dados.pode}", AoClicar: "btnSomar_clicar" }),
  node("Pilha", [1], { X: "16", Y: "48", Margem: "4" }, [node("Botao", [1, 0], { Nome: "btnB", Alinhamento: "Centro" })]),
  node("Combo", [2], { Nome: "cmb" }, [node("Item", [2, 0])]),
  node("Temporizador", [3], { Nome: "relogio" }),
]);

test("árvore de elementos com profundidade e sem Item", () => {
  assert.deepEqual(elementTreeRows(TREE), [
    { path: "", depth: 0, tag: "Janela", label: "J (Janela)" },
    { path: "0", depth: 1, tag: "Botao", label: "btnSomar (Botao)" },
    { path: "1", depth: 1, tag: "Pilha", label: "Pilha" },
    { path: "1.0", depth: 2, tag: "Botao", label: "btnB (Botao)" },
    { path: "2", depth: 1, tag: "Combo", label: "cmb (Combo)" },
    { path: "3", depth: 1, tag: "Temporizador", label: "relogio (Temporizador)" },
  ]);
});

test("propriedades mostram só o que vale no contêiner do controle", () => {
  assert.equal(parentLayout(TREE, [0]), "free");
  assert.equal(parentLayout(TREE, [1, 0]), "stack");
  const free = propertyRows(SCHEMA, TREE, [0]).map((row) => row.name);
  assert.deepEqual(free, ["Nome", "X", "Y", "Ancora", "Largura", "Habilitado", "Texto"]);
  const stacked = propertyRows(SCHEMA, TREE, [1, 0]).map((row) => row.name);
  assert.deepEqual(stacked, ["Nome", "Largura", "Alinhamento", "Habilitado", "Texto"]);
});

test("cada tipo ganha o editor certo; valor vinculado é editado como texto", () => {
  const rows = Object.fromEntries(propertyRows(SCHEMA, TREE, [0]).map((row) => [row.name, row]));
  assert.equal(rows.X.editor, "number");
  assert.equal(rows.X.value, "16");
  assert.equal(rows.X.min, -10000);
  assert.equal(rows.Ancora.editor, "anchors");
  assert.equal(rows.Habilitado.editor, "text", "{dados.pode} é vínculo");
  assert.equal(rows.Habilitado.bindable, true);
  const stacked = Object.fromEntries(propertyRows(SCHEMA, TREE, [1, 0]).map((row) => [row.name, row]));
  assert.equal(stacked.Alinhamento.editor, "choice");
  assert.equal(stacked.Alinhamento.value, "Centro");
  assert.equal(stacked.Habilitado.editor, "bool");
});

test("janela e componentes não filtram por contêiner; Formato fica de fora", () => {
  const window = propertyRows(SCHEMA, TREE, []);
  assert.deepEqual(window.map((row) => row.name), ["Nome", "Titulo"]);
  assert.equal(window[0].required, true);
  assert.deepEqual(propertyRows(SCHEMA, TREE, [3]).map((row) => row.name), ["Nome", "Intervalo"]);
  assert.deepEqual(propertyRows(SCHEMA, TREE, [9]), []);
});

test("eventos com nome sugerido no padrão do ADR 0018", () => {
  assert.equal(handlerSuggestion("btnSomar", "AoClicar"), "btnSomar_clicar");
  assert.equal(handlerSuggestion("relogio", "AoDisparar"), "relogio_disparar");
  assert.deepEqual(eventRows(SCHEMA, TREE, [0]), [
    { name: "AoClicar", description: "Descrição de AoClicar", value: "btnSomar_clicar", suggestion: "btnSomar_clicar" },
  ]);
  assert.equal(eventRows(SCHEMA, TREE, [1])[0], undefined);
  assert.equal(eventRows(SCHEMA, TREE, [])[0].suggestion, "J_abrir");
});

test("valor vazio tira o atributo; bordas na ordem do esquema", () => {
  assert.deepEqual(propertyEdit([0], "Texto", "  Somar "), { kind: "setAttribute", path: [0], name: "Texto", value: "Somar" });
  assert.deepEqual(propertyEdit([0], "Texto", "  "), { kind: "removeAttribute", path: [0], name: "Texto" });
  assert.equal(anchorsValue(["Cima", "Baixo", "Esquerda", "Direita"], new Set(["Esquerda", "Cima"])), "Cima,Esquerda");
});

test("envolver em Pilha herda a posição e o tamanho; o controle vai para dentro", () => {
  const plan = planWrap(TREE, [0], "Pilha", { Botao: [100, 32] });
  assert.deepEqual(plan, {
    edits: [
      { kind: "insert", parent: [], index: 0, tag: "Pilha", attributes: [["X", "16"], ["Y", "8"], ["Largura", "100"], ["Altura", "32"]] },
      { kind: "move", path: [1], parent: [0], index: 0 },
    ],
    select: [0],
  });
});

test("envolver em Painel deixa o controle no canto; atributos de posição passam ao contêiner", () => {
  const plan = planWrap(TREE, [1, 0], "Painel", {});
  assert.deepEqual(plan.edits[0], { kind: "insert", parent: [1], index: 0, tag: "Painel", attributes: [["Alinhamento", "Centro"]] });
  assert.deepEqual(plan.edits.slice(1), [
    { kind: "move", path: [1, 1], parent: [1, 0], index: 0 },
    { kind: "removeAttribute", path: [1, 0, 0], name: "Alinhamento" },
    { kind: "setAttribute", path: [1, 0, 0], name: "X", value: "0" },
    { kind: "setAttribute", path: [1, 0, 0], name: "Y", value: "0" },
  ]);
  assert.equal(planWrap(TREE, [], "Pilha", {}), null, "a janela não é envolvida");
  assert.equal(planWrap(TREE, [3], "Pilha", {}), null, "componentes não aparecem na tela");
});
