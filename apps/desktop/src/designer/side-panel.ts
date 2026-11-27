/**
 * Painel lateral do designer (ADR 0018, Fase 2): a árvore de elementos, com
 * "Envolver em", e o painel de propriedades com as abas *Propriedades* e
 * *Eventos*. Cada mudança vira edição estrutural, como no canvas.
 */
import type { DesignEdit, DesignNode, DesignSchema } from "../api.ts";
import { nodeAt, pathFromKey, type DefaultSizes } from "./canvas-model.ts";
import {
  WRAPPERS,
  anchorsValue,
  elementTreeRows,
  eventRows,
  planWrap,
  propertyEdit,
  propertyRows,
  type EventRow,
  type PropertyRow,
} from "./properties-model.ts";

export interface SidePanelHost {
  schema: () => DesignSchema | null;
  sizes: () => DefaultSizes;
  tree: () => DesignNode | null;
  selected: () => string | null;
  select: (path: string | null, moveCursor: boolean) => void;
  apply: (plan: { edits: DesignEdit[]; select: number[] | null }) => Promise<boolean>;
}

type Tab = "properties" | "events";

function element<K extends keyof HTMLElementTagNameMap>(tag: K, className?: string, text?: string): HTMLElementTagNameMap[K] {
  const created = document.createElement(tag);
  if (className) created.className = className;
  if (text !== undefined) created.textContent = text;
  return created;
}

