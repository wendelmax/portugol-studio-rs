import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";

const { createLatestOnlyRustSourceChecker } = await import("../src/rust-source-check.ts");
const main = ideSource();
const rustLinter = readFileSync(new URL("../src/editor/rust-linter.ts", import.meta.url), "utf8");
const editorSetup = readFileSync(new URL("../src/editor/setup.ts", import.meta.url), "utf8");
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test("serializes rustc checks and keeps only the newest queued document", async () => {
  const calls = [];
  let releaseFirst;
  const firstGate = new Promise((resolve) => { releaseFirst = resolve; });
  let active = 0;
  let peak = 0;
  const checker = createLatestOnlyRustSourceChecker(async (source) => {
    calls.push(source);
    active += 1;
    peak = Math.max(peak, active);
    if (source === "first") await firstGate;
    active -= 1;
    return [{ message: source }];
  });

  const first = checker("first");
  const superseded = checker("second");
  const latest = checker("third");
  assert.equal((await superseded).diagnostics.length, 0);
  releaseFirst();

  assert.deepEqual((await first).diagnostics, [{ message: "first" }]);
  assert.deepEqual((await latest).diagnostics, [{ message: "third" }]);
  assert.deepEqual(calls, ["first", "third"]);
  assert.equal(peak, 1);
});

test("a failed check does not block the next queued source", async () => {
  let calls = 0;
  const checker = createLatestOnlyRustSourceChecker(async (source) => {
    calls += 1;
    if (source === "bad") throw new Error("rustc failed");
    return [];
  });

  const [failed, next] = await Promise.all([checker("bad"), checker("good")]);

  assert.equal(failed.error, "rustc failed");
  assert.deepEqual(next.diagnostics, []);
  assert.equal(calls, 2);
});

test("Rust editor lint routes through the bounded Tauri source-check command", () => {
  assert.match(editorSetup, /const rustSourceLinter = linter\(createRustLintSourceCheck\(/);
  assert.match(editorSetup, /createLatestOnlyRustSourceChecker\(dependencies\.checkRustSource\)/);
  assert.match(main, /checkRustSource: api\.checkRustSource/);
  assert.match(rustLinter, /editor\.state\.doc !== doc/);
  assert.match(main, /rustExtensions: \(\) => \[rustSourceLinter\]/);
  assert.equal((main.match(/portugolTools\.reconfigure\(tab\.language === "rust" \? \[rustSourceLinter\]/g) ?? []).length, 2);
  assert.match(api, /invoke<RustSourceCompileDiagnostic\[]>\("check_rust_source", \{ source \}\)/);
  assert.match(tauri, /async fn check_rust_source\(/);
});
