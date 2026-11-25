import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createTabActivator } from "../src/state/tab-activation.ts";

function fixture({ activeId = 1, targetId = 2, tabLanguage = "portugol-rs" } = {}) {
  const current = { id: 1, language: "portugol", state: { doc: "old" }, symbols: [] };
  const target = { id: targetId, language: tabLanguage, state: { doc: "target" }, symbols: [], workspaceId: "ws" };
  const tabs = [current, target];
  const events = [];
  let active = activeId;
  let editorState = { doc: "edited" };
  const dependencies = {
    tabs: () => tabs,
    setActiveId: (id) => { active = id; events.push(["activeId", id]); },
    currentTab: () => tabs.find((tab) => tab.id === active),
    editorState: () => editorState,
    setHelpOpen: (value) => events.push(["helpOpen", value]),
    homeId: 0,
    helpId: -1,
    setPageVisibility: (id, tab) => events.push(["pages", id, tab?.id ?? null]),
    cancelPreview: () => events.push(["cancelPreview"]),
    renderTabs: () => events.push(["renderTabs"]),
    renderExerciseTests: () => events.push(["renderTests"]),
    clearCursor: () => events.push(["clearCursor"]),
    openHelp: () => events.push(["openHelp"]),
    focusHome: () => events.push(["focusHome"]),
    selectWorkspace: (tab) => events.push(["workspace", tab.workspaceId]),
    setEditorState: (tab) => events.push(["editorState", tab.id]),
    loadPanes: (tab) => events.push(["loadPanes", tab.workspaceId]),
    applyLearningMode: (tab) => events.push(["learning", tab.id]),
    configureEditor: (tab) => events.push(["configure", tab.language]),
    applyPanes: () => events.push(["applyPanes"]),
    clearRustDiagnostics: (tab) => events.push(["clearRustDiagnostics", tab.id]),
    setRustPreviewHidden: (hidden) => events.push(["previewHidden", hidden]),
    renderRustPreview: (tab) => events.push(["preview", tab.id]),
    renderMappingGutter: () => events.push(["mappingGutter"]),
    renderMessages: () => events.push(["messages"]),
    showOutline: (tab, changed) => events.push(["outline", tab.id, changed]),
    updateLibraryButtons: (tab) => events.push(["libraries", tab.id]),
    renderCursor: () => events.push(["cursor"]),
    centerSelectionEnabled: () => true,
    centerSelection: () => events.push(["center"]),
    schedulePreview: (tab) => events.push(["schedulePreview", tab.id]),
    bridgeMode: () => true,
    syncLanguageControls: (tab) => events.push(["controls", tab.id]),
    focusEditor: () => events.push(["focusEditor"]),
    ...arguments[0]?.dependencies,
  };
  const activator = createTabActivator(dependencies);
  return { activator, dependencies, events, current, target, tabs, getActive: () => active, setEditorState: (value) => { editorState = value; } };
}

test("unknown tab IDs do not alter the active editor or shell", () => {
  const { activator, events, getActive } = fixture();

  assert.equal(activator.activate(99), false);
  assert.equal(getActive(), 1);
  assert.deepEqual(events, []);
});

test("activating home preserves the active document and returns focus to home", () => {
  const { activator, current, events, getActive } = fixture();

  assert.equal(activator.activate(0), true);
  assert.equal(current.state.doc, "edited");
  assert.equal(getActive(), 0);
  assert.deepEqual(events, [
    ["activeId", 0], ["pages", 0, null], ["cancelPreview"], ["renderTabs"], ["renderTests"], ["clearCursor"], ["focusHome"],
  ]);
});

test("activating help opens help after storing the previous editor state", () => {
  const { activator, current, events, getActive } = fixture();

  assert.equal(activator.activate(-1), true);
  assert.equal(current.state.doc, "edited");
  assert.equal(getActive(), -1);
  assert.deepEqual(events.slice(0, 4), [["activeId", -1], ["helpOpen", true], ["pages", -1, null], ["cancelPreview"]]);
  assert.deepEqual(events.slice(-2), [["clearCursor"], ["openHelp"]]);
});

test("activating a file configures workspace, editor, previews, and diagnostics", () => {
  const { activator, current, events, target, getActive } = fixture();

  assert.equal(activator.activate(target.id), true);
  assert.equal(current.state.doc, "edited");
  assert.equal(getActive(), target.id);
  assert.deepEqual(events, [
    ["activeId", 2], ["pages", 2, 2], ["workspace", "ws"], ["editorState", 2], ["loadPanes", "ws"],
    ["learning", 2], ["configure", "portugol-rs"], ["applyPanes"], ["renderTabs"], ["renderTests"],
    ["previewHidden", false], ["preview", 2], ["mappingGutter"], ["messages"], ["outline", 2, true],
    ["libraries", 2], ["cursor"], ["center"], ["schedulePreview", 2], ["controls", 2], ["focusEditor"],
  ]);
});

test("IDE delegates tab activation sequencing to the state module", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(main, /createTabActivationShell\(/);
  assert.match(main, /return tabActivator\.activate\(id\)/);
});
