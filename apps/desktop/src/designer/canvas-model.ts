/**
 * Modelo do canvas do designer visual (ADR 0018, Fase 2): converte a árvore
 * do `.design` em caixas com estilo CSS, sem tocar no DOM. O DOM é montado por
 * `canvas-dom.ts`; separar as duas partes deixa as regras de layout testáveis.
 *
 * As regras seguem os três níveis do formato: posição livre por X/Y na
 * `Janela` e no `Painel`; `Pilha`, `Grade` e `Encaixe` posicionando os filhos;
 * e vínculos `{dados.campo}` mostrados como marcadores, porque o designer não
 * executa o programa.
 */
import type { DesignNode } from "../api.ts";

export type CanvasStyle = Record<string, string>;

/** Pedaço de texto: normal, a letra da tecla de acesso (`_Salvar`) ou um vínculo. */
export type TextRun = { text: string; kind: "plain" | "access" | "binding" };

export type CanvasKind =
  | "window"
  | "panel"
  | "stack"
  | "grid"
  | "dock"
  | "dock-rest"
  | "label"
  | "button"
  | "textbox"
  | "checkbox"
  | "radio"
  | "list"
  | "combo"
  | "image"
  | "unknown";

export type CanvasBox = {
  /** Caminho do elemento (`"0.1"`; `""` é a janela); `null` em caixas auxiliares do `Encaixe`. */
  path: string | null;
  tag: string;
  name: string | null;
  kind: CanvasKind;
  style: CanvasStyle;
  text: TextRun[] | null;
  placeholder: string | null;
  items: string[];
  checked: boolean;
  disabled: boolean;
  /** `Visivel="falso"`: esmaecido no designer, escondido na execução. */
  hidden: boolean;
  /** Item escolhido de `Lista` e `Combo` (`-1` para nenhum). */
  selected: number;
  children: CanvasBox[];
};

/** Componente sem aparência, mostrado na faixa abaixo da janela. */
export type CanvasComponent = { path: string; tag: string; name: string | null };

export type CanvasModel = {
  window: CanvasBox;
  title: TextRun[];
  components: CanvasComponent[];
};

/** Tamanho inicial de cada controle quando não há `Largura`/`Altura` (vem do esquema). */
export type DefaultSizes = Readonly<Record<string, readonly [number, number]>>;

const KINDS: Record<string, CanvasKind> = {
  Janela: "window",
  Painel: "panel",
  Pilha: "stack",
  Grade: "grid",
  Encaixe: "dock",
  Rotulo: "label",
  Botao: "button",
  CaixaTexto: "textbox",
  CaixaSelecao: "checkbox",
  BotaoOpcao: "radio",
  Lista: "list",
  Combo: "combo",
  Imagem: "image",
};
const COMPONENTS = new Set(["Temporizador", "DialogoArquivo"]);
const NOT_VISUAL = new Set(["Item", "DadosExemplo", ...COMPONENTS]);
const LAYOUTS = new Set<CanvasKind>(["stack", "grid", "dock"]);
const ALIGN: Record<string, string> = { Inicio: "start", Centro: "center", Fim: "end", Esticar: "stretch" };

/** Como um elemento posiciona os filhos. */
export type ContainerLayout = "free" | "stack" | "grid" | "dock" | "none";

export function layoutOf(tag: string): ContainerLayout {
  if (tag === "Janela" || tag === "Painel") return "free";
  if (tag === "Pilha") return "stack";
  if (tag === "Grade") return "grid";
  if (tag === "Encaixe") return "dock";
  return "none";
}

/** Componente sem aparência (`Temporizador`, `DialogoArquivo`): fica direto na `Janela`. */
export function isComponentTag(tag: string): boolean {
  return COMPONENTS.has(tag);
}

/** Elemento que aparece na janela (não é `Item`, `DadosExemplo` nem componente). */
export function isVisualTag(tag: string): boolean {
  return !NOT_VISUAL.has(tag);
}

export function attributeOf(node: DesignNode, name: string): string | undefined {
  return node.attributes.find((attribute) => attribute.name === name)?.value;
}

