import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { mappedPreviewSegments, sourceDiagnostic, sourceMapLinks } from "../src/source-map-ui.ts";
import { ideSource } from "./support/ide-source.mjs";
import { ideStyles } from "./support/ide-styles.mjs";

const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const main = ideSource();
const fileIO = readFileSync(new URL("../src/files/io.ts", import.meta.url), "utf8");
const ui = readFileSync(new URL("../src/source-map-ui.ts", import.meta.url), "utf8");
const css = ideStyles();
const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const generatedEditor = readFileSync(new URL("../src/generated-rust-editor.ts", import.meta.url), "utf8");
const editorThemeController = readFileSync(new URL("../src/editor/theme-controller.ts", import.meta.url), "utf8");

test("Rust export API exposes generated and source editor ranges", () => {
  assert.match(api, /interface SourceMapInfo[\s\S]*generatedStart: Position[\s\S]*generatedEnd: Position[\s\S]*sourceStart: Position \| null[\s\S]*sourceEnd: Position \| null[\s\S]*quality: "exact" \| "approximate" \| "generated_only"/);
  assert.match(api, /interface RustExport[\s\S]*sourceMap: SourceMapInfo\[\]/);
});

test("PORS run API distinguishes execution, compiler diagnostics, limits and cancellation", () => {
  assert.match(api, /status: "executed"[\s\S]*status: "compile_error"[\s\S]*diagnostics: PortugolRsCompileDiagnostic\[\][\s\S]*status: "compile_limit"[\s\S]*status: "runtime_limit"[\s\S]*reason: "time" \| "output" \| "string_memory"[\s\S]*status: "cancelled"/);
  assert.match(api, /runPortugolRs = \(id: number, source: string\)[\s\S]*\{ id, source \}/);
});

