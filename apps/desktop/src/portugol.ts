// Linguagem Portugol para o CodeMirror: realce (StreamLanguage) e
// autocompletar. As listas espelham o lexer em crates/portugol-syntax e as
// bibliotecas documentadas na ajuda e portadas em crates/portugol-vm/src/stdlib.

import {
  StreamLanguage,
  LanguageSupport,
  indentUnit,
  type StreamParser,
} from "@codemirror/language";
import {
  type Completion,
  type CompletionContext,
  type CompletionResult,
  snippetCompletion,
} from "@codemirror/autocomplete";
import { hoverTooltip, type Tooltip } from "@codemirror/view";
import { tags } from "@lezer/highlight";

const KEYWORDS = [
  "programa", "funcao", "inclua", "biblioteca", "const", "se", "senao",
  "enquanto", "faca", "para", "escolha", "caso", "contrario", "pare",
  "retorne", "e", "ou", "nao",
];
const TYPES = ["inteiro", "real", "cadeia", "caracter", "logico", "vazio"];
const BUILTINS = ["escreva", "leia", "limpa", "sorteia"];

export const LIBRARIES = [
  "Arquivos", "Calendario", "Graficos", "Internet", "Matematica", "Mouse",
  "Objetos", "ServicosWeb", "Sons", "Teclado", "Texto", "Tipos", "Util",
] as const;
const libraryNames = new Set<string>(LIBRARIES);
let documentedMembers: Promise<Map<string, Completion[]>> | undefined;

function libraryMembers() {
  documentedMembers ??= import("./help-assets/bibliotecas.json").then(({ default: libraries }) =>
    new Map(Object.entries(libraries).map(([name, library]) => [
      name, library.resources.map((resource) => ({
        label: resource.name,
        type: resource.signature.startsWith("funcao ") ? "function" : "constant",
        detail: resource.signature,
        info: resource.description,
      })),
    ])),
  );
  return documentedMembers;
}

const keywordSet = new Set(KEYWORDS);
const typeSet = new Set(TYPES);
const builtinSet = new Set(BUILTINS);

interface State {
  inComment: boolean;
  /** O token anterior foi um ponto: o identificador é membro de biblioteca. */
  afterDot: boolean;
}

const parser: StreamParser<State> = {
  name: "portugol",
  startState: () => ({ inComment: false, afterDot: false }),
  token(stream, state) {
    if (state.inComment) {
      if (stream.skipTo("*/")) {
        stream.next();
        stream.next();
        state.inComment = false;
      } else {
        stream.skipToEnd();
      }
      return "comment";
    }
    if (stream.eatSpace()) return null;

    if (stream.match("//")) {
      stream.skipToEnd();
      return "comment";
    }
    if (stream.match("/*")) {
      state.inComment = true;
      return "comment";
    }

    const afterDot = state.afterDot;
    state.afterDot = false;

    if (stream.match(/^"(?:\\.|[^"\\])*"?/)) return "string";
    if (stream.match(/^'(?:\\.|[^'\\])*'?/)) return "string";
    if (stream.match(/^0[xX][0-9a-fA-F]+/)) return "number";
    if (stream.match(/^\d+\.\d*|^\.\d+|^\d+/)) return "number";

    const word = stream.match(/^[A-Za-z_][A-Za-z0-9_]*/) as RegExpMatchArray | null;
    if (word) {
      const text = word[0];
      if (afterDot) return "propertyName";
      if (text === "verdadeiro" || text === "falso") return "bool";
      if (typeSet.has(text)) return "typeName";
      if (keywordSet.has(text)) return "keyword";
      if (builtinSet.has(text)) return "builtin";
      if (libraryNames.has(text)) return "namespace";
      if (stream.match(/^\s*\(/, false)) return "function";
      return "variableName";
    }

    if (stream.eat(".")) {
      state.afterDot = true;
      return "punctuation";
    }
    if (stream.match("-->")) return "operator";
    if (stream.match(/^[+\-*/%=<>!&|^~]+/)) return "operator";
    stream.next();
    return null;
  },
  languageData: {
    commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
    closeBrackets: { brackets: ["(", "[", "{", '"', "'"] },
    indentOnInput: /^\s*\}$/,
  },
  tokenTable: {
    builtin: tags.standard(tags.function(tags.variableName)),
    namespace: tags.namespace,
    function: tags.function(tags.variableName),
    punctuation: tags.punctuation,
  },
};

const language = StreamLanguage.define(parser);

