import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createIdeShortcuts } from "../src/commands/ide-shortcuts.ts";
import { ideSource } from "./support/ide-source.mjs";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const tauri = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test("home and command palette offer a new desktop project", () => {
  assert.match(html, /id="home-new-project"[^>]*>.*Novo Projeto<\/button>/);
  const calls = [];
  const shortcut = createIdeShortcuts(new Proxy({}, { get: (_, name) => () => calls.push(name) }))
    .find((item) => item.command === "Novo projeto desktop");
  assert.ok(shortcut);
  shortcut.action();
  assert.deepEqual(calls, ["newDesktopProject"]);
});

test("the dialog asks for the name before the system folder picker", () => {
  assert.match(html, /<dialog id="desktop-project-dialog"[\s\S]*id="desktop-project-name"[\s\S]*Escolher local e criar[\s\S]*<\/dialog>/);
  assert.match(html, /id="desktop-project-name" type="text" required maxlength="64"/);
  assert.match(api, /invoke<DesktopProjectCreated \| null>\("workspace_create_desktop_project", \{ name, example \}\)/);
  assert.match(tauri, /workspace::workspace_create_desktop_project,/);
});

test("a created project opens as workspace with its first screen", () => {
  const submit = ideSource().match(/async function submitDesktopProjectDialog[\s\S]*?\n  }\n/)?.[0] ?? "";
  assert.match(submit, /api\.createDesktopProject\(name, desktopProjectExample\)/);
  assert.match(submit, /if \(request !== workspaceRequest \|\| !created\) return;/);
  assert.match(submit, /workspaceState = created\.listing;/);
  assert.match(submit, /selectedWorkspacePath = created\.designPath;/);
  assert.match(submit, /await openWorkspaceFile\(/);
  assert.match(submit, /showDesktopProjectError\(/);
});

test("projetos de exemplo abrem o mesmo diálogo, com o nome já preenchido", () => {
  const source = ideSource();
  assert.match(source, /function openDesktopProjectDialog\(example: \{ id: string; name: string \} \| null = null\)/);
  assert.match(source, /desktopProjectExample = example\?\.id \?\? null;/);
  assert.match(source, /\(id, name\) => workspaceExplorer\.newDesktopProject\(\{ id, name \}\)/);
  assert.match(tauri, /desktop_example_projects,/);
});
