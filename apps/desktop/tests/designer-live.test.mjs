import test from "node:test";
import assert from "node:assert/strict";
import { applySnapshot, attributeName, resolveBindings } from "../src/designer/live-tree.ts";
import { buildCanvasModel } from "../src/designer/canvas-model.ts";
import { createScreenExecution } from "../src/designer/screen-execution.ts";

function node(tag, path, attributes = {}, children = [], text = null) {
  return {
    tag,
    path,
    attributes: Object.entries(attributes).map(([name, value]) => ({ name, value })),
    text,
    from: 0,
    to: 0,
    children,
  };
}

const TREE = node("Janela", [], { Nome: "J", Titulo: "Antes", Largura: "300", Altura: "200" }, [
  node("Botao", [0], { Nome: "btnOk", Texto: "OK", X: "8", Y: "8" }),
  node("Rotulo", [1], { Nome: "lbl", Texto: "Olá, {dados.nome}!", X: "8", Y: "48" }),
  node("Combo", [2], { Nome: "cmb", X: "8", Y: "80" }, [node("Item", [2, 0], {}, [], "Velho")]),
  node("CaixaTexto", [3], { Nome: "txt", X: "8", Y: "120" }),
]);

const SNAPSHOT = {
  janela: { titulo: "Depois", largura: 300, altura: 200, redimensionavel: false },
  controles: {
    btnOk: { texto: "Salvar", habilitado: false, visivel: true, x: 8, y: 8 },
    lbl: { texto: "", habilitado: true, visivel: false },
    cmb: { itens: ["Um", "Dois"], selecionado: 1, habilitado: true, visivel: true },
    txt: { texto: "digitado", texto_vazio: "Nome", habilitado: true, visivel: true },
  },
  dados: { nome: "Ana", idade: 9 },
};

test("campo do retrato volta a ser o atributo", () => {
  assert.equal(attributeName("texto_vazio"), "TextoVazio");
  assert.equal(attributeName("x"), "X");
});

test("vínculos mostram os valores de dados; chaves escapadas ficam", () => {
  assert.equal(resolveBindings("Olá, {dados.nome} ({dados.idade})", SNAPSHOT.dados), "Olá, Ana (9)");
  assert.equal(resolveBindings("\\{literal\\} {dados.falta}", SNAPSHOT.dados), "{literal} ");
});

test("o estado vivo muda texto, habilitado, visível, itens e seleção no desenho", () => {
  const model = buildCanvasModel(applySnapshot(TREE, SNAPSHOT));
  const [botao, rotulo, combo, caixa] = model.window.children;
  assert.deepEqual(model.title, [{ text: "Depois", kind: "plain" }]);
  assert.equal(botao.text.map((run) => run.text).join(""), "Salvar");
  assert.equal(botao.disabled, true);
  assert.equal(rotulo.hidden, true);
  assert.equal(rotulo.text.map((run) => run.text).join(""), "Olá, Ana!", "vinculado mostra dados, não o campo");
  assert.deepEqual(combo.items, ["Um", "Dois"]);
  assert.equal(combo.selected, 1);
  assert.equal(caixa.text.map((run) => run.text).join(""), "digitado");
  assert.equal(caixa.placeholder, "Nome");
});

function files({ tabs = [], present = [], disk = {} } = {}) {
  return {
    tabs: () => tabs,
    tabSource: (tab) => tab.text,
    hasFile: (_ws, path) => present.includes(path),
    readFile: async (_ws, path) => disk[path],
  };
}

test("executar a partir do código junta o .design aberto e o código", async () => {
  const started = [];
  const statuses = [];
  const tabs = [{ id: 1, workspaceId: "w", workspacePath: "App/J.design", text: "<Janela não salva />" }];
  const host = {
    files: files({ tabs, present: ["App/J.design", "App/J.fer"], disk: { "App/J.fer": "codigo" } }),
    start: async (design, code) => (started.push([design, code]), "screen-0"),
    setStatus: (message, kind) => statuses.push([message, kind]),
    appendConsole: () => {},
    showConsole: () => {},
  };
  const execution = createScreenExecution(host);
  const code = { id: 2, workspaceId: "w", workspacePath: "App/J.fer", text: "codigo" };
  assert.equal(execution.handles(code), true);
  await execution.run(code);
  assert.deepEqual(started, [["<Janela não salva />", "codigo"]]);
  assert.deepEqual(statuses.at(-1), ["Tela J.design em execução numa janela própria", "ok"]);
});

test("tela sem código roda com código vazio; erro ao iniciar vai para o console", async () => {
  const console = [];
  let shown = false;
  const host = {
    files: files({ present: ["App/J.design"], disk: { "App/J.design": "<Janela />" } }),
    start: async (_design, code) => { throw `sem handler (${JSON.stringify(code)})`; },
    setStatus: () => {},
    appendConsole: (text, kind) => console.push([text, kind]),
    showConsole: () => { shown = true; },
  };
  await createScreenExecution(host).run({ id: 3, workspaceId: "w", workspacePath: "App/J.design" });
  assert.deepEqual(console, [['Não foi possível executar a tela J.design: sem handler ("")\n', "error"]]);
  assert.equal(shown, true);
});
