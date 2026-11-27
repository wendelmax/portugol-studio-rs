import test from "node:test";
import assert from "node:assert/strict";
import {
  dockSide,
  freePosition,
  gridCell,
  isWithin,
  parseTracks,
  pathAfterMove,
  resizeRect,
  snap,
  stackIndex,
} from "../src/designer/placement.ts";
import { dockInsertIndex, planInsert, planInsertComponent, planMove } from "../src/designer/drop-edits.ts";
import { toolboxGroups } from "../src/designer/toolbox.ts";
import { keyboardTarget, insertPlan } from "../src/designer/interactions.ts";

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

test("snap e posição livre na grade de 8 px, sem coordenadas negativas", () => {
  assert.equal(snap(13), 16);
  assert.equal(snap(11), 8);
  assert.equal(snap(13, false), 13);
  assert.deepEqual(freePosition({ x: 50, y: 30 }, { x: 10, y: 5 }), { x: 40, y: 24 });
  assert.deepEqual(freePosition({ x: 3, y: 2 }, { x: 10, y: 10 }), { x: 0, y: 0 });
  assert.deepEqual(freePosition({ x: 13, y: 13 }, undefined, false), { x: 13, y: 13 });
});

test("posição na pilha pelo meio de cada filho", () => {
  const children = [
    { index: 0, rect: { x: 0, y: 0, width: 100, height: 20 } },
    { index: 2, rect: { x: 0, y: 30, width: 100, height: 20 } },
  ];
  assert.equal(stackIndex({ x: 5, y: 5 }, children, false, 3), 0);
  assert.equal(stackIndex({ x: 5, y: 35 }, children, false, 3), 2);
  assert.equal(stackIndex({ x: 5, y: 45 }, children, false, 3), 3);
  assert.equal(stackIndex({ x: 60, y: 0 }, children, true, 3), 3);
});

test("célula da grade com trilhas resolvidas e espaço entre elas", () => {
  assert.deepEqual(parseTracks("40px 120.5px"), [40, 120.5]);
  const rows = [30, 30, 100];
  const columns = [120, 200];
  assert.deepEqual(gridCell({ x: 10, y: 10 }, rows, columns, 6, 6), { row: 0, column: 0 });
  assert.deepEqual(gridCell({ x: 130, y: 40 }, rows, columns, 6, 6), { row: 1, column: 1 });
  assert.deepEqual(gridCell({ x: 999, y: 999 }, rows, columns, 6, 6), { row: 2, column: 1 });
});

test("borda do encaixe mais próxima", () => {
  const rect = { x: 0, y: 0, width: 300, height: 200 };
  assert.equal(dockSide({ x: 150, y: 5 }, rect), "Cima");
  assert.equal(dockSide({ x: 150, y: 195 }, rect), "Baixo");
  assert.equal(dockSide({ x: 4, y: 100 }, rect), "Esquerda");
  assert.equal(dockSide({ x: 296, y: 100 }, rect), "Direita");
});

test("alças redimensionam com snap e tamanho mínimo", () => {
  const rect = { x: 16, y: 16, width: 100, height: 32 };
  assert.deepEqual(resizeRect(rect, "se", { x: 21, y: 9 }), { x: 16, y: 16, width: 120, height: 40 });
  assert.deepEqual(resizeRect(rect, "w", { x: -9, y: 0 }), { x: 8, y: 16, width: 108, height: 32 });
  assert.deepEqual(resizeRect(rect, "n", { x: 0, y: 100 }), { x: 16, y: 40, width: 100, height: 8 });
  assert.deepEqual(resizeRect(rect, "e", { x: -500, y: 0 }).width, 8);
});

test("caminho depois de mover segue a regra do núcleo", () => {
  assert.deepEqual(pathAfterMove([0, 0], [0], 2), [0, 1]);
  assert.deepEqual(pathAfterMove([0, 1], [0], 0), [0, 0]);
  assert.deepEqual(pathAfterMove([0], [1], 1), [0, 1], "o botão vinha antes da pilha");
  assert.deepEqual(pathAfterMove([1, 1], [], 2), [2]);
  assert.equal(isWithin([1, 0], [1]), true);
  assert.equal(isWithin([1], [1]), true);
  assert.equal(isWithin([2], [1]), false);
});

