import test from "node:test";
import assert from "node:assert/strict";
import { createIdeShortcuts } from "../src/commands/ide-shortcuts.ts";

function fixture() {
  const events = [];
  const actions = new Proxy({}, {
    get: (_target, name) => () => events.push(name),
  });
  return { shortcuts: createIdeShortcuts(actions), events };
}

function keyboardEvent({ key, code = "", ctrlKey = false, metaKey = false, shiftKey = false, altKey = false }) {
  return { key, code, ctrlKey, metaKey, shiftKey, altKey };
}

test("matches and dispatches file, palette, debug, and navigation shortcuts", () => {
  const { shortcuts, events } = fixture();
  const cases = [
    ["Novo arquivo", keyboardEvent({ key: "n", code: "KeyN", ctrlKey: true }), "newFile"],
    ["Paleta de comandos", keyboardEvent({ key: "p", code: "KeyP", ctrlKey: true, shiftKey: true }), "commandPalette"],
    ["Continuar", keyboardEvent({ key: "F5" }), "continueDebug"],
    ["Próxima aba", keyboardEvent({ key: "ArrowRight", altKey: true }), "nextTab"],
  ];

  for (const [command, event, actionName] of cases) {
    const shortcut = shortcuts.find((candidate) => candidate.command === command);
    assert.ok(shortcut, `${command} should be registered`);
    assert.equal(shortcut.matches(event), true);
    shortcut.action();
    assert.equal(events.at(-1), actionName);
  }
});

test("registers all file, symbol-filter, and contextual actions in the catalog", () => {
  const { shortcuts } = fixture();
  const commands = new Set(shortcuts.map(({ command }) => command));

  for (const command of ["Abrir arquivo", "Salvar", "Salvar como", "Fechar todas as abas", "Filtrar tipo inteiro", "Filtrar variável", "Formatar código-fonte"]) {
    assert.ok(commands.has(command), `${command} should be available to the command palette`);
  }
});

test("shortcuts have unique key labels and respect modifier guards", () => {
  const { shortcuts } = fixture();
  const realLabels = shortcuts.map(({ label }) => label).filter((label) => label !== "via paleta de comandos");
  assert.equal(new Set(realLabels).size, realLabels.length, "nenhum atalho de teclado deve ser duplicado");

  const save = shortcuts.find(({ command }) => command === "Salvar");
  assert.equal(save.matches(keyboardEvent({ key: "s", code: "KeyS", ctrlKey: true, altKey: true })), false);

  const stepOver = shortcuts.find(({ command }) => command === "Próxima instrução");
  assert.equal(stepOver.matches(keyboardEvent({ key: "F10", shiftKey: true })), false);
  assert.equal(stepOver.matches(keyboardEvent({ key: "F10" })), true);
});
