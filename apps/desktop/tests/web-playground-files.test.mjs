import assert from "node:assert/strict";
import { test } from "node:test";
import { openFerFile, saveFerFile } from "../../web-playground/file-system.mjs";

function abortError() {
  return Object.assign(new Error("cancelado"), { name: "AbortError" });
}

test("abre .fer com o seletor nativo quando disponível", async () => {
  let options = null;
  const handle = {
    name: "media.fer",
    getFile: async () => ({
      name: "media.fer",
      text: async () => "funcao inicio() {}",
    }),
  };
  const host = {
    window: {
      showOpenFilePicker: async (input) => {
        options = input;
        return [handle];
      },
    },
  };

  const opened = await openFerFile(host);

  assert.deepEqual(opened, { name: "media.fer", content: "funcao inicio() {}", handle });
  assert.equal(options.multiple, false);
  assert.deepEqual(options.types[0].accept, { "text/plain": [".fer"] });
});

test("abertura nativa cancelada devolve null", async () => {
  const host = { window: { showOpenFilePicker: async () => { throw abortError(); } } };
  assert.equal(await openFerFile(host), null);
});

test("abre .fer com input de arquivo quando o seletor nativo não existe", async () => {
  const input = {
    tag: "input",
    type: "",
    accept: "",
    files: [{ name: "local.fer", text: async () => "escreval(42);" }],
    listeners: new Map(),
    addEventListener(type, fn) { this.listeners.set(type, fn); },
    click() { this.listeners.get("change")(); },
  };
  const host = {
    window: {},
    document: { createElement: () => input },
  };

  const opened = await openFerFile(host);

  assert.equal(input.type, "file");
  assert.equal(input.accept, ".fer,text/plain");
  assert.deepEqual(opened, { name: "local.fer", content: "escreval(42);", handle: null });
});

test("salva .fer no handle nativo com createWritable", async () => {
  const calls = [];
  const writable = {
    write: async (text) => { calls.push(["write", text]); },
    close: async () => { calls.push(["close"]); },
  };
  const existing = {
    name: "media.fer",
    createWritable: async () => writable,
  };
  let options = null;
  const host = {
    window: {
      showSaveFilePicker: async (input) => {
        options = input;
        return existing;
      },
    },
  };

  const saved = await saveFerFile(host, { name: "media.fer", content: "escreval(1);", handle: existing });

  assert.deepEqual(saved, { name: "media.fer", handle: existing, downloaded: false });
  assert.deepEqual(calls, [["write", "escreval(1);"], ["close"]]);
  assert.equal(options, null, "handle existente não deve abrir novo seletor");
});

test("salva cópia por download quando a API nativa não existe", async () => {
  const anchors = [];
  const revoked = [];
  let blob = null;
  const host = {
    window: {},
    document: {
      body: { append() {}, removeChild() {} },
      createElement: (tag) => {
        const anchor = {
          tag,
          download: "",
          href: "",
          removed: false,
          click() {},
          remove() { this.removed = true; },
        };
        anchors.push(anchor);
        return anchor;
      },
    },
    createObjectURL: (value) => { blob = value; return "blob:fer"; },
    revokeObjectURL: (url) => { revoked.push(url); },
    setTimeout: (fn) => { fn(); return 1; },
  };

  const saved = await saveFerFile(host, { name: "media.fer", content: "escreval(2);" });

  assert.deepEqual(saved, { name: "media.fer", handle: null, downloaded: true });
  assert.equal(anchors[0].tag, "a");
  assert.equal(anchors[0].download, "media.fer");
  assert.equal(anchors[0].href, "blob:fer");
  assert.equal(anchors[0].removed, true);
  assert.equal(await blob.text(), "escreval(2);");
  assert.deepEqual(revoked, ["blob:fer"]);
});
