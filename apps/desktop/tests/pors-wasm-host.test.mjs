import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import {
  createPorsUiSessionHost,
  createPorsWasmRuntime,
  loadPorsWasmRuntime,
} from "../src/pors-wasm-host.ts";
import { mountUiPreview } from "../src/ui-preview-renderer.ts";

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

function findByUiId(root, id) {
  if (root.getAttribute?.("data-ui-id") === id) return root;
  for (const child of root.children ?? []) {
    const found = findByUiId(child, id);
    if (found) return found;
  }
  return null;
}

function findFirstByTag(root, tag) {
  if (root.tagName === tag.toUpperCase()) return root;
  for (const child of root.children ?? []) {
    const found = findFirstByTag(child, tag);
    if (found) return found;
  }
  return null;
}

function createFakeWasmExports() {
  const memory = new WebAssembly.Memory({ initial: 2 });
  const encoder = new TextEncoder();
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const liveAllocs = new Map();
  const sessions = new Map();
  const calls = [];
  let nextAllocPtr = 1024;
  let resultPtr = 32768;
  let resultLen = 0;
  let nextSessionId = 1;

  function readUtf8(ptr, len) {
    if (len === 0) return "";
    return decoder.decode(new Uint8Array(memory.buffer, ptr, len));
  }

  function writeResult(payload) {
    const bytes = encoder.encode(JSON.stringify(payload));
    new Uint8Array(memory.buffer, resultPtr, bytes.length).set(bytes);
    resultLen = bytes.length;
    return bytes.length;
  }

  function buildTree(state) {
    return {
      kind: "coluna",
      children: [
        { kind: "texto", content: `Olá ${state.name}: ${state.count}` },
        { kind: "botao", id: "inc", label: "+1" },
        { kind: "entrada", id: "nome", value: state.name, placeholder: "Seu nome" },
      ],
    };
  }

  const exportsObj = {
    memory,
    pors_alloc(len) {
      const ptr = nextAllocPtr;
      nextAllocPtr += Math.max(len, 1);
      liveAllocs.set(ptr, len);
      return ptr;
    },
    pors_dealloc(ptr, len) {
      assert.equal(liveAllocs.get(ptr), len, `dealloc inconsistente para ptr=${ptr}`);
      liveAllocs.delete(ptr);
    },
    pors_result_ptr() {
      return resultPtr;
    },
    pors_result_len() {
      return resultLen;
    },
    pors_validate(srcPtr, srcLen) {
      const source = readUtf8(srcPtr, srcLen);
      calls.push({ op: "validate", source });
      if (source.includes("erro")) {
        return writeResult({
          ok: false,
          diagnostics: [{ code: "pors.syntax", message: "Erro sintático", start: 0, end: 4 }],
        });
      }
      return writeResult({ ok: true, diagnostics: [] });
    },
    pors_format(srcPtr, srcLen) {
      const source = readUtf8(srcPtr, srcLen);
      calls.push({ op: "format", source });
      return writeResult({ ok: true, changed: true, formatted: `${source.trim()}\n` });
    },
    pors_run(srcPtr, srcLen, inPtr, inLen, maxSteps, maxStringBytes, maxOutputBytes) {
      const source = readUtf8(srcPtr, srcLen);
      const input = readUtf8(inPtr, inLen);
      calls.push({ op: "run", source, input, maxSteps, maxStringBytes, maxOutputBytes });
      return writeResult({ ok: true, stdout: `out:${input.trim()}`, diagnostics: [] });
    },
    pors_ui_start(srcPtr, srcLen, maxSteps, maxNodes, maxDepth) {
      const source = readUtf8(srcPtr, srcLen);
      calls.push({ op: "ui_start", source, maxSteps, maxNodes, maxDepth });
      if (source.includes("BROKEN")) {
        return writeResult({
          ok: false,
          code: "missing_function",
          error: 'O programa visual precisa definir a função "iniciar".',
        });
      }
      const id = nextSessionId++;
      const state = { count: 0, name: "", step: 0 };
      sessions.set(id, state);
      return writeResult({
        ok: true,
        session: id,
        step: 0,
        history_len: 1,
        tree: buildTree(state),
        stdout: "",
      });
    },
    pors_ui_dispatch(sessionId, kindPtr, kindLen, targetPtr, targetLen, valuePtr, valueLen) {
      const kind = readUtf8(kindPtr, kindLen);
      const target = readUtf8(targetPtr, targetLen);
      const value = readUtf8(valuePtr, valueLen);
      calls.push({ op: "ui_dispatch", sessionId, kind, target, value });
      const state = sessions.get(sessionId);
      if (!state) {
        return writeResult({
          ok: false,
          code: "session_not_found",
          error: `Sessão visual ${sessionId} não encontrada.`,
        });
      }
      if (value === "BOOM") {
        return writeResult({
          ok: false,
          code: "time",
          error: "Limite de passos do evento excedido.",
          stdout: "",
        });
      }
      if (kind === "clique" && target === "inc") {
        state.count += 1;
      } else if (kind === "entrada_alterada" && target === "nome") {
        state.name = value;
      }
      state.step += 1;
      return writeResult({
        ok: true,
        session: sessionId,
        step: state.step,
        history_len: state.step + 1,
        tree: buildTree(state),
        stdout: "",
      });
    },
    pors_ui_snapshot(sessionId) {
      const state = sessions.get(sessionId);
      if (!state) {
        return writeResult({
          ok: false,
          code: "session_not_found",
          error: `Sessão visual ${sessionId} não encontrada.`,
        });
      }
      return writeResult({
        ok: true,
        session: sessionId,
        step: state.step,
        history_len: state.step + 1,
        tree: buildTree(state),
        stdout: "",
      });
    },
    pors_ui_close(sessionId) {
      calls.push({ op: "ui_close", sessionId });
      return sessions.delete(sessionId) ? 1 : 0;
    },
  };

  return { exportsObj, liveAllocs, calls, sessions };
}

