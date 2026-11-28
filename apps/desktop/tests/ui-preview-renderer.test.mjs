import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  mountUiPreview,
  parseUiNodeJson,
  serializeUiEvent,
  UiContractError,
  validateUiNode,
} from "../src/ui-preview-renderer.ts";

class FakeElement {
  tagName;
  ownerDocument;
  className = "";
  type = "";
  value = "";
  placeholder = "";
  disabled = false;
  textContent = "";
  selectionStart = null;
  selectionEnd = null;
  attributes = new Map();
  children = [];
  listeners = new Map();
  parent = null;

  constructor(tagName, ownerDocument) {
    this.tagName = tagName.toUpperCase();
    this.ownerDocument = ownerDocument;
  }

  setAttribute(name, value) {
    this.attributes.set(name, String(value));
  }

  getAttribute(name) {
    return this.attributes.has(name) ? this.attributes.get(name) : null;
  }

  appendChild(child) {
    child.parent = this;
    this.children.push(child);
    return child;
  }

  replaceChildren(...nextChildren) {
    for (const child of this.children) {
      child.parent = null;
    }
    this.children = [];
    for (const child of nextChildren) {
      this.appendChild(child);
    }
  }

  contains(target) {
    let cursor = target;
    while (cursor) {
      if (cursor === this) return true;
      cursor = cursor.parent;
    }
    return false;
  }

  addEventListener(type, listener) {
    if (!this.listeners.has(type)) this.listeners.set(type, new Set());
    this.listeners.get(type).add(listener);
  }

  removeEventListener(type, listener) {
    this.listeners.get(type)?.delete(listener);
  }

  trigger(type) {
    for (const listener of [...(this.listeners.get(type) ?? [])]) {
      listener();
    }
  }

  focus() {
    this.ownerDocument.activeElement = this;
  }

  setSelectionRange(start, end) {
    this.selectionStart = start;
    this.selectionEnd = end;
  }
}

function createFakeDom() {
  const doc = {
    activeElement: null,
    createElement(tagName) {
      return new FakeElement(tagName, doc);
    },
  };
  const container = doc.createElement("div");
  return { doc, container };
}

test("renders canonical JSON from ferro-vm and dispatches click and input events", () => {
  const { container } = createFakeDom();
  const events = [];
  const controller = mountUiPreview(container, (event) => events.push(event));

  // Payload exato emitido por `crates/ferro-vm/tests/ui_runtime.rs`
  const json =
    '{"kind":"coluna","children":[{"kind":"texto","content":"contagem0"},{"kind":"botao","id":"inc","label":"+1"},{"kind":"entrada","id":"reset","value":"","placeholder":"digite 0"}]}';
  const parsed = controller.renderJson(json);

  assert.equal(parsed.kind, "coluna");
  assert.equal(container.children.length, 1);
  const column = container.children[0];
  assert.equal(column.getAttribute("data-ui-kind"), "coluna");
  assert.equal(column.children.length, 3);

  const [textEl, buttonEl, inputEl] = column.children;
  assert.equal(textEl.tagName, "P");
  assert.equal(textEl.textContent, "contagem0");

  assert.equal(buttonEl.tagName, "BUTTON");
  assert.equal(buttonEl.type, "button");
  assert.equal(buttonEl.textContent, "+1");
  assert.equal(buttonEl.getAttribute("data-ui-id"), "inc");

  assert.equal(inputEl.tagName, "INPUT");
  assert.equal(inputEl.type, "text");
  assert.equal(inputEl.placeholder, "digite 0");
  assert.equal(inputEl.getAttribute("aria-label"), "digite 0");

  buttonEl.trigger("click");
  inputEl.value = "0";
  inputEl.trigger("input");

  assert.deepEqual(events, [
    { kind: "clique", id: "inc" },
    { kind: "entrada_alterada", id: "reset", value: "0" },
  ]);
  assert.equal(serializeUiEvent(events[0]), '{"kind":"clique","id":"inc"}');
  assert.equal(serializeUiEvent(events[1]), '{"kind":"entrada_alterada","id":"reset","value":"0"}');
});

