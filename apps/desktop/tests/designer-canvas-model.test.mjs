import test from "node:test";
import assert from "node:assert/strict";
import {
  buildCanvasModel,
  nodeAt,
  pathAtOffset,
  pathFromKey,
  pathKey,
  textRuns,
  thicknessCss,
  tracksCss,
} from "../src/designer/canvas-model.ts";

let offset = 0;
function node(tag, attributes = {}, children = [], path = [], text = null) {
  const from = offset;
  offset += 10;
  const built = {
    tag,
    path,
    attributes: Object.entries(attributes).map(([name, value]) => ({ name, value })),
    text,
    from,
    to: 0,
    children: children.map((child, index) => child([...path, index])),
  };
  built.to = Math.max(offset, ...built.children.map((child) => child.to)) + 1;
  offset = built.to;
  return built;
}
const el = (tag, attributes = {}, children = [], text = null) => (path) => node(tag, attributes, children, path, text);
const SIZES = { Botao: [100, 32], Rotulo: [100, 24], CaixaTexto: [160, 28] };

test("espessura e trilhas viram CSS como no WPF", () => {
  assert.equal(thicknessCss("8"), "8px");
  assert.equal(thicknessCss("8,4"), "4px 8px");
  assert.equal(thicknessCss("1,2,3,4"), "2px 3px 4px 1px");
  assert.equal(thicknessCss("1,2,3"), null);
  assert.equal(thicknessCss("x"), null);
  assert.equal(tracksCss("Auto,*,2*,120"), "auto 1fr 2fr 120px");
  assert.equal(tracksCss(undefined), "1fr");
});

test("texto com tecla de acesso, sublinhado duplo, vínculos e chaves escapadas", () => {
  assert.deepEqual(textRuns("_Salvar"), [
    { text: "S", kind: "access" },
    { text: "alvar", kind: "plain" },
  ]);
  assert.deepEqual(textRuns("a__b"), [{ text: "a_b", kind: "plain" }]);
  assert.deepEqual(textRuns("Olá, {dados.nome}! \\{x\\}"), [
    { text: "Olá, ", kind: "plain" },
    { text: "{dados.nome}", kind: "binding" },
    { text: "! {x}", kind: "plain" },
  ]);
  assert.deepEqual(textRuns("_a_b"), [
    { text: "a", kind: "access" },
    { text: "_b", kind: "plain" },
  ]);
  assert.deepEqual(textRuns("x_y", false), [{ text: "x_y", kind: "plain" }]);
});

test("posição livre usa X, Y e o tamanho padrão do controle", () => {
  const root = node("Janela", { Nome: "J", Titulo: "Conta", Largura: "360", Altura: "240", Preenchimento: "8" }, [
    el("Botao", { Nome: "btnOk", X: "10", Y: "20", Texto: "_OK" }),
    el("Temporizador", { Nome: "relogio" }),
  ]);
  const model = buildCanvasModel(root, SIZES);
  assert.deepEqual(model.title, [{ text: "Conta", kind: "plain" }]);
  assert.equal(model.window.style.width, "360px");
  assert.equal(model.window.style.padding, "8px");
  const [botao] = model.window.children;
  assert.equal(model.window.children.length, 1, "componentes não aparecem na janela");
  assert.equal(botao.path, "0");
  assert.equal(botao.name, "btnOk");
  assert.deepEqual(
    [botao.style.position, botao.style.left, botao.style.top, botao.style.width, botao.style.height],
    ["absolute", "10px", "20px", "100px", "32px"],
  );
  assert.equal(botao.text[0].kind, "access");
  assert.deepEqual(model.components, [{ path: "1", tag: "Temporizador", name: "relogio" }]);
});

test("pilha sozinha na janela ocupa tudo e empilha com espaço e alinhamento", () => {
  const root = node("Janela", { Largura: "300", Altura: "200" }, [
    el("Pilha", { Espaco: "8", Margem: "16" }, [
      el("Rotulo", { Texto: "Nome" }),
      el("CaixaTexto", { Nome: "txt", TextoVazio: "Digite", Alinhamento: "Inicio", Largura: "120" }),
    ]),
  ]);
  const [pilha] = buildCanvasModel(root, SIZES).window.children;
  assert.equal(pilha.style.width, "100%");
  assert.equal(pilha.style.position, undefined);
  assert.equal(pilha.style.margin, "16px");
  assert.equal(pilha.style["flex-direction"], "column");
  assert.equal(pilha.style.gap, "8px");
  const [rotulo, caixa] = pilha.children;
  assert.equal(rotulo.style["align-self"], "stretch");
  assert.equal(caixa.style["align-self"], "start");
  assert.equal(caixa.style.width, "120px");
  assert.equal(caixa.placeholder, "Digite");
  assert.equal(caixa.path, "0.1");
});

