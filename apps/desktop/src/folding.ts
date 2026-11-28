import { foldService } from "@codemirror/language";
import type { Text } from "@codemirror/state";

interface Fold {
  from: number;
  to: number;
}

const cache = new WeakMap<Text, Map<number, Fold>>();

function foldRanges(doc: Text): Map<number, Fold> {
  const previous = cache.get(doc);
  if (previous) return previous;

  const ranges = new Map<number, Fold>();
  const braces: number[] = [];
  const source = doc.toString();
  const tokens = /\/\*[^]*?(?:\*\/|$)|\/\/[^\n]*|"(?:\\[^\n]|[^"\\\n])*"?|'(?:\\[^\n]|[^'\\\n])*'?|[{}]/g;
  for (const match of source.matchAll(tokens)) {
    const [token] = match;
    const position = match.index!;
    if (token === "{") {
      braces.push(position);
      continue;
    }
    if (token === "}") {
      const opener = braces.pop();
      if (opener === undefined) continue;
      const openingLine = doc.lineAt(opener);
      const closingLine = doc.lineAt(position);
      if (openingLine.number === closingLine.number) continue;

      // A chave em linha própria é dobrada a partir do cabeçalho do bloco.
      const prefix = source.slice(openingLine.from, opener);
      let header = openingLine;
      if (!prefix.trim()) {
        while (header.number > 1) {
          const candidate = doc.line(header.number - 1);
          header = candidate;
          if (candidate.text.trim()) break;
        }
        if (!header.text.trim()) header = openingLine;
      }
      const to = position + 1;
      if (to > (ranges.get(header.from)?.to ?? header.to)) {
        ranges.set(header.from, { from: header.to, to });
      }
      continue;
    }
    if (token.startsWith("/*") && token.endsWith("*/")) {
      const first = doc.lineAt(position);
      const last = doc.lineAt(position + token.length - 1);
      if (last.number > first.number) {
        const to = position + token.length;
        if (to > (ranges.get(first.from)?.to ?? first.to)) {
          ranges.set(first.from, { from: first.to, to });
        }
      }
    }
  }
  cache.set(doc, ranges);
  return ranges;
}

export const ferroFolding = foldService.of((state, from) =>
  foldRanges(state.doc).get(from) ?? null,
);
