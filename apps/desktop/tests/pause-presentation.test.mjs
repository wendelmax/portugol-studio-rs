import test from "node:test";
import assert from "node:assert/strict";
import { createPausePresentation } from "../src/run/pause-presentation.ts";

const info = { position: { offset: 12, line: 4, column: 2 }, stack: [], variables: [] };
const frame = { function: "helper", position: { offset: 20, line: 7, column: 3 } };
function fixture() {
  const calls = [];
  let activeId = 1;
  let runTabId = 1;
  let runId = 9;
  let tabs = [{ id: 1, language: "portugol-rs" }];
  let paused;
  const present = createPausePresentation({
    setPausedState: (value) => { paused = value; calls.push(["paused", value]); },
    activeTabId: () => activeId,
    runTabId: () => runTabId,
    runId: () => runId,
    runTab: () => tabs.find(({ id }) => id === runTabId),
    setDebugLine: (line) => calls.push(["debugLine", line]),
    setRunning: (value) => calls.push(["running", value]),
    renderDebugInfo: (value, select) => calls.push(["render", value, select]),
    activate: (id) => { calls.push(["activate", id]); activeId = id; },
    goTo: (offset) => calls.push(["goTo", offset]),
    showUtilities: (visible) => calls.push(["utilities", visible]),
    showDebugPanel: () => calls.push(["debugPanel"]),
    setStatus: (message) => calls.push(["status", message]),
  });
  return {
    calls, present, paused: () => paused,
    setActiveId: (id) => { activeId = id; },
    setRunTabId: (id) => { runTabId = id; },
    setRunId: (id) => { runId = id; },
    setTabs: (value) => { tabs = value; },
  };
}

test("updates paused state, current editor debug line, and running state", () => {
  const f = fixture();
  f.present.setPaused(info);
  assert.equal(f.paused(), info);
  assert.deepEqual(f.calls.slice(0, 3), [["paused", info], ["debugLine", 4], ["running", true]]);
  const beforeClear = f.calls.length;
  f.setActiveId(2);
  f.present.setPaused(null);
  assert.equal(f.paused(), null);
  assert.deepEqual(f.calls.slice(beforeClear), [["paused", null], ["running", true]]);
});

test("shows debug panel and reports language-specific pause status", () => {
  const f = fixture();
  f.present.setPaused(info);
  assert.ok(f.calls.some(([name, value]) => name === "utilities" && value === true));
  assert.ok(f.calls.some(([name]) => name === "debugPanel"));
  assert.ok(f.calls.some(([name, message]) => name === "status" && message === "VM Portugol-RS pausada na linha 4."));
  f.setTabs([{ id: 1, language: "rust" }]);
  f.present.setPaused(info);
  assert.ok(f.calls.some(([name, message]) => name === "status" && message === "Pausado na linha 4."));
});

test("navigates to the paused source and selects valid stack frames safely", () => {
  const f = fixture();
  f.setActiveId(2);
  f.present.setPaused(info);
  const render = f.calls.find(([name]) => name === "render");
  render[2](frame);
  assert.ok(f.calls.some(([name, id]) => name === "activate" && id === 1));
  assert.deepEqual(f.calls.filter(([name]) => name === "goTo").map(([, offset]) => offset), [20]);

  f.setTabs([]);
  render[2](frame);
  assert.equal(f.calls.filter(([name]) => name === "goTo").length, 1);
});

test("does not set an editor debug line when the paused run is on another tab", () => {
  const f = fixture();
  f.setActiveId(2);
  f.present.setPaused(info);
  assert.equal(f.calls.filter(([name]) => name === "debugLine").length, 0);
});
