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

test("web playground CSS keeps elements with hidden attribute out of layout", () => {
  const css = readFileSync(new URL("../../web-playground/style.css", import.meta.url), "utf8");
  // Painéis como .pg-ui-section definem `display: flex`, que vence a regra da UA
  // para [hidden] sem uma declaração explícita com precedência.
  assert.match(css, /\[hidden\][^{}]*\{[^}]*display:\s*none\s*!important\s*;/);
});

test("web playground does not overwrite the execution status when refreshing generated Rust", () => {
  const app = readFileSync(new URL("../../web-playground/app.mjs", import.meta.url), "utf8");
  assert.match(app, /function runExportRust\(announce = true\)/);
  assert.match(app, /if \(rustWasOpen\) runExportRust\(false\)/);
});

test("web playground exposes a persistent theme toggle and a safe syntax highlight layer", () => {
  const html = readFileSync(new URL("../../web-playground/index.html", import.meta.url), "utf8");
  assert.match(html, /id="pg-theme"/);
  assert.match(html, /id="pg-highlight"[^>]*aria-hidden="true"/);

  const css = readFileSync(new URL("../../web-playground/style.css", import.meta.url), "utf8");
  assert.match(css, /:root\[data-theme="dark"\]/);
  assert.match(css, /@media \(max-width: 720px\)/);

  const app = readFileSync(new URL("../../web-playground/app.mjs", import.meta.url), "utf8");
  assert.match(app, /localStorage\.setItem\(THEME_STORAGE_KEY/);
  assert.match(app, /document\.createTextNode\(/);
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

test("Prévia UI loads a visual example when the editor holds a console program", async () => {
  class FakeElement {
    children = [];
    dataset = {};
    attributes = new Map();
    listeners = new Map();
    textContent = "";
    value = "";
    hidden = false;
    disabled = false;
    className = "";
    placeholder = "";
    scrollTop = 0;
    scrollLeft = 0;
    constructor(tag) { this.tag = tag; }
    append(...nodes) { this.children.push(...nodes); }
    replaceChildren(...nodes) { this.children = nodes; }
    setAttribute(key, value) { this.attributes.set(key, String(value)); }
    addEventListener(type, fn) {
      if (!this.listeners.has(type)) this.listeners.set(type, []);
      this.listeners.get(type).push(fn);
    }
    fire(type) {
      for (const fn of this.listeners.get(type) ?? []) fn({ preventDefault() {} });
    }
  }

  const ids = [
    "pg-status", "pg-status-text", "pg-theme", "pg-example", "pg-editor", "pg-highlight",
    "pg-stdin", "pg-output", "pg-output-empty", "pg-output-badge", "pg-diagnostics",
    "pg-ui-container", "pg-ui-section", "pg-rust-section", "pg-rust-code", "pg-copy-rust",
    "pg-validate", "pg-format", "pg-run", "pg-run-ui", "pg-export-rust",
  ];
  const elements = new Map(ids.map((id) => [id, new FakeElement("div")]));
  const doc = {
    documentElement: new FakeElement("html"),
    activeElement: null,
    getElementById: (id) => elements.get(id) ?? null,
    createElement: (tag) => new FakeElement(tag),
    createTextNode: (text) => ({ textContent: text }),
    createDocumentFragment: () => new FakeElement("fragment"),
  };

  const memory = new WebAssembly.Memory({ initial: 1 });
  const encoder = new TextEncoder();
  let nextPtr = 1024;
  let resultPtr = 2048;
  let resultLen = 0;
  const writeResult = (payload) => {
    const bytes = encoder.encode(JSON.stringify(payload));
    resultPtr = 2048;
    resultLen = bytes.length;
    new Uint8Array(memory.buffer, resultPtr, resultLen).set(bytes);
    return resultLen;
  };
  const readUtf8 = (ptr, len) => new TextDecoder().decode(new Uint8Array(memory.buffer, ptr, len));
  const exports = {
    memory,
    pors_alloc(len) { const ptr = nextPtr; nextPtr += len; return ptr; },
    pors_dealloc() {},
    pors_result_ptr: () => resultPtr,
    pors_result_len: () => resultLen,
    pors_validate: () => writeResult({ ok: true, diagnostics: [] }),
    pors_format: () => writeResult({ ok: true, changed: false, formatted: "" }),
    pors_export_rust: () => writeResult({ ok: true, code: "", diagnostics: [] }),
    pors_run(ptr, len) {
      return writeResult({ ok: true, stdout: readUtf8(ptr, len), diagnostics: [] });
    },
    pors_ui_start: () => writeResult({
      ok: true,
      session: 1,
      step: 0,
      tree: { kind: "texto", content: "Prévia visual" },
      stdout: "",
    }),
    pors_ui_dispatch: () => writeResult({
      ok: true,
      session: 1,
      step: 1,
      tree: { kind: "texto", content: "Prévia visual" },
      stdout: "",
    }),
    pors_ui_snapshot: () => writeResult({ ok: true, session: 1, step: 0, tree: { kind: "texto", content: "" }, stdout: "" }),
    pors_ui_close: () => 1,
  };

  const globals = {
    document: Object.getOwnPropertyDescriptor(globalThis, "document"),
    CSS: Object.getOwnPropertyDescriptor(globalThis, "CSS"),
    localStorage: Object.getOwnPropertyDescriptor(globalThis, "localStorage"),
    navigator: Object.getOwnPropertyDescriptor(globalThis, "navigator"),
    fetch: Object.getOwnPropertyDescriptor(globalThis, "fetch"),
    instantiate: Object.getOwnPropertyDescriptor(WebAssembly, "instantiate"),
  };
  globalThis.document = doc;
  globalThis.CSS = { escape: (value) => value };
  globalThis.localStorage = { getItem: () => null, setItem() {} };
  Object.defineProperty(globalThis, "navigator", { value: { clipboard: { writeText: async () => {} } }, configurable: true });
  globalThis.fetch = async () => ({
    ok: true,
    clone() { throw new Error("stream indisponível no teste"); },
    async arrayBuffer() { return new ArrayBuffer(8); },
  });
  Object.defineProperty(WebAssembly, "instantiate", {
    value: async () => ({ instance: { exports } }),
    configurable: true,
    writable: true,
  });

  try {
    await import("../../web-playground/app.mjs?ui-preview-regression");
    await new Promise((resolve) => setTimeout(resolve, 0));

    const editor = elements.get("pg-editor");
    assert.match(editor.value, /^funcao saudar\(/);

    elements.get("pg-run-ui").fire("click");

    assert.match(editor.value, /funcao iniciar\(/);
    assert.equal(elements.get("pg-example").value, "ui-contador");
    assert.equal(elements.get("pg-ui-section").hidden, false);
    assert.equal(elements.get("pg-ui-container").children.length, 1);
  } finally {
    for (const [key, descriptor] of Object.entries(globals)) {
      const target = key === "instantiate" ? WebAssembly : globalThis;
      if (descriptor) Object.defineProperty(target, key, descriptor);
      else delete target[key];
    }
  }
});

class FakeElement {
  children = [];
  dataset = {};
  className = "";
  _text = null;
  value = "";
  hidden = false;
  disabled = false;
  scrollTop = 0;
  scrollLeft = 0;
  attributes = new Map();
  listeners = new Map();
  constructor(tag) {
    this.tag = tag;
  }
  append(...nodes) {
    this.children.push(...nodes);
  }
  replaceChildren(...nodes) {
    this.children = [];
    for (const node of nodes) {
      if (node && node.isFragment) {
        this.children.push(...node.children);
      } else {
        this.children.push(node);
      }
    }
  }
  setAttribute(key, value) {
    this.attributes.set(key, String(value));
  }
  addEventListener(type, fn) {
    this.listeners.set(type, fn);
  }
  get textContent() {
    if (this._text === null) {
      return this.children.map((child) => child.textContent).join("");
    }
    return this._text;
  }
  set textContent(text) {
    this._text = String(text);
  }
}

function collectTokenClasses(root) {
  const classes = [];
  const visit = (node) => {
    const className = typeof node.className === "string" ? node.className : "";
    if (className.startsWith("pg-tok-")) classes.push(className);
    for (const child of node.children ?? []) visit(child);
  };
  visit(root);
  return classes;
}

test("web playground highlights interpolated strings and escreval builtins", async () => {
  const elementsById = new Map();
  const fakeDocument = {
    documentElement: new FakeElement("html"),
    createElement: (tag) => new FakeElement(tag),
    createDocumentFragment: () => {
      const fragment = new FakeElement("#document-fragment");
      fragment.isFragment = true;
      return fragment;
    },
    createTextNode: (text) => {
      const node = new FakeElement("#text");
      node.textContent = text;
      return node;
    },
    getElementById: (id) => {
      if (!elementsById.has(id)) elementsById.set(id, new FakeElement("div"));
      return elementsById.get(id);
    },
  };
  const previousDocument = globalThis.document;
  globalThis.document = fakeDocument;
  try {
    await import("../../web-playground/app.mjs?highlight-contract");

    const editorEl = fakeDocument.getElementById("pg-editor");
    const highlightEl = fakeDocument.getElementById("pg-highlight");

    const render = (source) => {
      editorEl.value = source;
      const listener = editorEl.listeners.get("input");
      if (typeof listener === "function") listener();
    };

    render('escreval("Olá, {nome}!");');
    const classes = collectTokenClasses(highlightEl);
    assert.ok(
      classes.includes("pg-tok-interpolation"),
      `esperava pg-tok-interpolation no realce: ${classes}`,
    );
    assert.ok(
      classes.includes("pg-tok-builtin"),
      `esperava pg-tok-builtin no realce: ${classes}`,
    );

    for (const doc of ['escreval("\\{nome}");', 'escreval("{}");', 'escreval("abri {");']) {
      render(doc);
      const docClasses = collectTokenClasses(highlightEl);
      assert.ok(
        !docClasses.includes("pg-tok-interpolation"),
        `${doc} não deve produzir pg-tok-interpolation`,
      );
    }

    render('escreval("Olá, {nome}!");');
    assert.equal(highlightEl.textContent, editorEl.value);

    const appSource = readFileSync(new URL("../../web-playground/app.mjs", import.meta.url), "utf8");
    assert.ok(!appSource.includes(".innerHTML"));
  } finally {
    globalThis.document = previousDocument;
  }
});