test("renders lista and linha components and treats untrusted strings strictly as text", () => {
  const { container } = createFakeDom();
  const controller = mountUiPreview(container, () => {});
  const xss = '<img src=x onerror="alert(1)">';
  const json =
    '{"kind":"lista","items":[{"kind":"linha","children":[{"kind":"texto","content":"<img src=x onerror=\\"alert(1)\\">"},{"kind":"botao","id":"ok","label":"Enviar\\t!"}]}]}';

  controller.renderJson(json);
  const list = container.children[0];
  assert.equal(list.tagName, "UL");
  assert.equal(list.children.length, 1);
  assert.equal(list.children[0].tagName, "LI");
  const row = list.children[0].children[0];
  assert.equal(row.getAttribute("data-ui-kind"), "linha");
  assert.equal(row.children[0].textContent, xss);
  assert.equal(row.children[1].textContent, "Enviar\t!");

  const source = readFileSync(new URL("../src/ui-preview-renderer.ts", import.meta.url), "utf8");
  assert.doesNotMatch(source, /\binnerHTML\b|\bouterHTML\b|\binsertAdjacentHTML\b/);
});

test("preserves focused input and caret position across MVU re-renders", () => {
  const { doc, container } = createFakeDom();
  const controller = mountUiPreview(container, () => {});

  controller.render({
    kind: "coluna",
    children: [{ kind: "entrada", id: "nome", value: "Ada", placeholder: "Nome" }],
  });
  const firstInput = container.children[0].children[0];
  firstInput.focus();
  firstInput.setSelectionRange(2, 2);

  controller.render({
    kind: "coluna",
    children: [{ kind: "entrada", id: "nome", value: "Ada L", placeholder: "Nome" }],
  });
  const nextInput = container.children[0].children[0];
  assert.notEqual(nextInput, firstInput);
  assert.equal(doc.activeElement, nextInput);
  assert.equal(nextInput.selectionStart, 2);
  assert.equal(nextInput.selectionEnd, 2);
});

test("blocks events when disabled, after destroy, and from stale detached controls", () => {
  const { container } = createFakeDom();
  const events = [];
  const controller = mountUiPreview(container, (event) => events.push(event));

  controller.render({ kind: "botao", id: "save", label: "Salvar" });
  const staleButton = container.children[0];

  controller.setDisabled(true);
  assert.equal(staleButton.disabled, true);
  staleButton.trigger("click");
  assert.equal(events.length, 0);

  controller.setDisabled(false);
  controller.render({ kind: "botao", id: "save", label: "Salvar 2" });
  const activeButton = container.children[0];

  staleButton.trigger("click");
  assert.equal(events.length, 0);

  activeButton.trigger("click");
  assert.deepEqual(events, [{ kind: "clique", id: "save" }]);

  controller.destroy();
  assert.equal(container.children.length, 0);
  activeButton.trigger("click");
  assert.equal(events.length, 1);
});

test("rejects malformed JSON, unknown components, and trees exceeding node or depth limits", () => {
  assert.throws(
    () => parseUiNodeJson("{invalid"),
    (error) => error instanceof UiContractError && error.code === "invalid_json",
  );
  assert.throws(
    () => parseUiNodeJson('{"kind":"canvas"}'),
    (error) => error instanceof UiContractError && error.code === "invalid_node",
  );
  assert.throws(
    () => validateUiNode({ kind: "botao", id: "ok", label: 42 }),
    (error) => error instanceof UiContractError && error.code === "invalid_node",
  );
  assert.throws(
    () =>
      validateUiNode(
        {
          kind: "coluna",
          children: [
            { kind: "texto", content: "1" },
            { kind: "texto", content: "2" },
            { kind: "texto", content: "3" },
          ],
        },
        { maxNodes: 3 },
      ),
    (error) => error instanceof UiContractError && error.code === "node_limit_exceeded",
  );
  assert.throws(
    () =>
      validateUiNode(
        {
          kind: "coluna",
          children: [{ kind: "linha", children: [{ kind: "texto", content: "x" }] }],
        },
        { maxDepth: 2 },
      ),
    (error) => error instanceof UiContractError && error.code === "depth_limit_exceeded",
  );
});
