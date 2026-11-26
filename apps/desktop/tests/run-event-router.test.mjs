import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { replayRunEvents, routeRunEvent } from "../src/run-event-router.ts";
import { ideSource } from "./support/ide-source.mjs";

const main = ideSource();

test("delivers events only to the matching run and drops queued events from an older run", () => {
  let activeRunId = null;
  const pending = [];
  const delivered = [];

  routeRunEvent(41, activeRunId, pending, () => delivered.push("old output"));
  routeRunEvent(42, activeRunId, pending, () => delivered.push("new output"));
  activeRunId = 42;
  replayRunEvents(pending, activeRunId);

  assert.deepEqual(delivered, ["new output"]);
});

test("routes matching run events immediately and ignores late events when no run is pending", () => {
  const pending = [];
  const delivered = [];

  routeRunEvent(7, 7, pending, () => delivered.push("active"));
  routeRunEvent(6, 7, null, () => delivered.push("stale"));

  assert.deepEqual(delivered, ["active"]);
  assert.deepEqual(pending, []);
});

test("replays matching startup events in arrival order", () => {
  const pending = [
    { id: 8, handle: () => delivered.push("output") },
    { id: 8, handle: () => delivered.push("paused") },
    { id: 7, handle: () => delivered.push("stale") },
  ];
  const delivered = [];

  replayRunEvents(pending, 8);

  assert.deepEqual(delivered, ["output", "paused"]);
});

test("IDE routes events and replays startup events through the tested run-id guard", () => {
  const runEvents = readFileSync(new URL("../src/run/run-events.ts", import.meta.url), "utf8");
  const mainEntry = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  const shell = readFileSync(new URL("../src/run/run-events-shell.ts", import.meta.url), "utf8");
  assert.doesNotMatch(mainEntry, /createRunEventsController\(/);
  assert.match(shell, /createRunEventsController\(\{[\s\S]*pendingEvents: \(\) => state\.pendingEvents/);
  assert.match(shell, /listenRun\(handlers\)/);
  assert.match(runEvents, /routeRunEvent\(id, dependencies\.runId\(\), dependencies\.pendingEvents\(\), handle\)/);
  assert.equal((main.match(/replayEvents: replayRunEvents/g) ?? []).length, 1);
  const porsExecution = readFileSync(new URL("../src/run/portugol-rs-execution.ts", import.meta.url), "utf8");
  assert.match(porsExecution, /dependencies\.replayEvents\(events, id\)/);
});
