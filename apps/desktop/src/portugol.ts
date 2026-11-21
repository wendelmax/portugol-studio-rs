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

const PORS_WORDS: Completion[] = [
  ...["funcao", "inicio", "seja", "mutavel", "se", "senao"].map((label) => ({ label, type: "keyword" })),
  ...["inteiro", "real", "logico", "cadeia", "caracter"].map((label) => ({ label, type: "type" })),
  { label: "verdadeiro", type: "constant" }, { label: "falso", type: "constant" },
  { label: "escreva", type: "function", apply: "escreva(" },
];
const porsKeywords = new Set(["funcao", "inicio", "seja", "mutavel", "se", "senao"]);
const porsTypes = new Set(["inteiro", "real", "logico", "cadeia", "caracter"]);
const porsParser: StreamParser<State> = {
  name: "portugol-rs",
  startState: () => ({ inComment: false, afterDot: false }),
  token(stream, state) {
    if (state.inComment) {
      if (stream.skipTo("*/")) { stream.next(); stream.next(); state.inComment = false; }
      else stream.skipToEnd();
      return "comment";
    }
    if (stream.eatSpace()) return null;
    if (stream.match("//")) { stream.skipToEnd(); return "comment"; }
    if (stream.match("/*")) { state.inComment = true; return "comment"; }
    if (stream.match(/^"(?:\\.|[^"\\])*"?/)) return "string";
    if (stream.match(/^'(?:\\.|[^'\\])*'?/)) return "string";
    if (stream.match(/^\d+(?:\.\d+)?/)) return "number";
    const word = stream.match(/^[A-Za-z_][A-Za-z0-9_]*/) as RegExpMatchArray | null;
    if (word) {
      const text = word[0];
      if (text === "verdadeiro" || text === "falso") return "bool";
      if (porsTypes.has(text)) return "typeName";
      if (porsKeywords.has(text)) return "keyword";
      if (text === "escreva") return "builtin";
      if (stream.match(/^\s*\(/, false)) return "function";
      return "variableName";
    }
    if (stream.match(/^[+\-*/%=<>!&|^]+/)) return "operator";
    stream.next();
    return null;
  },
  languageData: { commentTokens: { line: "//", block: { open: "/*", close: "*/" } }, closeBrackets: { brackets: ["(", "{", '"'] } },
  tokenTable: { builtin: tags.standard(tags.function(tags.variableName)), function: tags.function(tags.variableName) },
};
const porsLanguage = StreamLanguage.define(porsParser);

/** Suporte de edição para o subconjunto atual do dialeto Portugol-RS. */
export function portugolRs(): LanguageSupport {
  const completePors = (context: CompletionContext): CompletionResult | null => {
    const word = context.matchBefore(/[A-Za-z_]\w*/);
    if (!word && !context.explicit) return null;
    return { from: word?.from ?? context.pos, options: PORS_WORDS, validFor: /^\w*$/ };
  };
  return new LanguageSupport(porsLanguage, [indentUnit.of("    "), porsLanguage.data.of({ autocomplete: completePors })]);
}
