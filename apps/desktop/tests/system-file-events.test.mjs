import test from "node:test";
import assert from "node:assert/strict";
import { mountSystemFileEvents } from "../src/files/system-file-events.ts";

test("replays pending files at startup and after OS file events", async () => {
  const calls = [];
  let onFiles;
  mountSystemFileEvents({
    pendingFiles: async () => ["/tmp/a.pors"],
    listenForFiles: async (listener) => { onFiles = listener; },
    listenForDrop: async () => {},
    openFiles: async (paths) => { calls.push(paths); },
    setStatus: () => {},
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  onFiles();
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.deepEqual(calls, [["/tmp/a.pors"], ["/tmp/a.pors"]]);
});

test("reports startup retrieval errors and opens only dropped paths with drop context", async () => {
  const calls = [];
  let onDrop;
  mountSystemFileEvents({
    pendingFiles: async () => { throw new Error("unavailable"); },
    listenForFiles: async () => {},
    listenForDrop: async (listener) => { onDrop = listener; },
    openFiles: async (...args) => { calls.push(args); },
    setStatus: (...args) => { calls.push(args); },
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  onDrop({ type: "over", paths: ["ignored.pors"] });
  onDrop({ type: "drop", paths: ["dropped.pors"] });
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.equal(calls[0][0], "Não foi possível receber arquivos do sistema.");
  assert.deepEqual(calls[1], [["dropped.pors"], true]);
});
