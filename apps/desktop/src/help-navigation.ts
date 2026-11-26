// Lógica pura de navegação da Ajuda: histórico Voltar/Avançar, árvore hierárquica,
// ajuda contextual sob o cursor (F1), formatação de código inline e realce sintático PORS.

import { PORS_STDLIB_CATALOG, PORS_STDLIB_MODULES } from "./pors-stdlib-catalog.ts";

export interface HelpHistory {
  current: () => string;
  canBack: () => boolean;
  canForward: () => boolean;
  visit: (id: string) => void;
  back: () => string | null;
  forward: () => string | null;
}

/** Pilha de histórico estilo navegador (Voltar / Avançar) para tópicos da Ajuda. */
export function createHelpHistory(initialId: string): HelpHistory {
  const stack: string[] = [initialId];
  let index = 0;

  return {
    current: () => stack[index],
    canBack: () => index > 0,
    canForward: () => index < stack.length - 1,
    visit(id: string) {
      if (stack[index] === id) return;
      stack.splice(index + 1);
      stack.push(id);
      index = stack.length - 1;
    },
    back() {
      if (index <= 0) return null;
      index -= 1;
      return stack[index];
    },
    forward() {
      if (index >= stack.length - 1) return null;
      index += 1;
      return stack[index];
    },
  };
}

export interface HelpTopicSummary {
  readonly id: string;
  readonly title: string;
}

export interface HelpTreeNode {
  readonly id: string;
  readonly title: string;
  readonly children: HelpTreeNode[];
}

const SECTION_ORDER: readonly string[] = [
  "index.html",
  "tipos",
  "inteiro.html",
  "real.html",
  "logico.html",
  "caracter.html",
  "cadeia.html",
  "vazio.html",
  "declaracoes",
  "variavel.html",
  "constante.html",
  "vetor.html",
  "matriz.html",
  "funcao.html",
  "entrada_saida",
  "escreva.html",
  "leia.html",
  "limpa.html",
  "expressao",
  "atribuicao.html",
  "operacoes_aritimeticas",
  "operacao_adicao.html",
  "operacao_subtracao.html",
  "operacao_multiplicacao.html",
  "operacao_divisao.html",
  "operacao_modulo.html",
  "relacional.html",
  "operacoes_logicas",
  "e.html",
  "ou.html",
  "nao.html",
  "operacoes_bitwise",
  "bitwise_and.html",
  "bitwise_or.html",
  "bitwise_xor.html",
  "bitwise_not.html",
  "bitwise_shift.html",
  "estruturas_controle",
  "desvio",
  "se.html",
  "se_senao.html",
  "se_senao_se.html",
  "escolha_caso.html",
  "repeticao",
  "enquanto.html",
  "faca_enquanto.html",
  "para.html",
  "bibliotecas",
];

function orderRank(segment: string): number {
  const index = SECTION_ORDER.indexOf(segment);
  return index >= 0 ? index : 999;
}

function segmentOf(id: string): string {
  const parts = id.split("/");
  const last = parts[parts.length - 1] ?? "";
  return last === "index.html" ? (parts[parts.length - 2] ?? last) : last;
}

/** Constrói a árvore hierárquica ordenada dos tópicos HTML de referência da sintaxe. */
export function buildSyntaxTopicTree(topics: readonly HelpTopicSummary[]): HelpTreeNode[] {
  const byDir = new Map<string, HelpTopicSummary[]>();
  const indexByDir = new Map<string, HelpTopicSummary>();

  for (const topic of topics) {
    const parts = topic.id.split("/");
    const file = parts[parts.length - 1] ?? "";
    const dir = parts.slice(0, -1).join("/");
    if (file === "index.html") {
      indexByDir.set(dir, topic);
    } else {
      const list = byDir.get(dir) ?? [];
      list.push(topic);
      byDir.set(dir, list);
    }
  }

  function childDirsOf(parentDir: string): string[] {
    const prefix = parentDir ? `${parentDir}/` : "";
    const dirs = new Set<string>();
    for (const dir of [...indexByDir.keys(), ...byDir.keys()]) {
      if (!dir.startsWith(prefix) || dir === parentDir) continue;
      const rest = dir.slice(prefix.length);
      if (!rest.includes("/")) dirs.add(dir);
    }
    return [...dirs].sort((a, b) => {
      const aParts = a.split("/");
      const bParts = b.split("/");
      const diff = orderRank(aParts[aParts.length - 1] ?? "") - orderRank(bParts[bParts.length - 1] ?? "");
      return diff !== 0 ? diff : a.localeCompare(b, "pt-BR");
    });
  }

  function buildDir(dir: string): HelpTreeNode[] {
    const nodes: HelpTreeNode[] = [];
    for (const sub of childDirsOf(dir)) {
      const indexTopic = indexByDir.get(sub);
      const children = buildDir(sub);
      if (indexTopic) {
        nodes.push({ id: indexTopic.id, title: indexTopic.title, children });
      } else {
        nodes.push(...children);
      }
    }
    const leaves = [...(byDir.get(dir) ?? [])].sort((a, b) => {
      const diff = orderRank(segmentOf(a.id)) - orderRank(segmentOf(b.id));
      return diff !== 0 ? diff : a.title.localeCompare(b.title, "pt-BR");
    });
    for (const leaf of leaves) {
      nodes.push({ id: leaf.id, title: leaf.title, children: [] });
    }
    return nodes.sort((a, b) => {
      const diff = orderRank(segmentOf(a.id)) - orderRank(segmentOf(b.id));
      return diff !== 0 ? diff : a.title.localeCompare(b.title, "pt-BR");
    });
  }

  const rootDir = "topicos/linguagem_portugol";
  const rootIndex = indexByDir.get(rootDir);
  const rootChildren = buildDir(rootDir);
  if (rootIndex) {
    return [{ id: rootIndex.id, title: rootIndex.title, children: [] }, ...rootChildren];
  }
  return rootChildren;
}

