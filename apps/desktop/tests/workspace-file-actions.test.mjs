import test from "node:test";
import assert from "node:assert/strict";
import {
  applyWorkspaceRenameToTabs,
  deleteWorkspaceFileFlow,
  prepareWorkspaceRename,
} from "../src/workspace-file-actions.ts";

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

test("canceling file deletion has no save or filesystem side effects", async () => {
  let saves = 0;
  let deletions = 0;
  const tabs = [{ workspaceId: "w1", workspacePath: "aula.por", dirty: true }];

  const result = await deleteWorkspaceFileFlow({
    tabs,
    workspaceId: "w1",
    filePath: "aula.por",
    choice: "cancel",
    isDirty: (tab) => tab.dirty,
    saveTab: async () => { saves++; return true; },
    deleteFile: async () => { deletions++; return "deleted"; },
  });

  assert.equal(result.status, "cancelled");
  assert.equal(saves, 0);
  assert.equal(deletions, 0);
  assert.equal(result.remainingTabs, tabs);
});

test("saving dirty tabs keeps the file and saves source and exercise buffers", async () => {
  const saved = [];
  let deletions = 0;
  const tabs = [
    { workspaceId: "w1", workspacePath: "aula.por", id: 1, dirty: true },
    { workspaceId: "w1", workspaceExercisePath: "aula.por", id: 2, dirty: true },
  ];

  const result = await deleteWorkspaceFileFlow({
    tabs,
    workspaceId: "w1",
    filePath: "aula.por",
    choice: "save",
    isDirty: (tab) => tab.dirty,
    saveTab: async (tab) => { saved.push(tab.id); return true; },
    deleteFile: async () => { deletions++; return "deleted"; },
  });

  assert.equal(result.status, "saved");
  assert.deepEqual(saved, [1, 2]);
  assert.equal(deletions, 0);
  assert.equal(result.remainingTabs, tabs);
});

test("a failed save keeps the file and stops saving later tabs", async () => {
  const saved = [];
  let deletions = 0;
  const result = await deleteWorkspaceFileFlow({
    tabs: [
      { workspaceId: "w1", workspacePath: "aula.por", id: 1, dirty: true },
      { workspaceId: "w1", workspacePath: "aula.por", id: 2, dirty: true },
    ],
    workspaceId: "w1",
    filePath: "aula.por",
    choice: "save",
    isDirty: (tab) => tab.dirty,
    saveTab: async (tab) => { saved.push(tab.id); return false; },
    deleteFile: async () => { deletions++; return "deleted"; },
  });

  assert.equal(result.status, "save-failed");
  assert.deepEqual(saved, [1]);
  assert.equal(deletions, 0);
});

test("a dirty file cannot take the clean delete path without a discard choice", async () => {
  let deletions = 0;
  const result = await deleteWorkspaceFileFlow({
    tabs: [{ workspaceId: "w1", workspacePath: "aula.por", dirty: true }],
    workspaceId: "w1",
    filePath: "aula.por",
    choice: "delete",
    isDirty: (tab) => tab.dirty,
    saveTab: async () => true,
    deleteFile: async () => { deletions++; return "deleted"; },
  });

  assert.equal(result.status, "needs-decision");
  assert.equal(deletions, 0);
});

test("a failed backend delete leaves all tabs associated with the existing file", async () => {
  const tabs = [
    { workspaceId: "w1", workspacePath: "aula.por", path: "/tmp/aula.por" },
    { workspaceId: "w1", workspaceExercisePath: "aula.por" },
  ];

  await assert.rejects(deleteWorkspaceFileFlow({
    tabs,
    workspaceId: "w1",
    filePath: "aula.por",
    choice: "discard",
    isDirty: () => false,
    saveTab: async () => true,
    deleteFile: async () => { throw new Error("permission denied"); },
  }), /permission denied/);
  assert.equal(tabs[0].workspacePath, "aula.por");
  assert.equal(tabs[1].workspaceExercisePath, "aula.por");
});

test("a successful delete returns only unrelated tabs and removes source and exercise tabs", async () => {
  const deletion = { path: "/tmp/aula.por", relativePath: "aula.por" };
  const tabs = [
    { workspaceId: "w1", workspacePath: "aula.por", id: 1 },
    { workspaceId: "w1", workspaceExercisePath: "aula.por", id: 2 },
    { workspaceId: "w1", workspacePath: "outra.por", id: 3 },
    { workspaceId: "w2", workspacePath: "aula.por", id: 4 },
  ];
  let deleted = null;

  const result = await deleteWorkspaceFileFlow({
    tabs,
    workspaceId: "w1",
    filePath: "aula.por",
    choice: "discard",
    isDirty: () => false,
    saveTab: async () => true,
    deleteFile: async () => { deleted = deletion; return deletion; },
  });

  assert.equal(deleted, deletion);
  assert.equal(result.status, "deleted");
  assert.equal(result.deletion, deletion);
  assert.deepEqual(result.deletedTabs.map((tab) => tab.id), [1, 2]);
  assert.deepEqual(result.remainingTabs.map((tab) => tab.id), [3, 4]);
});
