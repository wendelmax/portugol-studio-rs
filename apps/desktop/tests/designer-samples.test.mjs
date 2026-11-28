import test from "node:test";
import assert from "node:assert/strict";
import { buildCanvasModel, sampleData } from "../src/designer/canvas-model.ts";
import { sampleEdit, sampleRows } from "../src/designer/properties-model.ts";

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

const TREE = node("Janela", [], { Nome: "J", Titulo: "Conta de {dados.nome}", Dados: "Conta" }, [
  node("Rotulo", [0], { Nome: "lbl", Texto: "Resultado: {dados.resultado} ({dados.pessoa.nome})", X: "0", Y: "0" }),
  node("Botao", [1], { Nome: "btn", Texto: "Salvar", Habilitado: "{dados.pode}", X: "0", Y: "40" }),
  node("DadosExemplo", [2], { resultado: "42", nome: "Ana", pode: "falso" }),
]);

test("o designer mostra os exemplos no lugar dos vínculos diretos de dados", () => {
  assert.deepEqual([...sampleData(TREE)], [["resultado", "42"], ["nome", "Ana"], ["pode", "falso"]]);
  const model = buildCanvasModel(TREE);
  assert.deepEqual(model.title, [{ text: "Conta de ", kind: "plain" }, { text: "Ana", kind: "sample" }]);
  const [rotulo, botao] = model.window.children;
  assert.deepEqual(rotulo.text, [
    { text: "Resultado: ", kind: "plain" },
    { text: "42", kind: "sample" },
    { text: " (", kind: "plain" },
    { text: "{dados.pessoa.nome}", kind: "binding" },
    { text: ")", kind: "plain" },
  ]);
  assert.equal(botao.disabled, true, "Habilitado vinculado usa o exemplo");
});

test("sem exemplos (ou na execução) os vínculos aparecem como marcadores", () => {
  const model = buildCanvasModel(TREE, {}, false);
  assert.equal(model.window.children[0].text[1].kind, "binding");
  assert.equal(model.window.children[1].disabled, false);
});

const FIELDS = [
  { name: "resultado", type: "inteiro", editable: true },
  { name: "pessoa", type: "Pessoa", editable: false },
  { name: "nova", type: "cadeia", editable: true },
];

test("linhas do painel com o valor de exemplo de cada campo", () => {
  assert.deepEqual(sampleRows(TREE, FIELDS), [
    { name: "resultado", type: "inteiro", editable: true, value: "42" },
    { name: "pessoa", type: "Pessoa", editable: false, value: "" },
    { name: "nova", type: "cadeia", editable: true, value: "" },
  ]);
});

test("editar exemplos cria, muda, tira e apaga o DadosExemplo", () => {
  assert.deepEqual(sampleEdit(TREE, "resultado", " 7 "), [{ kind: "setAttribute", path: [2], name: "resultado", value: "7" }]);
  assert.equal(sampleEdit(TREE, "resultado", "42"), null, "valor igual não muda nada");
  assert.deepEqual(sampleEdit(TREE, "nome", ""), [{ kind: "removeAttribute", path: [2], name: "nome" }]);
  const only = node("Janela", [], {}, [node("Botao", [0]), node("DadosExemplo", [1], { nome: "Ana" })]);
  assert.deepEqual(sampleEdit(only, "nome", ""), [{ kind: "remove", path: [1] }]);
  const none = node("Janela", [], {}, [node("Botao", [0])]);
  assert.deepEqual(sampleEdit(none, "nome", "Bia"), [
    { kind: "insert", parent: [], index: 1, tag: "DadosExemplo", attributes: [["nome", "Bia"]] },
  ]);
  assert.equal(sampleEdit(none, "nome", ""), null);
});
