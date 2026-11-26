import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { ideSource } from "./support/ide-source.mjs";

const moduleUrl = new URL("../src/process-arguments.ts", import.meta.url);

test("separa argumentos por espaços preservando aspas, espaços e acentos", async () => {
  const { parseProcessArguments } = await import(moduleUrl.href);
  assert.deepEqual(parseProcessArguments('olá "João Silva" 19'), ["olá", "João Silva", "19"]);
});

test("preserva argumento vazio e interpreta aspas e barras escapadas", async () => {
  const { parseProcessArguments } = await import(moduleUrl.href);
  assert.deepEqual(parseProcessArguments('"" "com \\"aspas\\"" "c:\\\\aula"'), ["", 'com "aspas"', "c:\\aula"]);
});

test("rejeita aspas não fechadas com mensagem legível", async () => {
  const { parseProcessArguments } = await import(moduleUrl.href);
  assert.throws(() => parseProcessArguments('"João Silva'), /aspas duplas não foram fechadas/i);
});

test("encaminha argv como lista tipada do editor ao processo Rust, sem shell", () => {
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();
  const rustExecution = readFileSync(new URL("../src/run/rust-execution.ts", import.meta.url), "utf8");
  const languageCapabilities = readFileSync(new URL("../src/state/language-capabilities.ts", import.meta.url), "utf8");
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const runner = readFileSync(new URL("../../../crates/portugol-service/src/pors_run.rs", import.meta.url), "utf8");

  assert.match(api, /runRustSource = \(id: number, source: string, args: string\[\]/);
  assert.match(api, /invoke<RustSourceRunResult>\("run_rust_source", \{ id, source, args \}\)/);
  assert.match(main, /args\.disabled = controls\.arguments\.disabled/);
  assert.match(languageCapabilities, /disabled: pors/);
  assert.match(main, /parseArguments: parseProcessArguments/);
  assert.match(rustExecution, /dependencies\.parseArguments\(dependencies\.argumentsText\(\)\)[\s\S]*dependencies\.runRustSource\(id, source, rustArgs\)/);
  assert.match(html, /placeholder='ex\.: 42 "João Silva"'/);
  assert.match(tauri, /args: Vec<String>[\s\S]*run_rust_source_with_args_and_cancel/);
  assert.match(runner, /\.args\(args\)/);
  assert.doesNotMatch(runner, /Command::new\(args|sh -c.*args/);
});

test("execution parameters are hidden for Portugol-RS, which takes no argv", async () => {
  const { isPaletteCommandAvailable } = await import("../src/command-palette.ts");
  const context = {
    language: "portugol-rs", cargoProject: false, cargoAvailable: false, canRunAsRust: false,
    runActive: false, paused: false, porsRunPending: false,
  };
  assert.equal(isPaletteCommandAvailable("Parâmetros de execução", context), false);
  assert.equal(isPaletteCommandAvailable("Parâmetros de execução", { ...context, language: "rust" }), true);
  assert.equal(isPaletteCommandAvailable("Parâmetros de execução", { ...context, language: null }), false);
});