export const pathKey = (path: readonly number[]): string => path.join(".");

export function pathFromKey(key: string): number[] {
  return key === "" ? [] : key.split(".").map(Number);
}

function attributes(node: DesignNode): Map<string, string> {
  return new Map(node.attributes.map((attribute) => [attribute.name, attribute.value]));
}

function integer(value: string | undefined): number | null {
  if (value === undefined || !/^-?\d+$/.test(value.trim())) return null;
  return Number(value);
}

/** `"8"`, `"8,4"` (horizontal, vertical) ou `"1,2,3,4"` (esquerda, cima, direita, baixo), em CSS. */
export function thicknessCss(value: string | undefined): string | null {
  if (value === undefined) return null;
  const parts = value.split(",").map((part) => integer(part));
  if (parts.some((part) => part === null || part < 0)) return null;
  const n = parts as number[];
  if (n.length === 1) return `${n[0]}px`;
  if (n.length === 2) return `${n[1]}px ${n[0]}px`;
  if (n.length === 4) return `${n[1]}px ${n[2]}px ${n[3]}px ${n[0]}px`;
  return null;
}

/** Trilhas da `Grade` (`"Auto,*,2*,120"`) em `grid-template`. */
export function tracksCss(value: string | undefined): string {
  if (value === undefined || value.trim() === "") return "1fr";
  return value
    .split(",")
    .map((part) => {
      const track = part.trim();
      if (track === "Auto") return "auto";
      if (track === "*") return "1fr";
      const star = /^(\d+)\*$/.exec(track);
      if (star) return `${star[1]}fr`;
      return integer(track) !== null ? `${track}px` : "1fr";
    })
    .join(" ");
}

/**
 * Texto com vínculos e tecla de acesso. `{dados.campo}` vira um marcador,
 * `\{`/`\}` são chaves literais, `_X` marca a tecla de acesso e `__` é um
 * sublinhado. O texto da `Janela` não tem tecla de acesso.
 */
export function textRuns(value: string, accessKey = true): TextRun[] {
  const runs: TextRun[] = [];
  let plain = "";
  let accessUsed = !accessKey;
  const flush = () => {
    if (plain) runs.push({ text: plain, kind: "plain" });
    plain = "";
  };
  for (let index = 0; index < value.length; index++) {
    const char = value[index];
    const next = value[index + 1];
    if (char === "\\" && (next === "{" || next === "}")) {
      plain += next;
      index++;
    } else if (char === "{") {
      const end = value.indexOf("}", index + 1);
      if (end < 0) {
        plain += value.slice(index);
        break;
      }
      flush();
      runs.push({ text: value.slice(index, end + 1), kind: "binding" });
      index = end;
    } else if (char === "_" && next === "_" && accessKey) {
      plain += "_";
      index++;
    } else if (char === "_" && !accessUsed && next !== undefined) {
      flush();
      runs.push({ text: next, kind: "access" });
      accessUsed = true;
      index++;
    } else {
      plain += char;
    }
  }
  flush();
  return runs;
}

function visualChildren(node: DesignNode): DesignNode[] {
  return node.children.filter((child) => !NOT_VISUAL.has(child.tag));
}

function box(node: DesignNode, kind: CanvasKind, style: CanvasStyle): CanvasBox {
  const attrs = attributes(node);
  return {
    path: pathKey(node.path),
    tag: node.tag,
    name: attrs.get("Nome") ?? null,
    kind,
    style,
    text: null,
    placeholder: null,
    items: [],
    checked: attrs.get("Marcado") === "verdadeiro",
    disabled: attrs.get("Habilitado") === "falso",
    hidden: attrs.get("Visivel") === "falso",
    selected: integer(attrs.get("Selecionado")) ?? -1,
    children: [],
  };
}

type ParentLayout = "free" | "free-fill" | "stack-vertical" | "stack-horizontal" | "grid" | "dock";

