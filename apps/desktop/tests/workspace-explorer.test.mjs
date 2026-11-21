import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { flattenWorkspaceEntries, workspaceFileKind, workspaceSourceFileName } from "../src/workspace-explorer.ts";

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

test("classifies Portugol, Portugol-RS, and exercise files case-insensitively", () => {
  assert.equal(workspaceFileKind("aula.POR"), "portugol");
  assert.equal(workspaceFileKind("ponte.PORS"), "portugol-rs");
  assert.equal(workspaceFileKind("listas.PEX"), "exercise");
  assert.equal(workspaceFileKind("leia-me.md"), null);
});

test("builds new Portugol and Portugol-RS filenames from the selected language", () => {
  assert.equal(workspaceSourceFileName("ola", "por"), "ola.por");
  assert.equal(workspaceSourceFileName("ponte.pors", "pors"), "ponte.pors");
  assert.equal(workspaceSourceFileName("classico.POR", "por"), "classico.POR");
});

test("rejects empty names, path separators, and extensions that conflict with the language", () => {
  assert.throws(() => workspaceSourceFileName("", "por"), /nome/i);
  assert.throws(() => workspaceSourceFileName("../fora", "por"), /nome/i);
  assert.throws(() => workspaceSourceFileName("subpasta\\programa", "pors"), /nome/i);
  assert.throws(() => workspaceSourceFileName("programa.por", "pors"), /extensão/i);
});

test("home action and explorer tree expose stable accessible controls", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="home-open-folder"/);
  assert.match(html, /id="workspace-explorer"/);
  assert.match(html, /id="workspace-tree"[^>]*role="tree"/);
  assert.match(html, /data-testid="project-tree"/);
});

test("workspace create dialog exposes the .por and .pors choice and an error status", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="workspace-new-file"/);
  assert.match(html, /id="workspace-create-dialog"[^>]*aria-labelledby="workspace-create-title"/);
  assert.match(html, /id="workspace-create-name"[^>]*aria-describedby="workspace-create-error"/);
  assert.match(html, /<option value="por">[^<]*\.por\)<\/option>/);
  assert.match(html, /<option value="pors">[^<]*\.pors\)<\/option>/);
  assert.match(html, /id="workspace-create-error"[^>]*role="status"/);
  assert.match(html, /id="workspace-create-submit"/);
  assert.match(html, /id="workspace-create-cancel"/);
});
