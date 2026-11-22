import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const css = readFileSync(new URL("../src/style.css", import.meta.url), "utf8");
const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");

test("shell tem marca, navegação de atividade e tabs dentro do workspace de edição", () => {
  assert.match(html, /class="app-brand"/);
  assert.ok(html.includes("app-brand-mark") && html.includes("Portugol Studio"));
  assert.match(html, /<aside[^>]*class="activity-rail"[^>]*aria-label="Atividades"/);
  for (const activity of ["project", "learn", "examples", "debug", "settings"]) {
    assert.match(html, new RegExp(`data-activity="${activity}"`));
  }
  assert.match(html, /<nav id="tabs" class="tabs" aria-label="Abas"><\/nav>/);
  assert.match(html, /<div class="workspace">\s*<nav id="tabs"/);
});

test("ações atuais e layout responsivo do shell continuam disponíveis", () => {
  for (const id of ["btn-run", "btn-debug", "btn-stop", "btn-save", "btn-open", "btn-help", "btn-explorer"]) {
    assert.ok(html.includes(`id="${id}"`), `a ação ${id} deve permanecer no DOM`);
  }
  assert.match(css, /\.activity-rail\s*\{[^}]*flex:\s*0 0 56px/s);
  assert.match(css, /\.utilities\s*\{[^}]*width:\s*clamp\(300px,\s*22vw,\s*380px\)/s);
  assert.match(css, /@media\s*\(max-width:\s*1439px\)/);
  assert.match(main, /setWorkspaceExplorerVisible\(window\.innerWidth\s*>=\s*1440\)/);
  assert.match(main, /showUtilities\(window\.innerWidth\s*>=\s*1440\)/);
});

test("aprender é servido pelo inspector; variáveis e ferramentas existentes continuam acessíveis", () => {
  assert.match(html, /id="learning-inspector-slot"/);
  assert.match(html, /id="variable-inspector"/);
  assert.match(html, /id="utilities-insert"/);
  assert.match(html, /id="outline"/);
  assert.match(html, /Simulação didática — não é execução nem trace de memória Rust\./);
});
