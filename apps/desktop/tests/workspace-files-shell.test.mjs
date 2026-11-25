import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

test("main composes workspace, recent files, and file I/O through one shell", async () => {
  const main = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const shell = await readFile(new URL("../src/files/workspace-files-shell.ts", import.meta.url), "utf8");

  assert.match(main, /createWorkspaceFilesShell\(/);
  assert.doesNotMatch(main, /mountWorkspaceExplorer\(|createFileIO\(|createRecentFilesView\(|createRecentFiles\(/);
  assert.match(shell, /mountWorkspaceExplorer\(/);
  assert.match(shell, /createFileIO\(/);
  assert.match(shell, /createRecentFilesView\(/);
  assert.match(shell, /createRecentFiles\(/);
  assert.doesNotMatch(shell, /\binvoke\s*\(/);
});
