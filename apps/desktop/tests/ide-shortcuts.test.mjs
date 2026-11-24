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
