/**
 * Dados do painel lateral do designer, sem DOM: as linhas da árvore de
 * elementos e as propriedades e eventos do elemento selecionado, lidas do
 * esquema (`design_schema`) e da árvore do `.design`.
 */
import type { DataField, DesignAttributeInfo, DesignEdit, DesignElementInfo, DesignNode, DesignSchema } from "../api.ts";
import { attributeOf, isComponentTag, layoutOf, nodeAt, pathKey, type ContainerLayout } from "./canvas-model.ts";

export type TreeRow = { path: string; depth: number; tag: string; label: string };

/** Árvore de elementos na ordem do arquivo; `Item` e `DadosExemplo` ficam de fora. */
export function elementTreeRows(root: DesignNode): TreeRow[] {
  const rows: TreeRow[] = [];
  const visit = (node: DesignNode, depth: number) => {
    if (node.tag === "Item" || node.tag === "DadosExemplo") return;
    const name = attributeOf(node, "Nome");
    rows.push({ path: pathKey(node.path), depth, tag: node.tag, label: name ? `${name} (${node.tag})` : node.tag });
    for (const child of node.children) visit(child, depth + 1);
  };
  visit(root, 0);
  return rows;
}

export type PropertyEditor = "number" | "text" | "bool" | "choice" | "anchors";

export type PropertyRow = {
  name: string;
  description: string;
  editor: PropertyEditor;
  value: string;
  required: boolean;
  min: number | null;
  max: number | null;
  choices: string[];
  /** Aceita vínculo `{dados.campo}`; um valor vinculado é editado como texto. */
  bindable: boolean;
};

export type EventRow = { name: string; description: string; value: string; suggestion: string };

const PLACEMENT_FITS: Record<DesignAttributeInfo["placement"], (layout: ContainerLayout) => boolean> = {
  any: () => true,
  free: (layout) => layout === "free",
  grid: (layout) => layout === "grid",
  dock: (layout) => layout === "dock",
  layout: (layout) => layout === "stack" || layout === "grid" || layout === "dock",
};

function editorFor(attribute: DesignAttributeInfo, value: string): PropertyEditor {
  if (attribute.binding !== "none" && value.trimStart().startsWith("{")) return "text";
  switch (attribute.type) {
    case "integer":
      return "number";
    case "bool":
      return "bool";
    case "choice":
      return "choice";
    case "anchors":
      return "anchors";
    default:
      return "text";
  }
}

export function schemaFor(schema: DesignSchema, tag: string): DesignElementInfo | null {
  if (tag === schema.window.name) return schema.window;
  return schema.elements.find((element) => element.name === tag) ?? null;
}

/** Layout do contêiner do elemento (`none` para a janela). */
export function parentLayout(root: DesignNode, path: readonly number[]): ContainerLayout {
  if (path.length === 0) return "none";
  const parent = nodeAt(root, path.slice(0, -1));
  return parent ? layoutOf(parent.tag) : "none";
}

/** Propriedades do elemento que valem no contêiner dele, na ordem do esquema. */
export function propertyRows(schema: DesignSchema, root: DesignNode, path: readonly number[]): PropertyRow[] {
  const node = nodeAt(root, path);
  const element = node && schemaFor(schema, node.tag);
  if (!node || !element) return [];
  const layout = parentLayout(root, path);
  const componentOrWindow = path.length === 0 || isComponentTag(node.tag);
  return element.attributes
    .filter((attribute) => attribute.type !== "event" && attribute.name !== "Formato")
    .filter((attribute) => componentOrWindow || PLACEMENT_FITS[attribute.placement](layout))
    .map((attribute) => {
      const value = attributeOf(node, attribute.name) ?? "";
      return {
        name: attribute.name,
        description: attribute.description,
        editor: editorFor(attribute, value),
        value,
        required: attribute.required,
        min: attribute.min,
        max: attribute.max,
        choices: attribute.choices,
        bindable: attribute.binding !== "none",
      };
    });
}

/**
 * Nome sugerido para o handler, como no ADR 0018: `btnSomar` + `AoClicar` →
 * `btnSomar_clicar`.
 */
export function handlerSuggestion(controlName: string, event: string): string {
  const action = event.startsWith("Ao") ? event.slice(2) : event;
  return `${controlName}_${action.charAt(0).toLowerCase()}${action.slice(1)}`;
}

export function eventRows(schema: DesignSchema, root: DesignNode, path: readonly number[]): EventRow[] {
  const node = nodeAt(root, path);
  const element = node && schemaFor(schema, node.tag);
  if (!node || !element) return [];
  const name = attributeOf(node, "Nome") ?? node.tag.charAt(0).toLowerCase() + node.tag.slice(1);
  return element.attributes
    .filter((attribute) => attribute.type === "event")
    .map((attribute) => ({
      name: attribute.name,
      description: attribute.description,
      value: attributeOf(node, attribute.name) ?? "",
      suggestion: handlerSuggestion(name, attribute.name),
    }));
}