export const SNIPPETS: Completion[] = [
  snippetCompletion("programa\n{\n\tfuncao inicio()\n\t{\n\t\t${}\n\t}\n}", {
    label: "programa", detail: "estrutura básica", type: "keyword",
  }),
  snippetCompletion("se (${condicao})\n{\n\t${}\n}", {
    label: "se", detail: "bloco se", type: "keyword",
  }),
  snippetCompletion("se (${condicao})\n{\n\t${}\n}\nsenao\n{\n\t\n}", {
    label: "se senao", detail: "bloco se/senao", type: "keyword",
  }),
  snippetCompletion("enquanto (${condicao})\n{\n\t${}\n}", {
    label: "enquanto", detail: "laço", type: "keyword",
  }),
  snippetCompletion("faca\n{\n\t${}\n}\nenquanto (${condicao})", {
    label: "faca", detail: "laço faça-enquanto", type: "keyword",
  }),
  snippetCompletion("para (inteiro ${i} = 0; ${i} < ${n}; ${i}++)\n{\n\t${}\n}", {
    label: "para", detail: "laço com contador", type: "keyword",
  }),
  snippetCompletion("escolha (${valor})\n{\n\tcaso ${1}:\n\t\t${}\n\t\tpare\n\tcaso contrario:\n\t\t\n}", {
    label: "escolha", detail: "escolha/caso", type: "keyword",
  }),
  snippetCompletion("funcao ${nome}(${})\n{\n\t\n}", {
    label: "funcao", detail: "declaração de função", type: "keyword",
  }),
];

const WORDS: Completion[] = [
  ...KEYWORDS.map((label) => ({ label, type: "keyword" })),
  ...TYPES.map((label) => ({ label, type: "type" })),
  { label: "verdadeiro", type: "constant" },
  { label: "falso", type: "constant" },
  ...BUILTINS.map((label) => ({ label, type: "function", apply: `${label}(` })),
];

/** `inclua biblioteca X --> y` no documento: alias/nome -> biblioteca. */
function libraryScopes(text: string): Map<string, string> {
  const scopes = new Map<string, string>();
  const include = /inclua\s+biblioteca\s+([A-Za-z_]\w*)(?:\s*-->\s*([A-Za-z_]\w*))?/g;
  for (const match of text.matchAll(include)) {
    scopes.set(match[1], match[1]);
    if (match[2]) scopes.set(match[2], match[1]);
  }
  return scopes;
}

/** Identificadores declarados no texto (variáveis, funções, parâmetros). */
function declaredNames(text: string): string[] {
  const names = new Set<string>();
  const decl = /\b(?:inteiro|real|cadeia|caracter|logico|vazio|funcao)\s+(?:[a-z]+\s+)?&?([A-Za-z_]\w*)/g;
  for (const match of text.matchAll(decl)) {
    if (!keywordSet.has(match[1]) && !typeSet.has(match[1])) names.add(match[1]);
  }
  return [...names];
}

function complete(context: CompletionContext): CompletionResult | null | Promise<CompletionResult> {
  const text = context.state.doc.toString();

  // Membro de biblioteca: `alias.`
  const member = context.matchBefore(/[A-Za-z_]\w*\.\w*/);
  if (member) {
    const [scope] = member.text.split(".");
    const library = libraryScopes(text).get(scope);
    if (library) {
      return libraryMembers().then((members) => ({
        from: member.from + scope.length + 1,
        options: members.get(library) ?? [],
        validFor: /^\w*$/,
      }));
    }
  }

  // Nome de biblioteca depois de `inclua biblioteca`.
  const include = context.matchBefore(/inclua\s+biblioteca\s+\w*/);
  if (include) {
    const word = context.matchBefore(/\w*/)!;
    return {
      from: word.from,
      options: LIBRARIES.map((label) => ({
        label,
        type: "namespace",
      })),
    };
  }

  const word = context.matchBefore(/[A-Za-z_]\w*/);
  if (!word && !context.explicit) return null;
  const from = word ? word.from : context.pos;
  const locals = declaredNames(text).map((label) => ({ label, type: "variable" }));
  return { from, options: [...SNIPPETS, ...WORDS, ...locals], validFor: /^\w*$/ };
}

export function portugol(): LanguageSupport {
  return new LanguageSupport(language, [
    indentUnit.of("\t"),
    language.data.of({ autocomplete: complete }),
  ]);
}

export interface PorsDocEntry {
  readonly label: string;
  readonly kind: "function" | "constant" | "type" | "keyword" | "namespace" | "enum";
  readonly signature: string;
  readonly description: string;
}

export interface PorsHoverInfo {
  readonly from: number;
  readonly to: number;
  readonly title: string;
  readonly signature: string;
  readonly description: string;
}

const PORS_KEYWORDS = [
  "funcao", "inicio", "seja", "mutavel", "se", "senao",
  "enquanto", "para", "pare", "continue", "retorne",
  "registro", "enum", "escolha", "caso",
] as const;

const PORS_TYPES = [
  "inteiro", "real", "logico", "cadeia", "caracter", "vetor", "Opcao", "Resultado",
] as const;

const PORS_BUILTINS = ["escreva", "leia", "tamanho"] as const;

export const PORS_MODULES = ["UI", "Matematica", "Texto", "Tipos"] as const;

