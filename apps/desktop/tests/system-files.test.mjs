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

test("opens supported lesson and source files in input order and ignores legacy .por/.pex/.pors", async () => {
  const { openIncomingFiles, events } = fixture();

  await openIncomingFiles(["a.licao", "b.POR", "c.fer", "d.pors", "e.rs", "f.pex"], true);

  assert.deepEqual(events, [
    ["exercise", "a.licao"], ["file", "c.fer"], ["file", "e.rs"],
  ]);
});

test("warns when requested and no incoming path has a supported extension", async () => {
  const { openIncomingFiles, events } = fixture();

  await openIncomingFiles(["readme.txt"], true);
  await openIncomingFiles(["legado.por"], true);
  await openIncomingFiles([], true);
  await openIncomingFiles(["programa.fer"], true);

  assert.deepEqual(events, [
    ["status", "Escolha um arquivo .fer, .rs ou .licao.", "error"],
    ["status", "Arquivos legados .por e .pex não são suportados no Ferro Studio. Use o Portugol Studio original para abri-los, ou escolha um arquivo .fer, .rs ou .licao.", "error"],
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

  await openIncomingFiles(["falha.rs", "ok.fer"]);

  assert.deepEqual(events, [["status", "falha ao abrir", "error"], ["file", "ok.fer"]]);
});
