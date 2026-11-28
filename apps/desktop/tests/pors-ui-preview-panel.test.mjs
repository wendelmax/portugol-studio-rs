import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { findPorsUiExample, PORS_UI_EXAMPLES } from "../src/pors-ui-examples.ts";
import {
  createWasmUiBackend,
  mountPorsUiPreviewPanel,
} from "../src/pors-ui-preview-panel.ts";
import { loadPorsWasmRuntime } from "../src/pors-wasm-host.ts";
import { mountUiPreview } from "../src/ui-preview-renderer.ts";

class FakeElement {
  tagName;
  ownerDocument;
  className = "";
  type = "";
  value = "";
  placeholder = "";
  disabled = false;
  hidden = false;
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

function createPanelDom() {
  const doc = {
    activeElement: null,
    createElement(tagName) {
      return new FakeElement(tagName, doc);
    },
  };
  const panel = doc.createElement("aside");
  panel.hidden = true;
  return {
    doc,
    elements: {
      panel,
      toggleButton: doc.createElement("button"),
      runButton: doc.createElement("button"),
      restartButton: doc.createElement("button"),
      cancelButton: doc.createElement("button"),
      closeButton: doc.createElement("button"),
      exampleSelect: doc.createElement("select"),
      loadExampleButton: doc.createElement("button"),
      status: doc.createElement("p"),
      canvas: doc.createElement("div"),
    },
  };
}

function findByUiId(root, id) {
  if (root.getAttribute?.("data-ui-id") === id) return root;
  for (const child of root.children ?? []) {
    const found = findByUiId(child, id);
    if (found) return found;
  }
  return null;
}

function collectTexts(root, out = []) {
  if (root.tagName === "P") out.push(root.textContent);
  for (const child of root.children ?? []) {
    collectTexts(child, out);
  }
  return out;
}

function flushMicrotasks() {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

test("catálogo visual expõe 2 exemplos .pors prontos e populados no seletor", () => {
  assert.equal(PORS_UI_EXAMPLES.length, 2);
  assert.equal(findPorsUiExample("contador-visual")?.fileName, "contador-visual.fer");
  assert.equal(findPorsUiExample("tarefas-visual")?.fileName, "tarefas-visual.fer");

  const { elements } = createPanelDom();
  const controller = mountPorsUiPreviewPanel({
    elements,
    backend: {
      startUi: () => ({ ok: false, error: "noop" }),
      dispatchUi: () => ({ ok: false, error: "noop" }),
      closeUi: () => true,
    },
    mount: mountUiPreview,
    examples: PORS_UI_EXAMPLES,
    getActiveSource: () => null,
  });

  assert.equal(elements.exampleSelect.children.length, 2);
  assert.equal(elements.exampleSelect.value, "contador-visual");
  return controller.destroy();
});

test("executa, interage, reinicia, cancela e fecha sessão MVU via backend assíncrono", async () => {
  const { elements } = createPanelDom();
  const sessions = new Map();
  const cancelled = [];
  let nextId = 1;
  let closedFocusCalled = false;

  const backend = {
    async startUi(source) {
      if (source.includes("LIMIT")) {
        return { ok: false, code: "time", error: "Limite de passos excedido." };
      }
      const id = nextId++;
      const state = { count: 0, step: 0 };
      sessions.set(id, state);
      return {
        ok: true,
        session: id,
        step: 0,
        history_len: 1,
        stdout: "",
        tree: {
          kind: "coluna",
          children: [
            { kind: "texto", content: "Total: 0" },
            { kind: "botao", id: "inc", label: "+1" },
          ],
        },
      };
    },
    async dispatchUi(sessionId, event) {
      const state = sessions.get(sessionId);
      if (!state) return { ok: false, code: "session_not_found", error: "Sessão ausente" };
      if (event.kind === "clique" && event.id === "inc") {
        state.count += 1;
      }
      state.step += 1;
      return {
        ok: true,
        session: sessionId,
        step: state.step,
        history_len: state.step + 1,
        stdout: "",
        tree: {
          kind: "coluna",
          children: [
            { kind: "texto", content: `Total: ${state.count}` },
            { kind: "botao", id: "inc", label: "+1" },
          ],
        },
      };
    },
    async cancelUi(sessionId) {
      cancelled.push(sessionId);
      return sessions.delete(sessionId);
    },
    async closeUi(sessionId) {
      return sessions.delete(sessionId);
    },
  };

  let currentDoc = { source: "app-pors", isPors: true };
  const loadedExamples = [];
  const controller = mountPorsUiPreviewPanel({
    elements,
    backend,
    mount: mountUiPreview,
    examples: PORS_UI_EXAMPLES,
    getActiveSource: () => currentDoc,
    onLoadExample: (example) => loadedExamples.push(example.id),
    onCloseFocus: () => {
      closedFocusCalled = true;
    },
  });

  await controller.open();
  assert.equal(controller.isOpen, true);
  assert.equal(controller.active, true);
  assert.equal(controller.sessionId, 1);
  assert.deepEqual(collectTexts(elements.canvas), ["Total: 0"]);
  assert.equal(elements.status.textContent, "Preview ativo — passo 0.");
  assert.equal(elements.status.getAttribute("data-error"), "false");

  // Clique no botão incrementa contador e atualiza DOM + status.
  findByUiId(elements.canvas, "inc").trigger("click");
  await flushMicrotasks();
  assert.equal(controller.step, 1);
  assert.deepEqual(collectTexts(elements.canvas), ["Total: 1"]);
  assert.equal(elements.status.textContent, "Preview ativo — passo 1.");

  // Reiniciar fecha a sessão anterior e reinicia no passo 0.
  await controller.restart();
  assert.equal(controller.sessionId, 2);
  assert.equal(controller.step, 0);
  assert.deepEqual(collectTexts(elements.canvas), ["Total: 0"]);

  // Cancelar chama cancelUi e desabilita controles.
  await controller.cancel();
  assert.equal(controller.active, false);
  assert.equal(cancelled.includes(2), true);
  assert.equal(findByUiId(elements.canvas, "inc").disabled, true);
  assert.equal(elements.status.textContent, "Preview cancelado.");

  // Erro de orçamento exibe mensagem explícita com data-error="true".
  currentDoc = { source: "LIMIT", isPors: true };
  await controller.runCurrentSource();
  assert.equal(elements.status.getAttribute("data-error"), "true");
  assert.match(elements.status.textContent, /Orçamento de passos excedido/);

  // Carregar exemplo abre o exemplo escolhido e inicia imediatamente.
  elements.exampleSelect.value = "tarefas-visual";
  elements.loadExampleButton.trigger("click");
  await flushMicrotasks();
  assert.deepEqual(loadedExamples, ["tarefas-visual"]);
  assert.equal(controller.active, true);

  // Fechar oculta o painel e devolve foco.
  await controller.close();
  assert.equal(controller.isOpen, false);
  assert.equal(closedFocusCalled, true);

  await controller.destroy();
});

test("descarta resposta assíncrona atrasada após cancelamento e fecha sessão obsoleta", async () => {
  const { elements } = createPanelDom();
  let resolveStart;
  const closed = [];

  const backend = {
    startUi() {
      return new Promise((resolve) => {
        resolveStart = resolve;
      });
    },
    dispatchUi() {
      return { ok: false, error: "noop" };
    },
    closeUi(sessionId) {
      closed.push(sessionId);
      return true;
    },
  };

  const controller = mountPorsUiPreviewPanel({
    elements,
    backend,
    mount: mountUiPreview,
    examples: PORS_UI_EXAMPLES,
    getActiveSource: () => ({ source: "app-lento", isPors: true }),
  });

  const pendingOpen = controller.open();
  await controller.cancel();
  resolveStart({
    ok: true,
    session: 77,
    step: 0,
    history_len: 1,
    stdout: "",
    tree: { kind: "texto", content: "Tarde demais" },
  });
  await pendingOpen;

  assert.equal(controller.active, false);
  assert.deepEqual(closed, [77]);
  assert.equal(elements.canvas.children.length, 0);
  await controller.destroy();
});

const realWasmPath = process.env.PORTUGOL_WASM_PATH;

test(
  "executa os 2 exemplos visuais oficiais no painel conectado ao ferro_wasm.wasm real",
  { skip: !realWasmPath },
  async () => {
    const runtime = await loadPorsWasmRuntime(readFileSync(realWasmPath));
    const backend = createWasmUiBackend(runtime);
    const { elements } = createPanelDom();

    const controller = mountPorsUiPreviewPanel({
      elements,
      backend,
      mount: mountUiPreview,
      examples: PORS_UI_EXAMPLES,
      getActiveSource: () => ({ source: PORS_UI_EXAMPLES[0].source, isPors: true }),
    });

    // Exemplo 1: Contador interativo
    await controller.open();
    assert.deepEqual(collectTexts(elements.canvas), [
      "Olá, Visitante!",
      "Contagem: 0 (Neutro)",
    ]);
    findByUiId(elements.canvas, "inc").trigger("click");
    await flushMicrotasks();
    assert.deepEqual(collectTexts(elements.canvas), [
      "Olá, Visitante!",
      "Contagem: 1 (Animado)",
    ]);

    // Exemplo 2: Lista de tarefas
    elements.exampleSelect.value = "tarefas-visual";
    elements.loadExampleButton.trigger("click");
    await flushMicrotasks();
    const draft = findByUiId(elements.canvas, "rascunho");
    draft.value = "Publicar PR";
    draft.trigger("input");
    await flushMicrotasks();
    findByUiId(elements.canvas, "salvar").trigger("click");
    await flushMicrotasks();
    assert.equal(collectTexts(elements.canvas).includes("[ ] Publicar PR"), true);

    await controller.destroy();
    assert.deepEqual(runtime.activeUiSessionIds, []);
  },
);
