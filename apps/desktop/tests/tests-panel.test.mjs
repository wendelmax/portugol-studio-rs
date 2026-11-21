import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { panelContentId, panelNameForKey } from "../src/panel-tabs.ts";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");

test("Tests tab controls an accessible empty-state panel", () => {
  assert.equal(panelContentId("tests"), "tests");
  assert.match(html, /<button id="tab-tests"[^>]*role="tab"[^>]*aria-controls="tests"[^>]*>Testes<\/button>/);
  assert.match(html, /<section id="tests"[^>]*role="tabpanel"[^>]*aria-labelledby="tab-tests"/);
  assert.match(html, /id="tests-status"[^>]*role="status"[^>]*>Nenhum teste configurado\.<\/p>/);
  assert.match(html, /runner de testes ainda não está integrado/i);
  assert.doesNotMatch(html, /\b\d+\s+(?:testes?\s+)?(?:aprovados?|falhos?|passaram?)\b/i);
  assert.match(main, /\$\("tab-tests"\)\.onclick\s*=\s*\(\)\s*=>\s*showPanel\("tests"\)/);
  assert.match(main, /\["console",\s*"tests",\s*"messages",\s*"debug",\s*"learning"\]/);
});

test("bottom tabs support arrow, Home, and End keyboard navigation", () => {
  const names = ["console", "tests", "messages", "debug", "learning"];
  assert.equal(panelNameForKey(names, "console", "ArrowRight"), "tests");
  assert.equal(panelNameForKey(names, "tests", "ArrowLeft"), "console");
  assert.equal(panelNameForKey(names, "learning", "ArrowRight"), "console");
  assert.equal(panelNameForKey(names, "console", "ArrowLeft"), "learning");
  assert.equal(panelNameForKey(names, "debug", "Home"), "console");
  assert.equal(panelNameForKey(names, "console", "End"), "learning");
  assert.equal(panelNameForKey(names, "console", "Enter"), null);
  assert.match(main, /panelNameForKey\(bottomPanelNames, current, event\.key\)/);
});
