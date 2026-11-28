export const PLAYGROUND_EXAMPLES = [
  {
    id: "ola",
    title: "Boas-vindas (.fer)",
    file: "boas-vindas.fer",
    group: "Básico",
    mode: "console",
    input: "",
    source: "funcao inicio() {\n    seja nome: cadeia = \"Ferro\";\n    seja mutavel ano: inteiro = 2026;\n    ano += 1;\n    escreva(\"Olá, \", nome, \"! Próximo ano: \", ano);\n}\n",
  },
  {
    id: "entrada",
    title: "Entrada tipada",
    file: "entrada.fer",
    group: "Básico",
    mode: "console",
    input: "Ada\\n17\\n",
    source: "funcao inicio() {\n    seja mutavel nome = leia();\n    seja mutavel idade: inteiro = leia();\n    escreva(\"Olá, \", nome, \"! Próxima idade: \", idade + 1);\n}\n",
  },
  {
    id: "tipos",
    title: "Matemática, texto e tipos",
    file: "tipos.fer",
    group: "Básico",
    mode: "console",
    input: "",
    source: "funcao somar_textos(a: cadeia, b: cadeia) -> Resultado<inteiro, cadeia> {\n    seja x: inteiro = Tipos.para_inteiro(Texto.aparar(a))?;\n    seja y: inteiro = Tipos.para_inteiro_base(b, 16)?;\n    retorne Resultado::Ok(x + y);\n}\n\nfuncao inicio() {\n    seja pi2: real = Matematica.arredondar(Matematica.PI, 2);\n    escolha somar_textos(\" 26 \", \"0x10\") {\n        caso Resultado::Ok(total) => {\n            seja codigo: cadeia = Texto.preencher_a_esquerda('0', 4, Tipos.inteiro_para_cadeia(total));\n            escreva(\"PI=\", pi2, \" total=\", codigo);\n        }\n        caso Resultado::Erro(e) => {\n            escreva(\"erro=\", e);\n        }\n    }\n}\n",
  },
  {
    id: "decisao",
    title: "Decisões encadeadas",
    file: "decisao.fer",
    group: "Controle",
    mode: "console",
    input: "",
    source: "funcao inicio() {\n    seja nota: real = 8.5;\n    seja frequencia: inteiro = 80;\n    se (nota >= 9.0 && frequencia >= 75) {\n        escreva(\"Aprovado com excelência\");\n    } senao se (nota >= 7.0 && frequencia >= 75) {\n        escreva(\"Aprovado\");\n    } senao se (frequencia >= 75) {\n        escreva(\"Em recuperação\");\n    } senao {\n        escreva(\"Reprovado\");\n    }\n}\n",
  },
  {
    id: "lacos",
    title: "Laços e controle",
    file: "lacos.fer",
    group: "Controle",
    mode: "console",
    input: "",
    source: "funcao inicio() {\n    seja mutavel soma = 0;\n    para (seja mutavel i: inteiro = 1; i <= 10; i += 1) {\n        se (i == 5) {\n            continue;\n        }\n        se (i == 8) {\n            pare;\n        }\n        soma += i;\n    }\n    escreva(\"Soma=\", soma);\n}\n",
  },
  {
    id: "funcoes",
    title: "Funções e recursão",
    file: "funcoes.fer",
    group: "Controle",
    mode: "console",
    input: "",
    source: "funcao fatorial(n: inteiro) -> inteiro {\n    se (n <= 1) {\n        retorne 1;\n    } senao {\n        retorne n * fatorial(n - 1);\n    }\n}\n\nfuncao inicio() {\n    escreva(\"5! = \", fatorial(5));\n}\n",
  },
  {
    id: "vetores",
    title: "Vetores",
    file: "vetores.fer",
    group: "Dados",
    mode: "console",
    input: "",
    source: "funcao somar_vetor(itens: vetor<inteiro>) -> inteiro {\n    seja mutavel total = 0;\n    para (seja mutavel i = 0; i < tamanho(itens); i += 1) {\n        total += itens[i];\n    }\n    retorne total;\n}\n\nfuncao inicio() {\n    seja mutavel notas: vetor<inteiro> = [7, 8, 9];\n    notas[2] = 10;\n    escreva(\"Total=\", somar_vetor(notas), \" Qtd=\", tamanho(notas));\n}\n",
  },
  {
    id: "registro",
    title: "Registros e enums",
    file: "registro.fer",
    group: "Dados",
    mode: "console",
    input: "",
    source: "enum Situacao {\n    Ativo,\n    Concluido(inteiro)\n}\n\nregistro Tarefa {\n    titulo: cadeia,\n    situacao: Situacao\n}\n\nfuncao inicio() {\n    seja tarefa: Tarefa = Tarefa {\n        titulo: \"Estudar Ferro\",\n        situacao: Situacao::Concluido(100)\n    };\n    escolha tarefa.situacao {\n        caso Situacao::Ativo => {\n            escreva(tarefa.titulo, \": em andamento\");\n        }\n        caso Situacao::Concluido(pontos) => {\n            escreva(tarefa.titulo, \": concluída (\", pontos, \"%)\");\n        }\n    }\n}\n",
  },
  {
    id: "valores",
    title: "Opção e Resultado",
    file: "valores.fer",
    group: "Valores",
    mode: "console",
    input: "",
    source: "funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {\n    se (b == 0) {\n        retorne Resultado::Erro(\"divisão por zero\");\n    }\n    retorne Resultado::Ok(a / b);\n}\n\nfuncao dobro_da_divisao(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {\n    seja quociente: inteiro = dividir(a, b)?;\n    retorne Resultado::Ok(quociente * 2);\n}\n\nfuncao inicio() {\n    escolha dobro_da_divisao(20, 4) {\n        caso Resultado::Ok(v) => {\n            escreva(\"ok=\", v);\n        }\n        caso Resultado::Erro(e) => {\n            escreva(\"erro=\", e);\n        }\n    }\n    escreva(\" | \");\n    escolha dobro_da_divisao(20, 0) {\n        caso Resultado::Ok(v) => {\n            escreva(\"ok=\", v);\n        }\n        caso Resultado::Erro(e) => {\n            escreva(\"erro=\", e);\n        }\n    }\n}\n",
  },
  {
    id: "bibliotecas",
    title: "Util e Calendário",
    file: "bibliotecas.fer",
    group: "Valores",
    mode: "console",
    input: "",
    source: "funcao inicio() {\n    Util.definir_semente(42);\n    seja ano: inteiro = 2024;\n    seja bissexto: logico = Calendario.eh_bissexto(ano);\n    escolha Calendario.dias_no_mes(ano, Calendario.MES_FEVEREIRO) {\n        caso Resultado::Ok(dias) => {\n            escolha Calendario.nome_mes(Calendario.MES_FEVEREIRO) {\n                caso Resultado::Ok(mes) => {\n                    escolha Util.sorteia(1, 6) {\n                        caso Resultado::Ok(dado) => {\n                            escreva(mes, \"/\", ano, \"=\", dias, \" bissexto=\", bissexto, \" dado_valido=\", dado >= 1 && dado <= 6);\n                        }\n                        caso Resultado::Erro(e) => {\n                            escreva(e);\n                        }\n                    }\n                }\n                caso Resultado::Erro(e) => {\n                    escreva(e);\n                }\n            }\n        }\n        caso Resultado::Erro(e) => {\n            escreva(e);\n        }\n    }\n}\n",
  },
  {
    id: "ui-contador",
    title: "Contador visual MVU",
    file: "ui-contador.fer",
    group: "Visual",
    mode: "ui",
    input: "",
    source: "registro Estado {\n    contagem: inteiro\n}\n\nfuncao iniciar() -> Estado {\n    retorne Estado { contagem: 0 };\n}\n\nfuncao atualizar(estado: Estado, alvo: cadeia) -> Estado {\n    se (alvo == \"inc\") {\n        retorne Estado { contagem: estado.contagem + 1 };\n    }\n    retorne estado;\n}\n\nfuncao visualizar(estado: Estado) -> inteiro {\n    retorne UI.coluna([\n        UI.texto(\"Contagem: \", estado.contagem),\n        UI.botao(\"inc\", \"+1\")\n    ]);\n}\n",
  },
  {
    id: "exercicio",
    title: "Exercício aberto",
    file: "exercicio.fer",
    group: "Exercício",
    mode: "console",
    input: "",
    source: "// Escreva seu programa Ferro aqui.\nfuncao inicio() {\n    escreval(\"Olá, Ferro!\");\n}\n",
  },
];

