import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { THEME_COLOR_FIELDS } from "../src/theme-presets.ts";

const EXPECTED_MODULES = [
  "styles/tokens.css",
  "styles/base.css",
  "styles/shell.css",
  "styles/editor.css",
  "styles/panels.css",
  "styles/dialogs.css",
  "styles/home.css",
];
const OTHER_STYLES = ["graphics-window.css"];

const RUNTIME_CSS_VARS = new Set([
  "--icon-light",
  "--icon-dark",
  "--editor-font",
  "--console-font",
]);

function readSrc(rel) {
  return readFileSync(new URL(`../src/${rel}`, import.meta.url), "utf8");
}

function stripComments(css) {
  return css.replace(/\/\*[\s\S]*?\*\//g, "");
}

test("style.css importa os módulos semânticos na ordem esperada", () => {
  const entry = stripComments(readSrc("style.css"));
  const imports = [...entry.matchAll(/@import\s+["']\.\/([^"']+)["'];/g)].map((m) => m[1]);
  assert.deepEqual(imports, EXPECTED_MODULES);
});

test("cores literais ficam restritas a styles/tokens.css", () => {
  const literalColorPattern = /#[0-9a-fA-F]{3,8}\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch)\s*\(/g;
  const namedColorPropPattern =
    /(?:^|[;{]\s*)(?:color|background(?:-color)?|border(?:-[a-z]+)?-color|outline-color|fill|stroke)\s*:\s*(?:white|black|red|green|blue|yellow|orange|gray|grey|silver|purple|pink|cyan|magenta)\b/gi;

  for (const mod of [...EXPECTED_MODULES.slice(1), ...OTHER_STYLES]) {
    const css = stripComments(readSrc(mod));
    const matches = css.match(literalColorPattern) ?? [];
    assert.deepEqual(matches, [], `${mod} não deve conter cores literais: ${matches.join(", ")}`);
    const namedMatches = css.match(namedColorPropPattern) ?? [];
    assert.deepEqual(
      namedMatches,
      [],
      `${mod} não deve usar cores nomeadas literais: ${namedMatches.join(", ")}`,
    );
  }
});

test("janela gráfica importa os tokens globais de cor", () => {
  assert.match(readSrc("graphics-window.css"), /@import\s+["']\.\/styles\/tokens\.css["'];/);
});

test("todos os tokens CSS usados pelos módulos existem em styles/tokens.css", () => {
  const tokensCss = stripComments(readSrc("styles/tokens.css"));
  const definedTokens = new Set([...tokensCss.matchAll(/(--[a-z0-9-]+)\s*:/gi)].map((m) => m[1]));

  for (const field of THEME_COLOR_FIELDS) {
    assert.ok(definedTokens.has(field), `tokens.css deve definir ${field}`);
    const hexDefs = [...tokensCss.matchAll(new RegExp(`${field}:\\s*(#[0-9a-f]{6});`, "gi"))];
    assert.ok(
      hexDefs.length >= 3,
      `${field} deve ter valor #rrggbb no tema claro, media dark e [data-theme="dark"]`,
    );
  }

  for (const mod of [...EXPECTED_MODULES.slice(1), ...OTHER_STYLES]) {
    const css = stripComments(readSrc(mod));
    for (const [, name] of css.matchAll(/var\(\s*(--[a-z0-9-]+)/gi)) {
      if (RUNTIME_CSS_VARS.has(name)) continue;
      assert.ok(definedTokens.has(name), `${mod} referencia token indefinido ${name}`);
    }
  }
});
