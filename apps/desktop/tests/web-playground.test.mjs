import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  PLAYGROUND_EXAMPLES,
  createPlaygroundRuntime,
  renderUiNodeTree,
} from "../../web-playground/playground-core.mjs";
import { LESSON_SOLUTIONS } from "../../web-playground/lessons.mjs";

test("web playground ships twelve organized .fer examples without innerHTML", () => {
  assert.equal(PLAYGROUND_EXAMPLES.length, 12);
  for (const example of PLAYGROUND_EXAMPLES) {
    for (const field of ["id", "title", "file", "group", "mode", "input", "source"]) {
      assert.equal(typeof example[field], "string", `${example.id}: campo ${field} ausente`);
    }
    assert.ok(["console", "ui"].includes(example.mode), `${example.id}: modo inválido`);
    assert.ok(example.source.trim().length > 0, `${example.id}: fonte vazia`);
  }
  assert.deepEqual(
    [...new Set(PLAYGROUND_EXAMPLES.map((example) => example.group))],
    ["Básico", "Controle", "Dados", "Valores", "Visual", "Exercício"],
  );
  assert.ok(PLAYGROUND_EXAMPLES.some((ex) => ex.mode === "ui" && ex.source.includes("UI.coluna")));

  for (const file of ["index.html", "app.mjs", "playground-core.mjs", "ferro-language.mjs"]) {
    const content = readFileSync(new URL(`../../web-playground/${file}`, import.meta.url), "utf8");
    assert.ok(!/\.innerHTML\b/.test(content), `${file} não deve usar innerHTML`);
  }
});

test("web playground ships a searchable Ferro syntax reference panel", () => {
  const html = readFileSync(new URL("../../web-playground/index.html", import.meta.url), "utf8");
  assert.match(html, /id="pg-syntax-panel"/);
  assert.match(html, /id="pg-syntax-search"[^>]*type="search"/);
  assert.match(html, /id="pg-syntax-list"/);
  assert.match(html, /id="pg-syntax-empty"/);
});