const REQUIRED_EXPORTS = [
  "memory",
  "ferro_alloc",
  "ferro_dealloc",
  "ferro_result_ptr",
  "ferro_result_len",
  "ferro_validate",
  "ferro_format",
  "ferro_run",
  "ferro_ui_start",
  "ferro_ui_dispatch",
  "ferro_ui_snapshot",
  "ferro_ui_close",
];

function sanitizeBudget(value) {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) return 0;
  return Math.floor(value);
}

/** Cria a fachada de execução a partir dos exports do módulo `ferro_wasm.wasm`. */
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
    const ptr = rawExports.ferro_alloc(len);
    if (ptr === 0) throw new Error("Falha ao alocar memória linear no módulo WASM.");
    try {
      new Uint8Array(rawExports.memory.buffer, ptr, len).set(encoded);
      return fn(ptr, len);
    } finally {
      rawExports.ferro_dealloc(ptr, len);
    }
  }

  function readJson(expectedLen) {
    const ptr = rawExports.ferro_result_ptr();
    const actualLen = rawExports.ferro_result_len();
    const len = actualLen === expectedLen ? actualLen : expectedLen;
    const raw = decoder.decode(new Uint8Array(rawExports.memory.buffer, ptr, len));
    return JSON.parse(raw);
  }

  function closeUi(sessionId) {
    const closed = rawExports.ferro_ui_close(sessionId) === 1;
    openSessions.delete(sessionId);
    return closed;
  }

  return {
    validate(source) {
      return withUtf8(source, (ptr, len) => readJson(rawExports.ferro_validate(ptr, len)));
    },
    format(source) {
      return withUtf8(source, (ptr, len) => readJson(rawExports.ferro_format(ptr, len)));
    },
    exportRust(source) {
      if (typeof rawExports.ferro_export_rust !== "function") {
        return {
          ok: false,
          error: "A exportação para Rust não está disponível nesta versão do módulo WASM.",
          diagnostics: [],
        };
      }
      return withUtf8(source, (ptr, len) => readJson(rawExports.ferro_export_rust(ptr, len)));
    },
    run(source, input = "", budget = {}) {
      return withUtf8(source, (srcPtr, srcLen) =>
        withUtf8(input, (inPtr, inLen) =>
          readJson(
            rawExports.ferro_run(
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
          rawExports.ferro_ui_start(
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
            readJson(rawExports.ferro_ui_dispatch(sessionId, kPtr, kLen, tPtr, tLen, vPtr, vLen)),
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

/** Carrega e instancia `ferro_wasm.wasm` via streaming ou ArrayBuffer. */
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