/** Edição para um valor digitado no painel: vazio tira o atributo (volta ao padrão). */
export function propertyEdit(path: readonly number[], name: string, value: string): DesignEdit {
  const trimmed = value.trim();
  return trimmed === ""
    ? { kind: "removeAttribute", path: [...path], name }
    : { kind: "setAttribute", path: [...path], name, value: trimmed };
}

/** Bordas marcadas de `Ancora` (`"Cima,Esquerda"`), na ordem do esquema. */
export function anchorsValue(choices: readonly string[], checked: ReadonlySet<string>): string {
  return choices.filter((choice) => checked.has(choice)).join(",");
}

export const WRAPPERS = ["Painel", "Pilha", "Grade"] as const;
export type WrapperTag = (typeof WRAPPERS)[number];

const PLACEMENT_ATTRIBUTES = ["X", "Y", "Linha", "Coluna", "Encaixar", "Ancora", "Alinhamento", "AlinhamentoVertical", "Margem"];

/**
 * Envolve o elemento num `Painel`, `Pilha` ou `Grade`: o contêiner novo entra
 * no lugar dele, herda os atributos de posição (e o tamanho, na posição
 * livre), e o elemento vai para dentro. O núcleo tira do elemento o que não
 * vale no contêiner novo.
 */
export function planWrap(root: DesignNode, path: readonly number[], wrapper: WrapperTag, sizes: Readonly<Record<string, readonly [number, number]>>): { edits: DesignEdit[]; select: number[] } | null {
  const node = nodeAt(root, path);
  if (!node || path.length === 0 || isComponentTag(node.tag)) return null;
  const index = path[path.length - 1];
  const parent = path.slice(0, -1);
  const attributes: [string, string][] = [];
  for (const name of PLACEMENT_ATTRIBUTES) {
    const value = attributeOf(node, name);
    if (value !== undefined) attributes.push([name, value]);
  }
  if (parentLayout(root, path) === "free") {
    const [width, height] = sizes[node.tag] ?? [100, 24];
    attributes.push(["Largura", attributeOf(node, "Largura") ?? String(width)]);
    attributes.push(["Altura", attributeOf(node, "Altura") ?? String(height)]);
  }
  const edits: DesignEdit[] = [
    { kind: "insert", parent, index, tag: wrapper, attributes },
    { kind: "move", path: [...parent, index + 1], parent: [...parent, index], index: 0 },
  ];
  // Dentro do Painel (posição livre), o elemento fica no canto.
  for (const name of PLACEMENT_ATTRIBUTES.filter((n) => n !== "X" && n !== "Y")) {
    if (attributeOf(node, name) !== undefined) edits.push({ kind: "removeAttribute", path: [...parent, index, 0], name });
  }
  if (wrapper === "Painel") {
    edits.push(
      { kind: "setAttribute", path: [...parent, index, 0], name: "X", value: "0" },
      { kind: "setAttribute", path: [...parent, index, 0], name: "Y", value: "0" },
    );
  }
  return { edits, select: [...parent, index] };
}

export type SampleRow = { name: string; type: string; editable: boolean; value: string };

/** Campos de `Dados` com o valor de exemplo de cada um (vazio se não houver). */
export function sampleRows(root: DesignNode, fields: readonly DataField[]): SampleRow[] {
  const sample = root.children.find((child) => child.tag === "DadosExemplo");
  return fields.map((field) => ({
    ...field,
    value: sample?.attributes.find((attribute) => attribute.name === field.name)?.value ?? "",
  }));
}

/**
 * Edição de um valor de exemplo: cria o `<DadosExemplo>` quando falta, tira o
 * atributo quando o valor fica vazio e apaga o elemento quando ele fica sem
 * campos. `null` quando nada muda.
 */
export function sampleEdit(root: DesignNode, field: string, value: string): DesignEdit[] | null {
  const trimmed = value.trim();
  const sample = root.children.find((child) => child.tag === "DadosExemplo");
  if (!sample) {
    return trimmed === "" ? null : [{ kind: "insert", parent: [], index: root.children.length, tag: "DadosExemplo", attributes: [[field, trimmed]] }];
  }
  const current = sample.attributes.find((attribute) => attribute.name === field);
  if (trimmed === "") {
    if (!current) return null;
    return sample.attributes.length === 1
      ? [{ kind: "remove", path: sample.path }]
      : [{ kind: "removeAttribute", path: sample.path, name: field }];
  }
  if (current?.value === trimmed) return null;
  return [{ kind: "setAttribute", path: sample.path, name: field, value: trimmed }];
}