const PORS_MODULE_MEMBERS: Record<(typeof PORS_MODULES)[number], readonly PorsDocEntry[]> = {
  UI: [
    { label: "texto", kind: "function", signature: "UI.texto(valor, ...) -> inteiro", description: "Cria um nó de texto declarativo concatenando valores escalares." },
    { label: "botao", kind: "function", signature: "UI.botao(id: cadeia, rotulo: cadeia) -> inteiro", description: "Cria um botão interativo que dispara eventos de clique com o identificador informado." },
    { label: "entrada", kind: "function", signature: "UI.entrada(id: cadeia, valor: cadeia, dica: cadeia) -> inteiro", description: "Cria um campo de texto controlado que dispara eventos `entrada_alterada`." },
    { label: "coluna", kind: "function", signature: "UI.coluna(filhos: vetor<inteiro>) -> inteiro", description: "Organiza nós filhos verticalmente em uma coluna." },
    { label: "linha", kind: "function", signature: "UI.linha(filhos: vetor<inteiro>) -> inteiro", description: "Organiza nós filhos horizontalmente em uma linha." },
    { label: "lista", kind: "function", signature: "UI.lista(itens: vetor<inteiro>) -> inteiro", description: "Renderiza uma lista de nós visuais filhos." },
  ],
  Matematica: [
    { label: "PI", kind: "constant", signature: "Matematica.PI: real", description: "Constante matemática π (≈ 3.141592653589793)." },
    { label: "E", kind: "constant", signature: "Matematica.E: real", description: "Constante de Euler e (≈ 2.718281828459045)." },
    { label: "potencia", kind: "function", signature: "Matematica.potencia(base: real, expoente: real) -> real", description: "Calcula `base` elevado a `expoente`." },
    { label: "raiz", kind: "function", signature: "Matematica.raiz(radicando: real, indice: real) -> real", description: "Calcula a raiz de índice `indice` de `radicando`." },
    { label: "raiz_quadrada", kind: "function", signature: "Matematica.raiz_quadrada(numero: real) -> real", description: "Calcula a raiz quadrada de `numero`." },
    { label: "arredondar", kind: "function", signature: "Matematica.arredondar(numero: real, casas: inteiro) -> real", description: "Arredonda `numero` para a quantidade de casas decimais informada (0 a 15)." },
    { label: "piso", kind: "function", signature: "Matematica.piso(numero: real) -> real", description: "Retorna o maior inteiro (como `real`) menor ou igual a `numero`." },
    { label: "teto", kind: "function", signature: "Matematica.teto(numero: real) -> real", description: "Retorna o menor inteiro (como `real`) maior ou igual a `numero`." },
    { label: "logaritmo", kind: "function", signature: "Matematica.logaritmo(numero: real, base: real) -> real", description: "Calcula o logaritmo de `numero` na base especificada." },
    { label: "seno", kind: "function", signature: "Matematica.seno(angulo: real) -> real", description: "Calcula o seno de `angulo` em radianos." },
    { label: "cosseno", kind: "function", signature: "Matematica.cosseno(angulo: real) -> real", description: "Calcula o cosseno de `angulo` em radianos." },
    { label: "tangente", kind: "function", signature: "Matematica.tangente(angulo: real) -> real", description: "Calcula a tangente de `angulo` em radianos." },
    { label: "valor_absoluto", kind: "function", signature: "Matematica.valor_absoluto(numero: real) -> real", description: "Retorna o módulo (valor absoluto) de um número `real`." },
    { label: "valor_absoluto_inteiro", kind: "function", signature: "Matematica.valor_absoluto_inteiro(numero: inteiro) -> inteiro", description: "Retorna o módulo (valor absoluto) de um número `inteiro`." },
    { label: "maior_numero", kind: "function", signature: "Matematica.maior_numero(a: real, b: real) -> real", description: "Retorna o maior entre dois valores `real`." },
    { label: "menor_numero", kind: "function", signature: "Matematica.menor_numero(a: real, b: real) -> real", description: "Retorna o menor entre dois valores `real`." },
    { label: "maior_inteiro", kind: "function", signature: "Matematica.maior_inteiro(a: inteiro, b: inteiro) -> inteiro", description: "Retorna o maior entre dois valores `inteiro`." },
    { label: "menor_inteiro", kind: "function", signature: "Matematica.menor_inteiro(a: inteiro, b: inteiro) -> inteiro", description: "Retorna o menor entre dois valores `inteiro`." },
  ],
  Texto: [
    { label: "numero_caracteres", kind: "function", signature: "Texto.numero_caracteres(texto: cadeia) -> inteiro", description: "Retorna a quantidade de caracteres Unicode (`char`) da cadeia." },
    { label: "caixa_alta", kind: "function", signature: "Texto.caixa_alta(texto: cadeia) -> cadeia", description: "Retorna uma nova cadeia convertida para letras maiúsculas." },
    { label: "caixa_baixa", kind: "function", signature: "Texto.caixa_baixa(texto: cadeia) -> cadeia", description: "Retorna uma nova cadeia convertida para letras minúsculas." },
    { label: "aparar", kind: "function", signature: "Texto.aparar(texto: cadeia) -> cadeia", description: "Remove espaços e quebras de linha no início e no fim da cadeia." },
    { label: "contem", kind: "function", signature: "Texto.contem(texto: cadeia, trecho: cadeia) -> logico", description: "Verifica se `texto` contém a subcadeia `trecho`." },
    { label: "comeca_com", kind: "function", signature: "Texto.comeca_com(texto: cadeia, prefixo: cadeia) -> logico", description: "Verifica se `texto` começa com `prefixo`." },
    { label: "termina_com", kind: "function", signature: "Texto.termina_com(texto: cadeia, sufixo: cadeia) -> logico", description: "Verifica se `texto` termina com `sufixo`." },
    { label: "substituir", kind: "function", signature: "Texto.substituir(texto: cadeia, antigo: cadeia, novo: cadeia) -> cadeia", description: "Substitui todas as ocorrências de `antigo` por `novo`, respeitando o limite de memória." },
    { label: "preencher_a_esquerda", kind: "function", signature: "Texto.preencher_a_esquerda(car: caracter, tamanho: inteiro, texto: cadeia) -> cadeia", description: "Preenche `texto` à esquerda com `car` até atingir `tamanho` caracteres." },
    { label: "obter_caracter", kind: "function", signature: "Texto.obter_caracter(texto: cadeia, indice: inteiro) -> Opcao<caracter>", description: "Retorna `Opcao::Algum(c)` no índice Unicode informado, ou `Opcao::Nenhum` fora dos limites." },
    { label: "posicao_texto", kind: "function", signature: "Texto.posicao_texto(trecho: cadeia, texto: cadeia, inicio: inteiro) -> Opcao<inteiro>", description: "Procura `trecho` em `texto` a partir do índice Unicode `inicio`, retornando `Opcao<inteiro>`." },
    { label: "extrair_subtexto", kind: "function", signature: "Texto.extrair_subtexto(texto: cadeia, inicio: inteiro, fim: inteiro) -> Resultado<cadeia, cadeia>", description: "Extrai a fatia Unicode `[inicio..fim)` retornando `Resultado<cadeia, cadeia>`." },
  ],
  Tipos: [
    { label: "para_inteiro", kind: "function", signature: "Tipos.para_inteiro(texto: cadeia) -> Resultado<inteiro, cadeia>", description: "Converte uma cadeia decimal em `inteiro`, retornando `Resultado::Erro` se for inválida." },
    { label: "para_inteiro_base", kind: "function", signature: "Tipos.para_inteiro_base(texto: cadeia, base: inteiro) -> Resultado<inteiro, cadeia>", description: "Converte uma cadeia em `inteiro` na base 2, 10 ou 16." },
    { label: "para_real", kind: "function", signature: "Tipos.para_real(texto: cadeia) -> Resultado<real, cadeia>", description: "Converte uma cadeia em `real` finito, retornando `Resultado<real, cadeia>`." },
    { label: "para_logico", kind: "function", signature: "Tipos.para_logico(texto: cadeia) -> Resultado<logico, cadeia>", description: "Converte `\"verdadeiro\"` ou `\"falso\"` em `logico`, retornando `Resultado<logico, cadeia>`." },
    { label: "para_caracter", kind: "function", signature: "Tipos.para_caracter(texto: cadeia) -> Resultado<caracter, cadeia>", description: "Converte uma cadeia de exatamente 1 caractere Unicode em `caracter`." },
    { label: "inteiro_para_real", kind: "function", signature: "Tipos.inteiro_para_real(valor: inteiro) -> real", description: "Converte explicitamente um `inteiro` em `real`." },
    { label: "real_para_inteiro", kind: "function", signature: "Tipos.real_para_inteiro(valor: real) -> inteiro", description: "Converte explicitamente um `real` em `inteiro` truncando a parte fracionária." },
    { label: "inteiro_para_cadeia", kind: "function", signature: "Tipos.inteiro_para_cadeia(valor: inteiro) -> cadeia", description: "Converte um `inteiro` para representação textual decimal." },
    { label: "inteiro_para_cadeia_base", kind: "function", signature: "Tipos.inteiro_para_cadeia_base(valor: inteiro, base: inteiro) -> Resultado<cadeia, cadeia>", description: "Formata um `inteiro` em cadeia na base 2, 10 ou 16." },
    { label: "real_para_cadeia", kind: "function", signature: "Tipos.real_para_cadeia(valor: real) -> cadeia", description: "Converte um `real` em cadeia." },
    { label: "logico_para_cadeia", kind: "function", signature: "Tipos.logico_para_cadeia(valor: logico) -> cadeia", description: "Converte um `logico` em `\"verdadeiro\"` ou `\"falso\"`." },
    { label: "caracter_para_cadeia", kind: "function", signature: "Tipos.caracter_para_cadeia(valor: caracter) -> cadeia", description: "Converte um `caracter` em cadeia de 1 caractere." },
    { label: "logico_para_inteiro", kind: "function", signature: "Tipos.logico_para_inteiro(valor: logico) -> inteiro", description: "Converte `verdadeiro` em `1` e `falso` em `0`." },
    { label: "caracter_para_inteiro", kind: "function", signature: "Tipos.caracter_para_inteiro(valor: caracter) -> Resultado<inteiro, cadeia>", description: "Converte um caractere de dígito (`'0'` a `'9'`) no inteiro correspondente." },
    { label: "inteiro_para_caracter", kind: "function", signature: "Tipos.inteiro_para_caracter(valor: inteiro) -> Resultado<caracter, cadeia>", description: "Converte um inteiro entre `0` e `9` no caractere de dígito correspondente." },
    { label: "cadeia_e_inteiro", kind: "function", signature: "Tipos.cadeia_e_inteiro(texto: cadeia) -> logico", description: "Verifica se `texto` representa um `inteiro` válido." },
    { label: "cadeia_e_real", kind: "function", signature: "Tipos.cadeia_e_real(texto: cadeia) -> logico", description: "Verifica se `texto` representa um `real` finito válido." },
    { label: "cadeia_e_logico", kind: "function", signature: "Tipos.cadeia_e_logico(texto: cadeia) -> logico", description: "Verifica se `texto` é `\"verdadeiro\"` ou `\"falso\"`." },
    { label: "cadeia_e_caracter", kind: "function", signature: "Tipos.cadeia_e_caracter(texto: cadeia) -> logico", description: "Verifica se `texto` contém exatamente 1 caractere Unicode." },
  ],
};

