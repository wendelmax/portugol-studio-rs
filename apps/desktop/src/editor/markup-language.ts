// Realce da marcação dos projetos desktop FERRO (ADR 0018): o pseudo-XAML dos
// arquivos .design e o manifesto .ferproj. O realce é só apresentação; a
// validade do documento vem dos diagnósticos do backend.

import { LanguageSupport, StreamLanguage, type StringStream } from "@codemirror/language";

interface MarkupState {
  inTag: boolean;
  expectTagName: boolean;
  inComment: boolean;
  quote: string | null;
}

function readComment(stream: StringStream, state: MarkupState): string {
  if (stream.skipTo("-->")) {
    stream.match("-->");
    state.inComment = false;
  } else {
    stream.skipToEnd();
  }
  return "comment";
}

/** Valor de atributo: texto comum, entidades e vínculos `{dados.campo}`. */
function readValue(stream: StringStream, state: MarkupState): string {
  const quote = state.quote as string;
  if (stream.peek() === quote) {
    stream.next();
    state.quote = null;
    return "attributeValue";
  }
  if (stream.match(/^\\[{}]/)) return "attributeValue";
  if (stream.peek() === "{") {
    stream.next();
    while (!stream.eol() && stream.peek() !== "}" && stream.peek() !== quote) stream.next();
    if (stream.peek() === "}") stream.next();
    return "string.special";
  }
  if (stream.match(/^&[a-z]+;/)) return "character";
  while (!stream.eol()) {
    const next = stream.peek();
    if (next === quote || next === "{" || next === "&" || next === "\\") break;
    stream.next();
  }
  if (stream.current() === "") stream.next();
  return "attributeValue";
}

export const markupStreamParser = {
  name: "ferro-markup",
  startState: (): MarkupState => ({ inTag: false, expectTagName: false, inComment: false, quote: null }),
  copyState: (state: MarkupState): MarkupState => ({ ...state }),
  token(stream: StringStream, state: MarkupState): string | null {
    if (state.inComment) return readComment(stream, state);
    if (state.quote) return readValue(stream, state);
    if (!state.inTag) {
      if (stream.match("<!--")) {
        state.inComment = true;
        return readComment(stream, state);
      }
      if (stream.match(/^<\/?/)) {
        state.inTag = true;
        state.expectTagName = true;
        return "angleBracket";
      }
      if (stream.match(/^&[a-z]+;/)) return "character";
      stream.eatWhile(/[^<&]/);
      if (stream.current() === "") stream.next();
      return null;
    }
    if (stream.eatSpace()) return null;
    if (stream.match(/^\/?>/)) {
      state.inTag = false;
      return "angleBracket";
    }
    if (stream.match(/^[\p{L}_][\p{L}\p{N}_.:-]*/u)) {
      if (state.expectTagName) {
        state.expectTagName = false;
        return "tagName";
      }
      return "attributeName";
    }
    if (stream.eat("=")) return "operator";
    const quote = stream.next();
    if (quote === "\"" || quote === "'") {
      state.quote = quote;
      return "attributeValue";
    }
    return "invalid";
  },
  languageData: {
    commentTokens: { block: { open: "<!--", close: "-->" } },
    closeBrackets: { brackets: ["\"", "'"] },
  },
};

const markupLanguage = StreamLanguage.define(markupStreamParser);

/** Linguagem CodeMirror dos arquivos .design e .ferproj. */
export function markup(): LanguageSupport {
  return new LanguageSupport(markupLanguage);
}
