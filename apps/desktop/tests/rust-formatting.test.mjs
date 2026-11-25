import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";

const main = ideSource();
const formatter = readFileSync(new URL("../src/editor/format.ts", import.meta.url), "utf8");
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
const languageCapabilities = readFileSync(new URL("../src/state/language-capabilities.ts", import.meta.url), "utf8");
const toolchainStatus = readFileSync(new URL("../src/shell/toolchain-status.ts", import.meta.url), "utf8");

test(".rs formatting uses rustfmt and applies only to the captured editor document", () => {
  assert.match(main, /function formatCode\(\)[\s\S]*editorFormatter\.formatCode\(\)/);
  assert.match(formatter, /dependencies\.formatRust\(source\)/);
  assert.match(formatter, /dependencies\.activeTab\(\) !== tab \|\| dependencies\.view\.state\.doc !== doc/);
  assert.match(formatter, /annotations: isolateHistory\.of\("full"\)/);
  assert.match(main, /formatRust: api\.formatRustSource/);
  assert.match(main, /deriveLanguageControls\(tab/);
  assert.match(languageCapabilities, /canFormatSource\(tab\.language, state\.rustfmtAvailable\)/);
  assert.match(main, /checkRustfmt: api\.rustfmtAvailable/);
  assert.match(toolchainStatus, /dependencies\.checkRustfmt\(\)/);
  assert.match(api, /invoke<string>\("format_rust_source", \{ source \}\)/);
  assert.match(tauri, /async fn format_rust_source\(source: String\)/);
});
