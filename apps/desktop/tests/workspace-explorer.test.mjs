import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  collectWorkspaceFiles,
  flattenWorkspaceEntries,
  workspaceFileKind,
  workspaceParentFolders,
  workspaceRenameFileName,
  workspaceSourceFileName,
  workspaceSubfolderPath,
} from "../src/workspace-explorer.ts";
import { ideSource } from "./support/ide-source.mjs";
import { ideStyles } from "./support/ide-styles.mjs";

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

test("classifies Portugol-RS, Rust, and PORS exercise files case-insensitively and ignores legacy .por/.pex", () => {
  assert.equal(workspaceFileKind("aula.POR"), null);
  assert.equal(workspaceFileKind("ponte.PORS"), "portugol-rs");
  assert.equal(workspaceFileKind("main.RS"), "rust");
  assert.equal(workspaceFileKind("listas.PORS.JSON"), "exercise");
  assert.equal(workspaceFileKind("listas.PEX"), null);
  assert.equal(workspaceFileKind("leia-me.md"), null);
});

test("validates Rust workspace source filenames", () => {
  assert.equal(workspaceSourceFileName("main", "rust"), "main.rs");
  assert.equal(workspaceSourceFileName("main.RS", "rust"), "main.RS");
  assert.throws(() => workspaceSourceFileName("main.pors", "rust"), /extensão/);
});

test("builds new Ferro and Rust filenames from the selected language", () => {
  assert.equal(workspaceSourceFileName("ola", "source"), "ola.fer");
  assert.equal(workspaceSourceFileName("ponte.pors", "source"), "ponte.pors");
  assert.equal(workspaceSourceFileName("ponte.fer", "source"), "ponte.fer");
  assert.equal(workspaceSourceFileName("main", "rust"), "main.rs");
  assert.equal(workspaceSourceFileName("moderno.FER", "source"), "moderno.FER");
});

test("rejects empty names, path separators, and extensions that conflict with the language", () => {
  assert.throws(() => workspaceSourceFileName("", "source"), /nome/i);
  assert.throws(() => workspaceSourceFileName("../fora", "source"), /nome/i);
  assert.throws(() => workspaceSourceFileName("subpasta\\programa", "source"), /nome/i);
  assert.throws(() => workspaceSourceFileName("programa.por", "source"), /extensão/i);
});

test("renames a workspace file while preserving its original extension or migrating to canonical", () => {
  assert.equal(workspaceRenameFileName("novo", "antigo.fer"), "novo.fer");
  assert.equal(workspaceRenameFileName("novo", "antigo.pors"), "novo.pors");
  assert.equal(workspaceRenameFileName("antigo.fer", "antigo.pors"), "antigo.fer");
  assert.equal(workspaceRenameFileName("ponte.pors", "classico.pors"), "ponte.pors");
  assert.equal(workspaceRenameFileName("crates.rs", "main.rs"), "crates.rs");
  assert.equal(workspaceRenameFileName("atividade", "exercicio.pors.json"), "atividade.pors.json");
  assert.equal(workspaceRenameFileName("atividade.pors.json", "exercicio.pors.json"), "atividade.pors.json");
  assert.equal(workspaceRenameFileName(" nome.pors ", " nome.pors "), " nome.pors ");
});

