import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { filterCommands, isDebugCommandAvailable, nextCommandIndex } from "../src/command-palette.ts";

const commands = [
  { command: "Abrir arquivo", label: "Ctrl+O" },
  { command: "Salvar", label: "Ctrl+S" },
  { command: "Ajuda da linguagem", label: "F1" },
];

test("searches command names and shortcuts without case or accent sensitivity", () => {
  assert.deepEqual(filterCommands(commands, "ABRIR"), [commands[0]]);
  assert.deepEqual(filterCommands(commands, "ajuda linguagem"), [commands[2]]);
  assert.deepEqual(filterCommands(commands, "ctrl+s"), [commands[1]]);
  assert.deepEqual(filterCommands(commands, "nao existe"), []);
});

test("moves the selected command cyclically with arrow keys", () => {
  assert.equal(nextCommandIndex(3, -1, "ArrowDown"), 0);
  assert.equal(nextCommandIndex(3, -1, "ArrowUp"), 2);
  assert.equal(nextCommandIndex(3, 2, "ArrowDown"), 0);
  assert.equal(nextCommandIndex(3, 0, "ArrowUp"), 2);
  assert.equal(nextCommandIndex(0, -1, "ArrowDown"), -1);
});

test("offers only debug actions supported by the current runtime state", () => {
  assert.equal(isDebugCommandAvailable("Depurar", "portugol", false, false), true);
  assert.equal(isDebugCommandAvailable("Depurar", "portugol", true, false), false);
  assert.equal(isDebugCommandAvailable("Interromper", "portugol", false, false), false);
  assert.equal(isDebugCommandAvailable("Pausar", "portugol", true, false), true);
  assert.equal(isDebugCommandAvailable("Pausar", "portugol", true, true), false);
  assert.equal(isDebugCommandAvailable("Continuar", "portugol", true, false), false);
  assert.equal(isDebugCommandAvailable("Continuar", "portugol", true, true), true);
  assert.equal(isDebugCommandAvailable("Depurar", "rust", false, false), false);
});

test("palette dialog exposes an accessible search, listbox, and empty state", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="command-palette"[^>]*aria-labelledby="command-palette-title"/);
  assert.match(html, /id="command-search"[^>]*role="combobox"[^>]*aria-controls="command-results"/);
  assert.match(html, /id="command-results"[^>]*role="listbox"/);
  assert.match(html, /id="command-empty"[^>]*role="status"/);
});

test("command palette shortcut is advertised", () => {
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(main, /Ctrl\+Shift\+P[^\n]*Paleta de comandos/i);
});