export function mountSidePanel(container: HTMLElement, host: SidePanelHost) {
  let tab: Tab = "properties";
  let uid = 0;

  const commit = (path: number[], name: string, value: string) =>
    void host.apply({ edits: [propertyEdit(path, name, value)], select: path });

  function propertyEditor(row: PropertyRow, path: number[], id: string): HTMLElement {
    switch (row.editor) {
      case "bool": {
        const select = element("select");
        for (const [value, label] of [["", "(padrão)"], ["verdadeiro", "verdadeiro"], ["falso", "falso"]]) {
          const option = element("option", undefined, label);
          option.value = value;
          select.append(option);
        }
        select.value = row.value;
        select.id = id;
        select.addEventListener("change", () => commit(path, row.name, select.value));
        return select;
      }
      case "choice": {
        const select = element("select");
        for (const value of ["", ...row.choices]) {
          const option = element("option", undefined, value || "(padrão)");
          option.value = value;
          select.append(option);
        }
        select.value = row.choices.includes(row.value) ? row.value : "";
        select.id = id;
        select.addEventListener("change", () => commit(path, row.name, select.value));
        return select;
      }
      case "anchors": {
        const group = element("div", "design-anchors");
        group.id = id;
        group.setAttribute("role", "group");
        const checked = new Set(row.value.split(",").map((part) => part.trim()).filter(Boolean));
        for (const choice of row.choices) {
          const label = element("label");
          const box = element("input");
          box.type = "checkbox";
          box.checked = checked.has(choice);
          box.addEventListener("change", () => {
            if (box.checked) checked.add(choice);
            else checked.delete(choice);
            commit(path, row.name, anchorsValue(row.choices, checked));
          });
          label.append(box, document.createTextNode(choice));
          group.append(label);
        }
        return group;
      }
      default: {
        const input = element("input");
        input.type = row.editor === "number" ? "number" : "text";
        if (row.editor === "number") {
          if (row.min !== null) input.min = String(row.min);
          if (row.max !== null) input.max = String(row.max);
        }
        input.value = row.value;
        input.id = id;
        input.spellcheck = false;
        if (row.bindable && row.editor === "text") input.placeholder = "texto ou {dados.campo}";
        input.addEventListener("change", () => commit(path, row.name, input.value));
        input.addEventListener("keydown", (event) => {
          if (event.key === "Enter") input.blur();
        });
        return input;
      }
    }
  }

  function propertiesList(rows: PropertyRow[], path: number[]): HTMLElement {
    const list = element("div", "design-property-list");
    for (const row of rows) {
      const id = `design-prop-${++uid}`;
      const label = element("label", "design-property-name", row.name + (row.required ? " *" : ""));
      label.htmlFor = id;
      label.title = row.description;
      const editor = propertyEditor(row, path, id);
      editor.dataset.property = row.name;
      editor.title = row.description;
      list.append(label, editor);
    }
    return list;
  }

  function eventsList(rows: EventRow[], path: number[]): HTMLElement {
    const list = element("div", "design-property-list");
    if (rows.length === 0) list.append(element("p", "design-side-empty", "Este elemento não tem eventos."));
    for (const row of rows) {
      const id = `design-event-${++uid}`;
      const label = element("label", "design-property-name", row.name);
      label.htmlFor = id;
      label.title = row.description;
      const line = element("div", "design-event-line");
      const input = element("input");
      input.id = id;
      input.dataset.property = row.name;
      input.value = row.value;
      input.placeholder = row.suggestion;
      input.spellcheck = false;
      input.title = `${row.description}. Nome da função no código da tela.`;
      input.addEventListener("change", () => commit(path, row.name, input.value));
      input.addEventListener("keydown", (event) => {
        if (event.key === "Enter") input.blur();
      });
      const suggest = element("button", undefined, "Sugerir");
      suggest.type = "button";
      suggest.title = `Usar ${row.suggestion}`;
      suggest.disabled = row.value === row.suggestion;
      suggest.addEventListener("click", () => commit(path, row.name, row.suggestion));
      line.append(input, suggest);
      list.append(label, line);
    }
    return list;
  }

  function structure(tree: DesignNode, selected: string | null): HTMLElement {
    const section = element("section", "design-structure");
    section.append(element("h2", "design-side-title", "Estrutura"));
    const list = element("div", "design-tree");
    list.setAttribute("role", "tree");
    list.setAttribute("aria-label", "Elementos da tela");
    for (const row of elementTreeRows(tree)) {
      const item = element("button", "design-tree-item", row.label);
      item.type = "button";
      item.setAttribute("role", "treeitem");
      item.setAttribute("aria-level", String(row.depth + 1));
      item.setAttribute("aria-selected", String(row.path === selected));
      item.style.paddingInlineStart = `${6 + row.depth * 12}px`;
      item.dataset.path = row.path;
      item.addEventListener("click", () => host.select(row.path, true));
      list.append(item);
    }
    section.append(list);

    const path = selected === null ? null : pathFromKey(selected);
    const wrap = element("div", "design-wrap");
    wrap.append(element("span", undefined, "Envolver em"));
    for (const wrapper of WRAPPERS) {
      const button = element("button", undefined, wrapper);
      button.type = "button";
      const plan = path ? planWrap(tree, path, wrapper, host.sizes()) : null;
      button.disabled = plan === null;
      button.title = plan ? `Coloca o elemento selecionado dentro de um ${wrapper} novo` : "Selecione um controle para envolver";
      button.addEventListener("click", () => {
        if (plan) void host.apply(plan);
      });
      wrap.append(button);
    }
    section.append(wrap);
    return section;
  }

  function properties(schema: DesignSchema, tree: DesignNode, selected: string | null): HTMLElement {
    const section = element("section", "design-properties");
    const path = selected === null ? [] : pathFromKey(selected);
    const node = nodeAt(tree, path);
    const name = node?.attributes.find((attribute) => attribute.name === "Nome")?.value;
    section.append(element("h2", "design-side-title", node ? (name ? `${name} (${node.tag})` : node.tag) : "Propriedades"));
    if (!node) return section;

    const tabs = element("div", "design-side-tabs");
    tabs.setAttribute("role", "tablist");
    const body = element("div", "design-side-body");
    body.setAttribute("role", "tabpanel");
    for (const [value, label] of [["properties", "Propriedades"], ["events", "Eventos"]] as const) {
      const button = element("button", undefined, label);
      button.type = "button";
      button.setAttribute("role", "tab");
      button.setAttribute("aria-selected", String(tab === value));
      button.addEventListener("click", () => {
        tab = value;
        refresh();
      });
      tabs.append(button);
    }
    body.append(tab === "properties" ? propertiesList(propertyRows(schema, tree, path), path) : eventsList(eventRows(schema, tree, path), path));
    section.append(tabs, body);
    return section;
  }

  function refresh() {
    const schema = host.schema();
    const tree = host.tree();
    if (!schema || !tree) {
      container.replaceChildren();
      return;
    }
    // Mantém o foco no mesmo campo depois de redesenhar.
    const focused = (document.activeElement as HTMLElement | null)?.dataset?.property;
    const selected = host.selected() ?? "";
    container.replaceChildren(structure(tree, selected), properties(schema, tree, selected));
    if (focused) container.querySelector<HTMLElement>(`[data-property="${CSS.escape(focused)}"]`)?.focus();
  }

  return { refresh };
}
