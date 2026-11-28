// Trilha curta do playground. Cada lição roda no mesmo `pors_run`/`ferro_run`
// do WASM; a verificação compara apenas as linhas de stdout.

export const LESSONS = [
  {
    id: "variaveis",
    title: "Variáveis",
    prompt: "Imprima `Olá, Ada`.",
    hint: "Troque `???` por `Ada`.",
    starter: `funcao inicio() {
    seja nome: cadeia = "???";
    escreval("Olá, ", nome, "!");
}
`,
    stdin: "",
    expected: ["Olá, Ada"],
  },
  {
    id: "decisoes",
    title: "Decisões",
    prompt: "Com nota 6, imprima `Aprovado`.",
    hint: "Ajuste a condição para 6.",
    starter: `funcao inicio() {
    seja nota: inteiro = 6;
    se (nota >= 7) {
        escreval("Aprovado");
    } senao {
        escreval("Recuperação");
    }
}
`,
    stdin: "",
    expected: ["Aprovado"],
  },
  {
    id: "lacos",
    title: "Laços",
    prompt: "Some de 1 até 4.",
    hint: "Inclua o 4 no laço.",
    starter: `funcao inicio() {
    seja mutavel total = 0;
    para (seja mutavel i: inteiro = 1; i <= 3; i += 1) {
        total += 1;
    }
    escreval(total);
}
`,
    stdin: "",
    expected: ["4"],
  },
  {
    id: "vetores",
    title: "Vetores",
    prompt: "Imprima o segundo elemento.",
    hint: "Use o índice 1.",
    starter: `funcao inicio() {
    seja notas: vetor<inteiro> = [7, 8, 9];
    escreval(notas[0]);
}
`,
    stdin: "",
    expected: ["8"],
  },
  {
    id: "cadeias",
    title: "Cadeias",
    prompt: "Imprima `FERRO`.",
    hint: "Use `Texto.caixa_alta`.",
    starter: `funcao inicio() {
    seja texto: cadeia = "ferro";
    escreval(texto);
}
`,
    stdin: "",
    expected: ["FERRO"],
  },
  {
    id: "resultado",
    title: "Resultado",
    prompt: "Com `dividir(10, 2)`, imprima `5`.",
    hint: "Só retorne erro quando `b == 0`.",
    starter: `funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    retorne Resultado::Erro("zero");
    se (b == 0) {
        retorne Resultado::Erro("zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao inicio() {
    escolha dividir(10, 2) {
        caso Resultado::Ok(v) => {
            escreval(v);
        }
        caso Resultado::Erro(e) => {
            escreval(e);
        }
    }
}
`,
    stdin: "",
    expected: ["5"],
  },
];

// Usadas apenas por scripts/check-web-lessons.mjs; a interface não mostra a solução.
export const LESSON_SOLUTIONS = {
  variaveis: `funcao inicio() {
    seja nome: cadeia = "Ada";
    escreval("Olá, ", nome, "!");
}
`,
  decisoes: `funcao inicio() {
    seja nota: inteiro = 6;
    se (nota >= 6) {
        escreval("Aprovado");
    } senao {
        escreval("Recuperação");
    }
}
`,
  lacos: `funcao inicio() {
    seja mutavel total = 0;
    para (seja mutavel i: inteiro = 1; i <= 4; i += 1) {
        total += 1;
    }
    escreval(total);
}
`,
  vetores: `funcao inicio() {
    seja notas: vetor<inteiro> = [7, 8, 9];
    escreval(notas[1]);
}
`,
  cadeias: `funcao inicio() {
    seja texto: cadeia = "ferro";
    escreval(Texto.caixa_alta(texto));
}
`,
  resultado: `funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao inicio() {
    escolha dividir(10, 2) {
        caso Resultado::Ok(v) => {
            escreval(v);
        }
        caso Resultado::Erro(e) => {
            escreval(e);
        }
    }
}
`,
};

export function checkLesson(runtime, lesson, source) {
  const result = runtime.run(source, lesson.stdin);
  const actual = String(result.stdout ?? "").split("\n");
  if (actual.length > 1 && actual.at(-1) === "") actual.pop();
  return {
    ok: Boolean(result.ok) && JSON.stringify(actual) === JSON.stringify(lesson.expected),
    actual,
    expected: lesson.expected,
    diagnostics: result.diagnostics ?? [],
  };
}
