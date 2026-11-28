import test from "node:test";
import assert from "node:assert/strict";
import { createTimers, timerSpecs } from "../src/designer/timers.ts";

function node(tag, path, attributes = {}, children = []) {
  return { tag, path, attributes: Object.entries(attributes).map(([name, value]) => ({ name, value })), text: null, from: 0, to: 0, children };
}

const TREE = node("Janela", [], { Nome: "J" }, [
  node("Temporizador", [0], { Nome: "relogio", Intervalo: "1000", AoDisparar: "relogio_disparar" }),
  node("Temporizador", [1], { Nome: "mudo", Intervalo: "500" }),
  node("Temporizador", [2], { Nome: "rapido", Intervalo: "1", AoDisparar: "r" }),
]);

test("só temporizadores nomeados e ligados a AoDisparar, com o estado vivo", () => {
  const snapshot = { janela: {}, dados: null, controles: { relogio: { intervalo: 250, ativo: true }, rapido: { intervalo: 1, ativo: false } } };
  assert.deepEqual(timerSpecs(TREE, snapshot), [
    { name: "relogio", interval: 250, active: true },
    { name: "rapido", interval: 10, active: false },
  ]);
});

function clock() {
  const timers = new Map();
  let next = 1;
  return {
    timers,
    setInterval: (callback, ms) => { const id = next++; timers.set(id, { callback, ms }); return id; },
    clearInterval: (id) => timers.delete(id),
    tickAll: () => [...timers.values()].forEach((timer) => timer.callback()),
  };
}

test("liga, reajusta o intervalo e desliga conforme o estado", () => {
  const c = clock();
  const timers = createTimers(c, async () => {});
  timers.sync([{ name: "relogio", interval: 1000, active: true }]);
  assert.deepEqual(timers.active(), [{ name: "relogio", interval: 1000 }]);
  timers.sync([{ name: "relogio", interval: 1000, active: true }]);
  assert.equal(c.timers.size, 1, "estado igual não recria o relógio");
  timers.sync([{ name: "relogio", interval: 200, active: true }]);
  assert.deepEqual(timers.active(), [{ name: "relogio", interval: 200 }]);
  assert.equal(c.timers.size, 1);
  timers.sync([{ name: "relogio", interval: 200, active: false }]);
  assert.deepEqual(timers.active(), []);
  assert.equal(c.timers.size, 0);
});

test("um disparo só começa quando o anterior terminou", async () => {
  const c = clock();
  const fired = [];
  let release;
  const timers = createTimers(c, (name) => {
    fired.push(name);
    return new Promise((resolve) => { release = resolve; });
  });
  timers.sync([{ name: "relogio", interval: 100, active: true }]);
  c.tickAll();
  c.tickAll();
  assert.deepEqual(fired, ["relogio"], "o segundo tique foi ignorado");
  release();
  await new Promise((resolve) => setTimeout(resolve, 0));
  c.tickAll();
  assert.deepEqual(fired, ["relogio", "relogio"]);
  timers.stopAll();
  assert.equal(c.timers.size, 0);
});
