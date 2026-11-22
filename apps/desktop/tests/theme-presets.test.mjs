import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { test } from "node:test";

const registryUrl = new URL("../src/theme-presets.ts", import.meta.url);

test("presets populares têm IDs estáveis e cores para todos os tokens da IDE", async () => {
  assert.ok(existsSync(registryUrl), "o registro de temas embutidos deve existir");
  const { BUILTIN_THEMES, THEME_COLOR_FIELDS } = await import(registryUrl.href);
  const expected = {
    "preset:github-light": ["GitHub Light", "light"],
    "preset:github-dark": ["GitHub Dark", "dark"],
    "preset:one-dark-pro": ["One Dark Pro", "dark"],
    "preset:dracula": ["Dracula", "dark"],
    "preset:catppuccin-mocha": ["Catppuccin Mocha", "dark"],
    "preset:night-owl": ["Night Owl", "dark"],
    "preset:portugol-studio-next": ["Portugol Studio Next", "dark"],
  };

  assert.deepEqual(Object.keys(BUILTIN_THEMES), Object.keys(expected));
  for (const [id, [name, base]] of Object.entries(expected)) {
    const theme = BUILTIN_THEMES[id];
    assert.equal(theme.name, name, `${id} deve manter o nome apresentado`);
    assert.equal(theme.base, base, `${id} deve selecionar a base correta`);
    for (const field of THEME_COLOR_FIELDS) {
      assert.match(theme.colors[field] ?? "", /^#[\da-f]{6}$/i, `${id} precisa definir ${field}`);
    }
    assert.ok(contrast(theme.colors["--text"], theme.colors["--bg"]) >= 4.5, `${id} precisa manter contraste do texto`);
    assert.ok(contrast(theme.colors["--text"], theme.colors["--surface"]) >= 4.5, `${id} precisa manter contraste dos painéis`);
    assert.ok(contrast(theme.colors["--muted"], theme.colors["--bg"]) >= 4.5, `${id} precisa manter contraste do texto secundário no fundo`);
    assert.ok(contrast(theme.colors["--muted"], theme.colors["--surface"]) >= 4.5, `${id} precisa manter contraste do texto secundário nos painéis`);
  }
  const next = BUILTIN_THEMES["preset:portugol-studio-next"];
  assert.equal(next.colors["--bg"], "#0c1118");
  assert.equal(next.colors["--surface"], "#111923");
  assert.equal(next.colors["--editor-bg"], "#0d141d");
  assert.equal(next.colors["--accent"], "#34d399");
});

test("seletor mantém os temas originais e oferece cada preset pelo ID estável", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  for (const value of ["system", "light", "dark", ...[
    "preset:github-light", "preset:github-dark", "preset:one-dark-pro", "preset:dracula",
    "preset:catppuccin-mocha", "preset:night-owl",
    "preset:portugol-studio-next",
  ]]) assert.ok(html.includes(`value="${value}"`), `seletor deve conter ${value}`);
});

function contrast(foreground, background) {
  const luminance = (color) => {
    const channels = color.slice(1).match(/../g).map((channel) => parseInt(channel, 16) / 255)
      .map((channel) => channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4);
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2];
  };
  const values = [luminance(foreground), luminance(background)].sort((a, b) => b - a);
  return (values[0] + 0.05) / (values[1] + 0.05);
}