function placement(node: DesignNode, parent: ParentLayout, sizes: DefaultSizes): CanvasStyle {
  const attrs = attributes(node);
  const style: CanvasStyle = {};
  const width = integer(attrs.get("Largura"));
  const height = integer(attrs.get("Altura"));
  const margin = thicknessCss(attrs.get("Margem"));
  if (margin) style.margin = margin;
  if (attrs.get("Visivel") === "falso") style.opacity = "0.45";
  const [defaultWidth, defaultHeight] = sizes[node.tag] ?? [100, 24];

  switch (parent) {
    case "free":
      style.position = "absolute";
      style.left = `${integer(attrs.get("X")) ?? 0}px`;
      style.top = `${integer(attrs.get("Y")) ?? 0}px`;
      style.width = `${width ?? defaultWidth}px`;
      style.height = `${height ?? defaultHeight}px`;
      break;
    case "free-fill":
      // Pilha, Grade ou Encaixe sozinhos na Janela ou no Painel ocupam tudo.
      style.width = width !== null ? `${width}px` : "100%";
      style.height = height !== null ? `${height}px` : "100%";
      break;
    case "stack-vertical":
    case "stack-horizontal": {
      const across = parent === "stack-vertical" ? attrs.get("Alinhamento") : attrs.get("AlinhamentoVertical");
      style["align-self"] = ALIGN[across ?? "Esticar"] ?? "stretch";
      if (width !== null) style.width = `${width}px`;
      if (height !== null) style.height = `${height}px`;
      if (parent === "stack-vertical" && height === null) style["min-height"] = `${defaultHeight}px`;
      if (parent === "stack-horizontal" && width === null) style["min-width"] = `${defaultWidth}px`;
      break;
    }
    case "grid":
      style["grid-row"] = String((integer(attrs.get("Linha")) ?? 0) + 1);
      style["grid-column"] = String((integer(attrs.get("Coluna")) ?? 0) + 1);
      style["justify-self"] = ALIGN[attrs.get("Alinhamento") ?? "Esticar"] ?? "stretch";
      style["align-self"] = ALIGN[attrs.get("AlinhamentoVertical") ?? "Esticar"] ?? "stretch";
      if (width !== null) style.width = `${width}px`;
      if (height !== null) style.height = `${height}px`;
      style["min-height"] = `${defaultHeight}px`;
      break;
    case "dock":
      if (width !== null) style.width = `${width}px`;
      if (height !== null) style.height = `${height}px`;
      break;
  }
  return style;
}

function element(node: DesignNode, parent: ParentLayout, sizes: DefaultSizes): CanvasBox {
  const attrs = attributes(node);
  const kind = KINDS[node.tag] ?? "unknown";
  const style = placement(node, parent, sizes);
  const result = box(node, kind, style);
  const padding = thicknessCss(attrs.get("Preenchimento"));
  if (padding) style.padding = padding;

  const text = attrs.get("Texto");
  switch (kind) {
    case "label":
    case "button":
    case "checkbox":
    case "radio":
      result.text = textRuns(text ?? (kind === "label" ? (result.name ?? "Rótulo") : (result.name ?? node.tag)));
      break;
    case "textbox":
      result.text = text ? textRuns(text, false) : null;
      result.placeholder = attrs.get("TextoVazio") ?? null;
      break;
    case "list":
    case "combo":
      result.items = node.children.filter((child) => child.tag === "Item").map((child) => child.text ?? "");
      break;
    case "image":
      result.text = [{ text: attrs.get("Arquivo") ?? "Imagem", kind: "plain" }];
      break;
    case "panel":
      result.text = attrs.has("Titulo") ? textRuns(attrs.get("Titulo") ?? "", false) : null;
      if (attrs.get("Borda") === "verdadeiro" || attrs.has("Titulo")) style.border = "1px solid var(--design-border)";
      result.children = children(node, "free", sizes);
      style.position ??= "relative";
      break;
    case "stack": {
      const horizontal = attrs.get("Orientacao") === "Horizontal";
      style.display = "flex";
      style["flex-direction"] = horizontal ? "row" : "column";
      style.gap = `${integer(attrs.get("Espaco")) ?? 0}px`;
      result.children = visualChildren(node).map((child) =>
        element(child, horizontal ? "stack-horizontal" : "stack-vertical", sizes),
      );
      break;
    }
    case "grid":
      style.display = "grid";
      style["grid-template-rows"] = tracksCss(attrs.get("Linhas"));
      style["grid-template-columns"] = tracksCss(attrs.get("Colunas"));
      style.gap = `${integer(attrs.get("Espaco")) ?? 0}px`;
      result.children = visualChildren(node).map((child) => element(child, "grid", sizes));
      break;
    case "dock":
      style.display = "flex";
      result.children = [dock(visualChildren(node), sizes)];
      break;
  }
  return result;
}

