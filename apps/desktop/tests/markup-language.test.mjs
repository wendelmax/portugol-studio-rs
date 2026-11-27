import test from "node:test";
import assert from "node:assert/strict";
import { StringStream } from "@codemirror/language";
import { markupStreamParser } from "../src/editor/markup-language.ts";

function tokens(line) {
  const state = markupStreamParser.startState();
  const stream = new StringStream(line, 4, 4);
  const out = [];
  while (!stream.eol()) {
    const style = markupStreamParser.token(stream, state);
    if (style) out.push([stream.current(), style]);
    stream.start = stream.pos;
  }
  return out;
}

test("highlights tags, attributes, values and bindings of a .design line", () => {
  assert.deepEqual(tokens('<Rotulo Texto="Olá, {dados.nome}!" />'), [
    ["<", "angleBracket"],
    ["Rotulo", "tagName"],
    ["Texto", "attributeName"],
    ["=", "operator"],
    ['"', "attributeValue"],
    ["Olá, ", "attributeValue"],
    ["{dados.nome}", "string.special"],
    ["!", "attributeValue"],
    ['"', "attributeValue"],
    ["/>", "angleBracket"],
  ]);
});

test("escaped braces and entities are not bindings", () => {
  const styles = tokens('<Rotulo Texto="\\{a\\} &amp;" />').map(([, style]) => style);
  assert.ok(!styles.includes("string.special"));
  assert.ok(styles.includes("character"));
});

test("comments and closing tags", () => {
  assert.deepEqual(tokens("<!-- nota -->"), [["<!-- nota -->", "comment"]]);
  assert.deepEqual(tokens("</Pilha>"), [["</", "angleBracket"], ["Pilha", "tagName"], [">", "angleBracket"]]);
});
