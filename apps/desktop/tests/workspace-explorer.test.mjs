import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { collectWorkspaceFiles, flattenWorkspaceEntries, workspaceFileKind, workspaceRenameFileName, workspaceSourceFileName } from "../src/workspace-explorer.ts";
import { ideSource } from "./support/ide-source.mjs";

test("collapsed folders hide descendants and expanded folders preserve tree depth", () => {
  const entries = [
    { kind: "folder", name: "Aulas", path: "Aulas", entries: [
      { kind: "file", name: "inicio.por", path: "Aulas/inicio.por" },
    ] },
    { kind: "file", name: "programa.pors", path: "programa.pors" },
  ];

  assert.deepEqual(flattenWorkspaceEntries(entries), [
    { entry: entries[0], depth: 0, position: 1, setSize: 2 },
    { entry: entries[1], depth: 0, position: 2, setSize: 2 },
  ]);
  assert.deepEqual(flattenWorkspaceEntries(entries, new Set(["Aulas"])), [
    { entry: entries[0], depth: 0, position: 1, setSize: 2 },
    { entry: entries[0].entries[0], depth: 1, position: 1, setSize: 1 },
    { entry: entries[1], depth: 0, position: 2, setSize: 2 },
  ]);
});

test("collects all workspace files recursively in tree order", () => {
  const entries = [
    { kind: "folder", name: "Aulas", path: "Aulas", entries: [
      { kind: "file", name: "inicio.por", path: "Aulas/inicio.por" },
      { kind: "folder", name: "Avançado", path: "Aulas/Avançado", entries: [
        { kind: "file", name: "ownership.rs", path: "Aulas/Avançado/ownership.rs" },
      ] },
    ] },
    { kind: "file", name: "atividade.pex", path: "atividade.pex" },
  ];
  assert.deepEqual(collectWorkspaceFiles(entries), [
    entries[0].entries[0], entries[0].entries[1].entries[0], entries[1],
  ]);
  assert.deepEqual(collectWorkspaceFiles([]), []);
});

test("classifies Portugol, Portugol-RS, Rust, and exercise files case-insensitively", () => {
  assert.equal(workspaceFileKind("aula.POR"), "portugol");
  assert.equal(workspaceFileKind("ponte.PORS"), "portugol-rs");
  assert.equal(workspaceFileKind("main.RS"), "rust");
  assert.equal(workspaceFileKind("listas.PEX"), "exercise");
  assert.equal(workspaceFileKind("leia-me.md"), null);
});

test("validates Rust workspace source filenames", () => {
  assert.equal(workspaceSourceFileName("main", "rs"), "main.rs");
  assert.equal(workspaceSourceFileName("main.RS", "rs"), "main.RS");
  assert.throws(() => workspaceSourceFileName("main.por", "rs"), /extensão/);
});

test("builds new Portugol, Portugol-RS, and Rust filenames from the selected language", () => {
  assert.equal(workspaceSourceFileName("ola", "por"), "ola.por");
  assert.equal(workspaceSourceFileName("ponte.pors", "pors"), "ponte.pors");
  assert.equal(workspaceSourceFileName("main", "rs"), "main.rs");
  assert.equal(workspaceSourceFileName("classico.POR", "por"), "classico.POR");
});

test("rejects empty names, path separators, and extensions that conflict with the language", () => {
  assert.throws(() => workspaceSourceFileName("", "por"), /nome/i);
  assert.throws(() => workspaceSourceFileName("../fora", "por"), /nome/i);
  assert.throws(() => workspaceSourceFileName("subpasta\\programa", "pors"), /nome/i);
  assert.throws(() => workspaceSourceFileName("programa.por", "pors"), /extensão/i);
});

test("renames a workspace file while preserving its original extension", () => {
  assert.equal(workspaceRenameFileName("novo", "antigo.por"), "novo.por");
  assert.equal(workspaceRenameFileName("ponte.pors", "classico.pors"), "ponte.pors");
  assert.equal(workspaceRenameFileName("crates.rs", "main.rs"), "crates.rs");
  assert.equal(workspaceRenameFileName("atividade.pex", "exercicio.pex"), "atividade.pex");
  assert.equal(workspaceRenameFileName(" nome.por ", " nome.por "), " nome.por ");
});

test("rejects unsafe rename names and extension changes", () => {
  assert.throws(() => workspaceRenameFileName("", "antigo.por"), /nome/i);
  assert.throws(() => workspaceRenameFileName("../fora", "antigo.por"), /nome/i);
  assert.throws(() => workspaceRenameFileName("novo.pors", "antigo.por"), /extensão/i);
  assert.throws(() => workspaceRenameFileName("novo", "notas.txt"), /extensão/i);
});

test("home action and explorer tree expose stable accessible controls", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="home-open-folder"/);
  assert.match(html, /id="workspace-explorer"/);
  assert.match(html, /id="workspace-tree"[^>]*role="tree"/);
  assert.match(html, /data-testid="project-tree"/);
});

