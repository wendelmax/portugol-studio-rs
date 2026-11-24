import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");

test("movimento reduzido desativa movimento não essencial e rolagem suave", () => {
  const css = read("../src/styles/base.css");
  assert.match(css, /@media\s*\(prefers-reduced-motion:\s*reduce\)/);
  assert.match(css, /animation-duration:\s*0\.01ms\s*!important/);
  assert.match(css, /transition-duration:\s*0\.01ms\s*!important/);
  assert.match(css, /scroll-behavior:\s*auto\s*!important/);
});

test("forced colors usa as cores do sistema e mantém um foco visível", () => {
  const tokens = read("../src/styles/tokens.css");
  const base = read("../src/styles/base.css");
  assert.match(tokens, /@media\s*\(forced-colors:\s*active\)/);
  assert.match(tokens, /--bg:\s*Canvas\s*!important/);
  assert.match(tokens, /--text:\s*CanvasText\s*!important/);
  assert.match(tokens, /--accent:\s*Highlight\s*!important/);
  assert.match(base, /:focus-visible\s*\{[^}]*outline:/s);
});

test("controles centrais da IDE têm data-testid estáveis", () => {
  const html = read("../index.html");
  const ids = {
    "btn-run": "run",
    "btn-debug": "debug",
    "btn-stop": "stop",
    "btn-save": "save",
    "btn-open": "open",
    "workspace-explorer": "workspace-explorer",
    tabs: "editor-tabs",
    "bottom-panel": "output-panel",
    editor: "code-editor",
    "tab-console": "tab-console",
    "tab-tests": "tab-tests",
    "tab-messages": "tab-messages",
    "tab-debug": "tab-debug",
    console: "console-panel",
    tests: "tests-panel",
    messages: "messages-panel",
    debug: "debug-panel",
  };

  for (const [id, testId] of Object.entries(ids)) {
    const tag = html.match(new RegExp(`<[^>]*\\bid="${id}"[^>]*>`))?.[0] ?? "";
    assert.match(tag, new RegExp(`\\bdata-testid="${testId}"`), `#${id} precisa de data-testid="${testId}"`);
  }
  assert.match(html, /<div class="panel-tabs" data-testid="output-tabs" role="tablist"/);
});

test("o campo editável do CodeMirror tem um nome acessível", () => {
  const source = read("../src/editor/extensions.ts");
  assert.match(source, /EditorView\.contentAttributes\.of\(\{\s*"aria-label": "Editor de código"/);
});

test("estado de execução e diagnósticos continua disponível como texto acessível", () => {
  const html = read("../index.html");
  const status = read("../src/shell/status.ts");
  const diagnostics = read("../src/diagnostics.ts");
  assert.match(html, /<span id="status" role="status" aria-live="polite">Pronto<\/span>/);
  assert.match(html, /<p id="no-diagnostics">Nenhum problema encontrado\.<\/p>/);
  assert.match(status, /status\.textContent = text;/);
  assert.match(diagnostics, /button\.setAttribute\("aria-label", `\$\{kind\(diagnostic\)\}/);
});