test("PORS execution uses the shared registered cancellation token", () => {
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const porsExecution = readFileSync(new URL("../src/run/portugol-rs-execution.ts", import.meta.url), "utf8");
  assert.match(tauri, /async fn run_portugol_rs\([\s\S]*id: u32,[\s\S]*run_portugol_rs_with_io[\s\S]*remove\(&id\)/);
  assert.match(main, /registerRun: api\.registerPortugolRsRun/);
  assert.match(porsExecution, /registeredId = await dependencies\.registerRun\(\)[\s\S]*dependencies\.setRunId\(registeredId\)[\s\S]*dependencies\.runGeneratedRust\(registeredId, source\)/);
  assert.match(porsExecution, /result\.status === "cancelled"[\s\S]*Execução Portugol-RS cancelada durante/);
});

test("standalone Rust API covers compile/runtime limits and Tauri registers the command", () => {
  assert.match(api, /interface RustSourceCompileDiagnostic[\s\S]*start: Position[\s\S]*end: Position/);
  assert.match(api, /status: "compile_limit"[\s\S]*status: "runtime_limit"[\s\S]*runRustSource[\s\S]*"run_rust_source"/);
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  assert.match(tauri, /async fn run_rust_source[\s\S]*portugol_service::run_rust_source/);
  assert.match(tauri, /generate_handler!\[[\s\S]*run_rust_source/);
  assert.match(main, /hasRustExecutionConsent[\s\S]*Executar código Rust sem sandbox[\s\S]*api\.runRustSource/);
});

test("standalone Rust execution registers an interruptible run before compiling", () => {
  const rustExecution = readFileSync(new URL("../src/run/rust-execution.ts", import.meta.url), "utf8");
  const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  assert.match(api, /registerRustSourceRun[\s\S]*register_rust_source_run/);
  assert.match(api, /runRustSource = \(id: number, source: string, args: string\[\] = \[\]\)[\s\S]*\{ id, source, args \}/);
  assert.match(api, /status: "cancelled"; phase: "compilation" \| "runtime"/);
  assert.match(tauri, /fn register_rust_source_run[\s\S]*rust_source_cancellations/);
  assert.match(tauri, /async fn run_rust_source[\s\S]*run_rust_source_with_args_and_cancel[\s\S]*remove\(&id\)/);
  assert.match(tauri, /fn request_stop\(&self, id: u32\) -> bool[\s\S]*cancel\.store\(true/);
  assert.match(tauri, /fn stop\([\s\S]*\) -> bool[\s\S]*runs\.request_stop\(id\)/);
  assert.match(tauri, /generate_handler!\[[\s\S]*register_rust_source_run[\s\S]*run_rust_source/);
  assert.match(main, /registerRun: api\.registerRustSourceRun/);
  assert.match(rustExecution, /const id = await dependencies\.registerRun\(\);[\s\S]*dependencies\.setRunId\(id\)[\s\S]*dependencies\.runRustSource\(id, source, rustArgs\)/);
  assert.match(rustExecution, /result\.status === "cancelled"[\s\S]*Execução Rust cancelada durante/);
});

test("mapped diagnostics and preview segments require source mappings and label approximate locations", () => {
  assert.match(ui, /if \(!diagnostic\.sourceStart \|\| !diagnostic\.sourceEnd\) return null/);
  assert.match(ui, /localização aproximada/);
  assert.match(ui, /mappedPreviewSegments/);
});

test("compile diagnostics navigate mapped locations and leave generated-only errors unmapped", () => {
  const sourceStart = { offset: 8, line: 2, column: 3 };
  const sourceEnd = { offset: 13, line: 2, column: 8 };
  const diagnostic = {
    severity: "error", code: "E0308", message: "tipos incompatíveis", rendered: "error E0308",
    generatedStart: { offset: 20, line: 2, column: 1 }, generatedEnd: { offset: 25, line: 2, column: 6 },
    sourceStart, sourceEnd, quality: "approximate",
  };
  assert.equal(sourceDiagnostic(diagnostic).start, sourceStart);
  assert.match(sourceDiagnostic(diagnostic).message, /aproximada/);
  assert.equal(sourceDiagnostic({ ...diagnostic, sourceStart: null, sourceEnd: null, quality: "generated_only" }), null);
});

test("Rust preview mappings use UTF-16 generated offsets and retain explicit source targets", () => {
  const segments = mappedPreviewSegments("fn 😀() {}", [{
    generatedStart: { offset: 3, line: 1, column: 4 }, generatedEnd: { offset: 7, line: 1, column: 8 },
    sourceStart: { offset: 4, line: 2, column: 1 }, sourceEnd: { offset: 8, line: 2, column: 5 }, quality: "exact",
  }]);
  assert.deepEqual(segments, [{ from: 3, to: 7, sourceOffset: 4, sourceLine: 2, generatedLine: 1, quality: "exact" }]);
});

test("overlapping preview mappings prefer the narrowest source segment", () => {
  const position = (offset) => ({ offset, line: 1, column: offset + 1 });
  const segments = mappedPreviewSegments("0123456789", [
    { generatedStart: position(1), generatedEnd: position(9), sourceStart: position(10), sourceEnd: position(20), quality: "approximate" },
    { generatedStart: position(3), generatedEnd: position(6), sourceStart: position(30), sourceEnd: position(33), quality: "exact" },
  ]);
  assert.deepEqual(segments, [{ from: 3, to: 6, sourceOffset: 30, sourceLine: 1, generatedLine: 1, quality: "exact" }]);
});

test("generated-only source maps never create preview navigation even if they contain source spans", () => {
  const position = (offset, line) => ({ offset, line, column: 1 });
  assert.deepEqual(mappedPreviewSegments("generated", [
    { generatedStart: position(0, 1), generatedEnd: position(9, 1), sourceStart: position(0, 1), sourceEnd: position(9, 1), quality: "generated_only" },
  ]), []);
});

test("source mapping gutter derives unique exact and approximate line links", () => {
  const position = (offset, line) => ({ offset, line, column: 1 });
  const links = sourceMapLinks([
    { generatedStart: position(10, 4), generatedEnd: position(15, 4), sourceStart: position(2, 2), sourceEnd: position(5, 2), quality: "approximate" },
    { generatedStart: position(11, 4), generatedEnd: position(14, 4), sourceStart: position(3, 2), sourceEnd: position(4, 2), quality: "exact" },
    { generatedStart: position(20, 8), generatedEnd: position(25, 8), sourceStart: position(8, 3), sourceEnd: position(9, 3), quality: "exact" },
    { generatedStart: position(40, 12), generatedEnd: position(45, 12), sourceStart: position(12, 4), sourceEnd: position(16, 4), quality: "approximate" },
    { generatedStart: position(30, 9), generatedEnd: position(35, 9), sourceStart: null, sourceEnd: null, quality: "generated_only" },
  ]);
  assert.deepEqual(links, [
    { sourceLine: 2, generatedLine: 4, sourceOffset: 3, generatedOffset: 11, quality: "exact" },
    { sourceLine: 3, generatedLine: 8, sourceOffset: 8, generatedOffset: 20, quality: "exact" },
    { sourceLine: 4, generatedLine: 12, sourceOffset: 12, generatedOffset: 40, quality: "approximate" },
  ]);
});

test("bridge mapping gutter exposes accessible two-way navigation between source and Rust", () => {
  assert.match(html, /id="rust-mapping-gutter"[^>]*aria-label="Mapeamento Portugol para Rust"/);
  assert.match(html, /id="rust-mapping-status"[^>]*aria-live="polite"/);
  assert.match(main, /function renderMappingGutter\(/);
  assert.match(generatedEditor, /"data-source-offset": String\(mapping\.sourceOffset\)/);
  assert.match(generatedEditor, /"data-quality": mapping\.quality/);
  assert.match(main, /focusRustMapping\(/);
  assert.match(main, /Linha \$\{sourceLine\} sem correspondência direta/);
  assert.match(main, /Localização aproximada/);
});

test("Rust editor keeps the per-tab artifact marked stale until current translation succeeds", () => {
  assert.match(main, /rustPreview\?: RustPreviewState/);
  assert.match(main, /function renderRustPreview\(tab: Tab\)/);
  assert.match(main, /tab\.rustPreview = beginRustPreview/);
  assert.match(main, /tab\.rustPreview = acceptRustPreview/);
  assert.match(main, /tab\.rustPreview = failRustPreview/);
  assert.match(main, /rustMappingState === "stale"/);
  assert.match(main, /const pending = Boolean\(state\?\.pending\)/);
  assert.match(main, /status\.dataset\.stale = "true"/);
  assert.match(main, /copy\.disabled = save\.disabled = detach\.disabled = !artifact \|\| Boolean\(pending \|\| stale\)/);
  assert.match(css, /#rust-status\[data-stale="true"\]/);
  assert.match(html, /id="rust-output" class="rust-generated-editor"/);
  assert.match(main, /const generatedRustEditor = new EditorView\([\s\S]*createGeneratedRustEditorState/);
  assert.match(main, /generatedRustEditor\.state\.doc\.toString\(\)/);
  assert.match(editorThemeController, /generatedTheme\.reconfigure\(dependencies\.editorTheme\(\)\)/);
  assert.match(main, /generatedTheme: generatedRustTheme/);
  const renderer = main.match(/\n( *)function renderRustPreview\(tab: Tab\) \{([\s\S]*?)\n\1\}/)?.[2] ?? "";
  assert.match(renderer, /artifact && !stale[\s\S]*mappedPreviewSegments\(artifact\.code, artifact\.sourceMap\)/);
  assert.match(renderer, /generatedRustMappings\.of\(mappings\)/);
  assert.match(renderer, /generatedDocument\.toString\(\) === code/);
  assert.doesNotMatch(renderer, /replaceChildren|createTextNode|innerHTML/);
  assert.match(generatedEditor, /EditorState\.readOnly\.of\(true\)/);
  assert.match(generatedEditor, /EditorView\.editable\.of\(false\)/);
  assert.match(generatedEditor, /class: "cm-rust-mapped-segment"/);
  assert.match(css, /\.cm-rust-mapped-segment\[data-quality="approximate"\]/);
  const updateListener = main.match(/EditorView\.updateListener\.of\(\(update\) => \{([\s\S]*?)\n    \}\),/)?.[1] ?? "";
  assert.doesNotMatch(updateListener, /renderRustPreview\(tab\)|rustPreview\.render\(/);
  assert.match(fileIO, /const languageChanged = tab\.language !== previousLanguage/);
  assert.match(main, /if \(languageChanged && tab\.rustPreview\?\.artifact\)/);
  assert.match(main, /if \(learningMode === "bridge" && tab\.language !== "rust"\) rustPreview\.schedule\(\)/);
  const save = fileIO.match(/async function save\(as = false\): Promise<boolean> \{([\s\S]*?)\n  \}/)?.[1] ?? "";
  assert.match(save, /await dependencies\.writeFile\(path, text\)[\s\S]*const previousLanguage = tab\.language[\s\S]*tab\.language = dependencies\.detectSourceLanguage\(path\)/);
  const schedule = main.match(/\n( *)function scheduleRustExport\(\) \{([\s\S]*?)\n\1\}\n\n\1function renderRustPreview/)?.[2] ?? "";
  assert.ok(schedule, "agendamento deve estar separado da renderização do artefato");
  assert.doesNotMatch(schedule, /\$<HTMLElement>\("rust-output"\)\.textContent = ""/);
});

test("generated Rust diff is read-only, accessible, bounded, and hidden when the preview is stale", () => {
  assert.match(html, /id="rust-diff-toggle"[^>]*aria-pressed="false"[^>]*disabled>Mostrar diferenças/);
  assert.match(html, /id="rust-diff-output"[^>]*aria-label="Diferenças entre traduções Rust"[^>]*hidden/);
  assert.match(main, /generatedRustComparison\(state\)/);
  assert.match(main, /generatedOutput\.hidden = showDiff[\s\S]*diffOutput\.hidden = !showDiff/);
  assert.match(main, /if \(showDiff && comparison\) renderGeneratedRustDiff/);
  assert.match(main, /code\.textContent = text/);
  assert.match(main, /rust-diff-toggle"\)\.onclick = \(\) =>[\s\S]*renderRustPreview\(tab\)/);
  assert.match(css, /\.rust-diff-table tr\[data-kind="removed"\]/);
  assert.match(css, /\.rust-diff-table tr\[data-kind="added"\]/);
});

test("PORS run response is discarded when its tab or document revision changes", () => {
  const porsExecution = readFileSync(new URL("../src/run/portugol-rs-execution.ts", import.meta.url), "utf8");
  assert.match(porsExecution, /dependencies\.isCurrent\(tab, document\)/);
  assert.match(porsExecution, /result\.status === "executed"/);
  assert.match(porsExecution, /result\.status === "runtime_limit"/);
  assert.match(porsExecution, /tempo máximo de 3 segundos/);
  assert.match(porsExecution, /limite de saída de 1 MiB/);
  assert.match(porsExecution, /limite de memória para cadeias de 16 MiB/);
  assert.match(porsExecution, /result\.diagnostics\.map\(dependencies\.sourceDiagnostic\)/);
  assert.match(generatedEditor, /mappedSourceOffsetAt\(editor\.state\.field\(generatedRustDecorations\), position\)/);
  assert.match(main, /function focusRustMapping\(link: SourceMapLink\)[\s\S]*generatedRustEditor\.dispatch\([\s\S]*scrollIntoView: true/);
});
