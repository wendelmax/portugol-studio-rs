import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

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
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const runner = readFileSync(new URL("../../../crates/portugol-service/src/pors_run.rs", import.meta.url), "utf8");

  assert.match(api, /runRustSource = \(id: number, source: string, args: string\[\]/);
  assert.match(api, /invoke<RustSourceRunResult>\("run_rust_source", \{ id, source, args \}\)/);
  assert.match(main, /args\.disabled = pors/);
  assert.match(main, /parseProcessArguments[\s\S]*api\.runRustSource\(id, source, rustArgs\)/);
  assert.match(main, /Parâmetros de execução" && tab\?\.language === "portugol-rs"/);
  assert.match(html, /placeholder='ex\.: 42 "João Silva"'/);
  assert.match(tauri, /args: Vec<String>[\s\S]*run_rust_source_with_args_and_cancel/);
  assert.match(runner, /\.args\(args\)/);
  assert.doesNotMatch(runner, /Command::new\(args|sh -c.*args/);
});