const PORS_BUILTIN_VARIANTS: Record<"Opcao" | "Resultado", readonly PorsDocEntry[]> = {
  Opcao: [
    { label: "Algum", kind: "enum", signature: "Opcao::Algum(valor: T) -> Opcao<T>", description: "Representa a presença de um valor dentro de `Opcao<T>` (equivale a `Option::Some` em Rust)." },
    { label: "Nenhum", kind: "enum", signature: "Opcao::Nenhum -> Opcao<T>", description: "Representa a ausência de valor em `Opcao<T>` (equivale a `Option::None` em Rust)." },
  ],
  Resultado: [
    { label: "Ok", kind: "enum", signature: "Resultado::Ok(valor: T) -> Resultado<T, E>", description: "Representa o resultado bem-sucedido de uma operação falível (equivale a `Result::Ok` em Rust)." },
    { label: "Erro", kind: "enum", signature: "Resultado::Erro(erro: E) -> Resultado<T, E>", description: "Representa uma falha tipada que pode ser tratada com `escolha` ou propagada com `?` (equivale a `Result::Err` em Rust)." },
  ],
};

const PORS_TOP_LEVEL_DOCS: Record<string, PorsDocEntry> = {
  Opcao: { label: "Opcao", kind: "type", signature: "Opcao<T> { Algum(T), Nenhum }", description: "Tipo opcional nativo do Portugol-RS, mapeado diretamente para `Option<T>` em Rust. Aceita `escolha` exaustivo e propagação com `?`." },
  Resultado: { label: "Resultado", kind: "type", signature: "Resultado<T, E> { Ok(T), Erro(E) }", description: "Tipo de resultado falível nativo do Portugol-RS, mapeado diretamente para `Result<T, E>` em Rust. Aceita `escolha` exaustivo e propagação com `?`." },
  vetor: { label: "vetor", kind: "type", signature: "vetor<T>", description: "Coleção homogênea tipada com cópia por valor e verificação de limites em tempo de execução." },
  tamanho: { label: "tamanho", kind: "function", signature: "tamanho(v: vetor<T>) -> inteiro", description: "Retorna a quantidade de elementos de um vetor tipado." },
  escreva: { label: "escreva", kind: "function", signature: "escreva(valor, ...)", description: "Escreve um ou mais valores escalares na saída padrão sem quebra de linha automática." },
  leia: { label: "leia", kind: "function", signature: "leia(variavel_mutavel)", description: "Lê um valor tipado da entrada padrão para uma variável declarada com `seja mutavel`." },
  seja: { label: "seja", kind: "keyword", signature: "seja [mutavel] nome: Tipo = valor;", description: "Declara um binding imutável por padrão (ou mutável com `seja mutavel`), equivalente a `let` / `let mut` em Rust." },
  mutavel: { label: "mutavel", kind: "keyword", signature: "seja mutavel nome: Tipo = valor;", description: "Permite reatribuição e mutação de campos ou índices do binding." },
  registro: { label: "registro", kind: "keyword", signature: "registro Nome { campo: Tipo }", description: "Declara um tipo composto nominal com campos tipados (equivale a `struct` em Rust)." },
  enum: { label: "enum", kind: "keyword", signature: "enum Nome { Variante, ComPayload(Tipo) }", description: "Declara uma enumeração nominal com variantes simples ou com payload tipado (equivale a `enum` em Rust)." },
  escolha: { label: "escolha", kind: "keyword", signature: "escolha valor { caso Tipo::Variante => { ... } }", description: "Despacho exaustivo sobre variantes de um `enum`, `Opcao<T>` ou `Resultado<T, E>` (equivale a `match` em Rust)." },
  UI: { label: "UI", kind: "namespace", signature: "modulo UI", description: "Biblioteca declarativa de interface gráfica MVU (`UI.coluna`, `UI.linha`, `UI.lista`, `UI.texto`, `UI.botao`, `UI.entrada`)." },
  Matematica: { label: "Matematica", kind: "namespace", signature: "modulo Matematica", description: "Constantes (`PI`, `E`) e funções matemáticas tipadas de Portugol-RS." },
  Texto: { label: "Texto", kind: "namespace", signature: "modulo Texto", description: "Manipulação segura de cadeias Unicode (`char`), busca opcional (`Opcao`), fatiamento (`Resultado`) e orçamento de memória." },
  Tipos: { label: "Tipos", kind: "namespace", signature: "modulo Tipos", description: "Conversões explícitas e validação entre tipos, com `Resultado<T, cadeia>` para conversões falíveis." },
};

