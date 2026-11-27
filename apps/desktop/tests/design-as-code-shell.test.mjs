import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { ideSource } from "./support/ide-source.mjs";
import { ideStyles } from "./support/ide-styles.mjs";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const css = ideStyles();
const main = ideSource();
const activityBar = readFileSync(new URL("../src/shell/activity-bar.ts", import.meta.url), "utf8");

test("shell tem marca, contexto de projeto/modo, configurações dedicadas e tabs dentro do workspace", () => {
  assert.match(html, /class="app-brand"/);
  assert.match(html, /<title>Ferro Studio<\/title>/);
  assert.match(html, /class="app-brand-mark"[^>]*>Fe<\/span>Ferro Studio/);
  assert.match(html, /id="titlebar-workspace"[^>]*data-testid="workspace-selector"/);
  assert.match(html, /id="titlebar-learning-mode"[^>]*data-testid="learning-mode-switcher"/);
  assert.match(html, /<dialog id="settings-dialog" data-testid="settings-view"/);
  assert.match(html, /id="settings-reset-layout"[^>]*data-testid="reset-layout"/);
  assert.match(html, /<aside[^>]*class="activity-rail"[^>]*aria-label="Atividades"/);
  for (const activity of ["learn", "examples", "debug", "settings"]) {
    assert.match(html, new RegExp(`data-activity="${activity}"`));
  }
  assert.doesNotMatch(html, /data-activity="project"/);
  assert.match(html, /data-activity="debug"[^>]*aria-label="Painel de depuração"/);
  assert.match(html, /<nav class="code-toolbar" aria-label="Comandos do editor">/);
  assert.match(html, /<nav id="tabs" data-testid="editor-tabs" class="tabs" aria-label="Abas"><\/nav>/);
  assert.match(html, /<div class="workspace">\s*<nav id="tabs"/);
});

test("ações atuais e layout responsivo do shell continuam disponíveis", () => {
  for (const id of ["btn-run", "btn-debug", "btn-stop", "btn-save", "btn-open", "btn-help", "btn-explorer"]) {
    assert.ok(html.includes(`id="${id}"`), `a ação ${id} deve permanecer no DOM`);
  }
  assert.match(css, /\.activity-rail\s*\{[^}]*flex:\s*0 0 56px/s);
  assert.match(css, /\.utilities\s*\{[^}]*width:\s*clamp\(300px,\s*22vw,\s*380px\)/s);
  assert.match(css, /@media\s*\(max-width:\s*1439px\)/);
  assert.doesNotMatch(css, /\.code\.expanded\s+\.code-toolbar\s*\{[^}]*flex-direction:\s*row/s);
  assert.match(main, /setWorkspaceExplorerVisible\(window\.innerWidth\s*>=\s*1440\)/);
  assert.match(main, /showUtilities\(window\.innerWidth\s*>=\s*1440\)/);
});

test("aprender é servido pelo inspector; variáveis e ferramentas existentes continuam acessíveis", () => {
  assert.match(html, /id="learning-inspector-slot"/);
  assert.match(html, /id="variable-inspector"/);
  assert.match(html, /id="utilities-insert"/);
  assert.match(html, /id="outline"/);
  assert.match(html, /Simulação didática — não é execução nem trace de memória Rust\./);
  assert.ok(activityBar.includes('if (button.dataset.activity !== "learn") dependencies.setInspectorView(false);'));
});
