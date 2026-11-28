// Renderizador DOM seguro para a árvore declarativa UiNode (ADR 0017 / crates/ferro-vm/src/ui.rs).
// Usa exclusivamente createElement e textContent (sem HTML bruto) e preserva foco/cursor em entradas
// durante re-renderizações MVU no WebView Tauri ou em páginas Web.

export type UiNode =
  | { kind: "coluna"; children: UiNode[] }
  | { kind: "linha"; children: UiNode[] }
  | { kind: "texto"; content: string }
  | { kind: "botao"; id: string; label: string }
  | { kind: "entrada"; id: string; value: string; placeholder: string }
  | { kind: "lista"; items: UiNode[] };

export type UiEvent =
  | { kind: "clique"; id: string }
  | { kind: "entrada_alterada"; id: string; value: string };

export interface UiRenderLimits {
  readonly maxNodes: number;
  readonly maxDepth: number;
}

export const DEFAULT_UI_RENDER_LIMITS: UiRenderLimits = {
  maxNodes: 1024,
  maxDepth: 32,
};

export type UiContractErrorCode =
  | "invalid_json"
  | "invalid_node"
  | "node_limit_exceeded"
  | "depth_limit_exceeded";

export class UiContractError extends Error {
  readonly code: UiContractErrorCode;

  constructor(code: UiContractErrorCode, message: string) {
    super(message);
    this.name = "UiContractError";
    this.code = code;
  }
}

export interface UiPreviewController {
  render(node: UiNode): void;
  renderJson(raw: string): UiNode;
  setDisabled(disabled: boolean): void;
  destroy(): void;
}

