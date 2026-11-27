import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { codeBehindCandidates, codeBehindPath, designPathForCode, isDesignTab, mountDesignerShell, textChange } from "../src/designer/designer-shell.ts";

function fakeElement() {
  const listeners = {};
  const attributes = {};
  return {
    hidden: false,
    disabled: false,
    title: "",
    textContent: "",
    dataset: {},
    firstElementChild: null,
    children: [],
    classList: { add() {}, remove() {} },
    setAttribute(name, value) { attributes[name] = value; },
    getAttribute(name) { return attributes[name]; },
    addEventListener(type, listener) { listeners[type] = listener; },
    replaceChildren(...children) { this.children = children; this.firstElementChild = children[0] ?? null; },
    querySelector() { return null; },
    click() { listeners.click?.({ target: null }); },
    dispatch(type, event) { listeners[type]?.(event); },
  };
}

function setup({ files = [], tab, tree = null, editDesign = async (source) => source, history = [], schema = { window: {}, elements: [] }, codeTexts = {} }) {
  const opened = [];
  const statuses = [];
  const edits = [];
  let source = "<Janela />";
  let cursorAt = -1;
  let active = tab;
  const buttons = { design: fakeElement(), split: fakeElement(), text: fakeElement(), code: fakeElement(), undo: fakeElement(), redo: fakeElement() };
  const viewport = fakeElement();
  const area = fakeElement();
  const toolbar = fakeElement();
  const surface = fakeElement();
  const shell = mountDesignerShell({
    area,
    toolbar,
    surface,
    viewport,
    status: fakeElement(),
    buttons,
    activeTab: () => active,
    source: () => source,
    cursorOffset: () => 0,
    selectInEditor: () => {},
    designTree: async () => tree,
    designSchema: async () => schema,
    hasFile: (_workspace, path) => files.includes(path),
    openFile: async (_workspace, path) => {
      opened.push(path);
      if (path.endsWith(".design")) active = { id: 9, workspaceId: "w", workspacePath: path, path: `/p/${path}` };
      else { active = { id: 10, workspaceId: "w", workspacePath: path, path: `/p/${path}` }; source = codeTexts[path] ?? ""; }
      return true;
    },
    createFile: async (_workspace, path) => {
      opened.push(`criado:${path}`);
      files.push(path);
      active = { id: 11, workspaceId: "w", workspacePath: path, path: `/p/${path}` };
      source = "";
      return true;
    },
    editActive: (insert, cursor) => {
      if (insert) source = source.slice(0, insert.from) + insert.insert + source.slice(insert.from);
      cursorAt = cursor;
    },
    editDesign: async (current, requested) => {
      edits.push(requested);
      return editDesign(current, requested);
    },
    replaceSource: (next) => { source = next; },
    canvas: { paint() {}, highlight() {} },
    undo: () => { history.push("undo"); return true; },
    redo: () => { history.push("redo"); return true; },
    setStatus: (message) => statuses.push(message),
    setTimeout: () => 1,
    clearTimeout: () => {},
  });
  return { shell, viewport, buttons, area, toolbar, surface, opened, statuses, edits, source: () => source, cursor: () => cursorAt, setActive: (next) => { active = next; } };
}

const designTab = { id: 1, workspaceId: "w", workspacePath: "App/JanelaPrincipal.design", path: "/p/App/JanelaPrincipal.design" };

test("caminhos do code-behind e do desenho de uma tela", () => {
  assert.equal(codeBehindPath("App/JanelaPrincipal.design"), "App/JanelaPrincipal.fer");
  assert.equal(codeBehindPath("Inicio.DESIGN"), "Inicio.fer");
  assert.deepEqual(codeBehindCandidates("App/JanelaPrincipal.design"), ["App/JanelaPrincipal.fer"]);
  assert.equal(designPathForCode("App/JanelaPrincipal.fer"), "App/JanelaPrincipal.design");
  assert.equal(designPathForCode("main.rs"), null);
  assert.equal(isDesignTab(designTab), true);
  assert.equal(isDesignTab({ id: 2, path: "/p/meu-app.ferproj" }), false);
  assert.equal(isDesignTab(undefined), false);
});

test(".design aberto direto mostra a marcação e as vistas na barra", () => {
  const { shell, toolbar, surface, area, buttons } = setup({ tab: designTab });
  shell.sync();
  assert.equal(toolbar.hidden, false);
  assert.equal(surface.hidden, true);
  assert.equal(area.dataset.designView, undefined);
  assert.equal(buttons.text.getAttribute("aria-pressed"), "true");
  assert.equal(buttons.code.disabled, true, "sem .fer, o botão Código fica desligado");
});

