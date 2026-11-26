import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  PLAYGROUND_EXAMPLES,
  createPlaygroundRuntime,
  renderUiNodeTree,
} from "../../web-playground/playground-core.mjs";

test("web playground ships console and visual MVU .pors examples without innerHTML", () => {
  assert.ok(PLAYGROUND_EXAMPLES.length >= 4);
  assert.ok(PLAYGROUND_EXAMPLES.some((ex) => ex.mode === "console"));
  assert.ok(PLAYGROUND_EXAMPLES.some((ex) => ex.mode === "ui" && ex.source.includes("UI.coluna")));

  for (const file of ["index.html", "app.mjs", "playground-core.mjs"]) {
    const content = readFileSync(new URL(`../../web-playground/${file}`, import.meta.url), "utf8");
    assert.ok(!/\.innerHTML\b/.test(content), `${file} não deve usar innerHTML`);
  }
});

test("playground runtime encodes/decodes UTF-8 across linear memory for validate/run/ui", () => {
  const memory = new WebAssembly.Memory({ initial: 1 });
  let nextPtr = 64;
  let lastResultPtr = 0;
  let lastResultLen = 0;
  const encoder = new TextEncoder();
  const decoder = new TextDecoder("utf-8");

  const writeResponse = (payload) => {
    const bytes = encoder.encode(JSON.stringify(payload));
    lastResultPtr = 2048;
    lastResultLen = bytes.length;
    new Uint8Array(memory.buffer, lastResultPtr, bytes.length).set(bytes);
    return bytes.length;
  };

  const readUtf8 = (ptr, len) => decoder.decode(new Uint8Array(memory.buffer, ptr, len));
  let closedSession = null;

  const runtime = createPlaygroundRuntime({
    memory,
    pors_alloc(len) {
      const ptr = nextPtr;
      nextPtr += len;
      return ptr;
    },
    pors_dealloc() {},
    pors_result_ptr: () => lastResultPtr,
    pors_result_len: () => lastResultLen,
    pors_validate(ptr, len) {
      const src = readUtf8(ptr, len);
      return writeResponse({ ok: src.includes("funcao"), diagnostics: [] });
    },
    pors_format(ptr, len) {
      const src = readUtf8(ptr, len);
      return writeResponse({ ok: true, changed: false, formatted: src });
    },
    pors_export_rust(ptr, len) {
      const src = readUtf8(ptr, len);
      return writeResponse({ ok: true, code: `fn main() { /* ${src.length} */ }`, diagnostics: [] });
    },
    pors_run(srcPtr, srcLen, inPtr, inLen) {
      const src = readUtf8(srcPtr, srcLen);
      const input = readUtf8(inPtr, inLen);
      return writeResponse({ ok: true, stdout: `${src.length}:${input.trim()}`, diagnostics: [] });
    },
    pors_ui_start() {
      return writeResponse({
        ok: true,
        session: 7,
        step: 0,
        history_len: 1,
        tree: { kind: "texto", content: "Olá" },
        stdout: "",
      });
    },
    pors_ui_dispatch(sessionId, kPtr, kLen, tPtr, tLen, vPtr, vLen) {
      return writeResponse({
        ok: true,
        session: sessionId,
        step: 1,
        history_len: 2,
        tree: {
          kind: "texto",
          content: `${readUtf8(kPtr, kLen)}:${readUtf8(tPtr, tLen)}:${readUtf8(vPtr, vLen)}`,
        },
        stdout: "",
      });
    },
    pors_ui_snapshot() {
      return writeResponse({ ok: true, session: 7, step: 1, tree: { kind: "texto", content: "ok" }, stdout: "" });
    },
    pors_ui_close(sessionId) {
      closedSession = sessionId;
      return 1;
    },
  });

  assert.equal(runtime.validate("funcao inicio() {}").ok, true);
  assert.equal(runtime.run("funcao inicio() {}", "Ada\n").stdout, "18:Ada");
  assert.equal(runtime.exportRust("funcao inicio() {}").code, "fn main() { /* 18 */ }");

  const started = runtime.startUi("funcao iniciar() {}");
  assert.equal(started.ok, true);
  assert.equal(started.session, 7);

  const dispatched = runtime.dispatchUi(7, { kind: "entrada_alterada", id: "nome", value: "Café" });
  assert.equal(dispatched.tree.content, "entrada_alterada:nome:Café");

  assert.equal(runtime.closeUi(7), true);
  assert.equal(closedSession, 7);
});

test("renderUiNodeTree builds interactive DOM nodes safely via createElement and textContent", () => {
  class FakeNode {
    children = [];
    dataset = {};
    className = "";
    textContent = "";
    value = "";
    placeholder = "";
    listeners = new Map();
    attributes = new Map();
    constructor(tag) { this.tag = tag; }
    append(...nodes) { this.children.push(...nodes); }
    replaceChildren(...nodes) { this.children = nodes; }
    setAttribute(k, v) { this.attributes.set(k, String(v)); }
    addEventListener(type, fn) { this.listeners.set(type, fn); }
  }

  const doc = {
    activeElement: null,
    createElement: (tag) => new FakeNode(tag),
  };
  const container = new FakeNode("div");
  const events = [];

  renderUiNodeTree(
    doc,
    container,
    {
      kind: "coluna",
      children: [
        { kind: "texto", content: "<script>nao-executa</script>" },
        { kind: "botao", id: "inc", label: "+1" },
        { kind: "entrada", id: "nome", value: "Ada", placeholder: "Nome" },
      ],
    },
    (event) => events.push(event),
  );

  const col = container.children[0];
  assert.equal(col.className, "pg-ui-coluna");
  assert.equal(col.children[0].textContent, "<script>nao-executa</script>");
  col.children[1].listeners.get("click")();
  col.children[2].value = "Lovelace";
  col.children[2].listeners.get("input")();

  assert.deepEqual(events, [
    { kind: "clique", id: "inc" },
    { kind: "entrada_alterada", id: "nome", value: "Lovelace" },
  ]);
});