test("rejects unsafe rename names and extension changes", () => {
  assert.throws(() => workspaceRenameFileName("", "antigo.pors"), /nome/i);
  assert.throws(() => workspaceRenameFileName("../fora", "antigo.pors"), /nome/i);
  assert.throws(() => workspaceRenameFileName("novo.rs", "antigo.pors"), /extensão/i);
  assert.throws(() => workspaceRenameFileName("novo", "notas.txt"), /extensão/i);
  assert.throws(() => workspaceRenameFileName("novo", "antigo.por"), /extensão/i);
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
  const css = ideStyles();

  assert.match(html, /<button id="home-new"[^>]*Portugol-RS \(\.pors\)[^>]*>[^<]*<span[^>]*>[^<]*<\/span>Novo Arquivo<\/button>/);
  assert.match(html, /<details id="home-new-menu"[^>]*data-testid="home-new-menu"/);
  assert.match(html, /<summary[^>]*aria-label="Criar arquivo em outra linguagem"/);
  assert.match(html, /id="home-new-pors"[^>]*data-home-new-language="portugol-rs"[^>]*>Novo Portugol-RS \(\.pors\)<\/button>/);
  assert.match(html, /id="home-new-rust"[^>]*data-home-new-language="rust"[^>]*>Novo Rust \(\.rs\)<\/button>/);
  assert.doesNotMatch(html, /id="home-new-por"/);
  assert.match(main, /\$\("home-new"\)\.onclick = \(\) => newTab\(\);/);
  assert.match(main, /function newTab\(\s*text = PORS_TEMPLATE/);
  assert.match(main, /mountHomeNewFileMenu\(/);
  assert.match(main, /"portugol-rs": \(\) => newTab\(PORS_TEMPLATE, null, undefined, undefined, undefined, "portugol-rs"\)/);
  assert.match(main, /rust: \(\) => newTab\("fn main\(\) \{\\n    \\n\}\\n", null, undefined, undefined, undefined, "rust"\)/);
  assert.doesNotMatch(main, /portugol: \(\) => newTab/);
  assert.doesNotMatch(main, /const newPorsButton = document\.createElement/);
  assert.doesNotMatch(main, /const newRustButton = document\.createElement/);
  assert.match(css, /\.home-actions > button,\s*\.home-new-file-group > button/);
  assert.match(css, /\.home-new-menu-items\s*\{[^}]*background: var\(--surface\)/);
  assert.match(css, /\.home-new-menu > summary:focus-visible,[\s\S]*?outline: 2px solid var\(--accent\)/);
});

test("workspace create dialog exposes the .pors (default) and .rs choices and an error status", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="workspace-new-file"/);
  assert.match(html, /id="workspace-create-dialog"[^>]*aria-labelledby="workspace-create-title"/);
  assert.match(html, /id="workspace-create-name"[^>]*aria-describedby="workspace-create-error"/);
  assert.match(html, /<option value="source" selected>[^<]*\.pors\)<\/option>/);
  assert.match(html, /<option value="rust">[^<]*\.rs\)<\/option>/);
  assert.doesNotMatch(html, /<option value="por">/);
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
  const css = ideStyles();
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