test("o nó de tela abre no Design e o botão Dividido troca a vista", () => {
  const { shell, surface, area, buttons } = setup({ tab: designTab });
  shell.screenOpened();
  assert.equal(surface.hidden, false);
  assert.equal(area.dataset.designView, "design");
  assert.equal(buttons.design.getAttribute("aria-pressed"), "true");
  buttons.split.click();
  assert.equal(area.dataset.designView, "split");
  buttons.text.click();
  assert.equal(area.dataset.designView, undefined);
  assert.equal(surface.hidden, true);
});

test("abas que não são telas escondem a barra", () => {
  const { shell, toolbar, setActive } = setup({ tab: designTab });
  setActive({ id: 3, path: "/p/programa.fer" });
  shell.sync();
  assert.equal(toolbar.hidden, true);
});

test("F7 vai do desenho ao código e volta ao Design", async () => {
  const files = ["App/JanelaPrincipal.design", "App/JanelaPrincipal.fer"];
  const { shell, buttons, opened, setActive, area } = setup({ files, tab: designTab });
  shell.sync();
  assert.equal(buttons.code.disabled, false);
  assert.match(buttons.code.title, /JanelaPrincipal\.fer/);
  assert.equal(await shell.toggleCode(), true);
  assert.deepEqual(opened, ["App/JanelaPrincipal.fer"]);

  setActive({ id: 4, workspaceId: "w", workspacePath: "App/JanelaPrincipal.fer", path: "/p/App/JanelaPrincipal.fer" });
  assert.equal(await shell.toggleCode(), true);
  assert.deepEqual(opened, ["App/JanelaPrincipal.fer", "App/JanelaPrincipal.design"]);
  assert.equal(area.dataset.designView, "design");
});

test("F7 numa tela sem código avisa e não abre nada; fora de telas não faz nada", async () => {
  const { shell, opened, statuses, setActive } = setup({ tab: designTab });
  assert.equal(await shell.toggleCode(), true);
  assert.deepEqual(opened, []);
  assert.match(statuses.at(-1), /primeiro evento/);
  setActive({ id: 5, workspaceId: "w", workspacePath: "solto.fer", path: "/p/solto.fer" });
  assert.equal(await shell.toggleCode(), false);
});

test("textChange troca só o trecho alterado", () => {
  assert.equal(textChange("abc", "abc"), null);
  assert.deepEqual(textChange("<A X=\"1\" />", "<A X=\"16\" />"), { from: 7, to: 7, insert: "6" });
  assert.deepEqual(textChange("abcdef", "abXYef"), { from: 2, to: 4, insert: "XY" });
  assert.deepEqual(textChange("aaa", "aaaa"), { from: 3, to: 3, insert: "a" });
});

