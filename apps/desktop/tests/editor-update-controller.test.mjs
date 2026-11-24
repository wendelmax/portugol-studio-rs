import test from "node:test";
import assert from "node:assert/strict";
import { createEditorUpdateController } from "../src/editor/update-controller.ts";

function fixture({ language = "portugol", learningMode = "bridge", panelVisible = true, centerActive = true } = {}) {
  const events = [];
  let state = { doc: { toString: () => "novo código" } };
  const tab = {
    id: 1,
    language,
    runtimeCompileDiagnostics: [{ message: "antigo" }],
    exercise: { id: 1 },
    exerciseTestGeneration: 4,
    exerciseTestRun: { status: "passed" },
    rustPreview: { stale: false, pending: true, artifact: { source: "código antigo" } },
  };
  const dependencies = {
    activeTab: () => tab,
    currentEditorState: () => state,
    renderMessages: () => events.push(["messages"]),
    renderExerciseTests: () => events.push(["exerciseTests"]),
    renderTabs: () => events.push(["tabs"]),
    markPreviewStale: (preview, source) => { events.push(["stale", source]); return { ...preview, stale: true, pending: false }; },
    learningMode: () => learningMode,
    schedulePreview: () => events.push(["schedulePreview"]),
    selectionChanged: () => events.push(["selectionChanged"]),
    learningPanelVisible: () => panelVisible,
    renderLearningPanel: () => events.push(["renderLearningPanel"]),
    renderCursor: () => events.push(["cursor"]),
    renderMappingGutter: () => events.push(["mappingGutter"]),
    centerActive: () => centerActive,
    centerSelection: () => events.push(["center"]),
  };
  return {
    handleUpdate: createEditorUpdateController(dependencies), dependencies, events, tab,
    setState: (next) => { state = next; },
  };
}

test("document changes clear transient state and refresh exercise, tabs, preview, and selection", async () => {
  const { handleUpdate, events, tab, setState } = fixture();
  const state = { doc: { toString: () => "novo código" } };

  setState(state);
  handleUpdate({ docChanged: true, selectionSet: true, state });
  await new Promise((resolve) => queueMicrotask(resolve));

  assert.deepEqual(tab.runtimeCompileDiagnostics, []);
  assert.equal(tab.exerciseTestGeneration, 5);
  assert.equal(tab.exerciseTestRun, undefined);
  assert.equal(tab.rustPreview.stale, true);
  assert.deepEqual(events.map(([name]) => name), [
    "messages", "exerciseTests", "tabs", "stale", "schedulePreview", "selectionChanged",
    "renderLearningPanel", "cursor", "mappingGutter", "center",
  ]);
});

test("Rust edits do not invalidate generated Portugol preview or schedule bridge updates", () => {
  const { handleUpdate, events, tab } = fixture({ language: "rust" });

  handleUpdate({ docChanged: true, selectionSet: false, state: { doc: { toString: () => "rust" } } });

  assert.deepEqual(tab.runtimeCompileDiagnostics, []);
  assert.deepEqual(events.map(([name]) => name), ["messages", "exerciseTests", "tabs", "renderLearningPanel", "cursor", "mappingGutter"]);
});

test("selection updates only render visible learning content and defer guarded centering", async () => {
  const { handleUpdate, events, setState } = fixture({ panelVisible: false });
  const updateState = {};

  handleUpdate({ docChanged: false, selectionSet: true, state: updateState });
  setState(updateState);
  await new Promise((resolve) => queueMicrotask(resolve));

  assert.deepEqual(events.map(([name]) => name), ["selectionChanged", "cursor", "mappingGutter", "center"]);
});

test("does not center a selection after the editor has moved to another state", async () => {
  const { handleUpdate, events, setState } = fixture();

  handleUpdate({ docChanged: false, selectionSet: true, state: {} });
  setState({});
  await new Promise((resolve) => queueMicrotask(resolve));

  assert.deepEqual(events.map(([name]) => name), ["selectionChanged", "renderLearningPanel", "cursor", "mappingGutter"]);
});