/** Filhos de um contêiner de posição livre. */
function children(node: DesignNode, layout: "free", sizes: DefaultSizes): CanvasBox[] {
  const visual = visualChildren(node);
  const fills = visual.length === 1 && LAYOUTS.has(KINDS[visual[0].tag] ?? "unknown");
  return visual.map((child) => element(child, fills ? "free-fill" : layout, sizes));
}

const DOCK_DIRECTION: Record<string, string> = {
  Cima: "column",
  Baixo: "column-reverse",
  Esquerda: "row",
  Direita: "row-reverse",
};

/**
 * O `Encaixe` vira uma cadeia de caixas flexíveis: cada filho ocupa a borda
 * do seu `Encaixar` e deixa o resto para os seguintes. O último filho sem
 * `Encaixar` preenche o que sobra; um filho sem `Encaixar` que não é o último
 * fica à esquerda, como no WPF.
 */
function dock(nodes: DesignNode[], sizes: DefaultSizes): CanvasBox {
  const rest: CanvasBox = {
    path: null,
    tag: "",
    name: null,
    kind: "dock-rest",
    style: { display: "flex", flex: "1 1 auto", "min-width": "0", "min-height": "0" },
    text: null,
    placeholder: null,
    items: [],
    checked: false,
    disabled: false,
    hidden: false,
    selected: -1,
    children: [],
  };
  const [first, ...others] = nodes;
  if (!first) return rest;
  const declared = attributes(first).get("Encaixar");
  const child = element(first, "dock", sizes);
  if (!declared && others.length === 0) {
    child.style.flex = "1 1 auto";
    rest.children = [child];
    return rest;
  }
  child.style.flex = "0 0 auto";
  rest.style["flex-direction"] = DOCK_DIRECTION[declared ?? "Esquerda"] ?? "row";
  rest.children = [child, dock(others, sizes)];
  return rest;
}

/** Monta o modelo do canvas a partir da raiz `<Janela>`. */
export function buildCanvasModel(root: DesignNode, sizes: DefaultSizes = {}): CanvasModel {
  const attrs = attributes(root);
  const style: CanvasStyle = {
    position: "relative",
    width: `${integer(attrs.get("Largura")) ?? 400}px`,
    height: `${integer(attrs.get("Altura")) ?? 300}px`,
  };
  const padding = thicknessCss(attrs.get("Preenchimento"));
  if (padding) style.padding = padding;
  const window = box(root, "window", style);
  window.children = children(root, "free", sizes);
  const components = root.children
    .filter((child) => COMPONENTS.has(child.tag))
    .map((child) => ({ path: pathKey(child.path), tag: child.tag, name: attributes(child).get("Nome") ?? null }));
  return { window, title: textRuns(attrs.get("Titulo") ?? "", false), components };
}

/** Caminho do elemento mais interno cuja tag contém `offset` (UTF-16), para seguir o cursor do editor. */
export function pathAtOffset(root: DesignNode, offset: number): number[] | null {
  if (offset < root.from || offset > root.to) return null;
  for (const child of root.children) {
    const inner = pathAtOffset(child, offset);
    if (inner) return inner;
  }
  return root.path;
}

/** Elemento no caminho. */
export function nodeAt(root: DesignNode, path: readonly number[]): DesignNode | null {
  let node: DesignNode | undefined = root;
  for (const index of path) {
    node = node?.children[index];
    if (!node) return null;
  }
  return node;
}