export const PORS_SNIPPETS: Completion[] = [
  snippetCompletion("funcao ${nome}(${param}: ${inteiro}) -> ${inteiro} {\n    ${}\n}", {
    label: "funcao",
    detail: "declaração de função PORS",
    type: "keyword",
  }),
  snippetCompletion("seja ${nome}: ${inteiro} = ${0};", {
    label: "seja",
    detail: "binding imutável",
    type: "keyword",
  }),
  snippetCompletion("seja mutavel ${nome}: ${inteiro} = ${0};", {
    label: "seja mutavel",
    detail: "binding mutável",
    type: "keyword",
  }),
  snippetCompletion("se (${condicao}) {\n    ${}\n}", {
    label: "se",
    detail: "condicional se",
    type: "keyword",
  }),
  snippetCompletion("se (${condicao}) {\n    ${}\n} senao {\n    \n}", {
    label: "se senao",
    detail: "condicional se/senao",
    type: "keyword",
  }),
  snippetCompletion("enquanto (${condicao}) {\n    ${}\n}", {
    label: "enquanto",
    detail: "laço enquanto",
    type: "keyword",
  }),
  snippetCompletion("para (seja mutavel ${i}: inteiro = 0; ${i} < ${n}; ${i} += 1) {\n    ${}\n}", {
    label: "para",
    detail: "laço contado PORS",
    type: "keyword",
  }),
  snippetCompletion("registro ${Nome} {\n    ${campo}: ${inteiro}\n}", {
    label: "registro",
    detail: "declaração de registro (struct)",
    type: "keyword",
  }),
  snippetCompletion("enum ${Nome} {\n    ${Variante}\n}", {
    label: "enum",
    detail: "declaração de enum",
    type: "keyword",
  }),
  snippetCompletion(
    "escolha ${valor} {\n    caso ${Opcao::Algum(v)} => {\n        ${}\n    }\n    caso ${Opcao::Nenhum} => {\n        \n    }\n}",
    {
      label: "escolha",
      detail: "escolha exaustiva (match)",
      type: "keyword",
    },
  ),
];

