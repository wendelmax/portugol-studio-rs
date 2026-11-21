import test from "node:test";
import assert from "node:assert/strict";
import { applyWorkspaceRenameToTabs, prepareWorkspaceRename } from "../src/workspace-file-actions.ts";

test("canceling the dirty-buffer confirmation never saves or renames", async () => {
  let saves = 0;
  const result = await prepareWorkspaceRename({
    tabs: [{ workspacePath: "aula.por", dirty: true }],
    sourcePath: "aula.por",
    isDirty: (tab) => tab.dirty,
    confirmSave: async () => false,
    saveTab: async () => { saves++; return true; },
  });

  assert.equal(result, "cancelled");
  assert.equal(saves, 0);
});

test("save failure stops before the file rename can proceed", async () => {
  const saveOrder = [];
  const result = await prepareWorkspaceRename({
    tabs: [
      { workspacePath: "aula.por", id: 1, dirty: true },
      { workspacePath: "aula.por", id: 2, dirty: true },
    ],
    sourcePath: "aula.por",
    isDirty: (tab) => tab.dirty,
    confirmSave: async () => true,
    saveTab: async (tab) => { saveOrder.push(tab.id); return tab.id !== 1; },
  });

  assert.equal(result, "save-failed");
  assert.deepEqual(saveOrder, [1]);
});

test("saves all dirty source tabs in order and ignores dirty exercise answer buffers", async () => {
  const saveOrder = [];
  const result = await prepareWorkspaceRename({
    tabs: [
      { workspacePath: "aula.por", id: 1, dirty: true },
      { workspacePath: "aula.por", id: 2, dirty: true },
      { workspaceExercisePath: "atividade.pex", id: 3, dirty: true },
    ],
    sourcePath: "aula.por",
    isDirty: (tab) => tab.dirty,
    confirmSave: async () => true,
    saveTab: async (tab) => { saveOrder.push(tab.id); return true; },
  });

  assert.equal(result, "ready");
  assert.deepEqual(saveOrder, [1, 2]);
});

test("renaming a .pex never tries to save its generated exercise answer buffer", async () => {
  let confirmations = 0;
  let saves = 0;
  const result = await prepareWorkspaceRename({
    tabs: [{ workspaceExercisePath: "atividade.pex", id: 3, dirty: true }],
    sourcePath: "atividade.pex",
    isDirty: (tab) => tab.dirty,
    confirmSave: async () => { confirmations++; return true; },
    saveTab: async () => { saves++; return true; },
  });

  assert.equal(result, "ready");
  assert.equal(confirmations, 0);
  assert.equal(saves, 0);
});

test("updates open source and exercise paths only after rename success, preserving buffers", () => {
  const source = { path: "/tmp/aula.por", workspacePath: "aula.por", source: "escreva(1)" };
  const exercise = { workspaceExercisePath: "atividade.pex", exampleName: "atividade.pex", source: "resposta" };
  const unchanged = { workspacePath: "outra.por", path: "/tmp/outra.por" };

  const recentPaths = applyWorkspaceRenameToTabs(
    [source, exercise, unchanged],
    "aula.por",
    { path: "/tmp/novo.por", relativePath: "novo.por" },
  );

  assert.equal(source.path, "/tmp/novo.por");
  assert.equal(source.workspacePath, "novo.por");
  assert.equal(source.source, "escreva(1)");
  assert.equal(exercise.workspaceExercisePath, "atividade.pex");
  assert.equal(exercise.exampleName, "atividade.pex");
  assert.equal(unchanged.workspacePath, "outra.por");
  assert.deepEqual(recentPaths, ["/tmp/aula.por"]);
});

test("updates an open exercise tab when a .pex file is renamed", () => {
  const exercise = { workspaceExercisePath: "atividade.pex", exampleName: "atividade.pex", source: "resposta" };

  applyWorkspaceRenameToTabs([exercise], "atividade.pex", {
    path: "/tmp/nova.pex",
    relativePath: "nova.pex",
  });

  assert.equal(exercise.workspaceExercisePath, "nova.pex");
  assert.equal(exercise.exampleName, "nova.pex");
  assert.equal(exercise.source, "resposta");
});
