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
  whatChanged: string;
  whyItMatters: string;
  practice: string;
  practicePorsExerciseId?: string;
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
    whatChanged: "Você dá um nome ao valor e pode consultar esse nome em outras instruções.",
    whyItMatters: "Nomes claros ajudam a acompanhar os dados e a entender o que o algoritmo está fazendo.",
    practice: "Leia o nome de uma pessoa em uma variável cadeia e escreva a saudação com esse nome na tela.",
    practicePorsExerciseId: "pors-12-saudacao-cadeia",
    portugolExample: example("seja idade = 16;\nescreva(idade);", "Exemplo Portugol-RS (.fer)"),
    rustExample: example('let idade: i32 = 16;\nprintln!("{idade}");', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "conditionals",
    title: "Decisões",
    definition: "Uma condição escolhe qual bloco executar. O bloco senao é opcional e roda quando a condição é falsa.",
    whatChanged: "O fluxo deixa de ser apenas sequencial: uma condição escolhe qual caminho será executado.",
    whyItMatters: "Decisões permitem que o mesmo programa responda de formas diferentes conforme os dados.",
    practice: "Leia um número inteiro e escreva verdadeiro se for par ou falso se for ímpar.",
    practicePorsExerciseId: "pors-11-par-ou-impar",
    portugolExample: example("se (idade >= 18) {\n    escreva(\"Maior de idade\");\n} senao {\n    escreva(\"Menor de idade\");\n}", "Exemplo Portugol-RS (.fer)"),
    rustExample: example('if idade >= 18 {\n    println!("Maior de idade");\n} else {\n    println!("Menor de idade");\n}', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "loops",
    title: "Repetições",
    definition: "Uma repetição executa um bloco várias vezes. No enquanto e no para, a condição é verificada antes de cada passagem.",
    whatChanged: "Um bloco pode voltar a executar enquanto uma condição continuar verdadeira.",
    whyItMatters: "Repetições evitam copiar instruções e tornam simples percorrer sequências de dados.",
    practice: "Leia um inteiro N e some todos os números de 1 até N usando um laço.",
    practicePorsExerciseId: "pors-03-soma-ate-n",
    portugolExample: example("seja mutavel contador = 0;\nenquanto (contador < 3) {\n    escreva(contador);\n    contador += 1;\n}", "Exemplo Portugol-RS (.fer)"),
    rustExample: example('let mut contador = 0;\nwhile contador < 3 {\n    println!("{contador}");\n    contador += 1;\n}', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
];

const bridgeConcepts: LearningConcept[] = [
  {
    id: "mutability",
    title: "Mutabilidade",
    definition: "Em Portugol-RS e em Rust, uma variável é imutável por padrão. Use `seja mutavel` (ou `let mut` em Rust) quando o valor precisar mudar.",
    whatChanged: "Em Portugol-RS, `seja` cria um vínculo imutável e `seja mutavel` libera reatribuição — espelhando diretamente `let` e `let mut` em Rust.",
    whyItMatters: "Tornar mudanças explícitas ajuda a localizar efeitos no código e deixa mais estados protegidos contra alterações acidentais.",
    practice: "Declare `tentativas` como mutável, incremente seu valor com `+= 1` e observe o erro do compilador se remover `mutavel` / `mut`.",
    portugolExample: example("seja mutavel tentativas = 0;\ntentativas += 1;", "Exemplo Portugol-RS (.fer)"),
    rustExample: example("let mut tentativas: i32 = 0;\ntentativas += 1;", "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "ownership",
    title: "Ownership e movimento",
    definition: "Um valor String em Rust tem um dono. A atribuição abaixo move a propriedade para b; depois disso, a não pode ser usada. O exportador Portugol-RS→Rust clona cadeias em declarações e atribuições para preservar a semântica de valor do Portugol-RS, então este exemplo ensina uma regra de Rust e não descreve a tradução da aba.",
    whatChanged: "Em Portugol-RS, atribuir uma cadeia mantém a variável original disponível por cópia de valor. Em Rust, mover uma String transfere sua posse e invalida o vínculo anterior.",
    whyItMatters: "Ownership permite que Rust controle a vida dos valores sem coletor de lixo e rejeite usos que poderiam acessar memória inválida.",
    practice: "Altere a atribuição para let b = &a e use a depois. Em seguida, experimente let b = a.clone() e compare as duas formas de manter a disponível.",
    portugolExample: example("seja origem = \"Olá\";\nseja copia = origem;\nescreva(origem);", "Exemplo Portugol-RS (.fer): a cadeia original continua disponível"),
    rustExample: example('let a = String::from("Olá");\nlet b = a;\n// println!("{a}"); // erro: o valor foi movido', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "borrowing",
    title: "Empréstimos e referências",
    definition: "Uma referência empresta acesso a um valor sem transferir sua posse. Em cada escopo, Rust permite várias referências imutáveis ou uma única referência mutável exclusiva.",
    whatChanged: "Uma função pode receber &str para ler uma cadeia sem tomar posse da String que pertence a quem a chamou.",
    whyItMatters: "Empréstimos permitem compartilhar acesso com regras verificadas pelo compilador, sem transferir nem duplicar automaticamente o valor.",
    practice: "Altere a função tamanho para receber &String e depois &str. Chame-a com &nome e confira se nome continua disponível.",
    portugolExample: example("seja nome = \"Ana\";\nescreva(nome);", "Exemplo Portugol-RS (.fer)"),
    rustExample: example('fn tamanho(texto: &str) -> usize { texto.len() }\nlet nome = String::from("Ana");\nlet letras = tamanho(&nome);\nprintln!("{nome}: {letras}");', "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "functions",
    title: "Funções tipadas",
    definition: "Em Portugol-RS e em Rust, funções declaram os tipos explícitos de cada parâmetro e do retorno (`-> Tipo`), garantindo verificação em tempo de compilação.",
    whatChanged: "Em vez de depender de variáveis globais, a função recebe entradas tipadas e devolve um valor explícito.",
    whyItMatters: "Assinaturas explícitas documentam o contrato da função e permitem ao compilador detectar argumentos incompatíveis antes da execução.",
    practice: "Implemente o fatorial usando uma função tipada em `.fer` e compare com a assinatura `fn` gerada em Rust.",
    practicePorsExerciseId: "pors-04-funcao-fatorial",
    portugolExample: example("funcao dobro(valor: inteiro) -> inteiro {\n    retorne valor * 2;\n}", "Exemplo Portugol-RS (.fer)"),
    rustExample: example("fn dobro(valor: i32) -> i32 {\n    valor * 2\n}", "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "option-result",
    title: "Opcao<T>, Resultado<T, E> e escolha",
    definition: "Portugol-RS modela ausência de valor com `Opcao<T>` e erros esperados com `Resultado<T, E>`, espelhando diretamente `Option<T>` e `Result<T, E>` em Rust com correspondência exaustiva.",
    whatChanged: "Em vez de retornar valores mágicos como `-1` ou lançar exceções ocultas, o tipo de retorno exige tratar sucesso e falha explicitamente.",
    whyItMatters: "Tratar todos os casos em `escolha` / `match` elimina erros silenciosos de valor ausente e aproxima a lógica diretamente do Rust idiomático.",
    practice: "Escreva uma função `dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia>` em `.fer` e trate `Resultado::Ok` e `Resultado::Erro` com `escolha`.",
    practicePorsExerciseId: "pors-05-divisao-segura",
    portugolExample: example("escolha obter(notas, 0) {\n    caso Opcao::Algum(n) => { escreva(n); }\n    caso Opcao::Nenhum => { escreva(\"vazio\"); }\n}", "Exemplo Portugol-RS (.fer)"),
    rustExample: example("match notas.first() {\n    Some(n) => println!(\"{n}\"),\n    None => println!(\"vazio\"),\n}", "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "collections",
    title: "Vetores dinâmicos (vetor<T> → Vec<T>)",
    definition: "Um `vetor<T>` em Portugol-RS armazena uma sequência homogênea de tamanho dinâmico, traduzida para `Vec<T>` em Rust com operações seguras de leitura e remoção.",
    whatChanged: "Em vez de tamanhos fixos rígidos, a coleção cresce com `adicionar` (`push`) e acessa índices com retorno `Opcao<T>`.",
    whyItMatters: "Acesso verificado e tipagem homogênea previnem leituras fora dos limites e tornam previsível o uso de memória.",
    practice: "Filtre os números positivos lidos para um `vetor<inteiro>` dinâmico e inspecione a tradução para `Vec<i32>` na prévia Rust.",
    practicePorsExerciseId: "pors-06-filtrar-positivos",
    portugolExample: example("seja mutavel itens: vetor<inteiro> = [];\nadicionar(itens, 10);\nescreva(tamanho(itens));", "Exemplo Portugol-RS (.fer)"),
    rustExample: example("let mut itens: Vec<i32> = Vec::new();\nitens.push(10);\nprintln!(\"{}\", itens.len());", "Exemplo didático; não é código gerado nem compilado pela IDE."),
    provenance: "manual-selection",
  },
  {
    id: "records-enums",
    title: "Registros e enums (registro/enum → struct/enum)",
    definition: "Tipos compostos nominais agrupam campos relacionados com `registro` (`struct`) ou modelam estados mutuamente exclusivos com `enum`.",
    whatChanged: "Dados relacionados deixam de ficar em variáveis soltas e passam a formar tipos próprios verificados pelo compilador.",
    whyItMatters: "Modelar o domínio com `registro`/`struct` e `enum` torna estados inválidos impossíveis de representar e facilita a leitura do programa.",
    practice: "Construa um `registro Retangulo` em `.fer`, calcule sua área em uma função tipada e observe o `struct` gerado em Rust.",
    practicePorsExerciseId: "pors-13-registro-ponto",
    portugolExample: example("registro Ponto {\n    x: inteiro,\n    y: inteiro,\n}", "Exemplo Portugol-RS (.fer)"),
    rustExample: example("struct Ponto {\n    x: i32,\n    y: i32,\n}", "Exemplo didático; não é código gerado nem compilado pela IDE."),
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