const PORS_WORDS: Completion[] = [
  ...PORS_KEYWORDS.map((label) => ({
    label,
    type: "keyword",
    ...(PORS_TOP_LEVEL_DOCS[label]
      ? { detail: PORS_TOP_LEVEL_DOCS[label].signature, info: PORS_TOP_LEVEL_DOCS[label].description }
      : {}),
  })),
  ...PORS_TYPES.map((label) => ({
    label,
    type: "type",
    ...(PORS_TOP_LEVEL_DOCS[label]
      ? { detail: PORS_TOP_LEVEL_DOCS[label].signature, info: PORS_TOP_LEVEL_DOCS[label].description }
      : {}),
  })),
  ...PORS_MODULES.map((label) => ({
    label,
    type: "namespace",
    detail: PORS_TOP_LEVEL_DOCS[label].signature,
    info: PORS_TOP_LEVEL_DOCS[label].description,
  })),
  { label: "verdadeiro", type: "constant" },
  { label: "falso", type: "constant" },
  ...PORS_BUILTINS.map((label) => ({
    label,
    type: "function",
    apply: `${label}(`,
    detail: PORS_TOP_LEVEL_DOCS[label].signature,
    info: PORS_TOP_LEVEL_DOCS[label].description,
  })),
];

const porsKeywords = new Set<string>(PORS_KEYWORDS);
const porsTypes = new Set<string>(PORS_TYPES);
const porsBuiltins = new Set<string>(PORS_BUILTINS);
const porsModules = new Set<string>(PORS_MODULES);