test("soltar da caixa gera Insert com os atributos do contêiner", () => {
  assert.deepEqual(planInsert("Botao", { kind: "free", parent: [], index: 2, x: 16, y: 24 }), {
    edits: [{ kind: "insert", parent: [], index: 2, tag: "Botao", attributes: [["X", "16"], ["Y", "24"]] }],
    select: [2],
  });
  assert.deepEqual(planInsert("Rotulo", { kind: "grid", parent: [0], index: 3, row: 1, column: 0 }).edits[0].attributes, [
    ["Linha", "1"],
    ["Coluna", "0"],
  ]);
  assert.deepEqual(planInsert("Rotulo", { kind: "dock", parent: [0], index: 1, side: "Baixo" }).edits[0].attributes, [["Encaixar", "Baixo"]]);
  assert.deepEqual(planInsert("Rotulo", { kind: "stack", parent: [0], index: 0 }).edits[0].attributes, []);
  const root = node("Janela", [], {}, [node("Botao", [0])]);
  assert.deepEqual(planInsertComponent("Temporizador", root), {
    edits: [{ kind: "insert", parent: [], index: 1, tag: "Temporizador" }],
    select: [1],
  });
});

test("mover no mesmo contêiner muda só a posição; em outro, move e posiciona", () => {
  assert.deepEqual(planMove([0], { kind: "free", parent: [], index: 3, x: 40, y: 8 }), {
    edits: [
      { kind: "setAttribute", path: [0], name: "X", value: "40" },
      { kind: "setAttribute", path: [0], name: "Y", value: "8" },
    ],
    select: [0],
  });
  assert.deepEqual(planMove([1, 0], { kind: "stack", parent: [1], index: 2 }), {
    edits: [{ kind: "move", path: [1, 0], parent: [1], index: 2 }],
    select: [1, 1],
  });
  assert.deepEqual(planMove([1, 1], { kind: "free", parent: [], index: 2, x: 8, y: 160 }), {
    edits: [
      { kind: "move", path: [1, 1], parent: [], index: 2 },
      { kind: "setAttribute", path: [2], name: "X", value: "8" },
      { kind: "setAttribute", path: [2], name: "Y", value: "160" },
    ],
    select: [2],
  });
  assert.equal(planMove([1], { kind: "stack", parent: [1, 0], index: 0 }), null, "não entra em si mesmo");
  assert.equal(planMove([], { kind: "stack", parent: [0], index: 0 }), null, "a janela não se move");
});

test("no encaixe, o controle entra antes do centro", () => {
  const dock = node("Encaixe", [0], {}, [
    node("Rotulo", [0, 0], { Encaixar: "Cima" }),
    node("CaixaTexto", [0, 1]),
  ]);
  assert.equal(dockInsertIndex(dock), 1);
  const allDocked = node("Encaixe", [0], {}, [node("Rotulo", [0, 0], { Encaixar: "Cima" })]);
  assert.equal(dockInsertIndex(allDocked), 1);
  assert.equal(dockInsertIndex(node("Encaixe", [0])), 0);
});

test("inserir pelo teclado usa o contêiner selecionado, ou o pai do controle selecionado", () => {
  const tree = node("Janela", [], {}, [
    node("Botao", [0], { X: "16", Y: "16" }),
    node("Pilha", [1], {}, [node("Rotulo", [1, 0])]),
    node("Grade", [2]),
  ]);
  assert.deepEqual(keyboardTarget(tree, null), { kind: "free", parent: [], index: 3, x: 32, y: 32 });
  assert.deepEqual(keyboardTarget(tree, "1"), { kind: "stack", parent: [1], index: 1 });
  assert.deepEqual(keyboardTarget(tree, "1.0"), { kind: "stack", parent: [1], index: 1 });
  assert.deepEqual(keyboardTarget(tree, "2"), { kind: "grid", parent: [2], index: 0, row: 0, column: 0 });
  assert.deepEqual(keyboardTarget(tree, "0").parent, []);
  assert.equal(insertPlan(tree, "Botao", null), null);
  assert.equal(insertPlan(tree, "DialogoArquivo", null).edits[0].index, 3);
});

test("caixa de ferramentas agrupa por categoria e deixa a Janela de fora", () => {
  const schema = {
    window: { name: "Janela", category: "container" },
    elements: [
      { name: "Botao", label: "Botão", category: "control" },
      { name: "Pilha", label: "Pilha", category: "container" },
      { name: "Temporizador", label: "Temporizador", category: "component" },
      { name: "Rotulo", label: "Rótulo", category: "control" },
    ],
  };
  const groups = toolboxGroups(schema);
  assert.deepEqual(groups.map((group) => group.title), ["Controles", "Contêineres", "Componentes"]);
  assert.deepEqual(groups[0].tools.map((tool) => tool.name), ["Botao", "Rotulo"]);
});
