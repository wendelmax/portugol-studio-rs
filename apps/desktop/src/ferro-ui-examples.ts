// Exemplos visuais declarativos em Ferro (.fer) para o Preview Visual MVU (ADR 0017).

export interface FerroUiExample {
  readonly id: string;
  readonly title: string;
  readonly fileName: string;
  readonly description: string;
  readonly source: string;
}

export const FERRO_UI_EXAMPLES: readonly FerroUiExample[] = [
  {
    id: "contador-visual",
    title: "Contador interativo",
    fileName: "contador-visual.fer",
    description: "Demonstra estado em registro/enum, escolha exaustiva, botões e entrada de texto.",
    source: `enum Humor {
    Neutro,
    Animado
}

registro Estado {
    contagem: inteiro,
    nome: cadeia,
    humor: Humor
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "Visitante", humor: Humor::Neutro };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome, humor: Humor::Animado };
    }
    se (tipo == "clique" && alvo == "dec") {
        retorne Estado { contagem: estado.contagem - 1, nome: estado.nome, humor: Humor::Neutro };
    }
    se (tipo == "clique" && alvo == "zerar") {
        retorne Estado { contagem: 0, nome: estado.nome, humor: Humor::Neutro };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor, humor: estado.humor };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    seja mutavel selo = "Neutro";
    escolha estado.humor {
        caso Humor::Neutro => { selo = "Neutro"; }
        caso Humor::Animado => { selo = "Animado"; }
    }
    retorne UI.coluna([
        UI.texto("Olá, ", estado.nome, "!"),
        UI.texto("Contagem: ", estado.contagem, " (", selo, ")"),
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
  {
    id: "tarefas-visual",
    title: "Lista de tarefas",
    fileName: "tarefas-visual.fer",
    description: "Demonstra UI.lista, composição com funções auxiliares e atualização imutável de estado.",
    source: `registro Estado {
    rascunho: cadeia,
    tarefa1: cadeia,
    concluida1: logico,
    tarefa2: cadeia,
    concluida2: logico
}

funcao iniciar() -> Estado {
    retorne Estado {
        rascunho: "",
        tarefa1: "Estudar Ferro",
        concluida1: falso,
        tarefa2: "Testar UI declarativa",
        concluida2: verdadeiro
    };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "entrada_alterada" && alvo == "rascunho") {
        retorne Estado {
            rascunho: valor,
            tarefa1: estado.tarefa1,
            concluida1: estado.concluida1,
            tarefa2: estado.tarefa2,
            concluida2: estado.concluida2
        };
    }
    se (tipo == "clique" && alvo == "salvar" && estado.rascunho != "") {
        retorne Estado {
            rascunho: "",
            tarefa1: estado.rascunho,
            concluida1: falso,
            tarefa2: estado.tarefa1,
            concluida2: estado.concluida1
        };
    }
    se (tipo == "clique" && alvo == "toggle1") {
        retorne Estado {
            rascunho: estado.rascunho,
            tarefa1: estado.tarefa1,
            concluida1: !estado.concluida1,
            tarefa2: estado.tarefa2,
            concluida2: estado.concluida2
        };
    }
    se (tipo == "clique" && alvo == "toggle2") {
        retorne Estado {
            rascunho: estado.rascunho,
            tarefa1: estado.tarefa1,
            concluida1: estado.concluida1,
            tarefa2: estado.tarefa2,
            concluida2: !estado.concluida2
        };
    }
    retorne estado;
}

funcao rotulo_tarefa(texto: cadeia, feita: logico) -> cadeia {
    se (feita) {
        retorne "[x] {texto}";
    } senao {
        retorne "[ ] {texto}";
    }
}

funcao visualizar(estado: Estado) -> inteiro {
    retorne UI.coluna([
        UI.texto("Lista de tarefas Ferro"),
        UI.linha([
            UI.entrada("rascunho", estado.rascunho, "Nova tarefa..."),
            UI.botao("salvar", "Adicionar")
        ]),
        UI.lista([
            UI.linha([
                UI.texto(rotulo_tarefa(estado.tarefa1, estado.concluida1)),
                UI.botao("toggle1", "Alternar")
            ]),
            UI.linha([
                UI.texto(rotulo_tarefa(estado.tarefa2, estado.concluida2)),
                UI.botao("toggle2", "Alternar")
            ])
        ])
    ]);
}
`,
  },
];

export function findFerroUiExample(id: string): FerroUiExample | undefined {
  return FERRO_UI_EXAMPLES.find((example) => example.id === id);
}