test("validates relative subfolder paths and computes parent folders for expansion", () => {
  assert.equal(workspaceSubfolderPath(""), "");
  assert.equal(workspaceSubfolderPath("   "), "");
  assert.equal(workspaceSubfolderPath("aulas/modulo1"), "aulas/modulo1");
  assert.equal(workspaceSubfolderPath("aulas/modulo1/"), "aulas/modulo1");
  assert.throws(() => workspaceSubfolderPath("../fora"), /subpasta/i);
  assert.throws(() => workspaceSubfolderPath("aulas/./sub"), /subpasta/i);
  assert.throws(() => workspaceSubfolderPath("aulas//sub"), /subpasta/i);
  assert.throws(() => workspaceSubfolderPath("/absoluto"), /relativa/i);
  assert.throws(() => workspaceSubfolderPath("C:/windows"), /relativa/i);
  assert.throws(() => workspaceSubfolderPath("aulas\\modulo1"), /\//);
  assert.deepEqual(workspaceParentFolders("ola.pors"), []);
  assert.deepEqual(workspaceParentFolders("aulas/modulo1/ola.pors"), ["aulas", "aulas/modulo1"]);
});

test("workspace refresh, subfolder creation, and Cargo scaffold export expose accessible controls and IPC", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
  const libRust = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
  const main = ideSource();

  assert.match(html, /id="workspace-refresh"[^>]*aria-label="Atualizar projeto"/);
  assert.match(html, /id="workspace-create-folder"[^>]*aria-describedby="workspace-create-error"/);
  assert.match(html, /id="rust-export-cargo"[^>]*>Exportar Cargo<\/button>/);
  assert.match(api, /refreshWorkspace[\s\S]*invoke<WorkspaceListing>\("workspace_refresh"/);
  assert.match(api, /exportCargoProject[\s\S]*invoke<CargoScaffoldExportResult \| null>\("export_cargo_project"/);
  assert.match(libRust, /workspace::workspace_refresh/);
  assert.match(libRust, /workspace::export_cargo_project/);
  assert.match(main, /api\.refreshWorkspace/);
  assert.match(main, /api\.exportCargoProject/);
  assert.match(main, /workspaceSubfolderPath/);
});

test("classifies desktop project markup and gives each file a marker", async () => {
  const { workspaceFileMarker } = await import("../src/workspace-explorer.ts");
  assert.equal(workspaceFileKind("App/JanelaPrincipal.DESIGN"), "markup");
  assert.equal(workspaceFileKind("App/meu-app.ferproj"), "markup");
  assert.equal(workspaceFileMarker("App/JanelaPrincipal.design"), "TL");
  assert.equal(workspaceFileMarker("App/meu-app.ferproj"), "PJ");
  assert.equal(workspaceFileMarker("ponte.fer"), "Fe");
  assert.equal(workspaceFileMarker("ponte.pors"), "Fe");
  assert.equal(workspaceFileMarker("main.rs"), "R");
  assert.equal(workspaceFileMarker("lista.pors.json"), "EX");
});

test("groups each .design with .fer/.pors of the same name and flags duplicate code-behind", async () => {
  const { groupScreens, screensContaining, SCREEN_PATH_SUFFIX } = await import("../src/workspace-explorer.ts");
  const file = (path) => ({ kind: "file", name: path.split("/").pop(), path });
  const entries = [
    { kind: "folder", name: "App", path: "App", entries: [
      file("App/Cadastro.design"),
      file("App/JanelaPrincipal.DESIGN"),
      file("App/JanelaPrincipal.pors"),
      file("App/Duplicada.design"),
      file("App/Duplicada.pors"),
      file("App/Duplicada.fer"),
      file("App/meu-app.ferproj"),
      file("App/util.fer"),
    ] },
    file("solto.fer"),
  ];

  const grouped = groupScreens(entries);
  const app = grouped[0];
  assert.equal(app.kind, "folder");
  assert.deepEqual(app.entries.map((entry) => [entry.kind, entry.name]), [
    ["screen", "Cadastro"],
    ["screen", "JanelaPrincipal"],
    ["screen", "Duplicada"],
    ["file", "meu-app.ferproj"],
    ["file", "util.fer"],
  ]);
  const [cadastro, principal, duplicada] = app.entries;
  assert.equal(cadastro.code, null);
  assert.equal(cadastro.duplicateCode, false);
  assert.deepEqual(cadastro.entries.map((f) => f.path), ["App/Cadastro.design"]);
  assert.equal(principal.path, `App/JanelaPrincipal.DESIGN${SCREEN_PATH_SUFFIX}`);
  assert.equal(principal.duplicateCode, false);
  assert.deepEqual(principal.entries.map((f) => f.path), ["App/JanelaPrincipal.DESIGN", "App/JanelaPrincipal.pors"]);
  assert.equal(duplicada.duplicateCode, true);
  assert.equal(duplicada.code?.path, "App/Duplicada.fer");
  assert.deepEqual(duplicada.entries.map((f) => f.path), [
    "App/Duplicada.design",
    "App/Duplicada.fer",
    "App/Duplicada.pors",
  ]);
  assert.deepEqual(grouped[1], file("solto.fer"));

  assert.deepEqual(screensContaining(grouped, "App/JanelaPrincipal.pors"), [principal.path]);
  assert.deepEqual(screensContaining(grouped, "App/util.fer"), []);

  const rows = flattenWorkspaceEntries(grouped, new Set(["App", principal.path]));
  assert.deepEqual(rows.map((row) => [row.entry.name, row.depth]), [
    ["App", 0],
    ["Cadastro", 1],
    ["JanelaPrincipal", 1],
    ["JanelaPrincipal.DESIGN", 2],
    ["JanelaPrincipal.pors", 2],
    ["Duplicada", 1],
    ["meu-app.ferproj", 1],
    ["util.fer", 1],
    ["solto.fer", 0],
  ]);
});
