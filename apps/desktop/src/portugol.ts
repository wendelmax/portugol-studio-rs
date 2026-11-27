// Linguagem Portugol-RS (.fer) para o CodeMirror: realce (StreamLanguage),
// snippets, autocompletar e documentação de hover.

import {
  StreamLanguage,
  LanguageSupport,
  indentUnit,
  type StreamParser,
  type StringStream,
} from "@codemirror/language";
import {
  type Completion,
  type CompletionContext,
  type CompletionResult,
  snippetCompletion,
} from "@codemirror/autocomplete";
import { hoverTooltip, type Tooltip } from "@codemirror/view";
import { tags } from "@lezer/highlight";
import { PORS_STDLIB_CATALOG, PORS_STDLIB_MODULES } from "./pors-stdlib-catalog.ts";
import { resolveHelpTopicForEditor } from "./help-navigation.ts";

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

const PORS_BUILTINS = ["escreva", "escreval", "leia", "tamanho", "adicionar", "remover", "obter"] as const;

export const PORS_MODULES = PORS_STDLIB_MODULES;

const PORS_MODULE_MEMBERS = new Map<string, readonly PorsDocEntry[]>(
  PORS_STDLIB_CATALOG.map((lib) => [
    lib.module,
    lib.items.map<PorsDocEntry>((item) => ({
      label: item.name,
      kind: item.kind,
      signature: item.signature,
      description: item.description,
    })),
  ]),
);

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
  adicionar: { label: "adicionar", kind: "function", signature: "adicionar(v: mutavel vetor<T>, valor: T)", description: "Insere `valor` ao final de um vetor mutável (equivale a `Vec::push` em Rust)." },
  remover: { label: "remover", kind: "function", signature: "remover(v: mutavel vetor<T>) -> Opcao<T>", description: "Remove e retorna o último elemento de um vetor mutável como `Opcao<T>` (equivale a `Vec::pop`)." },
  obter: { label: "obter", kind: "function", signature: "obter(v: vetor<T>, indice: inteiro) -> Opcao<T>", description: "Lê um elemento do vetor com segurança retornando `Opcao<T>` sem encerrar o programa fora dos limites." },
  escreva: { label: "escreva", kind: "function", signature: "escreva(valor, ...)", description: "Função embutida que escreve um ou mais valores escalares na saída padrão sem quebra automática; continua válido para prompts e saídas sem quebra final. Para acrescentar a quebra, use `escreval`." },
  escreval: { label: "escreval", kind: "function", signature: "escreval(valor, ...)", description: "Função embutida (nunca palavra reservada) que escreve os valores na saída padrão com quebra de linha ao final; equivale a `escreva(valor, ..., \"\\n\")`." },
  leia: { label: "leia", kind: "function", signature: "leia() | leia(variavel_mutavel)", description: "`leia()` é uma expressão: lê uma linha e converte implicitamente ao tipo esperado pelo contexto (`seja n: inteiro = leia();`, `n = leia();`); sem contexto tipado devolve `cadeia`. `leia(variavel_mutavel)` continua válido e converte ao tipo da variável, sendo útil em laços com leitura repetida." },
  seja: { label: "seja", kind: "keyword", signature: "seja [mutavel] nome[: Tipo] = valor;", description: "Declara um binding imutável por padrão (ou mutável com `seja mutavel`), equivalente a `let` / `let mut` em Rust. O tipo é inferido do valor inicial quando omitido; mantenha a anotação quando necessário." },
  mutavel: { label: "mutavel", kind: "keyword", signature: "seja mutavel nome[: Tipo] = valor;", description: "Permite reatribuição e mutação de campos ou índices do binding." },
  registro: { label: "registro", kind: "keyword", signature: "registro Nome { campo: Tipo }", description: "Declara um tipo composto nominal com campos tipados (equivale a `struct` em Rust)." },
  enum: { label: "enum", kind: "keyword", signature: "enum Nome { Variante, ComPayload(Tipo) }", description: "Declara uma enumeração nominal com variantes simples ou com payload tipado (equivale a `enum` em Rust)." },
  escolha: { label: "escolha", kind: "keyword", signature: "escolha valor { caso Tipo::Variante => { ... } }", description: "Despacho exaustivo sobre variantes de um `enum`, `Opcao<T>` ou `Resultado<T, E>` (equivale a `match` em Rust)." },
  UI: { label: "UI", kind: "namespace", signature: "modulo UI", description: "Biblioteca declarativa de interface gráfica MVU (`UI.coluna`, `UI.linha`, `UI.lista`, `UI.texto`, `UI.botao`, `UI.entrada`)." },
  Matematica: { label: "Matematica", kind: "namespace", signature: "modulo Matematica", description: "Constantes (`PI`, `E`) e funções matemáticas tipadas de Portugol-RS." },
  Texto: { label: "Texto", kind: "namespace", signature: "modulo Texto", description: "Manipulação segura de cadeias Unicode (`char`), busca opcional (`Opcao`), fatiamento (`Resultado`) e orçamento de memória." },
  Tipos: { label: "Tipos", kind: "namespace", signature: "modulo Tipos", description: "Conversões explícitas e validação entre tipos, com `Resultado<T, cadeia>` para conversões falíveis." },
  Util: { label: "Util", kind: "namespace", signature: "modulo Util", description: "Utilitários determinísticos e seguros (`Util.sorteia`, `Util.definir_semente`, `Util.tempo_decorrido`, `Util.aguarde`)." },
  Calendario: { label: "Calendario", kind: "namespace", signature: "modulo Calendario", description: "Data e hora UTC determinísticas, constantes de dias/meses e cálculos gregorianos." },
  Arquivos: { label: "Arquivos", kind: "namespace", signature: "modulo Arquivos", description: "Leitura e escrita de arquivos UTF-8 confinada ao workspace, com limite de 2 MiB e tratamento via `Resultado<T, cadeia>`." },
  Json: { label: "Json", kind: "namespace", signature: "modulo Json", description: "Validação, consulta, formatação e construção imutável de JSON (RFC 8259) zero-dependência com `Resultado<T, cadeia>`." },
  Internet: { label: "Internet", kind: "namespace", signature: "modulo Internet", description: "Requisições HTTP/HTTPS seguras com consentimento explícito, timeout/cancelamento e limite de 1 MiB." },
  Graficos: { label: "Graficos", kind: "namespace", signature: "modulo Graficos", description: "Janela gráfica nativa, formas, texto, imagens e GIFs com ponte tipada para o host da VM." },
  Teclado: { label: "Teclado", kind: "namespace", signature: "modulo Teclado", description: "Leitura e consulta tipada de teclas na janela gráfica (`Resultado<inteiro, cadeia>` para espera bloqueante)." },
  Mouse: { label: "Mouse", kind: "namespace", signature: "modulo Mouse", description: "Consulta de botões, posição e cursor do mouse na janela gráfica." },
  Sons: { label: "Sons", kind: "namespace", signature: "modulo Sons", description: "Carregamento confinado ao workspace e controle de reprodução de áudio no host nativo da VM." },
};

