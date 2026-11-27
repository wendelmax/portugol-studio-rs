import test from "node:test";
import assert from "node:assert/strict";
import { createIncomingFileOpener } from "../src/files/system-files.ts";

function fixture(overrides = {}) {
  const events = [];
  const dependencies = {
    readExercise: async (path) => ({ path }),
    openExercise: (exercise) => events.push(["exercise", exercise.path]),
    openFile: async (path) => events.push(["file", path]),
    setStatus: (message, kind) => events.push(["status", message, kind]),
    ...overrides,
  };
  return { openIncomingFiles: createIncomingFileOpener(dependencies), dependencies, events };
}

test("opens supported exercise and source files in input order and ignores legacy .por/.pex", async () => {
  const { openIncomingFiles, events } = fixture();

  await openIncomingFiles(["a.pors.json", "b.POR", "c.fer", "d.pors", "e.rs", "f.pex"], true);

  assert.deepEqual(events, [
    ["exercise", "a.pors.json"], ["file", "c.fer"], ["file", "d.pors"], ["file", "e.rs"],
  ]);
});

test("warns when requested and no incoming path has a supported extension", async () => {
  const { openIncomingFiles, events } = fixture();

  await openIncomingFiles(["readme.txt"], true);
  await openIncomingFiles(["legado.por"], true);
  await openIncomingFiles([], true);
  await openIncomingFiles(["programa.fer"], true);

  assert.deepEqual(events, [
    ["status", "Escolha um arquivo .fer, .rs ou .pors.json.", "error"],
    ["status", "Arquivos legados .por e .pex não são suportados no Portugol Studio RS. Use o Portugol Studio original para abri-los, ou escolha um arquivo .fer, .rs ou .pors.json.", "error"],
    ["file", "programa.fer"],
  ]);
});

test("reports a failed path and continues opening remaining files", async () => {
  const { openIncomingFiles, events } = fixture({
    openFile: async (path) => {
      if (path === "falha.rs") throw "falha ao abrir";
      events.push(["file", path]);
    },
  });

  await openIncomingFiles(["falha.rs", "ok.pors"]);

  assert.deepEqual(events, [["status", "falha ao abrir", "error"], ["file", "ok.pors"]]);
});
