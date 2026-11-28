import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { deriveLanguageControls } from "../src/state/language-capabilities.ts";

function fixture(overrides = {}) {
  const tab = {
    language: "portugol-rs",
    symbols: [],
    ...overrides.tab,
  };
  const state = {
    rustcAvailable: true,
    cargoAvailable: true,
    rustfmtAvailable: true,
    runId: null,
    paused: null,
    busy: false,
    defaultTitles: {},
    libraryNames: ["Matematica", "Texto"],
    ...overrides.state,
  };
  return deriveLanguageControls(tab, state);
}

test("Ferro keeps run/debug, enables PORS modules, and disables unsupported arguments", () => {
  const controls = fixture({
    state: { libraryNames: ["Matematica", "Texto", "Objetos"] },
  });

  assert.equal(controls.buttons["btn-run"].disabled, false);
  assert.equal(controls.buttons["btn-debug"].disabled, false);
  assert.equal(controls.buttons["btn-run"].title, "Executar na VM do Ferro (Shift+F6)");
  assert.equal(controls.arguments.disabled, true);
  assert.equal(controls.arguments.title, "Parâmetros ainda não são suportados em Ferro.");
  assert.equal(controls.libraries.Texto.disabled, false);
  assert.equal(controls.libraries.Texto.title, "Inserir módulo PORS Texto.");
  assert.equal(controls.libraries.Objetos.disabled, true);
  assert.equal(controls.format.disabled, false);
});

test("Rust without detected Cargo shows a discovery message and locks execution", () => {
  const controls = fixture({
    tab: { language: "rust", workspaceId: "workspace", workspacePath: "src/main.rs", cargoProject: undefined },
    state: { busy: true, rustcAvailable: true },
  });

  assert.equal(controls.buttons["btn-run"].disabled, true);
  assert.equal(controls.buttons["btn-run"].title, "Verificando se este arquivo pertence a um projeto Cargo.");
  assert.equal(controls.buttons["btn-run"].description, "Verificando se este arquivo pertence a um projeto Cargo.");
  assert.equal(controls.buttons["btn-debug"].disabled, true);
  assert.equal(controls.arguments.disabled, false);
  assert.equal(controls.libraries.Matematica.title, "Módulos PORS não se aplicam a Rust.");
});

test("Rust reports missing tools, formatter, and unsupported symbol actions", () => {
  const controls = fixture({
    tab: { language: "rust", cargoProject: false },
    state: { rustcAvailable: false, rustfmtAvailable: false },
  });

  assert.equal(controls.buttons["btn-run"].title, "Instale a toolchain Rust para executar arquivos .rs.");
  assert.equal(controls.buttons["btn-debug"].title, "Instale a toolchain Rust para executar arquivos .rs.");
  assert.equal(controls.format.disabled, true);
  assert.equal(controls.format.title, "Instale o componente rustfmt com `rustup component add rustfmt`.");
  assert.deepEqual(controls.symbolActions, {
    disabled: true,
    description: "Navegação e renomeação de símbolos Rust ainda não estão implementadas.",
  });
});

test("execution and pause state gate run buttons while preserving pause actions", () => {
  const active = fixture({ state: { busy: true, runId: 7 } });
  assert.equal(active.buttons["btn-run"].disabled, true);
  assert.equal(active.buttons["btn-debug"].disabled, true);
  assert.equal(active.buttons["btn-debug-pause"].disabled, false);
  assert.equal(active.buttons["btn-debug-continue"].disabled, true);

  const paused = fixture({ state: { busy: true, runId: 7, paused: { position: { line: 2 } } } });
  assert.equal(paused.buttons["btn-run"].disabled, false);
  assert.equal(paused.buttons["btn-debug-pause"].disabled, true);
  assert.equal(paused.buttons["btn-debug-continue"].disabled, false);
});

test("IDE applies derived language controls through the tested state module", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const view = await readFile(new URL("../src/state/language-controls-view.ts", import.meta.url), "utf8");
  assert.match(main, /createLanguageControlsView\(/);
  assert.match(main, /languageControlsView\.sync\(tab\)/);
  assert.match(view, /deriveLanguageControls\(tab/);
  assert.match(view, /controls\.buttons\[id\]/);
});

test("desktop markup only formats: no run, debug, arguments, PORS modules or symbol actions", () => {
  const controls = fixture({ tab: { language: "markup" } });

  assert.equal(controls.buttons["btn-run"].disabled, true);
  assert.equal(controls.buttons["btn-debug"].disabled, true);
  assert.equal(
    controls.buttons["btn-run"].description,
    "Arquivos de marcação descrevem a tela; ela executa pelo projeto desktop.",
  );
  assert.equal(controls.arguments.disabled, true);
  assert.equal(controls.libraries.Texto.disabled, true);
  assert.equal(controls.libraries.Texto.title, "Módulos PORS não se aplicam a arquivos de marcação.");
  assert.equal(controls.symbolActions.disabled, true);
  assert.equal(controls.format.disabled, false);
});