const CONTEXTUAL_TOPIC_MAP: Readonly<Record<string, string>> = {
  seja: "topicos/linguagem_portugol/declaracoes/variavel.html",
  mutavel: "topicos/linguagem_portugol/declaracoes/variavel.html",
  funcao: "pors:funcoes",
  inicio: "pors:visao-geral",
  retorne: "pors:funcoes",
  se: "pors:condicionais",
  senao: "pors:condicionais",
  enquanto: "topicos/linguagem_portugol/estruturas_controle/repeticao/enquanto.html",
  faca: "topicos/linguagem_portugol/estruturas_controle/repeticao/faca_enquanto.html",
  para: "pors:lacos",
  pare: "pors:lacos",
  continue: "pors:lacos",
  escolha: "topicos/linguagem_portugol/estruturas_controle/desvio/escolha_caso.html",
  caso: "topicos/linguagem_portugol/estruturas_controle/desvio/escolha_caso.html",
  Opcao: "pors:opcao-resultado",
  Resultado: "pors:opcao-resultado",
  registro: "pors:colecoes-tipos-compostos",
  enum: "pors:colecoes-tipos-compostos",
  vetor: "pors:colecoes-tipos-compostos",
  tamanho: "pors:colecoes-tipos-compostos",
  adicionar: "pors:colecoes-tipos-compostos",
  remover: "pors:colecoes-tipos-compostos",
  obter: "pors:colecoes-tipos-compostos",
  escreva: "topicos/linguagem_portugol/entrada_saida/escreva.html",
  leia: "topicos/linguagem_portugol/entrada_saida/leia.html",
  inteiro: "topicos/linguagem_portugol/tipos/inteiro.html",
  real: "topicos/linguagem_portugol/tipos/real.html",
  logico: "topicos/linguagem_portugol/tipos/logico.html",
  verdadeiro: "topicos/linguagem_portugol/tipos/logico.html",
  falso: "topicos/linguagem_portugol/tipos/logico.html",
  cadeia: "topicos/linguagem_portugol/tipos/cadeia.html",
  caracter: "topicos/linguagem_portugol/tipos/caracter.html",
  iniciar: "pors:ui-mvu",
  atualizar: "pors:ui-mvu",
  visualizar: "pors:ui-mvu",
};

const MODULE_SET: ReadonlySet<string> = new Set(PORS_STDLIB_MODULES);
const MODULE_MEMBERS_SET = new Map<string, ReadonlySet<string>>(
  PORS_STDLIB_CATALOG.map((lib) => [lib.module, new Set(lib.items.map((item) => item.name))]),
);

/** Resolve o ID do tópico da Ajuda correspondente ao símbolo sob o cursor no editor `.pors`. */
export function resolveHelpTopicForEditor(doc: string, pos: number): string | null {
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
    if (MODULE_SET.has(modName)) {
      if (MODULE_MEMBERS_SET.get(modName)?.has(word)) {
        return `pors-lib:${modName}.${word}`;
      }
      return `pors-lib:${modName}`;
    }
  }

  // Variante qualificada `Opcao::Algum`, `Resultado::Erro`
  if (start >= 3 && doc.slice(start - 2, start) === "::") {
    let enumStart = start - 2;
    while (enumStart > 0 && isIdent(doc[enumStart - 1])) enumStart -= 1;
    const enumName = doc.slice(enumStart, start - 2);
    if (enumName === "Opcao" || enumName === "Resultado") {
      return "pors:opcao-resultado";
    }
  }

  if (MODULE_SET.has(word)) return `pors-lib:${word}`;
  return CONTEXTUAL_TOPIC_MAP[word] ?? null;
}