interface PorsState {
  inComment: boolean;
  afterDot: boolean;
  afterColonColon: boolean;
}

const porsParser: StreamParser<PorsState> = {
  name: "portugol-rs",
  startState: () => ({ inComment: false, afterDot: false, afterColonColon: false }),
  token(stream, state) {
    if (state.inComment) {
      if (stream.skipTo("*/")) {
        stream.next();
        stream.next();
        state.inComment = false;
      } else {
        stream.skipToEnd();
      }
      return "comment";
    }
    if (stream.eatSpace()) return null;
    if (stream.match("//")) {
      stream.skipToEnd();
      return "comment";
    }
    if (stream.match("/*")) {
      state.inComment = true;
      return "comment";
    }

    const afterDot = state.afterDot;
    const afterColonColon = state.afterColonColon;
    state.afterDot = false;
    state.afterColonColon = false;

    if (stream.match(/^"(?:\\.|[^"\\])*"?/)) return "string";
    if (stream.match(/^'(?:\\.|[^'\\])*'?/)) return "string";
    if (stream.match(/^\d+(?:\.\d+)?/)) return "number";
    const word = stream.match(/^[A-Za-z_][A-Za-z0-9_]*/) as RegExpMatchArray | null;
    if (word) {
      const text = word[0];
      if (afterDot || afterColonColon) return "propertyName";
      if (text === "verdadeiro" || text === "falso") return "bool";
      if (porsTypes.has(text)) return "typeName";
      if (porsKeywords.has(text)) return "keyword";
      if (porsBuiltins.has(text)) return "builtin";
      if (porsModules.has(text)) return "namespace";
      if (stream.match(/^\s*\(/, false)) return "function";
      return "variableName";
    }
    if (stream.match("::")) {
      state.afterColonColon = true;
      return "punctuation";
    }
    if (stream.eat(".")) {
      state.afterDot = true;
      return "punctuation";
    }
    if (stream.match("->") || stream.match("=>")) return "operator";
    if (stream.match(/^[+\-*/%=<>!&|^?]+/)) return "operator";
    stream.next();
    return null;
  },
  languageData: {
    commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
    closeBrackets: { brackets: ["(", "[", "{", '"', "'"] },
    indentOnInput: /^\s*\}$/,
  },
  tokenTable: {
    builtin: tags.standard(tags.function(tags.variableName)),
    namespace: tags.namespace,
    function: tags.function(tags.variableName),
    punctuation: tags.punctuation,
  },
};
const porsLanguage = StreamLanguage.define(porsParser);

function porsDeclaredEnums(text: string): Map<string, string[]> {
  const enums = new Map<string, string[]>();
  const pattern = /\benum\s+([A-Za-z_]\w*)\s*\{([^}]*)\}/g;
  for (const match of text.matchAll(pattern)) {
    const name = match[1];
    const body = match[2];
    const variants: string[] = [];
    for (const part of body.split(",")) {
      const variant = part.trim().match(/^([A-Za-z_]\w*)/);
      if (variant) variants.push(variant[1]);
    }
    enums.set(name, variants);
  }
  return enums;
}

function porsDeclaredSymbols(text: string): Completion[] {
  const seen = new Set<string>();
  const symbols: Completion[] = [];
  const add = (label: string, type: string) => {
    if (
      seen.has(label)
      || porsKeywords.has(label)
      || porsTypes.has(label)
      || porsBuiltins.has(label)
      || porsModules.has(label)
    ) {
      return;
    }
    seen.add(label);
    symbols.push({ label, type });
  };
  for (const match of text.matchAll(/\bfuncao\s+([A-Za-z_]\w*)/g)) add(match[1], "function");
  for (const match of text.matchAll(/\b(?:registro|enum)\s+([A-Za-z_]\w*)/g)) add(match[1], "type");
  for (const match of text.matchAll(/\bseja\s+(?:mutavel\s+)?([A-Za-z_]\w*)/g)) add(match[1], "variable");
  return symbols;
}

function toMemberCompletion(entry: PorsDocEntry): Completion {
  return {
    label: entry.label,
    type: entry.kind,
    detail: entry.signature,
    info: entry.description,
    ...(entry.kind === "function" ? { apply: `${entry.label}(` } : {}),
  };
}

