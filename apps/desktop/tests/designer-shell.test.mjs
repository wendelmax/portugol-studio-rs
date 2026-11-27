import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { codeBehindPath, designPathForCode, isDesignTab, mountDesignerShell } from "../src/designer/designer-shell.ts";

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
    click() { listeners.click?.({ target: null }); },
  };
}

function setup({ files = [], tab }) {
  const opened = [];
  const statuses = [];
  let active = tab;
  const buttons = { design: fakeElement(), split: fakeElement(), text: fakeElement(), code: fakeElement() };
  const area = fakeElement();
  const toolbar = fakeElement();
  const surface = fakeElement();
  const shell = mountDesignerShell({
    area,
    toolbar,
    surface,
    status: fakeElement(),
    buttons,
    activeTab: () => active,
    source: () => "",
    cursorOffset: () => 0,
    selectInEditor: () => {},
    designTree: async () => null,
    designSchema: async () => ({ window: {}, elements: [] }),
    hasFile: (_workspace, path) => files.includes(path),
    openFile: async (_workspace, path) => {
      opened.push(path);
      if (path.endsWith(".design")) active = { id: 9, workspaceId: "w", workspacePath: path, path: `/p/${path}` };
      return true;
    },
    setStatus: (message) => statuses.push(message),
    setTimeout: () => 1,
    clearTimeout: () => {},
  });
  return { shell, buttons, area, toolbar, surface, opened, statuses, setActive: (next) => { active = next; } };
}

const designTab = { id: 1, workspaceId: "w", workspacePath: "App/JanelaPrincipal.design", path: "/p/App/JanelaPrincipal.design" };

test("caminhos do code-behind e do desenho de uma tela", () => {
  assert.equal(codeBehindPath("App/JanelaPrincipal.design"), "App/JanelaPrincipal.pors");
  assert.equal(codeBehindPath("Inicio.DESIGN"), "Inicio.pors");
  assert.equal(designPathForCode("App/JanelaPrincipal.pors"), "App/JanelaPrincipal.design");
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
  assert.equal(buttons.code.disabled, true, "sem .pors, o botão Código fica desligado");
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
  setActive({ id: 3, path: "/p/programa.pors" });
  shell.sync();
  assert.equal(toolbar.hidden, true);
});

test("F7 vai do desenho ao código e volta ao Design", async () => {
  const files = ["App/JanelaPrincipal.design", "App/JanelaPrincipal.pors"];
  const { shell, buttons, opened, setActive, area } = setup({ files, tab: designTab });
  shell.sync();
  assert.equal(buttons.code.disabled, false);
  assert.match(buttons.code.title, /JanelaPrincipal\.pors/);
  assert.equal(await shell.toggleCode(), true);
  assert.deepEqual(opened, ["App/JanelaPrincipal.pors"]);

  setActive({ id: 4, workspaceId: "w", workspacePath: "App/JanelaPrincipal.pors", path: "/p/App/JanelaPrincipal.pors" });
  assert.equal(await shell.toggleCode(), true);
  assert.deepEqual(opened, ["App/JanelaPrincipal.pors", "App/JanelaPrincipal.design"]);
  assert.equal(area.dataset.designView, "design");
});

test("F7 numa tela sem código avisa e não abre nada; fora de telas não faz nada", async () => {
  const { shell, opened, statuses, setActive } = setup({ tab: designTab });
  assert.equal(await shell.toggleCode(), true);
  assert.deepEqual(opened, []);
  assert.match(statuses.at(-1), /primeiro evento/);
  setActive({ id: 5, workspaceId: "w", workspacePath: "solto.pors", path: "/p/solto.pors" });
  assert.equal(await shell.toggleCode(), false);
});

test("a IDE monta as vistas da tela e o atalho F7", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  for (const id of ["design-toolbar", "design-view-design", "design-view-split", "design-view-text", "design-view-code", "design-surface"]) {
    assert.match(html, new RegExp(`id="${id}"`));
  }
  const shortcuts = readFileSync(new URL("../src/commands/ide-shortcuts.ts", import.meta.url), "utf8");
  assert.match(shortcuts, /label: "F7", command: "Alternar desenho e código da tela"/);
});