export interface InlineCodeRun {
  readonly code: boolean;
  readonly text: string;
}

/** Divide texto com trechos entre crases (`...`) em segmentos estruturados sem usar innerHTML. */
export function parseInlineCodeRuns(text: string): InlineCodeRun[] {
  const runs: InlineCodeRun[] = [];
  let cursor = 0;
  while (cursor < text.length) {
    const open = text.indexOf("`", cursor);
    if (open === -1) {
      runs.push({ code: false, text: text.slice(cursor) });
      break;
    }
    const close = text.indexOf("`", open + 1);
    if (close === -1) {
      runs.push({ code: false, text: text.slice(cursor) });
      break;
    }
    if (open > cursor) runs.push({ code: false, text: text.slice(cursor, open) });
    if (close > open + 1) runs.push({ code: true, text: text.slice(open + 1, close) });
    cursor = close + 1;
  }
  return runs;
}

export type PorsTokenKind = "keyword" | "type" | "function" | "string" | "number" | "comment" | "operator" | "plain";

export interface PorsCodeToken {
  readonly kind: PorsTokenKind;
  readonly text: string;
}

const HIGHLIGHT_KEYWORDS = new Set([
  "funcao", "inicio", "seja", "mutavel", "se", "senao", "enquanto", "faca",
  "para", "pare", "continue", "retorne", "registro", "enum", "escolha", "caso",
  "verdadeiro", "falso",
]);
const HIGHLIGHT_TYPES = new Set([
  "inteiro", "real", "logico", "cadeia", "caracter", "vetor", "Opcao", "Resultado",
  ...PORS_STDLIB_MODULES,
]);
const HIGHLIGHT_BUILTINS = new Set(["escreva", "leia", "tamanho", "adicionar", "remover", "obter"]);

/** Tokenizador leve de Portugol-RS para realce sintático nos exemplos da Ajuda. */
export function tokenizePorsCode(source: string): PorsCodeToken[] {
  const tokens: PorsCodeToken[] = [];
  let i = 0;
  const push = (kind: PorsTokenKind, text: string) => {
    const last = tokens[tokens.length - 1];
    if (last && last.kind === kind) {
      tokens[tokens.length - 1] = { kind, text: last.text + text };
    } else {
      tokens.push({ kind, text });
    }
  };

  while (i < source.length) {
    const ch = source[i];
    if (ch === "/" && source[i + 1] === "/") {
      const end = source.indexOf("\n", i);
      const next = end === -1 ? source.length : end;
      push("comment", source.slice(i, next));
      i = next;
      continue;
    }
    if (ch === "/" && source[i + 1] === "*") {
      const end = source.indexOf("*/", i + 2);
      const next = end === -1 ? source.length : end + 2;
      push("comment", source.slice(i, next));
      i = next;
      continue;
    }
    if (ch === '"' || ch === "'") {
      const quote = ch;
      let j = i + 1;
      while (j < source.length && source[j] !== quote) {
        if (source[j] === "\\") j += 1;
        j += 1;
      }
      const next = Math.min(source.length, j + 1);
      push("string", source.slice(i, next));
      i = next;
      continue;
    }
    if (/[0-9]/.test(ch)) {
      let j = i + 1;
      while (j < source.length && /[0-9._]/.test(source[j])) j += 1;
      push("number", source.slice(i, j));
      i = j;
      continue;
    }
    if (/[A-Za-z_]/.test(ch)) {
      let j = i + 1;
      while (j < source.length && /[A-Za-z0-9_]/.test(source[j])) j += 1;
      const word = source.slice(i, j);
      if (HIGHLIGHT_KEYWORDS.has(word)) push("keyword", word);
      else if (HIGHLIGHT_TYPES.has(word)) push("type", word);
      else if (HIGHLIGHT_BUILTINS.has(word) || source[j] === "(") push("function", word);
      else push("plain", word);
      i = j;
      continue;
    }
    if (/[+\-*/%=<>!&|^~?:]/.test(ch)) {
      push("operator", ch);
      i += 1;
      continue;
    }
    push("plain", ch);
    i += 1;
  }
  return tokens;
}