test("pilha com irmãos é posicionada por X e Y", () => {
  const root = node("Janela", { Largura: "300", Altura: "200" }, [
    el("Pilha", { X: "5", Y: "6", Orientacao: "Horizontal" }),
    el("Botao", { X: "0", Y: "0" }),
  ]);
  const [pilha] = buildCanvasModel(root, SIZES).window.children;
  assert.equal(pilha.style.position, "absolute");
  assert.equal(pilha.style.left, "5px");
  assert.equal(pilha.style["flex-direction"], "row");
});

test("grade posiciona por Linha e Coluna", () => {
  const root = node("Janela", { Largura: "300", Altura: "200" }, [
    el("Grade", { Linhas: "Auto,*", Colunas: "120,*", Espaco: "4" }, [
      el("Rotulo", { Linha: "1", Coluna: "0", AlinhamentoVertical: "Centro" }),
    ]),
  ]);
  const [grade] = buildCanvasModel(root, SIZES).window.children;
  assert.equal(grade.style["grid-template-rows"], "auto 1fr");
  assert.equal(grade.style["grid-template-columns"], "120px 1fr");
  const [rotulo] = grade.children;
  assert.equal(rotulo.style["grid-row"], "2");
  assert.equal(rotulo.style["grid-column"], "1");
  assert.equal(rotulo.style["align-self"], "center");
});

test("encaixe ocupa as bordas em ordem e o último preenche", () => {
  const root = node("Janela", { Largura: "300", Altura: "200" }, [
    el("Encaixe", {}, [
      el("Rotulo", { Nome: "topo", Encaixar: "Cima" }),
      el("Rotulo", { Nome: "lado", Encaixar: "Direita" }),
      el("CaixaTexto", { Nome: "centro" }),
    ]),
  ]);
  const [encaixe] = buildCanvasModel(root, SIZES).window.children;
  const [chain] = encaixe.children;
  assert.equal(chain.kind, "dock-rest");
  assert.equal(chain.style["flex-direction"], "column");
  const [topo, rest] = chain.children;
  assert.equal(topo.name, "topo");
  assert.equal(topo.style.flex, "0 0 auto");
  assert.equal(rest.style["flex-direction"], "row-reverse");
  const [lado, last] = rest.children;
  assert.equal(lado.name, "lado");
  assert.equal(last.children[0].name, "centro");
  assert.equal(last.children[0].style.flex, "1 1 auto");
});

test("lista e combo mostram os itens; painel posiciona livre por dentro", () => {
  const root = node("Janela", { Largura: "300", Altura: "200" }, [
    el("Combo", { X: "0", Y: "0" }, [el("Item", {}, [], "Somar"), el("Item", {}, [], "Subtrair")]),
    el("Painel", { X: "0", Y: "40", Titulo: "Opções", Habilitado: "falso" }, [el("CaixaSelecao", { X: "4", Y: "4", Marcado: "verdadeiro" })]),
  ]);
  const [combo, painel] = buildCanvasModel(root, SIZES).window.children;
  assert.deepEqual(combo.items, ["Somar", "Subtrair"]);
  assert.equal(combo.children.length, 0);
  assert.equal(painel.disabled, true);
  assert.equal(painel.style.border, "1px solid var(--design-border)");
  assert.equal(painel.children[0].style.position, "absolute");
  assert.equal(painel.children[0].checked, true);
});

test("caminhos, nó por caminho e nó sob o cursor", () => {
  assert.equal(pathKey([0, 2]), "0.2");
  assert.deepEqual(pathFromKey("0.2"), [0, 2]);
  assert.deepEqual(pathFromKey(""), []);
  const root = node("Janela", {}, [el("Pilha", {}, [el("Botao"), el("Rotulo")])]);
  const rotulo = nodeAt(root, [0, 1]);
  assert.equal(rotulo.tag, "Rotulo");
  assert.equal(nodeAt(root, [3]), null);
  assert.deepEqual(pathAtOffset(root, rotulo.from + 1), [0, 1]);
  assert.deepEqual(pathAtOffset(root, root.from), []);
  assert.equal(pathAtOffset(root, root.to + 5), null);
});
