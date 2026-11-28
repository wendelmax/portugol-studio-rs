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
    input: "Ada\n17\n",
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
