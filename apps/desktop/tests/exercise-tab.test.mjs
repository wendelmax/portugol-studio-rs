import test from "node:test";
import assert from "node:assert/strict";
import { createExerciseTabOpener } from "../src/state/exercise-tab.ts";

const exercise = (overrides = {}) => ({
  name: "Variáveis", exercise: { statement: "Use */ no comentário", cases: 1 },
  ...overrides,
});
function fixture(initialTabs = []) {
  const calls = [];
  let tabs = initialTabs;
  const opener = createExerciseTabOpener({
    tabs: () => tabs,
    replaceTabs: (next) => { tabs = next; calls.push(["replaceTabs", next]); },
    activeTab: () => tabs.at(-1),
    isDirty: (tab) => tab.dirty === true,
    template: "programa { funcao inicio() {} }",
    newTab: (...args) => {
      calls.push(["newTab", ...args]);
      const tab = { id: calls.length, path: args[1], exampleName: args[2], language: args[5] };
      tabs = [...tabs, tab];
      return tab;
    },
    activate: (id) => calls.push(["activate", id]),
    selectWorkspace: (...args) => calls.push(["selectWorkspace", ...args]),
  });
  return { calls, opener, tabs: () => tabs };
}

test("creates a classic exercise tab with a safely wrapped statement", () => {
  const f = fixture();
  f.opener.open(exercise());
  const [, source, path, name, examplePath, document, language] = f.calls[0];
  assert.equal(path, null);
  assert.equal(name, "Variáveis");
  assert.equal(examplePath, undefined);
  assert.equal(document.name, "Variáveis");
  assert.equal(language, "portugol");
  assert.match(source, /Use \* \/ no comentário/);
  assert.match(source, /programa \{ funcao inicio\(\) \{\} \}/);
});

test("uses the PORS starter and Portugol-RS language", () => {
  const f = fixture();
  f.opener.open(exercise({ pors: { starter_code: "funcao inicio() {}" } }));
  assert.equal(f.calls[0][1], "/*\nUse * / no comentário\n*/\n\nfuncao inicio() {}");
  assert.equal(f.calls[0][6], "portugol-rs");
});

test("replaces only a clean single blank tab before opening the exercise", () => {
  const blank = { id: 1, path: null, exampleName: undefined, dirty: false };
  const f = fixture([blank]);
  f.opener.open(exercise());
  assert.equal(f.calls[0][0], "replaceTabs");
  assert.deepEqual(f.tabs().map(({ exampleName }) => exampleName), ["Variáveis"]);
});

test("reactivates an existing workspace exercise instead of opening a duplicate", () => {
  const existing = { id: 7, workspaceId: "course", workspaceExercisePath: "lesson/1.pors" };
  const f = fixture([existing]);
  f.opener.open(exercise(), { id: "course", path: "lesson/1.pors", preferenceKey: "course", exercise: true });
  assert.deepEqual(f.calls, [["activate", 7]]);
});

test("stores workspace exercise metadata, selects it, and activates the new tab", () => {
  const f = fixture();
  f.opener.open(exercise(), { id: "course", path: "lesson/1.pors", preferenceKey: "course", exercise: true });
  const tab = f.tabs()[0];
  assert.equal(tab.workspaceId, "course");
  assert.equal(tab.workspaceExercisePath, "lesson/1.pors");
  assert.equal(tab.workspacePreferenceKey, "course");
  assert.deepEqual(f.calls.slice(1), [["selectWorkspace", "course", "lesson/1.pors"], ["activate", tab.id]]);
});