test("rejeita módulo WASM sem exports obrigatórios", () => {
  assert.throws(() => createPorsWasmRuntime({}), /memory/);
  const memory = new WebAssembly.Memory({ initial: 1 });
  assert.throws(() => createPorsWasmRuntime({ memory }), /pors_alloc/);
});

test("encapsula validate, format, run e sessões UI sem vazar memória linear", () => {
  const { exportsObj, liveAllocs, calls } = createFakeWasmExports();
  const runtime = createPorsWasmRuntime(exportsObj);

  assert.deepEqual(runtime.validate("funcao inicio() { escreva(\"ação 🦀\"); }"), {
    ok: true,
    diagnostics: [],
  });
  assert.equal(runtime.validate("erro").ok, false);

  const formatted = runtime.format("funcao inicio(){}");
  assert.equal(formatted.ok, true);
  assert.equal(formatted.changed, true);

  const run = runtime.run("funcao inicio() {}", "Ada 🦀\n", {
    maxSteps: 1500.9,
    maxStringBytes: -5,
    maxOutputBytes: Number.NaN,
  });
  assert.equal(run.ok, true);
  assert.equal(run.stdout, "out:Ada 🦀");
  const runCall = calls.find((entry) => entry.op === "run");
  assert.equal(runCall.maxSteps, 1500);
  assert.equal(runCall.maxStringBytes, 0);
  assert.equal(runCall.maxOutputBytes, 0);

  const s1 = runtime.startUi("app-1");
  const s2 = runtime.startUi("app-2");
  assert.equal(s1.ok, true);
  assert.equal(s2.ok, true);
  assert.deepEqual(runtime.activeUiSessionIds, [1, 2]);
  assert.equal(runtime.closeAllUiSessions(), 2);
  assert.deepEqual(runtime.activeUiSessionIds, []);
  assert.equal(liveAllocs.size, 0, "toda alocação UTF-8 deve ser liberada");
});

