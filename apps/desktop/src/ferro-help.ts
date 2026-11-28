import {
  FERRO_STDLIB_CATALOG,
  type FerroStdlibExample,
  type FerroStdlibGroup,
  type FerroStdlibMember,
  type FerroStdlibModule,
} from "./ferro-stdlib-catalog.ts";

export type FerroHelpExample = FerroStdlibExample;

export interface FerroHelpTopic {
  readonly id: string;
  readonly title: string;
  readonly summary: string;
  readonly paragraphs: readonly string[];
  readonly comparisonRows?: readonly [string, string, string][];
  readonly example?: FerroHelpExample;
}

export type FerroHelpLibraryItem = FerroStdlibMember;
export type FerroHelpLibraryGroup = FerroStdlibGroup;
export type FerroHelpLibrary = FerroStdlibModule;

export const FERRO_HELP_DEFAULT_TOPIC = "ferro:visao-geral";

export const FERRO_HELP_TOPICS: readonly FerroHelpTopic[] = [
  {
    id: "ferro:visao-geral",
    title: "Ferro: Visão geral e tipos",
    summary: "Sintaxe tipada com imutabilidade padrão e compilação direta para VM, WASM e Rust.",
    paragraphs: [
      "Ferro (`.fer`) é pseudocódigo em português para aprender programação. Sua sintaxe e seus conceitos se inspiram em Rust, permitindo avançar gradualmente dos algoritmos para uma linguagem de uso profissional. Todo vínculo começa imutável com `seja`; use `seja mutavel` quando precisar alterar o valor.",
      "Para começar no Ferro Studio, crie um arquivo em `Novo Arquivo` → `Novo Ferro (.fer)`, escreva uma função `inicio` e selecione `Executar` (Shift+F6). A saída aparece no Console e erros ficam em Mensagens. Use `Depurar` (Shift+F5) para acompanhar a execução passo a passo.",
      "Abra a Ajuda com F1 para consultar a sintaxe e as bibliotecas. No painel `Aprender`, escolha um conceito, leia a explicação e pratique pelo exercício relacionado. Os exemplos podem ser abertos no editor para você alterar e executar.",
      "Os tipos escalares disponíveis são `inteiro` (`i32`), `real` (`f64`), `logico` (`bool`), `cadeia` (`String` UTF-8 com orçamento de memória) e `caracter` (`char` Unicode), sem coerções implícitas entre tipos.",
    ],
    example: {
      title: "Variáveis, funções e controle de fluxo",
      fileName: "guia-ferro-tipos.fer",
      source: `funcao dobro(valor: inteiro) -> inteiro {\n    retorne valor * 2;\n}\n\nfuncao inicio() {\n    seja mutavel total = 0;\n    para (seja mutavel i = 1; i <= 3; i += 1) {\n        total += dobro(i);\n    }\n    escreva("Total: ", total);\n}\n`,
    },
  },
  {
    id: "ferro:condicionais",
    title: "Condicionais encadeadas",
    summary: "Use `senao se` para testar alternativas em sequência, sem criar blocos extras.",
    paragraphs: [
      "Cada `senao se` verifica uma nova condição somente quando as anteriores forem falsas. O bloco `senao` final é opcional e cobre os demais casos.",
      "O encadeamento é representado como condicionais aninhadas e funciona da mesma forma na VM, em WebAssembly e no Rust gerado.",
    ],
    example: {
      title: "Classificação por nota",
      fileName: "condicionais-encadeadas.fer",
      source: `funcao classificar(nota: inteiro) -> inteiro {\n    se (nota >= 9) {\n        retorne 1;\n    } senao se (nota >= 7) {\n        retorne 2;\n    } senao {\n        retorne 3;\n    }\n}\n\nfuncao inicio() {\n    escreva(classificar(8));\n}\n`,
    },
  },
  {
    id: "ferro:lacos",
    title: "Laços de repetição (para, enquanto, faca-enquanto)",
    summary: "Estruturas de repetição tipadas com controle explícito por `pare` e `continue`.",
    paragraphs: [
      "Em Ferro você pode usar `para (seja mutavel i = 0; i < n; i += 1)`, `enquanto (condicao)` e `faca { ... } enquanto (condicao);`.",
      "Use `continue;` para pular para a próxima iteração e `pare;` para interromper o laço imediatamente, com o mesmo comportamento na VM e no Rust gerado.",
    ],
    example: {
      title: "Soma seletiva com para, continue e pare",
      fileName: "guia-ferro-lacos.fer",
      source: `funcao inicio() {\n    seja mutavel soma = 0;\n    para (seja mutavel i = 1; i <= 10; i += 1) {\n        se (i % 2 == 0) {\n            continue;\n        }\n        se (i > 7) {\n            pare;\n        }\n        soma += i;\n    }\n    escreva("Soma dos ímpares até 7: ", soma);\n}\n`,
    },
  },
  {
    id: "ferro:funcoes",
    title: "Funções tipadas e recursão",
    summary: "Funções auxiliares com parâmetros tipados, passagem por valor e retorno explícito.",
    paragraphs: [
      "Funções em Ferro declaram tipos explícitos nos parâmetros e no retorno (`-> Tipo`), omitindo `->` quando não retornam valor.",
      "Parâmetros são imutáveis por padrão (ou `mutavel` para cópia local mutável) e podem ser chamados em qualquer ordem ou recursivamente.",
    ],
    example: {
      title: "Fatorial recursivo tipado",
      fileName: "guia-ferro-funcoes.fer",
      source: `funcao fatorial(n: inteiro) -> inteiro {\n    se (n <= 1) {\n        retorne 1;\n    }\n    retorne n * fatorial(n - 1);\n}\n\nfuncao inicio() {\n    escreva("5! = ", fatorial(5));\n}\n`,
    },
  },
  {
    id: "ferro:opcao-resultado",
    title: "Opcao<T>, Resultado<T, E> e operador ?",
    summary: "Tratamento explícito de ausência e falhas sem null ou exceções, mapeado para `Option` e `Result` em Rust.",
    paragraphs: [
      "`Opcao<T>` possui as variantes `Opcao::Algum(valor)` e `Opcao::Nenhum`. `Resultado<T, E>` possui `Resultado::Ok(valor)` e `Resultado::Erro(erro)`.",
      "Use `escolha` para tratar todas as variantes de forma exaustiva, ou o operador pós-fixo `?` dentro de funções que retornam `Opcao<...>` ou `Resultado<..., E>` compatível.",
    ],
    example: {
      title: "Divisão segura com Resultado e ?",
      fileName: "guia-ferro-resultado.fer",
      source: `funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {\n    se (b == 0) {\n        retorne Resultado::Erro("divisão por zero");\n    }\n    retorne Resultado::Ok(a / b);\n}\n\nfuncao dobro_divisao(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {\n    seja q: inteiro = dividir(a, b)?;\n    retorne Resultado::Ok(q * 2);\n}\n\nfuncao inicio() {\n    escolha dobro_divisao(20, 4) {\n        caso Resultado::Ok(v) => { escreva(v); }\n        caso Resultado::Erro(e) => { escreva(e); }\n    }\n}\n`,
    },
  },
  {
    id: "ferro:colecoes-tipos-compostos",
    title: "Vetores dinâmicos, registros e enums",
    summary: "Construção vazia tipada `[]`, `adicionar`/`remover`/`obter` e tipos nominais (`registro` e `enum`).",
    paragraphs: [
      "Vetores tipados `vetor<T>` aceitam construção vazia tipada `[]`, inserção com `adicionar(v, valor);`, remoção segura com `remover(v) -> Opcao<T>`, leitura segura com `obter(v, indice) -> Opcao<T>` e tamanho com `tamanho(v)`.",
      "Tipos compostos nominais usam `registro Nome { campo: Tipo }` e `enum Nome { A, B(Tipo) }`, sempre com cópia por valor e `escolha` exaustiva.",
    ],
    example: {
      title: "Vetor dinâmico com obter e remover",
      fileName: "guia-ferro-vetores.fer",
      source: `funcao inicio() {\n    seja mutavel notas: vetor<inteiro> = [];\n    adicionar(notas, 8);\n    adicionar(notas, 10);\n    escolha obter(notas, 1) {\n        caso Opcao::Algum(n) => { escreva("Nota=", n, " Total=", tamanho(notas)); }\n        caso Opcao::Nenhum => { escreva("Sem nota"); }\n    }\n}\n`,
    },
  },
  {
    id: "ferro:ui-mvu",
    title: "UI visual declarativa (MVU)",
    summary: "Aplicativos interativos com `iniciar()`, `atualizar()` e `visualizar()` usando `UI.*`.",
    paragraphs: [
      "Programas visuais em Ferro declaram `iniciar() -> Estado`, `atualizar(estado, tipo, alvo, valor) -> Estado` e `visualizar(estado) -> inteiro`.",
      "A visualização compõe nós imutáveis com `UI.coluna`, `UI.linha`, `UI.lista`, `UI.texto`, `UI.botao` e `UI.entrada`, executáveis na prévia visual da IDE, em WebAssembly e em projetos Cargo standalone.",
    ],
    example: {
      title: "Contador MVU declarativo",
      fileName: "guia-ferro-ui.fer",
      source: `funcao iniciar() -> inteiro {\n    retorne 0;\n}\n\nfuncao atualizar(estado: inteiro, tipo: cadeia, alvo: cadeia) -> inteiro {\n    se (tipo == "clique" && alvo == "inc") {\n        retorne estado + 1;\n    }\n    retorne estado;\n}\n\nfuncao visualizar(estado: inteiro) -> inteiro {\n    retorne UI.coluna([\n        UI.texto("Contagem: ", estado),\n        UI.botao("inc", "+1")\n    ]);\n}\n`,
    },
  },
  {
    id: "ferro:ponte-rust",
    title: "Ponte Ferro ↔ Rust",
    summary: "Equivalência direta de conceitos entre `.fer` e `.rs`, do código didático ao Rust nativo.",
    paragraphs: [
      "Cada construção de Ferro foi desenhada para espelhar Rust real com o mínimo de atrito. Ative o botão `Rust` na barra superior para inspecionar o código Rust gerado lado a lado com source maps interativos.",
      "Quando quiser ir além da semântica de cópia por valor do Ferro e estudar ownership, movimento e empréstimos (`&T` / `&mut T`), use o painel `Aprender` no modo Ponte ou Rust para abrir o Laboratório de Ownership.",
    ],
    comparisonRows: [
      ["Binding imutável", "seja x: inteiro = 10;", "let x: i32 = 10;"],
      ["Binding mutável", "seja mutavel n: inteiro = 0;", "let mut n: i32 = 0;"],
      ["Função com retorno", "funcao dobro(x: inteiro) -> inteiro", "fn dobro(x: i32) -> i32"],
      ["Vetor dinâmico", "vetor<inteiro> / adicionar(v, x)", "Vec<i32> / v.push(x)"],
      ["Tipo composto", "registro Ponto { x: inteiro }", "struct Ponto { x: i32 }"],
      ["Enum + exaustividade", "escolha v { caso A => { ... } }", "match v { A => { ... } }"],
      ["Ausência / Erro", "Opcao<T> / Resultado<T, E> / ?", "Option<T> / Result<T, E> / ?"],
      ["Operadores lógicos", "&&, ||, !", "&&, ||, !"],
    ],
    example: {
      title: "Registro + Resultado pronto para comparar em Rust",
      fileName: "guia-ponte-rust.fer",
      source: `registro Retangulo {\n    largura: inteiro,\n    altura: inteiro,\n}\n\nfuncao area(r: Retangulo) -> inteiro {\n    retorne r.largura * r.altura;\n}\n\nfuncao inicio() {\n    seja caixa: Retangulo = Retangulo { largura: 5, altura: 3 };\n    escreva("Área = ", area(caixa));\n}\n`,
    },
  },
];

export const FERRO_HELP_LIBRARIES: readonly FerroHelpLibrary[] = FERRO_STDLIB_CATALOG;

function normalizeHelpQuery(value: string): string {
  return value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase("pt-BR");
}

export function filterFerroHelp(query: string): {
  topics: readonly FerroHelpTopic[];
  libraries: readonly FerroHelpLibrary[];
} {
  const normalized = normalizeHelpQuery(query.trim());
  if (!normalized) {
    return { topics: FERRO_HELP_TOPICS, libraries: FERRO_HELP_LIBRARIES };
  }
  const topics = FERRO_HELP_TOPICS.filter((topic) =>
    normalizeHelpQuery(
      `${topic.title} ${topic.summary} ${topic.paragraphs.join(" ")} ${(topic.comparisonRows ?? []).flat().join(" ")}`,
    ).includes(normalized),
  );
  const libraries = FERRO_HELP_LIBRARIES.filter((lib) =>
    normalizeHelpQuery(
      `${lib.name} ${lib.description} ${lib.items.map((item) => `${item.name} ${item.signature} ${item.description}`).join(" ")}`,
    ).includes(normalized),
  );
  return { topics, libraries };
}
