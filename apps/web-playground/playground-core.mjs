// Núcleo sem dependências externas do Web Playground Portugol-RS (WASM).
// Reutiliza o mesmo protocolo de memória linear de `crates/portugol-wasm`
// e renderiza nós `UiNode` (ADR 0017) exclusivamente com createElement/textContent.

export const PLAYGROUND_EXAMPLES = [
  {
    id: "ola",
    title: "Olá, Portugol-RS (console + entrada)",
    mode: "console",
    input: "Ada\n",
    source: `funcao saudar(nome: cadeia) -> cadeia {
    retorne "Olá, " + nome + "! Bem-vindo ao Portugol-RS no navegador.";
}

funcao inicio() {
    seja mutavel nome: cadeia = "";
    leia(nome);
    escreva(saudar(nome), "\\n");
}
`,
  },
  {
    id: "resultado",
    title: "Erros seguros com Resultado e ?",
    mode: "console",
    input: "",
    source: `funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("divisão por zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao calcular(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    seja quociente: inteiro = dividir(a, b)?;
    retorne Resultado::Ok(quociente * 10);
}

funcao inicio() {
    escolha calcular(20, 4) {
        caso Resultado::Ok(v) => {
            escreva("20 / 4 * 10 = ", v, "\\n");
        }
        caso Resultado::Erro(e) => {
            escreva("Erro: ", e, "\\n");
        }
    }
    escolha calcular(20, 0) {
        caso Resultado::Ok(v) => {
            escreva("resultado = ", v, "\\n");
        }
        caso Resultado::Erro(e) => {
            escreva("20 / 0 -> Erro: ", e, "\\n");
        }
    }
}
`,
  },
  {
    id: "vetores",
    title: "Vetores dinâmicos e laço para",
    mode: "console",
    input: "",
    source: `funcao inicio() {
    seja mutavel notas: vetor<inteiro> = [7, 8, 10];
    adicionar(notas, 9);
    seja mutavel soma: inteiro = 0;
    para (seja mutavel i: inteiro = 0; i < tamanho(notas); i += 1) {
        soma += notas[i];
    }
    escreva("Quantidade: ", tamanho(notas), "\\n");
    escreva("Soma: ", soma, "\\n");
    escreva("Média inteira: ", soma / tamanho(notas), "\\n");
}
`,
  },
  {
    id: "ui-contador",
    title: "Contador Visual MVU (UI.*)",
    mode: "ui",
    input: "",
    source: `registro Estado {
    contagem: inteiro,
    nome: cadeia
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "Estudante" };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome };
    }
    se (tipo == "clique" && alvo == "dec") {
        retorne Estado { contagem: estado.contagem - 1, nome: estado.nome };
    }
    se (tipo == "clique" && alvo == "zerar") {
        retorne Estado { contagem: 0, nome: estado.nome };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    retorne UI.coluna([
        UI.texto("Olá, ", estado.nome, "!"),
        UI.texto("Contagem atual: ", estado.contagem),
        UI.linha([
            UI.botao("dec", "-1"),
            UI.botao("inc", "+1"),
            UI.botao("zerar", "Zerar")
        ]),
        UI.entrada("nome", estado.nome, "Digite seu nome")
    ]);
}
`,
  },
];

const REQUIRED_EXPORTS = [
  "memory",
  "pors_alloc",
  "pors_dealloc",
  "pors_result_ptr",
  "pors_result_len",
  "pors_validate",
  "pors_format",
  "pors_run",
  "pors_ui_start",
  "pors_ui_dispatch",
  "pors_ui_snapshot",
  "pors_ui_close",
];

function sanitizeBudget(value) {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) return 0;
  return Math.floor(value);
}