test("integra PorsWasmRuntime com mountUiPreview preservando foco, reinício, cancelamento e erros", () => {
  const { exportsObj, liveAllocs, sessions } = createFakeWasmExports();
  const runtime = createPorsWasmRuntime(exportsObj);
  const { doc, container } = createFakeDom();
  const snapshots = [];
  const errors = [];

  const host = createPorsUiSessionHost({
    runtime,
    mount: mountUiPreview,
    container,
    onSnapshot: (snap) => snapshots.push(snap.step),
    onError: (err) => errors.push(err),
  });

  const started = host.start("app-valido");
  assert.equal(started.ok, true);
  assert.equal(host.active, true);
  assert.equal(host.sessionId, 1);
  assert.equal(findFirstByTag(container, "p").textContent, "Olá : 0");

  // Clique no botão incrementa estado e re-renderiza DOM.
  findByUiId(container, "inc").trigger("click");
  assert.equal(host.step, 1);
  assert.equal(findFirstByTag(container, "p").textContent, "Olá : 1");

  // Digitação em entrada preserva foco e posição do cursor após re-renderização.
  const input = findByUiId(container, "nome");
  input.focus();
  input.value = "Ada";
  input.setSelectionRange(2, 2);
  input.trigger("input");
  assert.equal(host.step, 2);
  assert.equal(findFirstByTag(container, "p").textContent, "Olá Ada: 1");
  const updatedInput = findByUiId(container, "nome");
  assert.equal(doc.activeElement, updatedInput);
  assert.equal(updatedInput.selectionStart, 2);
  assert.equal(updatedInput.selectionEnd, 2);

  // Reinício fecha a sessão anterior e volta para o estado inicial.
  const restarted = host.restart();
  assert.equal(restarted?.ok, true);
  assert.equal(host.sessionId, 2);
  assert.equal(host.step, 0);
  assert.equal(sessions.has(1), false);
  assert.equal(findFirstByTag(container, "p").textContent, "Olá : 0");

  // Erro de orçamento desabilita controles e notifica onError sem quebrar host.
  const inputAfterRestart = findByUiId(container, "nome");
  inputAfterRestart.value = "BOOM";
  inputAfterRestart.trigger("input");
  assert.equal(host.active, false);
  assert.equal(errors.length, 1);
  assert.equal(errors[0].code, "time");
  assert.equal(findByUiId(container, "inc").disabled, true);

  // Cancelamento libera a sessão WASM e ignora eventos subsequentes.
  host.start("app-valido");
  assert.equal(host.active, true);
  const activeId = host.sessionId;
  host.cancel();
  assert.equal(host.active, false);
  assert.equal(sessions.has(activeId), false);
  assert.equal(findByUiId(container, "inc").disabled, true);
  assert.equal(host.dispatch({ kind: "clique", id: "inc" }), null);

  // Erro de inicialização mantém host desabilitado e informa diagnóstico.
  const broken = host.start("BROKEN");
  assert.equal(broken.ok, false);
  assert.equal(errors.at(-1).code, "missing_function");

  // Destroy limpa DOM e encerra qualquer sessão aberta.
  host.start("app-valido");
  const finalSession = host.sessionId;
  host.destroy();
  assert.equal(sessions.has(finalSession), false);
  assert.equal(container.children.length, 0);
  assert.equal(liveAllocs.size, 0);
  assert.deepEqual(snapshots.slice(0, 4), [0, 1, 2, 0]);
});

test("falha de contrato no renderizador fecha a sessão WASM e reporta render_contract_error", () => {
  const { exportsObj, sessions } = createFakeWasmExports();
  const runtime = createPorsWasmRuntime(exportsObj);
  const { container } = createFakeDom();
  const errors = [];

  const host = createPorsUiSessionHost({
    runtime,
    mount: mountUiPreview,
    container,
    limits: { maxNodes: 2 },
    onError: (err) => errors.push(err),
  });

  const result = host.start("app-valido");
  assert.equal(result.ok, false);
  assert.equal(result.code, "render_contract_error");
  assert.equal(host.active, false);
  assert.equal(sessions.size, 0, "sessão WASM deve ser fechada quando o renderizador rejeitar a árvore");
  assert.equal(errors.length, 1);
});

const realWasmPath = process.env.PORTUGOL_WASM_PATH;

test(
  "executa um programa .pors visual MVU real dentro de portugol_wasm.wasm conectado ao mountUiPreview",
  { skip: !realWasmPath },
  async () => {
    const bytes = readFileSync(realWasmPath);
    const runtime = await loadPorsWasmRuntime(bytes);
    const { container } = createFakeDom();
    const host = createPorsUiSessionHost({
      runtime,
      mount: mountUiPreview,
      container,
    });

    const source = `
enum Status { Inicial, Ativo }

registro Estado {
    contagem: inteiro,
    nome: cadeia,
    status: Status
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "", status: Status::Inicial };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome, status: Status::Ativo };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor, status: Status::Ativo };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    seja mutavel selo: cadeia = "inicial";
    escolha estado.status {
        caso Status::Inicial => { selo = "inicial"; }
        caso Status::Ativo => { selo = "ativo"; }
    }
    retorne UI.coluna([
        UI.texto("Olá ", estado.nome, " [", selo, "] = ", estado.contagem),
        UI.linha([
            UI.botao("inc", "+1"),
            UI.entrada("nome", estado.nome, "Seu nome")
        ]),
        UI.lista([UI.texto("Item ", estado.contagem)])
    ]);
}
`;

    const started = host.start(source);
    assert.equal(started.ok, true);
    assert.equal(findFirstByTag(container, "p").textContent, "Olá  [inicial] = 0");

    findByUiId(container, "inc").trigger("click");
    assert.equal(findFirstByTag(container, "p").textContent, "Olá  [ativo] = 1");

    const input = findByUiId(container, "nome");
    input.value = "Ada";
    input.trigger("input");
    assert.equal(findFirstByTag(container, "p").textContent, "Olá Ada [ativo] = 1");
    assert.equal(findByUiId(container, "nome").value, "Ada");

    host.destroy();
    assert.deepEqual(runtime.activeUiSessionIds, []);
  },
);
