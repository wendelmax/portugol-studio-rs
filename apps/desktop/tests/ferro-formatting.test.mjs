import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";

const main = ideSource();
const shell = readFileSync(new URL("../src/editor/actions-shell.ts", import.meta.url), "utf8");
const formatter = readFileSync(new URL("../src/editor/format.ts", import.meta.url), "utf8");
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test(".fer formatting uses the FERRO backend command and applies only to the captured editor document", () => {
  assert.match(shell, /formatFerroRust: api\.formatFerroRust/);
  assert.match(formatter, /dependencies\.formatFerroRust\(source\)/);
  assert.match(formatter, /dependencies\.activeTab\(\) !== tab \|\| dependencies\.view\.state\.doc !== doc/);
  assert.match(formatter, /annotations: isolateHistory\.of\("full"\)/);
  assert.match(api, /invoke<\{ from: number; to: number; insert: string \}\[\] \| null>\("format_ferro_rs", \{ source \}\)/);
  assert.match(tauri, /fn format_ferro_rs\(source: String\) -> Option<Vec<ferro_service::TextEdit>>/);
});