function treeNode(tag, path, attributes = {}, children = []) {
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

test("item da caixa de ferramentas insere em cascata na janela e troca o texto do editor", async () => {
  const tree = treeNode("Janela", [], { Largura: "300", Altura: "200" }, [treeNode("Botao", [0], { X: "16", Y: "16" })]);
  const { shell, edits, source } = setup({ tab: designTab, tree, editDesign: async () => "<Janela>novo</Janela>" });
  shell.screenOpened();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(await shell.insertTool("Rotulo"), true);
  assert.deepEqual(edits.at(-1), [
    { kind: "insert", parent: [], index: 1, tag: "Rotulo", attributes: [["X", "32"], ["Y", "32"]] },
  ]);
  assert.equal(source(), "<Janela>novo</Janela>");
});

test("componente da caixa vai para o fim da janela, mesmo com uma pilha selecionada", async () => {
  const tree = treeNode("Janela", [], {}, [treeNode("Pilha", [0])]);
  const { shell, edits } = setup({ tab: designTab, tree });
  shell.screenOpened();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(await shell.insertTool("Temporizador"), true);
  assert.deepEqual(edits.at(-1), [{ kind: "insert", parent: [], index: 1, tag: "Temporizador" }]);
});

test("edição recusada pelo núcleo vira mensagem e não troca o texto", async () => {
  const tree = treeNode("Janela", [], {}, []);
  const { shell, statuses, source } = setup({
    tab: designTab,
    tree,
    editDesign: async () => { throw "Posição fora da janela"; },
  });
  shell.screenOpened();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.equal(await shell.insertTool("Botao"), false);
  assert.equal(statuses.at(-1), "Posição fora da janela");
  assert.equal(source(), "<Janela />");
});

test("desfazer e refazer usam o histórico do editor, pelos botões e pelo teclado do canvas", () => {
  const history = [];
  const { viewport, buttons } = setup({ tab: designTab, history });
  buttons.undo.click();
  buttons.redo.click();
  const key = (key, extra = {}) => ({ key, ctrlKey: true, metaKey: false, altKey: false, shiftKey: false, preventDefault() {}, ...extra });
  viewport.dispatch("keydown", key("z"));
  viewport.dispatch("keydown", key("y"));
  viewport.dispatch("keydown", key("Z", { shiftKey: true }));
  viewport.dispatch("keydown", key("z", { ctrlKey: false }));
  assert.deepEqual(history, ["undo", "redo", "undo", "redo", "redo"]);
});

const HANDLER_SCHEMA = {
  window: { name: "Janela", label: "Janela", defaultEvent: "AoAbrir", attributes: [] },
  elements: [
    { name: "Botao", label: "Botão", defaultEvent: "AoClicar", defaultWidth: 100, defaultHeight: 32, attributes: [] },
    { name: "Rotulo", label: "Rótulo", defaultEvent: null, defaultWidth: 100, defaultHeight: 24, attributes: [] },
  ],
};

async function handlerSetup(options) {
  const tree = treeNode("Janela", [], { Nome: "J", Dados: "Conta" }, [
    treeNode("Botao", [0], { Nome: "btnOk" }),
    treeNode("Rotulo", [1], { Nome: "lbl" }),
  ]);
  const env = setup({ tab: designTab, tree, schema: HANDLER_SCHEMA, ...options });
  env.shell.screenOpened();
  await new Promise((resolve) => setTimeout(resolve, 0));
  return env;
}

test("duplo clique numa tela sem código liga o evento, cria o .fer com o modelo e a função", async () => {
  const { shell, edits, opened, source, cursor } = await handlerSetup({ files: ["App/JanelaPrincipal.design"] });
  assert.equal(await shell.createHandler("0"), true);
  assert.deepEqual(edits[0], [{ kind: "setAttribute", path: [0], name: "AoClicar", value: "btnOk_clicar" }]);
  assert.deepEqual(opened, ["criado:App/JanelaPrincipal.fer"]);
  const code = source();
  assert.match(code, /^\/\/ Código da tela J\./);
  assert.match(code, /registro Conta \{/);
  assert.match(code, /funcao btnOk_clicar\(tela: J\) -> J \{/);
  assert.equal(code.slice(0, cursor()).endsWith("seja mutavel t: J = tela;\n    "), true);
});

test("duplo clique com a função já escrita só leva o cursor até ela", async () => {
  const existing = "funcao btnOk_clicar(tela: J) -> J {\n    retorne tela;\n}\n";
  const files = ["App/JanelaPrincipal.design", "App/JanelaPrincipal.fer"];
  const { shell, edits, opened, source, cursor } = await handlerSetup({ files, codeTexts: { "App/JanelaPrincipal.fer": existing } });
  assert.equal(await shell.createHandler("0"), true);
  assert.equal(edits.length, 1, "liga o evento que ainda não estava na marcação");
  assert.deepEqual(opened, ["App/JanelaPrincipal.fer"]);
  assert.equal(source(), existing);
  assert.equal(existing.slice(cursor()).startsWith("    retorne"), true);
});

test("controle sem evento padrão avisa e não muda nada", async () => {
  const { shell, edits, statuses } = await handlerSetup({ files: ["App/JanelaPrincipal.design"] });
  assert.equal(await shell.createHandler("1"), false);
  assert.equal(edits.length, 0);
  assert.match(statuses.at(-1), /Rótulo não tem evento padrão/);
});

test("a IDE monta as vistas da tela e o atalho F7", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  for (const id of ["design-toolbar", "design-view-design", "design-view-split", "design-view-text", "design-view-code", "design-surface", "design-toolbox", "design-viewport", "design-side", "design-undo", "design-redo"]) {
    assert.match(html, new RegExp(`id="${id}"`));
  }
  const shortcuts = readFileSync(new URL("../src/commands/ide-shortcuts.ts", import.meta.url), "utf8");
  assert.match(shortcuts, /label: "F7", command: "Alternar desenho e código da tela"/);
});
