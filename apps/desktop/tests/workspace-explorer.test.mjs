import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { flattenWorkspaceEntries, workspaceFileKind } from "../src/workspace-explorer.ts";

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

test("home action and explorer tree expose stable accessible controls", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.match(html, /id="home-open-folder"/);
  assert.match(html, /id="workspace-explorer"/);
  assert.match(html, /id="workspace-tree"[^>]*role="tree"/);
  assert.match(html, /data-testid="project-tree"/);
});
