import test from "node:test";
import assert from "node:assert/strict";
import { LANGUAGE_BUTTON_IDS } from "../src/state/language-capabilities.ts";
import { createLanguageControlsView } from "../src/state/language-controls-view.ts";

function fakeButton(title = "") {
  return {
    title,
    disabled: false,
    dataset: {},
    attributes: {},
    setAttribute(name, value) { this.attributes[name] = value; },
  };
}

function fixture() {
  const buttons = new Map(LANGUAGE_BUTTON_IDS.map((id) => [id, fakeButton(`Padrão ${id}`)]));
  buttons.set("editor-definition", fakeButton());
  buttons.set("editor-rename", fakeButton());
  const args = fakeButton();
  const format = fakeButton();
  const libraries = new Map([["Matematica", fakeButton()]]);
  const state = {
    rustcAvailable: true,
    cargoAvailable: true,
    rustfmtAvailable: true,
    runId: null,
    paused: null,
    busy: false,
  };
  const view = createLanguageControlsView({
    button: (id) => buttons.get(id),
    argumentsInput: () => args,
    formatButton: () => format,
    libraryButtons: () => libraries,
    state: () => state,
  });
  return { args, buttons, format, libraries, state, view };
}

test("applies derived language controls and remembers button default titles", () => {
  const f = fixture();
  const tab = { language: "portugol-rs", symbols: [] };

  f.view.sync(tab);

  assert.equal(f.buttons.get("btn-run").disabled, false);
  assert.equal(f.buttons.get("btn-run").title, "Executar na VM do Portugol-RS (Shift+F6)");
  assert.equal(f.buttons.get("btn-run").dataset.defaultTitle, "Padrão btn-run");
  assert.equal(f.buttons.get("btn-run").attributes["aria-description"], "");
  assert.equal(f.args.disabled, true);
  assert.equal(f.format.disabled, false);
  assert.equal(f.libraries.get("Matematica").disabled, false);
  assert.equal(f.buttons.get("editor-rename").disabled, false);
});

test("refreshes controls when the active tab becomes Rust with missing tools", () => {
  const f = fixture();
  const run = f.buttons.get("btn-run");
  f.view.sync({ language: "portugol-rs", symbols: [] });
  f.state.rustcAvailable = false;
  f.state.rustfmtAvailable = false;

  f.view.sync({ language: "rust", cargoProject: false, symbols: [] });

  assert.equal(run.disabled, true);
  assert.equal(run.title, "Instale a toolchain Rust para executar arquivos .rs.");
  assert.equal(run.dataset.defaultTitle, "Padrão btn-run");
  assert.equal(f.format.disabled, true);
  assert.equal(f.libraries.get("Matematica").disabled, true);
  assert.equal(f.buttons.get("editor-definition").disabled, true);
  assert.equal(f.buttons.get("editor-rename").attributes["aria-description"], "Navegação e renomeação de símbolos Rust ainda não estão implementadas.");
});
