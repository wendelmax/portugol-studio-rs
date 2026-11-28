import assert from "node:assert/strict";
import { test } from "node:test";
import { openFerFile, saveFerFile } from "../../web-playground/file-system.mjs";

function abortError() {
  return Object.assign(new Error("cancelado"), { name: "AbortError" });
}

function fallbackInput({ files = [], fail = false, cancel = false } = {}) {
  return {
    tag: "input",
    type: "",
    accept: "",
    files,
    listeners: new Map(),
    removed: false,
    addEventListener(type, fn) { this.listeners.set(type, fn); },
    click() {
      const listener = this.listeners.get(cancel ? "cancel" : "change");
      if (listener) void listener();
    },
    remove() { this.removed = true; },
  };
}

function fallbackHost(input) {
  const body = {
    children: [],
    append(node) { this.children.push(node); },
    removeChild(node) { this.children = this.children.filter((item) => item !== node); },
  };
  input.remove = () => {
    input.removed = true;
    body.removeChild(input);
  };
  return { window: {}, document: { body, createElement: () => input } };
}

async function withTimeout(promise) {
  const timeout = Symbol("timeout");
  return Promise.race([
    promise,
    new Promise((resolve) => setTimeout(() => resolve(timeout), 50)),
  ]);
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
  const input = fallbackInput({ files: [{ name: "local.fer", text: async () => "escreval(42);" }] });
  const host = fallbackHost(input);

  const opened = await openFerFile(host);

  assert.equal(input.type, "file");
  assert.equal(input.accept, ".fer,text/plain");
  assert.deepEqual(opened, { name: "local.fer", content: "escreval(42);", handle: null });
  assert.equal(input.removed, true);
  assert.deepEqual(host.document.body.children, []);
});

test("leitura falha no fallback rejeita em vez de deixar a promessa pendente", async () => {
  const input = fallbackInput({
    files: [{ name: "ruim.fer", text: async () => { throw new Error("falha de leitura"); } }],
  });

  await assert.rejects(withTimeout(openFerFile(fallbackHost(input))), /falha de leitura/);
  assert.equal(input.removed, true);
});

test("cancelamento no fallback devolve null sem depender do evento change", async () => {
  const input = fallbackInput({ cancel: true });

  assert.equal(await withTimeout(openFerFile(fallbackHost(input))), null);
  assert.equal(input.removed, true);
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

test("salva em arquivo nativo novo com suggestedName", async () => {
  const calls = [];
  const writable = {
    write: async (text) => { calls.push(["write", text]); },
    close: async () => { calls.push(["close"]); },
  };
  const handle = { name: "novo.fer", createWritable: async () => writable };
  let options = null;
  const host = {
    window: {
      showSaveFilePicker: async (input) => {
        options = input;
        return handle;
      },
    },
  };

  const saved = await saveFerFile(host, { name: "novo.fer", content: "escreval(3);" });

  assert.deepEqual(saved, { name: "novo.fer", handle, downloaded: false });
  assert.equal(options.suggestedName, "novo.fer");
  assert.deepEqual(options.types[0].accept, { "text/plain": [".fer"] });
  assert.deepEqual(calls, [["write", "escreval(3);"], ["close"]]);
});

test("salvamento nativo cancelado devolve null", async () => {
  const host = { window: { showSaveFilePicker: async () => { throw abortError(); } } };
  assert.equal(await saveFerFile(host, { name: "novo.fer", content: "" }), null);
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
