import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";

const main = ideSource();
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test(".pors formatting uses the PORS backend command and applies only to the captured editor document", () => {
  const formatter = main.match(/async function formatCode\(\) \{[\s\S]*?\n\}/)?.[0];
  assert.ok(formatter, "the editor format action must remain discoverable");
  assert.match(formatter, /tab\.language === "portugol-rs"/);
  assert.match(formatter, /api\.formatPortugolRs\(source\)/);
  assert.match(formatter, /activeTab\(\) !== tab \|\| view\.state\.doc !== doc/);
  assert.match(formatter, /annotations: isolateHistory\.of\("full"\)/);
  assert.match(api, /invoke<\{ from: number; to: number; insert: string \}\[\] \| null>\("format_portugol_rs", \{ source \}\)/);
  assert.match(tauri, /fn format_portugol_rs\(source: String\) -> Option<Vec<portugol_service::TextEdit>>/);
});
