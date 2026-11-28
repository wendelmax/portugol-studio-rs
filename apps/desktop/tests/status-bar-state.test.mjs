import assert from "node:assert/strict";
import { test } from "node:test";
import { readFileSync } from "node:fs";
import { statusBarState } from "../src/shell/status-bar-state.ts";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const component = readFileSync(new URL("../src/shell/status-bar.ts", import.meta.url), "utf8");

function tab(overrides = {}) {
  return {
    language: "portugol-rs",
    diagnostics: [],
    runtimeCompileDiagnostics: [],
    ...overrides,
  };
}

test("does not report clean diagnostics until the active document revision was analyzed", () => {
  const state = statusBarState(
    tab({ diagnosticsAnalysis: { source: "versão anterior", status: "complete" } }),
    "versão atual",
    "bridge",
    true,
    4,
  );

  assert.equal(state.diagnostics.state, "pending");
  assert.equal(state.diagnostics.label, "Analisando…");
});

test("counts current errors and warnings, including runtime compiler diagnostics", () => {
  const state = statusBarState(tab({
    diagnosticsAnalysis: { source: "fonte", status: "complete" },
    diagnostics: [{ severity: "error" }, { severity: "warning" }, { severity: "note" }],
    runtimeCompileDiagnostics: [{ severity: "error" }],
  }), "fonte", "beginner", false, 4);

  assert.deepEqual(state.diagnostics, { state: "ready", errors: 2, warnings: 1, label: "2 erros · 1 aviso" });
});

test("reports no errors only after a successful analysis of the current document", () => {
  const ready = statusBarState(tab({ diagnosticsAnalysis: { source: "fonte", status: "complete" } }), "fonte", "rust", false, 4);
  const unavailable = statusBarState(tab({ diagnosticsAnalysis: { source: "fonte", status: "unavailable" } }), "fonte", "rust", false, 4);
  const partial = statusBarState(tab({
    diagnosticsAnalysis: { source: "fonte", status: "unavailable" },
    runtimeCompileDiagnostics: [{ severity: "error" }],
  }), "fonte", "rust", false, 4);

  assert.equal(ready.diagnostics.label, "Sem erros");
  assert.equal(unavailable.diagnostics.state, "unavailable");
  assert.equal(unavailable.diagnostics.label, "Análise indisponível");
  assert.equal(partial.diagnostics.label, "1 erro · 0 avisos · análise parcial");
});

test("presents file language, learning mode, execution engines, indentation and encoding", () => {
  const state = statusBarState(tab(), "fonte", "bridge", true, 4);

  assert.equal(state.language, "Ferro");
  assert.equal(state.mode, "Ponte para Rust");
  assert.equal(state.engines, "VM e Rust nativo");
  assert.equal(state.indentation, "4 espaços");
  assert.equal(state.encoding, "UTF-8 ao salvar");
});

test("hides file context when no file tab is active and omits unavailable alternate engines", () => {
  assert.equal(statusBarState(undefined, "", "beginner", null, 4), null);
  const state = statusBarState(tab(), "fonte", "beginner", false, 4);
  assert.equal(state.engines, null);
});

test("status bar exposes named context items and opens Messages from diagnostic counts", () => {
  assert.match(html, /id="statusbar-diagnostics"[^>]*aria-label="Contagem de erros e avisos"/);
  assert.match(html, /id="statusbar-context"[^>]*role="group" aria-label="Contexto do arquivo"/);
  assert.match(html, /id="status" role="status" aria-live="polite"/);
  assert.match(component, /diagnostics\.addEventListener\("click", dependencies\.openMessages\)/);
  assert.match(component, /state\.diagnostics\.state === "pending"/);
});