test("web playground is organized as an editor shell with sidebar, tabs and bottom panels", () => {
  const html = readFileSync(new URL("../../web-playground/index.html", import.meta.url), "utf8");
  for (const id of ["pg-validate", "pg-format", "pg-run-ui", "pg-export-rust", "pg-output", "pg-example"]) {
    assert.doesNotMatch(html, new RegExp(`id="${id}"`), `${id} não deve existir`);
  }
  for (const id of [
    "pg-sidebar", "pg-file-list", "pg-example-list", "pg-lesson-list",
    "pg-tabs", "pg-active-file", "pg-panel-tabs", "pg-panel-terminal",
    "pg-panel-ui", "pg-panel-rust", "pg-panel-syntax", "pg-new-file",
    "pg-open-file", "pg-save-file", "pg-lesson-check", "pg-lesson-progress",
    "pg-lesson-title", "pg-lesson-prompt", "pg-lesson-hint", "pg-lesson-bar",
    "pg-action", "pg-run", "pg-terminal-history", "pg-stdin",
  ]) {
    assert.match(html, new RegExp(`id="${id}"`), `id ${id} ausente`);
  }
  for (const panel of ["terminal", "ui", "rust", "syntax"]) {
    assert.match(html, new RegExp(`data-panel="${panel}"`));
  }

  const css = readFileSync(new URL("../../web-playground/style.css", import.meta.url), "utf8");
  assert.match(css, /\.pg-workspace\s*\{[^}]*grid-template-columns:\s*240px minmax\(0, 1fr\)/);
  for (const width of [980, 720, 420]) {
    assert.match(css, new RegExp(`@media \\(max-width: ${width}px\\)`));
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
    ferro_alloc(len) {
      const ptr = nextPtr;
      nextPtr += len;
      return ptr;
    },
    ferro_dealloc() {},
    ferro_result_ptr: () => lastResultPtr,
    ferro_result_len: () => lastResultLen,
    ferro_validate(ptr, len) {
      const src = readUtf8(ptr, len);
      return writeResponse({ ok: src.includes("funcao"), diagnostics: [] });
    },
    ferro_format(ptr, len) {
      const src = readUtf8(ptr, len);
      return writeResponse({ ok: true, changed: false, formatted: src });
    },
    ferro_export_rust(ptr, len) {
      const src = readUtf8(ptr, len);
      return writeResponse({ ok: true, code: `fn main() { /* ${src.length} */ }`, diagnostics: [] });
    },
    ferro_run(srcPtr, srcLen, inPtr, inLen) {
      const src = readUtf8(srcPtr, srcLen);
      const input = readUtf8(inPtr, inLen);
      return writeResponse({ ok: true, stdout: `${src.length}:${input.trim()}`, diagnostics: [] });
    },
    ferro_ui_start() {
      return writeResponse({
        ok: true,
        session: 7,
        step: 0,
        history_len: 1,
        tree: { kind: "texto", content: "Olá" },
        stdout: "",
      });
    },
    ferro_ui_dispatch(sessionId, kPtr, kLen, tPtr, tLen, vPtr, vLen) {
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
    ferro_ui_snapshot() {
      return writeResponse({ ok: true, session: 7, step: 1, tree: { kind: "texto", content: "ok" }, stdout: "" });
    },
    ferro_ui_close(sessionId) {
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
    "pg-status", "pg-status-text", "pg-theme", "pg-editor", "pg-highlight",
    "pg-stdin", "pg-terminal-history", "pg-terminal-badge",
    "pg-ui-container", "pg-ui-section", "pg-rust-section", "pg-rust-code", "pg-copy-rust",
    "pg-action", "pg-run", "pg-completions",
    "pg-syntax-search", "pg-syntax-list", "pg-syntax-empty",
    "pg-sidebar", "pg-file-list", "pg-example-list", "pg-lesson-list", "pg-tabs",
    "pg-active-file", "pg-cursor", "pg-panel-tabs", "pg-panel-terminal", "pg-panel-ui",
    "pg-panel-rust", "pg-panel-syntax", "pg-new-file", "pg-open-file", "pg-save-file",
    "pg-lesson-bar", "pg-lesson-title", "pg-lesson-prompt", "pg-lesson-hint",
    "pg-lesson-check", "pg-lesson-progress",
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
    ferro_alloc(len) { const ptr = nextPtr; nextPtr += len; return ptr; },
    ferro_dealloc() {},
    ferro_result_ptr: () => resultPtr,
    ferro_result_len: () => resultLen,
    ferro_validate: () => writeResult({ ok: true, diagnostics: [] }),
    ferro_format: () => writeResult({ ok: true, changed: false, formatted: "" }),
    ferro_export_rust: () => writeResult({ ok: true, code: "", diagnostics: [] }),
    ferro_run(ptr, len) {
      return writeResult({ ok: true, stdout: readUtf8(ptr, len), diagnostics: [] });
    },
    ferro_ui_start: () => writeResult({
      ok: true,
      session: 1,
      step: 0,
      tree: { kind: "texto", content: "Prévia visual" },
      stdout: "",
    }),
    ferro_ui_dispatch: () => writeResult({
      ok: true,
      session: 1,
      step: 1,
      tree: { kind: "texto", content: "Prévia visual" },
      stdout: "",
    }),
    ferro_ui_snapshot: () => writeResult({ ok: true, session: 1, step: 0, tree: { kind: "texto", content: "" }, stdout: "" }),
    ferro_ui_close: () => 1,
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
    assert.match(editor.value, /^funcao inicio\(\)/);

    const action = elements.get("pg-action");
    action.value = "ui";
    action.fire("change");

    assert.match(editor.value, /funcao iniciar\(/);
    assert.equal(elements.get("pg-active-file").textContent, "ui-contador.fer");
    assert.equal(elements.get("pg-tabs").children.length, 2);
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

function collectTokenTexts(root, className) {
  const texts = [];
  const visit = (node) => {
    if (node.className === className) texts.push(node.textContent);
    for (const child of node.children ?? []) visit(child);
  };
  visit(root);
  return texts;
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

    for (const [doc, expected] of [
      ['escreval("{a{b}");', "{a{b}"],
      ['escreval("{\n nome \n}");', "{\n nome \n}"],
    ]) {
      render(doc);
      assert.deepEqual(collectTokenTexts(highlightEl, "pg-tok-interpolation"), [expected], doc);
      assert.equal(highlightEl.textContent, editorEl.value);
    }

    const appSource = readFileSync(new URL("../../web-playground/app.mjs", import.meta.url), "utf8");
    assert.ok(!appSource.includes(".innerHTML"));
  } finally {
    globalThis.document = previousDocument;
  }
});

test("web playground autocomplete suggests Ferro words and supports keyboard selection", async () => {
  class CompletionFakeElement {
    children = [];
    dataset = {};
    attributes = new Map();
    listeners = new Map();
    _text = null;
    value = "";
    hidden = false;
    disabled = false;
    className = "";
    placeholder = "";
    scrollTop = 0;
    scrollLeft = 0;
    selectionStart = 0;
    selectionEnd = 0;
    isFragment = false;
    constructor(tag) { this.tag = tag; }
    append(...nodes) { this.children.push(...nodes); }
    replaceChildren(...nodes) {
      this.children = [];
      for (const node of nodes) {
        if (node?.isFragment) this.children.push(...node.children);
        else this.children.push(node);
      }
    }
    setAttribute(key, value) { this.attributes.set(key, String(value)); }
    addEventListener(type, fn) {
      if (!this.listeners.has(type)) this.listeners.set(type, []);
      this.listeners.get(type).push(fn);
    }
    fire(type, event = {}) {
      for (const fn of this.listeners.get(type) ?? []) fn(event);
    }
    focus() {}
    get textContent() {
      if (this._text !== null) return this._text;
      return this.children.map((child) => child.textContent).join("");
    }
    set textContent(text) { this._text = String(text); }
  }

  const elements = new Map();
  const getElement = (id) => {
    if (!elements.has(id)) elements.set(id, new CompletionFakeElement("div"));
    return elements.get(id);
  };
  const previous = {
    document: Object.getOwnPropertyDescriptor(globalThis, "document"),
    CSS: Object.getOwnPropertyDescriptor(globalThis, "CSS"),
    localStorage: Object.getOwnPropertyDescriptor(globalThis, "localStorage"),
    fetch: Object.getOwnPropertyDescriptor(globalThis, "fetch"),
  };
  globalThis.document = {
    documentElement: new CompletionFakeElement("html"),
    activeElement: null,
    getElementById: getElement,
    createElement: (tag) => new CompletionFakeElement(tag),
    createTextNode: (text) => {
      const node = new CompletionFakeElement("#text");
      node.textContent = text;
      return node;
    },
    createDocumentFragment: () => {
      const fragment = new CompletionFakeElement("#fragment");
      fragment.isFragment = true;
      return fragment;
    },
  };
  globalThis.CSS = { escape: (value) => value };
  globalThis.localStorage = { getItem: () => null, setItem() {} };
  globalThis.fetch = async () => ({ ok: false, status: 404 });

  try {
    await import("../../web-playground/app.mjs?autocomplete");

    const editor = getElement("pg-editor");
    const list = getElement("pg-completions");

    editor.value = "escr";
    editor.selectionStart = editor.selectionEnd = 4;
    editor.fire("input");
    assert.equal(list.hidden, false);
    assert.ok(list.children.some((child) => child.textContent === "escreval"));

    editor.fire("keydown", { key: "ArrowDown", ctrlKey: false, metaKey: false, preventDefault() {} });
    editor.fire("keydown", { key: "Enter", ctrlKey: false, metaKey: false, preventDefault() {} });
    assert.equal(editor.value, "escreval");
    assert.equal(list.hidden, true);

    editor.value = "escr";
    editor.selectionStart = editor.selectionEnd = 4;
    editor.fire("input");
    const item = list.children.find((child) => child.textContent === "escreval");
    assert.ok(item, "escreval deve estar na lista");
    let defaultPrevented = false;
    item.fire("mousedown", { preventDefault() { defaultPrevented = true; } });
    if (!defaultPrevented) editor.fire("blur");
    item.fire("click");
    assert.equal(editor.value, "escreval");

    editor.value = "// escr";
    editor.selectionStart = editor.selectionEnd = 7;
    editor.fire("input");
    assert.equal(list.hidden, true);
  } finally {
    for (const [key, descriptor] of Object.entries(previous)) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    }
  }
});

test("web playground syntax panel filters entries and inserts the selected snippet", async () => {
  class SyntaxFakeElement {
    children = [];
    dataset = {};
    attributes = new Map();
    listeners = new Map();
    _text = null;
    value = "";
    hidden = false;
    disabled = false;
    className = "";
    placeholder = "";
    scrollTop = 0;
    scrollLeft = 0;
    selectionStart = 0;
    selectionEnd = 0;
    isFragment = false;
    constructor(tag) { this.tag = tag; }
    append(...nodes) { this.children.push(...nodes); }
    replaceChildren(...nodes) {
      this.children = [];
      for (const node of nodes) {
        if (node?.isFragment) this.children.push(...node.children);
        else this.children.push(node);
      }
    }
    setAttribute(key, value) { this.attributes.set(key, String(value)); }
    addEventListener(type, fn) {
      if (!this.listeners.has(type)) this.listeners.set(type, []);
      this.listeners.get(type).push(fn);
    }
    fire(type, event = {}) {
      for (const fn of this.listeners.get(type) ?? []) fn(event);
    }
    focus() {}
    get textContent() {
      if (this._text !== null) return this._text;
      return this.children.map((child) => child.textContent).join("");
    }
    set textContent(text) { this._text = String(text); }
  }

  const elements = new Map();
  const getElement = (id) => {
    if (!elements.has(id)) elements.set(id, new SyntaxFakeElement("div"));
    return elements.get(id);
  };
  const findItem = (node) => {
    if (node.className === "pg-syntax-item" && node.textContent.includes("escreval")) return node;
    for (const child of node.children ?? []) {
      const found = findItem(child);
      if (found) return found;
    }
    return null;
  };
  const previous = {
    document: Object.getOwnPropertyDescriptor(globalThis, "document"),
    CSS: Object.getOwnPropertyDescriptor(globalThis, "CSS"),
    localStorage: Object.getOwnPropertyDescriptor(globalThis, "localStorage"),
    fetch: Object.getOwnPropertyDescriptor(globalThis, "fetch"),
  };
  globalThis.document = {
    documentElement: new SyntaxFakeElement("html"),
    activeElement: null,
    getElementById: getElement,
    createElement: (tag) => new SyntaxFakeElement(tag),
    createTextNode: (text) => {
      const node = new SyntaxFakeElement("#text");
      node.textContent = text;
      return node;
    },
    createDocumentFragment: () => {
      const fragment = new SyntaxFakeElement("#fragment");
      fragment.isFragment = true;
      return fragment;
    },
  };
  globalThis.CSS = { escape: (value) => value };
  globalThis.localStorage = { getItem: () => null, setItem() {} };
  globalThis.fetch = async () => ({ ok: false, status: 404 });

  try {
    await import("../../web-playground/app.mjs?syntax-panel");

    const list = getElement("pg-syntax-list");
    const empty = getElement("pg-syntax-empty");
    const search = getElement("pg-syntax-search");
    const editor = getElement("pg-editor");
    assert.ok(list.children.length > 0);

    search.value = "inexistente";
    search.fire("input");
    assert.equal(list.children.length, 0);
    assert.equal(empty.hidden, false);

    search.value = "escreval";
    search.fire("input");
    const item = findItem(list);
    assert.ok(item, "item escreval deve estar na lista filtrada");
    item.fire("click");
    assert.ok(editor.value.includes("escreval"));
  } finally {
    for (const [key, descriptor] of Object.entries(previous)) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    }
  }
});

test("web playground terminal shows stdin history and literal stdout with the action combobox", async () => {
  class TerminalFakeElement {
    children = [];
    dataset = {};
    attributes = new Map();
    listeners = new Map();
    _text = null;
    value = "";
    hidden = false;
    disabled = false;
    className = "";
    placeholder = "";
    scrollTop = 0;
    scrollLeft = 0;
    selectionStart = 0;
    selectionEnd = 0;
    isFragment = false;
    constructor(tag) { this.tag = tag; }
    append(...nodes) { this.children.push(...nodes); }
    replaceChildren(...nodes) {
      this.children = [];
      for (const node of nodes) {
        if (node?.isFragment) this.children.push(...node.children);
        else this.children.push(node);
      }
    }
    setAttribute(key, value) { this.attributes.set(key, String(value)); }
    addEventListener(type, fn) {
      if (!this.listeners.has(type)) this.listeners.set(type, []);
      this.listeners.get(type).push(fn);
    }
    fire(type, event = {}) {
      for (const fn of this.listeners.get(type) ?? []) fn(event);
    }
    focus() {}
    get textContent() {
      if (this._text !== null) return this._text;
      return this.children.map((child) => child.textContent).join("");
    }
    set textContent(text) { this._text = String(text); }
  }

  const elements = new Map();
  const getElement = (id) => {
    if (!elements.has(id)) elements.set(id, new TerminalFakeElement("div"));
    return elements.get(id);
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
  const exports = {
    memory,
    ferro_alloc(len) { const ptr = nextPtr; nextPtr += len; return ptr; },
    ferro_dealloc() {},
    ferro_result_ptr: () => resultPtr,
    ferro_result_len: () => resultLen,
    ferro_validate: () => writeResult({ ok: true, diagnostics: [] }),
    ferro_format: () => writeResult({ ok: true, changed: false, formatted: "" }),
    ferro_export_rust: () => writeResult({ ok: true, code: "", diagnostics: [] }),
    ferro_run: () => writeResult({
      ok: true,
      stdout: "Olá\n<script>x</script>\n",
      diagnostics: [],
    }),
    ferro_ui_start: () => writeResult({ ok: true, session: 1, step: 0, tree: { kind: "texto", content: "" }, stdout: "" }),
    ferro_ui_dispatch: () => writeResult({ ok: true, session: 1, step: 1, tree: { kind: "texto", content: "" }, stdout: "" }),
    ferro_ui_snapshot: () => writeResult({ ok: true, session: 1, step: 0, tree: { kind: "texto", content: "" }, stdout: "" }),
    ferro_ui_close: () => 1,
  };
  const previous = {
    document: Object.getOwnPropertyDescriptor(globalThis, "document"),
    CSS: Object.getOwnPropertyDescriptor(globalThis, "CSS"),
    localStorage: Object.getOwnPropertyDescriptor(globalThis, "localStorage"),
    navigator: Object.getOwnPropertyDescriptor(globalThis, "navigator"),
    fetch: Object.getOwnPropertyDescriptor(globalThis, "fetch"),
    instantiate: Object.getOwnPropertyDescriptor(WebAssembly, "instantiate"),
  };
  globalThis.document = {
    documentElement: new TerminalFakeElement("html"),
    activeElement: null,
    getElementById: getElement,
    createElement: (tag) => new TerminalFakeElement(tag),
    createTextNode: (text) => {
      const node = new TerminalFakeElement("#text");
      node.textContent = text;
      return node;
    },
    createDocumentFragment: () => {
      const fragment = new TerminalFakeElement("#fragment");
      fragment.isFragment = true;
      return fragment;
    },
  };
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
    await import("../../web-playground/app.mjs?terminal-contract");
    await new Promise((resolve) => setTimeout(resolve, 0));

    const stdin = getElement("pg-stdin");
    const history = getElement("pg-terminal-history");
    stdin.value = "Ada\n\n";
    getElement("pg-run").fire("click");

    const lines = history.children.map((child) => ({
      kind: child.dataset.kind,
      text: child.textContent,
    }));
    assert.deepEqual(lines, [
      { kind: "input", text: "Ada" },
      { kind: "input", text: "" },
      { kind: "output", text: "Olá" },
      { kind: "output", text: "<script>x</script>" },
    ]);

    const action = getElement("pg-action");
    action.value = "validate";
    action.fire("change");
    assert.match(getElement("pg-status-text").textContent, /válido/);
    assert.equal(action.value, "");
  } finally {
    for (const [key, descriptor] of Object.entries(previous)) {
      const target = key === "instantiate" ? WebAssembly : globalThis;
      if (descriptor) Object.defineProperty(target, key, descriptor);
      else delete target[key];
    }
  }
});

class EditorFakeElement {
  children = [];
  dataset = {};
  attributes = new Map();
  listeners = new Map();
  _text = null;
  value = "";
  hidden = false;
  disabled = false;
  className = "";
  placeholder = "";
  scrollTop = 0;
  scrollLeft = 0;
  selectionStart = 0;
  selectionEnd = 0;
  isFragment = false;
  constructor(tag) { this.tag = tag; }
  append(...nodes) { this.children.push(...nodes); }
  replaceChildren(...nodes) {
    this.children = [];
    for (const node of nodes) {
      if (node?.isFragment) this.children.push(...node.children);
      else this.children.push(node);
    }
  }
  setAttribute(key, value) { this.attributes.set(key, String(value)); }
  getAttribute(key) { return this.attributes.get(key) ?? null; }
  addEventListener(type, fn) {
    if (!this.listeners.has(type)) this.listeners.set(type, []);
    this.listeners.get(type).push(fn);
  }
  remove() { this.removed = true; }
  fire(type, event = {}) {
    for (const fn of this.listeners.get(type) ?? []) {
      fn({ preventDefault() {}, stopPropagation() {}, ...event });
    }
  }
  click() { this.fire("click"); }
  focus() {}
  get textContent() {
    if (this._text !== null) return this._text;
    return this.children.map((child) => child.textContent).join("");
  }
  set textContent(text) { this._text = String(text); }
}

function createEditorHarness(query, { wasm = false, lessons = null } = {}) {
  const elements = new Map();
  const created = [];
  const storage = new Map();
  if (lessons !== null) storage.set("pg-lessons", JSON.stringify(lessons));

  const getElement = (id) => {
    if (!elements.has(id)) elements.set(id, new EditorFakeElement("div"));
    return elements.get(id);
  };
  const findAll = (root, predicate) => {
    const found = [];
    const visit = (node) => {
      if (predicate(node)) found.push(node);
      for (const child of node.children ?? []) visit(child);
    };
    visit(root);
    return found;
  };
  const findByDataset = (root, key, value) =>
    findAll(root, (node) => node.dataset?.[key] === value)[0] ?? null;

  const memory = new WebAssembly.Memory({ initial: 1 });
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
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
  const readUtf8 = (ptr, len) => decoder.decode(new Uint8Array(memory.buffer, ptr, len));
  const exports = {
    memory,
    ferro_alloc(len) { const ptr = nextPtr; nextPtr += len; return ptr; },
    ferro_dealloc() {},
    ferro_result_ptr: () => resultPtr,
    ferro_result_len: () => resultLen,
    ferro_validate: () => writeResult({ ok: true, diagnostics: [] }),
    ferro_format: () => writeResult({ ok: true, changed: false, formatted: "" }),
    ferro_export_rust: () => writeResult({ ok: true, code: "", diagnostics: [] }),
    ferro_run(srcPtr, srcLen) {
      const source = readUtf8(srcPtr, srcLen);
      return writeResult({
        ok: true,
        stdout: source.includes("Ada") ? "Olá, Ada\n" : "sem Ada\n",
        diagnostics: [],
      });
    },
    ferro_ui_start: () => writeResult({
      ok: true,
      session: 1,
      step: 0,
      tree: { kind: "texto", content: "Prévia" },
      stdout: "",
    }),
    ferro_ui_dispatch: () => writeResult({
      ok: true,
      session: 1,
      step: 1,
      tree: { kind: "texto", content: "Prévia" },
      stdout: "",
    }),
    ferro_ui_snapshot: () => writeResult({
      ok: true, session: 1, step: 0, tree: { kind: "texto", content: "" }, stdout: "",
    }),
    ferro_ui_close: () => 1,
  };

  const previous = {
    document: Object.getOwnPropertyDescriptor(globalThis, "document"),
    CSS: Object.getOwnPropertyDescriptor(globalThis, "CSS"),
    localStorage: Object.getOwnPropertyDescriptor(globalThis, "localStorage"),
    navigator: Object.getOwnPropertyDescriptor(globalThis, "navigator"),
    window: Object.getOwnPropertyDescriptor(globalThis, "window"),
    fetch: Object.getOwnPropertyDescriptor(globalThis, "fetch"),
    instantiate: Object.getOwnPropertyDescriptor(WebAssembly, "instantiate"),
    createObjectURL: Object.getOwnPropertyDescriptor(URL, "createObjectURL"),
    revokeObjectURL: Object.getOwnPropertyDescriptor(URL, "revokeObjectURL"),
  };

  const documentFake = {
    documentElement: new EditorFakeElement("html"),
    body: new EditorFakeElement("body"),
    activeElement: null,
    getElementById: getElement,
    createElement: (tag) => {
      const node = new EditorFakeElement(tag);
      created.push(node);
      return node;
    },
    createTextNode: (text) => {
      const node = new EditorFakeElement("#text");
      node.textContent = text;
      return node;
    },
    createDocumentFragment: () => {
      const fragment = new EditorFakeElement("#fragment");
      fragment.isFragment = true;
      return fragment;
    },
  };
  globalThis.document = documentFake;
  globalThis.CSS = { escape: (value) => value };
  globalThis.localStorage = {
    getItem: (key) => storage.get(key) ?? null,
    setItem: (key, value) => storage.set(key, String(value)),
  };
  Object.defineProperty(globalThis, "navigator", {
    value: { clipboard: { writeText: async () => {} } }, configurable: true,
  });
  globalThis.window = {};
  URL.createObjectURL = () => "blob:fer";
  URL.revokeObjectURL = () => {};
  globalThis.fetch = wasm
    ? async () => ({
        ok: true,
        clone() { throw new Error("stream indisponível no teste"); },
        async arrayBuffer() { return new ArrayBuffer(8); },
      })
    : async () => ({ ok: false, status: 404 });
  Object.defineProperty(WebAssembly, "instantiate", {
    value: async () => ({ instance: { exports } }), configurable: true, writable: true,
  });

  return {
    get: getElement,
    created,
    storage,
    findAll,
    findByDataset,
    async load() {
      await import(`../../web-playground/app.mjs?${query}`);
      await new Promise((resolve) => setTimeout(resolve, 0));
    },
    restore() {
      for (const [key, descriptor] of Object.entries(previous)) {
        const target = key === "instantiate"
          ? WebAssembly
          : ["createObjectURL", "revokeObjectURL"].includes(key)
            ? URL
            : globalThis;
        if (descriptor) Object.defineProperty(target, key, descriptor);
        else delete target[key];
      }
    },
  };
}

function setEditorSource(editor, source) {
  editor.value = source;
  editor.selectionStart = editor.selectionEnd = source.length;
  editor.fire("input");
}

test("playground opens examples as tabs and preserves edited buffers", async () => {
  const h = createEditorHarness("tabs-contract", { wasm: true });
  try {
    await h.load();
    const editor = h.get("pg-editor");
    const tabs = h.get("pg-tabs");
    assert.equal(h.get("pg-active-file").textContent, "boas-vindas.fer");
    assert.equal(tabs.children.length, 1);

    h.findByDataset(h.get("pg-example-list"), "exampleId", "vetores").fire("click");
    assert.equal(h.get("pg-active-file").textContent, "vetores.fer");
    assert.equal(tabs.children.length, 2);

    setEditorSource(editor, "funcao inicio() { escreval(42); }");
    h.findByDataset(tabs, "tabId", "boas-vindas.fer").fire("click");
    assert.notEqual(editor.value, "funcao inicio() { escreval(42); }");
    assert.equal(h.get("pg-stdin").value, "");

    h.findByDataset(tabs, "tabId", "vetores.fer").fire("click");
    assert.equal(editor.value, "funcao inicio() { escreval(42); }");
  } finally {
    h.restore();
  }
});

test("playground activates an open file and closes tabs", async () => {
  const h = createEditorHarness("file-tabs-contract", { wasm: true });
  try {
    await h.load();
    h.findByDataset(h.get("pg-example-list"), "exampleId", "vetores").fire("click");
    h.findByDataset(h.get("pg-file-list"), "fileTabId", "boas-vindas.fer").fire("click");
    assert.equal(h.get("pg-active-file").textContent, "boas-vindas.fer");

    h.findByDataset(h.get("pg-tabs"), "closeTabId", "vetores.fer").fire("click");
    const ids = h.findAll(h.get("pg-tabs"), (node) => node.dataset.tabId).map((node) => node.dataset.tabId);
    assert.deepEqual(ids, ["boas-vindas.fer"]);
  } finally {
    h.restore();
  }
});

test("playground lesson locks stdin and records progress after a check", async () => {
  const h = createEditorHarness("lesson-contract", { wasm: true });
  try {
    await h.load();
    h.findByDataset(h.get("pg-lesson-list"), "lessonId", "variaveis").fire("click");

    assert.equal(h.get("pg-stdin").disabled, true);
    assert.equal(h.get("pg-lesson-bar").hidden, false);
    assert.match(h.get("pg-lesson-title").textContent, /Variáveis/);

    setEditorSource(h.get("pg-editor"), LESSON_SOLUTIONS.variaveis);
    h.get("pg-lesson-check").fire("click");
    assert.deepEqual(JSON.parse(h.storage.get("pg-lessons")), ["variaveis"]);
    assert.match(
      h.findByDataset(h.get("pg-lesson-list"), "lessonId", "variaveis").textContent,
      /✓/,
    );

    h.findByDataset(h.get("pg-tabs"), "tabId", "boas-vindas.fer").fire("click");
    assert.equal(h.get("pg-stdin").disabled, false);
    assert.equal(h.get("pg-lesson-bar").hidden, true);
  } finally {
    h.restore();
  }
});

test("playground panel tabs switch between terminal and reference", async () => {
  const h = createEditorHarness("panels-contract");
  try {
    await h.load();
    assert.equal(h.get("pg-panel-terminal").hidden, false);
    assert.equal(h.get("pg-panel-syntax").hidden, true);

    h.findByDataset(h.get("pg-panel-tabs"), "panel", "syntax").fire("click");
    assert.equal(h.get("pg-panel-syntax").hidden, false);
    assert.equal(h.get("pg-panel-terminal").hidden, true);
    assert.equal(
      h.findByDataset(h.get("pg-panel-tabs"), "panel", "syntax").attributes.get("aria-selected"),
      "true",
    );
  } finally {
    h.restore();
  }
});

test("playground save button downloads the active buffer and clears the dirty mark", async () => {
  const h = createEditorHarness("save-contract");
  try {
    await h.load();
    setEditorSource(h.get("pg-editor"), "funcao inicio() { escreval(7); }");
    assert.match(
      h.findByDataset(h.get("pg-tabs"), "tabId", "boas-vindas.fer").textContent,
      /\*/,
    );

    h.get("pg-save-file").fire("click");
    await new Promise((resolve) => setTimeout(resolve, 0));
    await new Promise((resolve) => setTimeout(resolve, 0));

    const anchors = h.created.filter((node) => node.tag === "a");
    assert.equal(anchors.length, 1);
    assert.equal(anchors[0].download, "boas-vindas.fer");
    assert.match(h.get("pg-status-text").textContent, /baixado|salvo/i);
    assert.doesNotMatch(
      h.findByDataset(h.get("pg-tabs"), "tabId", "boas-vindas.fer").textContent,
      /\*/,
    );
  } finally {
    h.restore();
  }
});
