import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { test } from "node:test";

const registryUrl = new URL("../src/theme-presets.ts", import.meta.url);

test("presets populares têm IDs estáveis e cores para todos os tokens da IDE", async () => {
  assert.ok(existsSync(registryUrl), "o registro de temas embutidos deve existir");
  const { BUILTIN_THEMES, THEME_COLOR_FIELDS } = await import(registryUrl.href);
  const expected = {
    "preset:intellij-rust-dark": ["IntelliJ Rust Dark", "dark"],
    "preset:intellij-rust-light": ["IntelliJ Rust Light", "light"],
    "preset:vscode-dark-modern": ["VS Code Dark Modern", "dark"],
    "preset:vscode-light-modern": ["VS Code Light Modern", "light"],
    "preset:github-light": ["GitHub Light", "light"],
    "preset:github-dark": ["GitHub Dark", "dark"],
    "preset:one-dark-pro": ["One Dark Pro", "dark"],
    "preset:dracula": ["Dracula", "dark"],
    "preset:catppuccin-mocha": ["Catppuccin Mocha", "dark"],
    "preset:night-owl": ["Night Owl", "dark"],
    "preset:high-contrast": ["Alto contraste", "dark"],
  };

  assert.deepEqual(Object.keys(BUILTIN_THEMES), Object.keys(expected));
  for (const [id, [name, base]] of Object.entries(expected)) {
    const theme = BUILTIN_THEMES[id];
    assert.equal(theme.name, name, `${id} deve manter o nome apresentado`);
    assert.equal(theme.base, base, `${id} deve selecionar a base correta`);
    for (const field of THEME_COLOR_FIELDS) {
      assert.match(theme.colors[field] ?? "", /^#[\da-f]{6}$/i, `${id} precisa definir ${field}`);
    }
    assert.match(theme.colors["--error-text"] ?? "", /^#[\da-f]{6}$/i, `${id} precisa definir a cor do texto sobre erros`);
    for (const foreground of [
      "--text", "--muted", "--editor-text", "--editor-gutter", "--syntax-keyword", "--syntax-type",
      "--syntax-function", "--syntax-logical", "--syntax-number", "--syntax-string", "--syntax-interpolation", "--syntax-comment",
      "--syntax-operator",
    ]) {
      assert.match(theme.colors[foreground] ?? "", /^#[\da-f]{6}$/i, `${id} precisa definir ${foreground}`);
      const background = foreground.startsWith("--editor-") || foreground.startsWith("--syntax-")
        ? "--editor-bg"
        : null;
      const backgrounds = foreground === "--editor-gutter"
        ? ["--editor-bg", "--editor-line"]
        : background
        ? ["--editor-bg", "--editor-line", "--editor-selection"]
        : foreground === "--text"
          ? ["--bg", "--surface", "--tab-active", "--console-surface", "--examples-surface"]
          : foreground === "--muted"
            ? ["--bg", "--surface", "--console-surface"]
            : ["--bg", "--surface"];
      for (const backdrop of backgrounds) {
        assert.ok(
          contrast(theme.colors[foreground], theme.colors[backdrop]) >= 4.5,
          `${id}: ${foreground} precisa manter contraste AA com ${backdrop}`,
        );
      }
    }
    assert.ok(contrast(theme.colors["--home-title-text"], theme.colors["--frame"]) >= 4.5, `${id}: texto da tela inicial precisa de contraste AA`);
    assert.ok(contrast(theme.colors["--input-echo"], theme.colors["--console-surface"]) >= 4.5, `${id}: entrada do console precisa de contraste AA`);
    assert.ok(contrast(theme.colors["--editor-text"], theme.colors["--editor-selection"]) >= 4.5, `${id}: texto selecionado no editor precisa de contraste AA`);
    for (const foreground of ["--accent", "--error", "--warning", "--ok"]) {
      for (const background of ["--bg", "--surface"]) {
        assert.ok(
          contrast(theme.colors[foreground], theme.colors[background]) >= 4.5,
          `${id}: ${foreground} precisa manter contraste AA com ${background}`,
        );
      }
    }
    assert.ok(contrast(theme.colors["--accent-text"], theme.colors["--accent"]) >= 4.5, `${id}: texto do destaque precisa de contraste AA`);
    assert.ok(contrast(theme.colors["--error-text"], theme.colors["--error"]) >= 4.5, `${id}: texto do erro precisa de contraste AA`);
  }
  const themeSource = readFileSync(new URL("../src/editor/theme.ts", import.meta.url), "utf8");
  assert.match(themeSource, /export const DEFAULT_THEME = "system";/);
});

test("seletor mantém os temas do sistema e oferece cada preset pelo ID estável", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.ok(!html.includes('value="preset:portugol-studio-next"'), "Portugol Studio Next é especificação de UI/UX, não preset de cor");
  for (const value of ["system", "light", "dark", ...[
    "preset:intellij-rust-dark", "preset:intellij-rust-light",
    "preset:vscode-dark-modern", "preset:vscode-light-modern",
    "preset:github-light", "preset:github-dark", "preset:one-dark-pro", "preset:dracula",
    "preset:catppuccin-mocha", "preset:night-owl",
    "preset:high-contrast",
  ]]) assert.ok(html.includes(`value="${value}"`), `seletor deve conter ${value}`);
});

test("paletas originais clara e escura atendem ao contraste de texto e indicadores", () => {
  const css = readFileSync(new URL("../src/styles/tokens.css", import.meta.url), "utf8");
  const lightBlock = css.match(/:root,\s*\.theme-palette\s*\{([^}]+)\}/s)?.[1];
  const darkBlock = css.match(/:root\[data-theme="dark"\],\s*\.theme-palette\[data-theme="dark"\]\s*\{([^}]+)\}/s)?.[1];
  assert.ok(lightBlock, "tokens.css deve definir a paleta original clara");
  assert.ok(darkBlock, "tokens.css deve definir a paleta original escura");
  const palette = (block) => Object.fromEntries(
    [...block.matchAll(/(--[a-z-]+):\s*(#[\da-f]{6})\s*;/gi)].map(([, field, color]) => [field, color]),
  );

  for (const [id, colors] of [["claro", palette(lightBlock)], ["escuro", palette(darkBlock)]]) {
    assert.match(colors["--error-text"] ?? "", /^#[\da-f]{6}$/i, `${id}: tokens.css precisa definir --error-text`);
    for (const foreground of ["--text", "--muted"]) {
      const backgrounds = foreground === "--text"
        ? ["--bg", "--surface", "--tab-active", "--console-surface", "--examples-surface"]
        : ["--bg", "--surface", "--console-surface"];
      for (const background of backgrounds) {
        assert.ok(contrast(colors[foreground], colors[background]) >= 4.5, `${id}: ${foreground}/${background} precisa de 4,5:1`);
      }
    }
    assert.ok(contrast(colors["--home-title-text"], colors["--frame"]) >= 4.5, `${id}: --home-title-text/--frame precisa de 4,5:1`);
    assert.ok(contrast(colors["--input-echo"], colors["--console-surface"]) >= 4.5, `${id}: --input-echo/--console-surface precisa de 4,5:1`);
    assert.ok(contrast(colors["--editor-text"], colors["--editor-selection"]) >= 4.5, `${id}: --editor-text/--editor-selection precisa de 4,5:1`);
    for (const foreground of [
      "--editor-text", "--editor-gutter", "--syntax-keyword", "--syntax-type", "--syntax-function",
      "--syntax-logical", "--syntax-number", "--syntax-string", "--syntax-comment", "--syntax-operator",
    ]) {
      assert.ok(contrast(colors[foreground], colors["--editor-bg"]) >= 4.5, `${id}: ${foreground}/--editor-bg precisa de 4,5:1`);
    }
    for (const foreground of ["--accent", "--error", "--warning", "--ok"]) {
      for (const background of ["--bg", "--surface"]) {
        assert.ok(contrast(colors[foreground], colors[background]) >= 4.5, `${id}: ${foreground}/${background} precisa de 4,5:1`);
      }
    }
    assert.ok(contrast(colors["--accent-text"], colors["--accent"]) >= 4.5, `${id}: --accent-text/--accent precisa de 4,5:1`);
    assert.ok(contrast(colors["--error-text"], colors["--error"]) >= 4.5, `${id}: --error-text/--error precisa de 4,5:1`);
  }
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