/** Cria a fachada de execução a partir dos exports do módulo `portugol_wasm.wasm`. */
export function createPlaygroundRuntime(rawExports) {
  for (const name of REQUIRED_EXPORTS) {
    if (name === "memory") {
      if (!(rawExports.memory instanceof WebAssembly.Memory)) {
        throw new Error('O módulo WASM não exporta uma memória linear válida ("memory").');
      }
    } else if (typeof rawExports[name] !== "function") {
      throw new Error(`O módulo WASM não exporta a função obrigatória "${name}".`);
    }
  }

  const encoder = new TextEncoder();
  const decoder = new TextDecoder("utf-8", { fatal: true });
  const openSessions = new Set();

  function withUtf8(text, fn) {
    const encoded = encoder.encode(text);
    const len = encoded.length;
    if (len === 0) return fn(0, 0);
    const ptr = rawExports.pors_alloc(len);
    if (ptr === 0) throw new Error("Falha ao alocar memória linear no módulo WASM.");
    try {
      new Uint8Array(rawExports.memory.buffer, ptr, len).set(encoded);
      return fn(ptr, len);
    } finally {
      rawExports.pors_dealloc(ptr, len);
    }
  }

  function readJson(expectedLen) {
    const ptr = rawExports.pors_result_ptr();
    const actualLen = rawExports.pors_result_len();
    const len = actualLen === expectedLen ? actualLen : expectedLen;
    const raw = decoder.decode(new Uint8Array(rawExports.memory.buffer, ptr, len));
    return JSON.parse(raw);
  }

  function closeUi(sessionId) {
    const closed = rawExports.pors_ui_close(sessionId) === 1;
    openSessions.delete(sessionId);
    return closed;
  }

  return {
    validate(source) {
      return withUtf8(source, (ptr, len) => readJson(rawExports.pors_validate(ptr, len)));
    },
    format(source) {
      return withUtf8(source, (ptr, len) => readJson(rawExports.pors_format(ptr, len)));
    },
    run(source, input = "", budget = {}) {
      return withUtf8(source, (srcPtr, srcLen) =>
        withUtf8(input, (inPtr, inLen) =>
          readJson(
            rawExports.pors_run(
              srcPtr,
              srcLen,
              inPtr,
              inLen,
              sanitizeBudget(budget.maxSteps),
              sanitizeBudget(budget.maxStringBytes),
              sanitizeBudget(budget.maxOutputBytes),
            ),
          ),
        ),
      );
    },
    startUi(source, budget = {}) {
      const result = withUtf8(source, (srcPtr, srcLen) =>
        readJson(
          rawExports.pors_ui_start(
            srcPtr,
            srcLen,
            sanitizeBudget(budget.maxSteps),
            sanitizeBudget(budget.maxNodes),
            sanitizeBudget(budget.maxDepth),
          ),
        ),
      );
      if (result.ok) openSessions.add(result.session);
      return result;
    },
    dispatchUi(sessionId, event) {
      const value = event.kind === "entrada_alterada" ? event.value : "";
      return withUtf8(event.kind, (kPtr, kLen) =>
        withUtf8(event.id, (tPtr, tLen) =>
          withUtf8(value, (vPtr, vLen) =>
            readJson(rawExports.pors_ui_dispatch(sessionId, kPtr, kLen, tPtr, tLen, vPtr, vLen)),
          ),
        ),
      );
    },
    closeUi,
    closeAllUiSessions() {
      for (const id of [...openSessions]) closeUi(id);
    },
  };
}

/** Carrega e instancia `portugol_wasm.wasm` via streaming ou ArrayBuffer. */
export async function loadPlaygroundRuntime(urlOrResponse) {
  const response = typeof urlOrResponse === "string" ? await fetch(urlOrResponse) : await urlOrResponse;
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} ao carregar módulo WASM.`);
  }
  if (typeof WebAssembly.instantiateStreaming === "function") {
    try {
      const streamed = await WebAssembly.instantiateStreaming(response.clone(), {});
      return createPlaygroundRuntime(streamed.instance.exports);
    } catch {
      // Fallback para servidores estáticos sem MIME application/wasm.
    }
  }
  const bytes = await response.arrayBuffer();
  const instantiated = await WebAssembly.instantiate(bytes, {});
  return createPlaygroundRuntime(instantiated.instance.exports);
}

/** Renderiza uma árvore `UiNode` usando exclusivamente DOM seguro (`createElement`/`textContent`). */
export function renderUiNodeTree(doc, container, node, onEvent) {
  const active = doc.activeElement;
  const focusedId = active?.dataset?.uiInputId ?? null;
  const selStart = typeof active?.selectionStart === "number" ? active.selectionStart : null;
  const selEnd = typeof active?.selectionEnd === "number" ? active.selectionEnd : null;

  function build(current, depth) {
    if (depth > 32) {
      const err = doc.createElement("p");
      err.className = "pg-ui-error";
      err.textContent = "Limite de profundidade da árvore visual excedido.";
      return err;
    }
    switch (current.kind) {
      case "coluna":
      case "linha": {
        const box = doc.createElement("div");
        box.className = `pg-ui-${current.kind}`;
        for (const child of current.children ?? []) {
          box.append(build(child, depth + 1));
        }
        return box;
      }
      case "texto": {
        const p = doc.createElement("p");
        p.className = "pg-ui-texto";
        p.textContent = current.content ?? "";
        return p;
      }
      case "botao": {
        const btn = doc.createElement("button");
        btn.type = "button";
        btn.className = "pg-ui-botao";
        btn.textContent = current.label ?? current.id;
        btn.addEventListener("click", () => onEvent({ kind: "clique", id: current.id }));
        return btn;
      }
      case "entrada": {
        const input = doc.createElement("input");
        input.type = "text";
        input.className = "pg-ui-entrada";
        input.dataset.uiInputId = current.id;
        input.value = current.value ?? "";
        input.placeholder = current.placeholder ?? "";
        input.setAttribute("aria-label", current.placeholder || current.id);
        input.addEventListener("input", () =>
          onEvent({ kind: "entrada_alterada", id: current.id, value: input.value }),
        );
        return input;
      }
      case "lista": {
        const ul = doc.createElement("ul");
        ul.className = "pg-ui-lista";
        for (const item of current.items ?? []) {
          const li = doc.createElement("li");
          li.append(build(item, depth + 1));
          ul.append(li);
        }
        return ul;
      }
      default: {
        const fallback = doc.createElement("span");
        fallback.textContent = `[nó desconhecido: ${String(current.kind)}]`;
        return fallback;
      }
    }
  }

  container.replaceChildren(build(node, 0));

  if (focusedId && typeof container.querySelector === "function") {
    const restored = container.querySelector(`[data-ui-input-id="${CSS.escape(focusedId)}"]`);
    if (restored && typeof restored.focus === "function") {
      restored.focus();
      if (selStart !== null && selEnd !== null && typeof restored.setSelectionRange === "function") {
        restored.setSelectionRange(selStart, selEnd);
      }
    }
  }
}
