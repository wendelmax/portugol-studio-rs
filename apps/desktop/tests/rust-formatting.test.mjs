import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test(".rs formatting uses rustfmt and applies only to the captured editor document", () => {
  const formatter = main.match(/async function formatCode\(\) \{[\s\S]*?\n\}/)?.[0];
  assert.ok(formatter, "the editor format action must remain discoverable");
  assert.match(formatter, /api\.formatRustSource\(source\)/);
  assert.match(formatter, /activeTab\(\) !== tab \|\| view\.state\.doc !== doc/);
  assert.match(formatter, /annotations: isolateHistory\.of\("full"\)/);
  assert.match(main, /canFormatSource\(tab\.language, rustfmtAvailable\)/);
  assert.match(main, /api\.rustfmtAvailable\(\)/);
  assert.match(api, /invoke<string>\("format_rust_source", \{ source \}\)/);
  assert.match(tauri, /async fn format_rust_source\(source: String\)/);
});
