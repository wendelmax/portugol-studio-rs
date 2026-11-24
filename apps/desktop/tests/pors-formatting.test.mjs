import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";

const main = ideSource();
const formatter = readFileSync(new URL("../src/editor/format.ts", import.meta.url), "utf8");
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test(".pors formatting uses the PORS backend command and applies only to the captured editor document", () => {
  assert.match(main, /function formatCode\(\)[\s\S]*editorFormatter\.formatCode\(\)/);
  assert.match(formatter, /tab\.language === "portugol-rs"/);
  assert.match(formatter, /dependencies\.formatPortugolRs\(source\)/);
  assert.match(formatter, /dependencies\.activeTab\(\) !== tab \|\| dependencies\.view\.state\.doc !== doc/);
  assert.match(formatter, /annotations: isolateHistory\.of\("full"\)/);
  assert.match(main, /formatPortugolRs: api\.formatPortugolRs/);
  assert.match(api, /invoke<\{ from: number; to: number; insert: string \}\[\] \| null>\("format_portugol_rs", \{ source \}\)/);
  assert.match(tauri, /fn format_portugol_rs\(source: String\) -> Option<Vec<portugol_service::TextEdit>>/);
});
