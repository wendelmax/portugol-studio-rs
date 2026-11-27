import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createPaletteContext, filterCommands, isDebugCommandAvailable, isPaletteCommandAvailable, nextCommandIndex } from "../src/command-palette.ts";

const commands = [
  { command: "Abrir arquivo", label: "Ctrl+O" },
  { command: "Salvar", label: "Ctrl+S" },
  { command: "Ajuda da linguagem", label: "F1" },
];

test("builds palette capabilities from the active tab and current execution state", () => {
  const context = createPaletteContext({
    activeTab: { language: "rust", cargoProject: true },
    cargoAvailable: true,
    rustcAvailable: false,
    runActive: true,
    paused: true,
    porsRunPending: false,
    canRunAsRust: () => false,
  });

  assert.deepEqual(context, {
    language: "rust", cargoProject: true, cargoAvailable: true, canRunAsRust: false,
    runActive: true, paused: true, porsRunPending: false,
  });
});

test("empty palette context does not advertise a language or Cargo project", () => {
  assert.deepEqual(createPaletteContext({
    activeTab: undefined,
    cargoAvailable: true,
    rustcAvailable: true,
    runActive: false,
    paused: false,
    porsRunPending: false,
    canRunAsRust: () => true,
  }), {
    language: null, cargoProject: false, cargoAvailable: true, canRunAsRust: false,
    runActive: false, paused: false, porsRunPending: false,
  });
});

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
  assert.equal(isDebugCommandAvailable("Depurar", "portugol-rs", false, false), true);
  assert.equal(isDebugCommandAvailable("Depurar", "portugol-rs", true, false), false);
  assert.equal(isDebugCommandAvailable("Interromper", "portugol-rs", false, false), false);
  assert.equal(isDebugCommandAvailable("Pausar", "portugol-rs", true, false), true);
  assert.equal(isDebugCommandAvailable("Pausar", "portugol-rs", true, true), false);
  assert.equal(isDebugCommandAvailable("Continuar", "portugol-rs", true, false), false);
  assert.equal(isDebugCommandAvailable("Continuar", "portugol-rs", true, true), true);
  assert.equal(isDebugCommandAvailable("Depurar", "rust", false, false), false);
});

test("PORS palette debugger commands follow VM state while Rust keeps them unavailable", () => {
  const idlePors = {
    language: "portugol-rs", cargoProject: false, cargoAvailable: false, canRunAsRust: false,
    runActive: false, paused: false, porsRunPending: false,
  };
  assert.equal(isPaletteCommandAvailable("Depurar", idlePors), true);
  assert.equal(isPaletteCommandAvailable("Interromper", idlePors), false);
  assert.equal(isPaletteCommandAvailable("Pausar", idlePors), false);

  const runningPors = { ...idlePors, runActive: true };
  assert.equal(isPaletteCommandAvailable("Interromper", runningPors), true);
  assert.equal(isPaletteCommandAvailable("Pausar", runningPors), true);
  assert.equal(isPaletteCommandAvailable("Continuar", runningPors), false);

  const pausedPors = { ...runningPors, paused: true };
  for (const command of ["Continuar", "Próxima instrução", "Entrar na função", "Sair da função"]) {
    assert.equal(isPaletteCommandAvailable(command, pausedPors), true, command);
  }

  const runningRust = { ...pausedPors, language: "rust" };
  for (const command of ["Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução", "Entrar na função", "Sair da função"]) {
    assert.equal(isPaletteCommandAvailable(command, runningRust), false, command);
  }
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
  const commandShell = readFileSync(new URL("../src/commands/ide-command-shell.ts", import.meta.url), "utf8");
  const shortcuts = readFileSync(new URL("../src/commands/ide-shortcuts.ts", import.meta.url), "utf8");
  assert.doesNotMatch(main, /createIdeShortcuts\(/);
  assert.match(commandShell, /createIdeShortcuts\(/);
  assert.match(commandShell, /mountKeyboardShortcutDispatch\(/);
  assert.match(commandShell, /renderShortcutList\(/);
  assert.match(shortcuts, /label: "Ctrl\+Shift\+P", command: "Paleta de comandos"/i);
});

test("palette hides tab commands without a tab and Rust-unsupported commands in Rust files", () => {
  const idle = {
    language: "portugol-rs", cargoProject: false, cargoAvailable: false, canRunAsRust: false,
    runActive: false, paused: false, porsRunPending: false,
  };
  assert.equal(isPaletteCommandAvailable("Paleta de comandos", idle), false);
  assert.equal(isPaletteCommandAvailable("Salvar", idle), true);
  assert.equal(isPaletteCommandAvailable("Salvar", { ...idle, language: null }), false);
  assert.equal(isPaletteCommandAvailable("Tela inicial", { ...idle, language: null }), true);
  assert.equal(isPaletteCommandAvailable("Renomear símbolo", { ...idle, language: "rust" }), false);
  assert.equal(isPaletteCommandAvailable("Salvar", { ...idle, language: "rust" }), true);
});

test("Executar como Rust needs rustc for the active Portugol-RS file and no run in progress", () => {
  const ready = {
    language: "portugol-rs", cargoProject: false, cargoAvailable: false, canRunAsRust: true,
    runActive: false, paused: false, porsRunPending: false,
  };
  assert.equal(isPaletteCommandAvailable("Executar como Rust", ready), true);
  assert.equal(isPaletteCommandAvailable("Executar como Rust", { ...ready, canRunAsRust: false }), false);
  assert.equal(isPaletteCommandAvailable("Executar como Rust", { ...ready, runActive: true }), false);
  assert.equal(isPaletteCommandAvailable("Executar como Rust", { ...ready, language: null }), false);
});

test("markup tabs hide run, debug and PORS symbol commands but keep formatting", () => {
  const context = { language: "markup", cargoProject: false, cargoAvailable: false, canRunAsRust: false, runActive: false, paused: false, porsRunPending: false };
  for (const command of ["Executar", "Depurar", "Renomear símbolo", "Alternar prévia Rust", "Parâmetros de execução"]) {
    assert.equal(isPaletteCommandAvailable(command, context), false, command);
  }
  assert.equal(isPaletteCommandAvailable("Formatar código-fonte", context), true);
  assert.equal(isPaletteCommandAvailable("Executar como Rust", context), false);
});