function completePors(context: CompletionContext): CompletionResult | null {
  const text = context.state.doc.toString();

  // Membro de módulo padrão PORS: `UI.`, `Matematica.`, `Texto.`, `Tipos.`
  const member = context.matchBefore(/[A-Za-z_]\w*\.\w*/);
  if (member) {
    const [scope] = member.text.split(".");
    if (scope in PORS_MODULE_MEMBERS) {
      const entries = PORS_MODULE_MEMBERS[scope as keyof typeof PORS_MODULE_MEMBERS];
      return {
        from: member.from + scope.length + 1,
        options: entries.map(toMemberCompletion),
        validFor: /^\w*$/,
      };
    }
  }

  // Variante de enum nativo ou declarado: `Opcao::`, `Resultado::`, `MeuEnum::`
  const variant = context.matchBefore(/[A-Za-z_]\w*::\w*/);
  if (variant) {
    const [enumName] = variant.text.split("::");
    const from = variant.from + enumName.length + 2;
    if (enumName === "Opcao" || enumName === "Resultado") {
      return {
        from,
        options: PORS_BUILTIN_VARIANTS[enumName].map((entry) => ({
          label: entry.label,
          type: "enum",
          detail: entry.signature,
          info: entry.description,
          ...(entry.label === "Nenhum" ? {} : { apply: `${entry.label}(` }),
        })),
        validFor: /^\w*$/,
      };
    }
    const userVariants = porsDeclaredEnums(text).get(enumName);
    if (userVariants) {
      return {
        from,
        options: userVariants.map((label) => ({
          label,
          type: "enum",
          detail: `${enumName}::${label}`,
        })),
        validFor: /^\w*$/,
      };
    }
  }

  const word = context.matchBefore(/[A-Za-z_]\w*/);
  if (!word && !context.explicit) return null;
  const from = word?.from ?? context.pos;
  const locals = porsDeclaredSymbols(text);
  return { from, options: [...PORS_SNIPPETS, ...PORS_WORDS, ...locals], validFor: /^\w*$/ };
}

/** Resolve documentação contextual para hover sobre símbolos Portugol-RS. */
export function porsHoverInfo(doc: string, pos: number): PorsHoverInfo | null {
  const isIdent = (ch: string | undefined) => !!ch && /[A-Za-z0-9_]/.test(ch);
  let start = pos;
  let end = pos;
  if (!isIdent(doc[start]) && start > 0 && isIdent(doc[start - 1])) {
    start -= 1;
    end = start;
  }
  if (!isIdent(doc[start])) return null;
  while (start > 0 && isIdent(doc[start - 1])) start -= 1;
  while (end < doc.length && isIdent(doc[end])) end += 1;
  const word = doc.slice(start, end);

  // Acesso qualificado `Modulo.membro`
  if (start >= 2 && doc[start - 1] === ".") {
    let modStart = start - 1;
    while (modStart > 0 && isIdent(doc[modStart - 1])) modStart -= 1;
    const modName = doc.slice(modStart, start - 1);
    if (modName in PORS_MODULE_MEMBERS) {
      const entry = PORS_MODULE_MEMBERS[modName as keyof typeof PORS_MODULE_MEMBERS].find(
        (item) => item.label === word,
      );
      if (entry) {
        return {
          from: modStart,
          to: end,
          title: `${modName}.${entry.label}`,
          signature: entry.signature,
          description: entry.description,
        };
      }
    }
  }

  // Variante qualificada `Opcao::Algum`, `Resultado::Erro`
  if (start >= 3 && doc.slice(start - 2, start) === "::") {
    let enumStart = start - 2;
    while (enumStart > 0 && isIdent(doc[enumStart - 1])) enumStart -= 1;
    const enumName = doc.slice(enumStart, start - 2);
    if (enumName === "Opcao" || enumName === "Resultado") {
      const entry = PORS_BUILTIN_VARIANTS[enumName].find((item) => item.label === word);
      if (entry) {
        return {
          from: enumStart,
          to: end,
          title: `${enumName}::${entry.label}`,
          signature: entry.signature,
          description: entry.description,
        };
      }
    }
  }

  const top = PORS_TOP_LEVEL_DOCS[word];
  if (!top) return null;
  return {
    from: start,
    to: end,
    title: top.label,
    signature: top.signature,
    description: top.description,
  };
}

const porsHoverExtension = hoverTooltip((view, pos): Tooltip | null => {
  const info = porsHoverInfo(view.state.doc.toString(), pos);
  if (!info) return null;
  return {
    pos: info.from,
    end: info.to,
    above: true,
    create() {
      const dom = document.createElement("div");
      dom.className = "cm-pors-hover";
      const sig = document.createElement("div");
      sig.className = "cm-pors-hover-signature";
      sig.textContent = info.signature;
      const desc = document.createElement("div");
      desc.className = "cm-pors-hover-description";
      desc.textContent = info.description;
      dom.append(sig, desc);
      return { dom };
    },
  };
});

/** Suporte de edição para o dialeto Portugol-RS (realce, snippets, autocompletar e hover). */
export function portugolRs(): LanguageSupport {
  return new LanguageSupport(porsLanguage, [
    indentUnit.of("    "),
    porsLanguage.data.of({ autocomplete: completePors }),
    porsHoverExtension,
  ]);
}