export const PORS_SNIPPETS: Completion[] = [
  snippetCompletion("funcao ${nome}(${param}: ${inteiro}) -> ${inteiro} {\n    ${}\n}", {
    label: "funcao",
    detail: "declaração de função PORS",
    type: "keyword",
  }),
  snippetCompletion("seja ${nome} = ${0};", {
    label: "seja",
    detail: "binding imutável com inferência",
    type: "keyword",
  }),
  snippetCompletion("seja mutavel ${nome} = ${0};", {
    label: "seja mutavel",
    detail: "binding mutável com inferência",
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
  snippetCompletion("se (${condicao1}) {\n    ${}\n} senao se (${condicao2}) {\n    ${}\n} senao {\n    ${}\n}", {
    label: "se senao se",
    detail: "condicionais se/senao se encadeadas",
    type: "keyword",
  }),
  snippetCompletion("enquanto (${condicao}) {\n    ${}\n}", {
    label: "enquanto",
    detail: "laço enquanto",
    type: "keyword",
  }),
  snippetCompletion("para (seja mutavel ${i} = 0; ${i} < ${n}; ${i} += 1) {\n    ${}\n}", {
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
  inString: boolean;
  inInterpolation: boolean;
  inNestedString: boolean;
}

/** Índice do `}` que fecha uma interpolação, ou -1 se o trecho for literal. */
function interpolationEnd(line: string, from: number): number {
  for (let i = from; i < line.length; i += 1) {
    const ch = line[i];
    if (ch === "\\") {
      i += 1;
      continue;
    }
    if (ch === "}") return i > from ? i : -1;
  }
  return -1;
}

/** Verdadeiro quando um `{` pode fechar em uma linha seguinte da mesma cadeia. */
function interpolationContinuesOnNextLine(line: string, from: number): boolean {
  for (let i = from; i < line.length; i += 1) {
    const ch = line[i];
    if (ch === "\\") {
      i += 1;
      continue;
    }
    if (ch === '"' || ch === "}") return false;
  }
  return true;
}

function classifyPorsToken(stream: StringStream, state: PorsState): string | null {
  if (stream.match(/^\d+(?:\.\d+)?/)) return "number";
  const word = stream.match(/^[A-Za-z_][A-Za-z0-9_]*/) as RegExpMatchArray | null;
  if (word) {
    const text = word[0];
    if (state.afterDot || state.afterColonColon) return "propertyName";
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
  return null;
}

function scanPorsString(stream: StringStream, state: PorsState): string | null {
  let consumed = false;
  while (!stream.eol()) {
    const ch = stream.peek();
    if (state.inNestedString) {
      if (ch === "\\") {
        stream.next();
        if (!stream.eol()) stream.next();
        consumed = true;
        continue;
      }
      if (ch === '"') {
        if (consumed) return "string";
        stream.next();
        state.inNestedString = false;
        return "string";
      }
      stream.next();
      consumed = true;
      continue;
    }
    if (state.inInterpolation && ch === '"') {
      if (consumed) return "string";
      stream.next();
      state.inNestedString = true;
      return "string";
    }
    if (state.inInterpolation && ch === "}") {
      if (consumed) return "string";
      stream.next();
      state.inInterpolation = false;
      return "interpolation";
    }
    if (state.inInterpolation) {
      if (stream.eatSpace()) return null;
      if (!consumed) {
        const token = classifyPorsToken(stream, state);
        if (token) {
          if (token !== "punctuation") {
            state.afterDot = false;
            state.afterColonColon = false;
          }
          return token;
        }
      }
    }
    if (ch === '"') {
      if (consumed) return "string";
      stream.next();
      state.inString = false;
      return "string";
    }
    if (
      !state.inInterpolation &&
      ch === "{" &&
      (interpolationEnd(stream.string, stream.pos + 1) >= 0 ||
        interpolationContinuesOnNextLine(stream.string, stream.pos + 1))
    ) {
      if (consumed) return "string";
      stream.next();
      state.inInterpolation = true;
      return "interpolation";
    }
    if (ch === "\\") {
      stream.next();
      if (!stream.eol()) stream.next();
      consumed = true;
      continue;
    }
    stream.next();
    consumed = true;
  }
  return consumed ? "string" : null;
}

const porsParser: StreamParser<PorsState> = {
  name: "portugol-rs",
  startState: () => ({
    inComment: false,
    afterDot: false,
    afterColonColon: false,
    inString: false,
    inInterpolation: false,
    inNestedString: false,
  }),
  token(stream, state) {
    if (state.inString) return scanPorsString(stream, state);
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

    if (stream.peek() === '"') {
      stream.next();
      state.inString = true;
      return scanPorsString(stream, state);
    }
    if (stream.match(/^'(?:\\.|[^'\\])*'?/)) return "string";
    const token = classifyPorsToken(stream, state);
    if (token !== "punctuation") {
      state.afterDot = false;
      state.afterColonColon = false;
    }
    if (token) return token;
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
    interpolation: tags.special(tags.string),
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
    const entries = PORS_MODULE_MEMBERS.get(scope);
    if (entries) {
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
    const members = PORS_MODULE_MEMBERS.get(modName);
    if (members) {
      const entry = members.find((item) => item.label === word);
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
  const doc = view.state.doc.toString();
  const info = porsHoverInfo(doc, pos);
  if (!info) return null;
  const helpTopicId = resolveHelpTopicForEditor(doc, pos);
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
      if (helpTopicId) {
        const footer = document.createElement("div");
        footer.className = "cm-pors-hover-footer";
        const helpButton = document.createElement("button");
        helpButton.type = "button";
        helpButton.className = "cm-pors-hover-help";
        helpButton.textContent = "Ver na Ajuda (F1)";
        helpButton.onclick = (event) => {
          event.preventDefault();
          view.dom.dispatchEvent(new CustomEvent("pors-open-help", {
            bubbles: true,
            detail: { topicId: helpTopicId },
          }));
        };
        footer.append(helpButton);
        dom.append(footer);
      }
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
