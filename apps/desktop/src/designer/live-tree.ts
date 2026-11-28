/**
 * Estado vivo de uma tela em execução sobre a árvore do `.design`: cada campo
 * do retrato (`texto_vazio`) volta a ser o atributo (`TextoVazio`), e o
 * desenho da tela passa a mostrar os valores que o código mudou. Sem DOM.
 */
import type { DesignNode } from "../api.ts";
import { attributeOf } from "./canvas-model.ts";

/** Retrato devolvido pela execução: `{ janela, controles, dados }`. */
export type ScreenSnapshot = {
  janela: Record<string, unknown>;
  controles: Record<string, Record<string, unknown>>;
  dados: unknown;
};

/** `texto_vazio` → `TextoVazio`. */
export function attributeName(field: string): string {
  return field
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");
}

function attributeValue(value: unknown): string | null {
  if (typeof value === "boolean") return value ? "verdadeiro" : "falso";
  if (typeof value === "number") return String(value);
  if (typeof value === "string") return value;
  return null;
}

/** Valor de `dados.a.b` no retrato, como texto; `null` se o caminho não existe. */
function dataValue(data: unknown, path: string[]): string | null {
  let current: unknown = { dados: data };
  for (const part of path) {
    if (typeof current !== "object" || current === null || !(part in current)) return null;
    current = (current as Record<string, unknown>)[part];
  }
  return attributeValue(current);
}

/**
 * Atributo com vínculos (`"Olá, {dados.nome}"`) com os valores de `dados`
 * (vínculo de mão única para exibição). `\{` e `\}` são chaves literais.
 */
export function resolveBindings(raw: string, data: unknown): string {
  let out = "";
  for (let index = 0; index < raw.length; index++) {
    const char = raw[index];
    const next = raw[index + 1];
    if (char === "\\" && (next === "{" || next === "}")) {
      out += next;
      index++;
    } else if (char === "{") {
      const end = raw.indexOf("}", index + 1);
      if (end < 0) {
        out += raw.slice(index);
        break;
      }
      const path = raw.slice(index + 1, end).trim().split(".");
      out += dataValue(data, path) ?? "";
      index = end;
    } else {
      out += char;
    }
  }
  return out;
}

const hasBinding = (raw: string) => /(^|[^\\])\{/.test(raw);

function withAttributes(node: DesignNode, values: Record<string, unknown>, data: unknown): DesignNode {
  const attributes = node.attributes.map((attribute) => {
    if (hasBinding(attribute.value)) return { ...attribute, value: resolveBindings(attribute.value, data) };
    return attribute;
  });
  for (const [field, value] of Object.entries(values)) {
    if (field === "itens") continue;
    const name = attributeName(field);
    const text = attributeValue(value);
    if (text === null) continue;
    const original = attributeOf(node, name);
    // Vinculado na marcação: a tela mostra `dados`, não o campo do controle.
    if (original !== undefined && hasBinding(original)) continue;
    const index = attributes.findIndex((attribute) => attribute.name === name);
    if (index >= 0) attributes[index] = { name, value: text };
    else attributes.push({ name, value: text });
  }
  return { ...node, attributes };
}

/** Árvore do `.design` com o estado vivo aplicado. */
export function applySnapshot(root: DesignNode, snapshot: ScreenSnapshot): DesignNode {
  const visit = (node: DesignNode, isRoot: boolean): DesignNode => {
    const name = attributeOf(node, "Nome");
    const values = isRoot ? snapshot.janela : name !== undefined ? snapshot.controles[name] : undefined;
    let next = values ? withAttributes(node, values, snapshot.dados) : { ...node };
    let children = node.children.map((child) => visit(child, false));
    const items = values?.itens;
    if (Array.isArray(items)) {
      const others = children.filter((child) => child.tag !== "Item");
      const itemNodes = items.map((text, index): DesignNode => ({
        tag: "Item",
        path: [...node.path, others.length + index],
        attributes: [],
        text: String(text),
        from: 0,
        to: 0,
        children: [],
      }));
      children = [...others, ...itemNodes];
    }
    next = { ...next, children };
    return next;
  };
  return visit(root, true);
}
