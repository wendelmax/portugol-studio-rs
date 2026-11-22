import type { LearningMode } from "./learning-mode";

export interface LearningExample {
  code: string;
  caption: string;
  provenance: "editorial";
}

export interface LearningConcept {
  id: string;
  title: string;
  definition: string;
  portugolExample: LearningExample;
  rustExample?: LearningExample;
  provenance: "manual-selection";
}

const example = (code: string, caption: string): LearningExample => ({
  code,
  caption,
  provenance: "editorial",
});

const fundamentals: LearningConcept[] = [
  {
    id: "variables",
    title: "Variáveis",
    definition: "Uma variável associa um nome a um valor que o programa pode consultar. O tipo indica quais valores cabem nela.",
    portugolExample: example("inteiro idade = 16;\nescreva(idade);", "Exemplo Portugol"),
    rustExample: example('let idade: i32 = 16;\nprintln!("{idade}");', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "conditionals",
    title: "Decisões",
    definition: "Uma condição escolhe qual bloco executar. O bloco senao é opcional e roda quando a condição é falsa.",
    portugolExample: example("se (idade >= 18) {\n    escreva(\"Maior de idade\");\n} senao {\n    escreva(\"Menor de idade\");\n}", "Exemplo Portugol"),
    rustExample: example('if idade >= 18 {\n    println!("Maior de idade");\n} else {\n    println!("Menor de idade");\n}', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "loops",
    title: "Repetições",
    definition: "Uma repetição executa um bloco várias vezes. No enquanto, a condição é verificada antes de cada passagem.",
    portugolExample: example("inteiro contador = 0;\nenquanto (contador < 3) {\n    escreva(contador);\n    contador++;\n}", "Exemplo Portugol"),
    rustExample: example('let mut contador = 0;\nwhile contador < 3 {\n    println!("{contador}");\n    contador += 1;\n}', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
];

const bridgeConcepts: LearningConcept[] = [
  {
    id: "mutability",
    title: "Mutabilidade",
    definition: "Em Rust, uma variável é imutável por padrão. Use mut na declaração quando o valor precisar mudar. O Portugol atual permite reatribuição sem essa anotação.",
    portugolExample: example("inteiro tentativas = 0;\ntentativas++;", "Exemplo Portugol"),
    rustExample: example("let mut tentativas: i32 = 0;\ntentativas += 1;", "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "ownership",
    title: "Ownership e movimento",
    definition: "Um valor String tem um dono. A atribuição abaixo move a propriedade para b; depois disso, a não pode ser usada. O exportador Portugol→Rust atual clona cadeias em declarações e atribuições para preservar a semântica Portugol, então este exemplo ensina uma regra de Rust e não descreve a tradução da aba.",
    portugolExample: example("cadeia origem = \"Olá\";\ncadeia copia = origem;\nescreva(origem);", "Exemplo Portugol: a cadeia original continua disponível"),
    rustExample: example('let a = String::from("Olá");\nlet b = a;\n// println!("{a}"); // erro: o valor foi movido', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "borrowing",
    title: "Empréstimos e referências",
    definition: "Uma referência empresta acesso a um valor sem transferir sua posse. Em cada escopo, Rust permite várias referências imutáveis ou uma única referência mutável exclusiva.",
    portugolExample: example("cadeia nome = \"Ana\";\nescreva(nome);", "Valor Portugol"),
    rustExample: example('fn tamanho(texto: &String) -> usize { texto.len() }\nlet nome = String::from("Ana");\nlet letras = tamanho(&nome);\nprintln!("{nome}: {letras}");', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
];

const bridge = [...fundamentals, ...bridgeConcepts];
const rust = bridgeConcepts.filter(({ id }) => ["mutability", "ownership", "borrowing"].includes(id));

export function conceptsForMode(mode: LearningMode): LearningConcept[] {
  if (mode === "rust") return rust;
  return mode === "bridge" ? bridge : fundamentals;
}

export function findLearningConcept(id: string): LearningConcept | undefined {
  return bridge.find((concept) => concept.id === id);
}

export function resolveConceptForMode(mode: LearningMode, id: string): LearningConcept {
  const concepts = conceptsForMode(mode);
  return concepts.find((concept) => concept.id === id) ?? concepts[0];
}