test("home start page defaults new file to Portugol-RS and groups alternate languages", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const main = ideSource();
  const css = readFileSync(new URL("../src/style.css", import.meta.url), "utf8");

  assert.match(html, /<button id="home-new"[^>]*Portugol-RS \(\.pors\)[^>]*>[^<]*<span[^>]*>[^<]*<\/span>Novo Arquivo<\/button>/);
  assert.match(html, /<details id="home-new-menu"[^>]*data-testid="home-new-menu"/);
  assert.match(html, /<summary[^>]*aria-label="Criar arquivo em outra linguagem"/);
  assert.match(html, /id="home-new-pors"[^>]*data-home-new-language="portugol-rs"[^>]*>Novo Portugol-RS \(\.pors\)<\/button>/);
  assert.match(html, /id="home-new-rust"[^>]*data-home-new-language="rust"[^>]*>Novo Rust \(\.rs\)<\/button>/);
  assert.match(html, /id="home-new-por"[^>]*data-home-new-language="portugol"[^>]*>Novo Portugol clássico \(\.por\)<\/button>/);
  assert.match(main, /\$\("home-new"\)\.onclick = \(\) => newTab\(\);/);
  assert.match(main, /function newTab\(text = PORS_TEMPLATE/);
  assert.match(main, /mountHomeNewFileMenu\(/);
  assert.match(main, /"portugol-rs": \(\) => newTab\(PORS_TEMPLATE, null, undefined, undefined, undefined, "portugol-rs"\)/);
  assert.match(main, /rust: \(\) => newTab\("fn main\(\) \{\\n    \\n\}\\n", null, undefined, undefined, undefined, "rust"\)/);
  assert.match(main, /portugol: \(\) => newTab\(NEW_PROGRAM, null, undefined, undefined, undefined, "portugol"\)/);
  assert.doesNotMatch(main, /const newPorsButton = document\.createElement/);
  assert.doesNotMatch(main, /const newRustButton = document\.createElement/);
  assert.match(css, /\.home-actions > button,\s*\.home-new-file-group > button/);
  assert.match(css, /\.home-new-menu-items\s*\{[^}]*background: var\(--surface\)/);
  assert.match(css, /\.home-new-menu > summary:focus-visible,[\s\S]*?outline: 2px solid var\(--accent\)/);
});

test("workspace create dialog exposes the .pors (default), .rs, and .por choices and an error status", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="workspace-new-file"/);
  assert.match(html, /id="workspace-create-dialog"[^>]*aria-labelledby="workspace-create-title"/);
  assert.match(html, /id="workspace-create-name"[^>]*aria-describedby="workspace-create-error"/);
  assert.match(html, /<option value="pors" selected>[^<]*\.pors\)<\/option>/);
  assert.match(html, /<option value="rs">[^<]*\.rs\)<\/option>/);
  assert.match(html, /<option value="por">[^<]*\.por\)<\/option>/);
  assert.match(html, /id="workspace-create-error"[^>]*role="status"/);
  assert.match(html, /id="workspace-create-submit"/);
  assert.match(html, /id="workspace-create-cancel"/);
});

test("startup has a typed Tauri route to restore the last workspace", () => {
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();
  assert.match(api, /restoreWorkspace\s*=\s*\(\)\s*=>\s*invoke<WorkspaceListing\s*\|\s*null>\("workspace_restore"\)/);
  assert.match(main, /restoreWorkspaceOnStartup\(\)/);
  assert.match(main, /api\.restoreWorkspace\(\)/);
});

test("workspace rename has an accessible dialog and a typed Tauri route", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();
  const css = readFileSync(new URL("../src/style.css", import.meta.url), "utf8");
  assert.match(main, /workspace-rename-file/);
  assert.match(html, /id="workspace-rename-dialog"[^>]*aria-labelledby="workspace-rename-title"/);
  assert.match(html, /id="workspace-rename-name"[^>]*aria-describedby="workspace-rename-error"/);
  assert.match(html, /id="workspace-rename-error"[^>]*role="status"/);
  assert.match(html, /id="workspace-rename-submit"/);
  assert.match(html, /id="workspace-rename-cancel"/);
  assert.match(api, /renameWorkspaceFile/);
  assert.match(api, /"workspace_rename_file"/);
  assert.match(main, /prepareWorkspaceRename/);
  assert.match(css, /@media \(hover: none\), \(pointer: coarse\)[\s\S]*?\.workspace-file-actions\s*\{\s*opacity: 1;/);
});

test("workspace delete has an accessible destructive confirmation and typed Tauri route", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const main = ideSource();

  assert.match(main, /workspace-delete-file/);
  assert.match(html, /id="workspace-delete-dialog"[^>]*aria-labelledby="workspace-delete-title"[^>]*aria-describedby="workspace-delete-description"/);
  assert.match(html, /id="workspace-delete-warning"[^>]*role="status"[^>]*aria-live="polite"/);
  assert.match(html, /id="workspace-delete-save"/);
  assert.match(html, /id="workspace-delete-submit"/);
  assert.match(html, /id="workspace-delete-cancel"/);
  assert.match(api, /deleteWorkspaceFile/);
  assert.match(api, /"workspace_delete_file"/);
  assert.match(main, /deleteWorkspaceFileFlow/);
});
