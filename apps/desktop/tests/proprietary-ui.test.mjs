import assert from "node:assert/strict";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { test } from "node:test";
import { ICON_NAMES, resolveIconName } from "../src/icons.ts";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const homeCss = readFileSync(new URL("../src/styles/home.css", import.meta.url), "utf8");
const tipsJson = JSON.parse(readFileSync(new URL("../src/assets/tips/index.json", import.meta.url), "utf8"));

test("interface proprietária não mantém ícones PNG nem capturas da IDE Java antiga", () => {
  assert.equal(existsSync(new URL("../src/ui-assets", import.meta.url)), false);

  const tipFiles = readdirSync(new URL("../src/assets/tips", import.meta.url));
  assert.deepEqual(tipFiles, ["index.json"]);
});

test("todos os data-icon do HTML resolvem para ícones vetoriais SVG próprios", () => {
  const icons = [...html.matchAll(/data-icon="([^"]+)"/g)].map((match) => match[1]);
  assert.ok(icons.length >= 20, "esperava ícones declarados no shell");
  assert.ok(ICON_NAMES.length >= 20);
  for (const name of icons) {
    assert.ok(!name.includes("/"), `ícone no HTML deve usar nome semântico proprietário: ${name}`);
    assert.ok(resolveIconName(name), `ícone não registrado em icons.ts: ${name}`);
  }
});

test("dicas da IDE são proprietárias de Portugol-RS/Rust sem imagens legadas", () => {
  assert.ok(Array.isArray(tipsJson) && tipsJson.length >= 6);
  for (const tip of tipsJson) {
    assert.ok(tip.id && tip.badge && tip.title && tip.shortcut && tip.visual && tip.description);
    assert.equal("image" in tip, false);
  }
  assert.match(html, /id="tips-visual"/);
  assert.doesNotMatch(html, /id="tips-image"/);
});

test("tela inicial e prévia de temas usam identidade própria de Portugol-RS", () => {
  assert.doesNotMatch(homeCss, /border:\s*9px\s+solid\s+var\(--home-border\)/);
  assert.match(html, /class="home-tagline"/);
  assert.match(html, /class="titlebar-actions"[\s\S]*?id="btn-run"/);
  const preview = html.match(/id="theme-preview"[\s\S]*?<\/pre>/)?.[0] ?? "";
  assert.match(preview, /\bseja\b/);
  assert.doesNotMatch(preview, /\bprograma\b/);
});