function resolveLimits(limits?: Partial<UiRenderLimits>): UiRenderLimits {
  return {
    maxNodes: limits?.maxNodes ?? DEFAULT_UI_RENDER_LIMITS.maxNodes,
    maxDepth: limits?.maxDepth ?? DEFAULT_UI_RENDER_LIMITS.maxDepth,
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function expectStringField(record: Record<string, unknown>, field: string, kind: string): string {
  const value = record[field];
  if (typeof value !== "string") {
    throw new UiContractError("invalid_node", `O campo "${field}" de "${kind}" deve ser texto.`);
  }
  return value;
}

function expectArrayField(record: Record<string, unknown>, field: string, kind: string): unknown[] {
  const value = record[field];
  if (!Array.isArray(value)) {
    throw new UiContractError("invalid_node", `O campo "${field}" de "${kind}" deve ser lista.`);
  }
  return value;
}

/** Valida um objeto arbitrário contra o contrato UiNode e os limites de nós/profundidade. */
export function validateUiNode(value: unknown, limits?: Partial<UiRenderLimits>): UiNode {
  const resolved = resolveLimits(limits);
  let count = 0;

  function walk(candidate: unknown, depth: number): UiNode {
    if (depth > resolved.maxDepth) {
      throw new UiContractError(
        "depth_limit_exceeded",
        `A profundidade máxima da árvore (${resolved.maxDepth}) foi excedida.`,
      );
    }
    count += 1;
    if (count > resolved.maxNodes) {
      throw new UiContractError(
        "node_limit_exceeded",
        `O limite máximo de ${resolved.maxNodes} nós de interface foi excedido.`,
      );
    }
    if (!isRecord(candidate)) {
      throw new UiContractError("invalid_node", "Cada nó de interface deve ser um objeto.");
    }
    const kind = candidate.kind;
    if (typeof kind !== "string") {
      throw new UiContractError("invalid_node", "Nó de interface sem campo \"kind\" textual.");
    }
    switch (kind) {
      case "coluna": {
        const rawChildren = expectArrayField(candidate, "children", kind);
        return { kind: "coluna", children: rawChildren.map((child) => walk(child, depth + 1)) };
      }
      case "linha": {
        const rawChildren = expectArrayField(candidate, "children", kind);
        return { kind: "linha", children: rawChildren.map((child) => walk(child, depth + 1)) };
      }
      case "texto":
        return { kind: "texto", content: expectStringField(candidate, "content", kind) };
      case "botao":
        return {
          kind: "botao",
          id: expectStringField(candidate, "id", kind),
          label: expectStringField(candidate, "label", kind),
        };
      case "entrada":
        return {
          kind: "entrada",
          id: expectStringField(candidate, "id", kind),
          value: expectStringField(candidate, "value", kind),
          placeholder: expectStringField(candidate, "placeholder", kind),
        };
      case "lista": {
        const rawItems = expectArrayField(candidate, "items", kind);
        return { kind: "lista", items: rawItems.map((item) => walk(item, depth + 1)) };
      }
      default:
        throw new UiContractError("invalid_node", `Componente de interface desconhecido: "${kind}".`);
    }
  }

  return walk(value, 1);
}

/** Faz parse e valida o JSON canônico emitido por `ferro_vm::ui::UiNode::to_json`. */
export function parseUiNodeJson(raw: string, limits?: Partial<UiRenderLimits>): UiNode {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    throw new UiContractError("invalid_json", "JSON da árvore de interface inválido.");
  }
  return validateUiNode(parsed, limits);
}

/** Serializa um evento de UI no formato canônico esperado pelo runtime MVU. */
export function serializeUiEvent(event: UiEvent): string {
  if (event.kind === "clique") {
    return JSON.stringify({ kind: "clique", id: event.id });
  }
  return JSON.stringify({ kind: "entrada_alterada", id: event.id, value: event.value });
}

interface SavedInputFocus {
  id: string;
  selectionStart: number | null;
  selectionEnd: number | null;
}

function captureFocusedInput(container: HTMLElement): SavedInputFocus | null {
  const active = container.ownerDocument?.activeElement as HTMLElement | null | undefined;
  if (!active || active.tagName?.toUpperCase() !== "INPUT") return null;
  if (typeof container.contains === "function" && !container.contains(active)) return null;
  const id = active.getAttribute("data-ui-id");
  if (active.getAttribute("data-ui-kind") !== "entrada" || id === null) return null;
  const input = active as HTMLInputElement;
  return {
    id,
    selectionStart: typeof input.selectionStart === "number" ? input.selectionStart : null,
    selectionEnd: typeof input.selectionEnd === "number" ? input.selectionEnd : null,
  };
}

/**
 * Monta um renderizador MVU seguro dentro de `container`.
 * Eventos disparados por controles antigos ou após `destroy()` são descartados.
 */
export function mountUiPreview(
  container: HTMLElement,
  onEvent: (event: UiEvent) => void,
  limits?: Partial<UiRenderLimits>,
): UiPreviewController {
  const doc = container.ownerDocument;
  let disabled = false;
  let destroyed = false;
  let renderGeneration = 0;
  let interactiveControls: Array<HTMLButtonElement | HTMLInputElement> = [];
  let cleanupListeners: Array<() => void> = [];

  function detachListeners() {
    for (const cleanup of cleanupListeners) {
      cleanup();
    }
    cleanupListeners = [];
    interactiveControls = [];
  }

  function dispatch(generation: number, event: UiEvent) {
    if (destroyed || disabled || generation !== renderGeneration) return;
    onEvent(event);
  }

  function buildElement(
    node: UiNode,
    generation: number,
    inputsById: Map<string, HTMLInputElement>,
  ): HTMLElement {
    switch (node.kind) {
      case "coluna": {
        const element = doc.createElement("div");
        element.className = "ferro-ui-column";
        element.setAttribute("data-ui-kind", "coluna");
        for (const child of node.children) {
          element.appendChild(buildElement(child, generation, inputsById));
        }
        return element;
      }
      case "linha": {
        const element = doc.createElement("div");
        element.className = "ferro-ui-row";
        element.setAttribute("data-ui-kind", "linha");
        for (const child of node.children) {
          element.appendChild(buildElement(child, generation, inputsById));
        }
        return element;
      }
      case "texto": {
        const element = doc.createElement("p");
        element.className = "ferro-ui-text";
        element.setAttribute("data-ui-kind", "texto");
        element.textContent = node.content;
        return element;
      }
      case "botao": {
        const button = doc.createElement("button");
        button.type = "button";
        button.className = "ferro-ui-button";
        button.setAttribute("data-ui-kind", "botao");
        button.setAttribute("data-ui-id", node.id);
        button.textContent = node.label;
        button.disabled = disabled;
        const onClick = () => dispatch(generation, { kind: "clique", id: node.id });
        button.addEventListener("click", onClick);
        cleanupListeners.push(() => button.removeEventListener("click", onClick));
        interactiveControls.push(button);
        return button;
      }
      case "entrada": {
        const input = doc.createElement("input");
        input.type = "text";
        input.className = "ferro-ui-input";
        input.setAttribute("data-ui-kind", "entrada");
        input.setAttribute("data-ui-id", node.id);
        input.value = node.value;
        input.placeholder = node.placeholder;
        if (node.placeholder) {
          input.setAttribute("aria-label", node.placeholder);
        }
        input.disabled = disabled;
        const onInput = () =>
          dispatch(generation, { kind: "entrada_alterada", id: node.id, value: input.value });
        input.addEventListener("input", onInput);
        cleanupListeners.push(() => input.removeEventListener("input", onInput));
        interactiveControls.push(input);
        if (!inputsById.has(node.id)) {
          inputsById.set(node.id, input);
        }
        return input;
      }
      case "lista": {
        const list = doc.createElement("ul");
        list.className = "ferro-ui-list";
        list.setAttribute("data-ui-kind", "lista");
        for (const item of node.items) {
          const itemElement = doc.createElement("li");
          itemElement.className = "ferro-ui-list-item";
          itemElement.appendChild(buildElement(item, generation, inputsById));
          list.appendChild(itemElement);
        }
        return list;
      }
    }
  }

  function renderValidated(node: UiNode) {
    if (destroyed) return;
    const savedFocus = captureFocusedInput(container);
    detachListeners();
    renderGeneration += 1;
    const generation = renderGeneration;
    const inputsById = new Map<string, HTMLInputElement>();
    const rootElement = buildElement(node, generation, inputsById);
    container.replaceChildren(rootElement);

    if (savedFocus) {
      const target = inputsById.get(savedFocus.id);
      if (target && !target.disabled) {
        target.focus();
        if (
          savedFocus.selectionStart !== null &&
          savedFocus.selectionEnd !== null &&
          typeof target.setSelectionRange === "function"
        ) {
          const max = target.value.length;
          target.setSelectionRange(
            Math.min(savedFocus.selectionStart, max),
            Math.min(savedFocus.selectionEnd, max),
          );
        }
      }
    }
  }

  return {
    render(node: UiNode) {
      const validated = validateUiNode(node, limits);
      renderValidated(validated);
    },
    renderJson(raw: string): UiNode {
      const validated = parseUiNodeJson(raw, limits);
      renderValidated(validated);
      return validated;
    },
    setDisabled(nextDisabled: boolean) {
      disabled = nextDisabled;
      for (const control of interactiveControls) {
        control.disabled = nextDisabled;
      }
    },
    destroy() {
      if (destroyed) return;
      destroyed = true;
      renderGeneration += 1;
      detachListeners();
      container.replaceChildren();
    },
  };
}
