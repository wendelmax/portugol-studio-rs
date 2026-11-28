import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { mountSystemFileEvents } from "../src/files/system-file-events.ts";

test("main composes incoming files and operating system events through a shell", () => {
  const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
  const shell = readFileSync(new URL("../src/files/system-files-shell.ts", import.meta.url), "utf8");

  assert.match(main, /mountSystemFilesShell\(/);
  assert.doesNotMatch(main, /createIncomingFileOpener\(|mountSystemFileEvents\(/);
  assert.match(shell, /createIncomingFileOpener\(/);
  assert.match(shell, /mountSystemFileEvents\(/);
});

test("replays pending files at startup and after OS file events", async () => {
  const calls = [];
  let onFiles;
  mountSystemFileEvents({
    pendingFiles: async () => ["/tmp/a.fer"],
    listenForFiles: async (listener) => { onFiles = listener; },
    listenForDrop: async () => {},
    openFiles: async (paths) => { calls.push(paths); },
    setStatus: () => {},
  });
  await new Promise((resolve) => setTimeout(resolve, 0));
  onFiles();
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.deepEqual(calls, [["/tmp/a.fer"], ["/tmp/a.fer"]]);
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
  onDrop({ type: "over", paths: ["ignored.fer"] });
  onDrop({ type: "drop", paths: ["dropped.fer"] });
  await new Promise((resolve) => setTimeout(resolve, 0));

  assert.equal(calls[0][0], "Não foi possível receber arquivos do sistema.");
  assert.deepEqual(calls[1], [["dropped.fer"], true]);
});
