import type { SymbolInfo, SymbolKind } from "./api";

export const symbolKindLabels: Record<SymbolKind, string> = {
  library: "Biblioteca",
  function: "Função",
  constant: "Constante",
  variable: "Variável",
  parameter: "Parâmetro",
  array: "Vetor",
  matrix: "Matriz",
};

const types = ["inteiro", "caracter", "real", "cadeia", "logico", "vazio"];
const typeLabels: Record<string, string> = { logico: "Lógico" };
const normalize = (text: string) => text.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();

export function mountOutline(jump: (offset: number) => void, rename: (offset: number) => void) {
  const filter = document.querySelector<HTMLInputElement>("#outline-search")!;
  const type = document.querySelector<HTMLSelectElement>("#outline-type")!;
  const kind = document.querySelector<HTMLSelectElement>("#outline-kind")!;
  const options = document.querySelector<HTMLDetailsElement>("#outline-options")!;
  const tree = document.querySelector<HTMLUListElement>("#outline-tree")!;
  const empty = document.querySelector<HTMLElement>("#outline-empty")!;
  let symbols: SymbolInfo[] = [];
  let collapsed = new Set<string>();

  for (const key of types) type.add(new Option(typeLabels[key] ?? key, key));
  for (const [key, label] of Object.entries(symbolKindLabels)) kind.add(new Option(label, key));

  function entry(symbol: SymbolInfo, query: string, parent = ""): HTMLLIElement | null {
    const key = parent + ":" + symbol.name + ":" + symbol.kind;
    const children = symbol.children.flatMap((child) => entry(child, query, key) ?? []);
    const matchesText = !query || normalize(symbol.name + " " + symbolKindLabels[symbol.kind] + " " + symbol.ty).includes(query);
    const matches = matchesText && (!type.value || type.value === symbol.ty) &&
      (!kind.value || kind.value === symbol.kind);
    if (!matches && !children.length) return null;

    const item = document.createElement("li");
    const row = document.createElement("div");
    row.className = "outline-row";
    const link = document.createElement("button");
    link.className = "outline-symbol";
    const label = document.createElement("span");
    label.className = "outline-kind";
    label.textContent = symbolKindLabels[symbol.kind];
    const name = document.createElement("span");
    name.textContent = symbol.name;
    const annotation = document.createElement("small");
    annotation.textContent = symbol.ty + (symbol.kind === "array" ? "[]" : symbol.kind === "matrix" ? "[][]" : "");
    link.append(label, name, annotation);
    link.title = `Linha ${symbol.position.line}, coluna ${symbol.position.column}`;
    link.onclick = () => jump(symbol.position.offset);
    if (symbol.kind !== "library") link.ondblclick = () => rename(symbol.position.offset);

    if (children.length) {
      const expand = document.createElement("button");
      expand.className = "outline-expand";
      const nested = document.createElement("ul");
      nested.append(...children);
      nested.hidden = collapsed.has(key) && !query && !kind.value && !type.value;
      const update = () => {
        expand.textContent = nested.hidden ? "▸" : "▾";
        expand.setAttribute("aria-expanded", String(!nested.hidden));
        expand.setAttribute("aria-label", `${nested.hidden ? "Expandir" : "Recolher"} ${symbol.name}`);
      };
      expand.onclick = () => {
        nested.hidden = !nested.hidden;
        if (nested.hidden) collapsed.add(key);
        else collapsed.delete(key);
        update();
      };
      update();
      row.append(expand, link);
      item.append(row, nested);
    } else {
      row.append(link);
      item.append(row);
    }
    return item;
  }

  function render() {
    const query = normalize(filter.value.trim());
    const entries = symbols.flatMap((symbol) => entry(symbol, query) ?? []);
    tree.replaceChildren(...entries);
    options.classList.toggle("has-filters", Boolean(type.value || kind.value));
    empty.hidden = Boolean(entries.length);
    empty.textContent = symbols.length ? "Nenhum símbolo encontrado." : "O programa ainda não tem símbolos.";
  }

  function setFilter(select: HTMLSelectElement, value: string) {
    select.value = select.value === value ? "" : value;
    render();
  }

  filter.oninput = type.onchange = kind.onchange = render;
  return {
    focus: () => filter.focus(),
    show(symbolsForTab: SymbolInfo[], tabChanged = false) {
      if (tabChanged) collapsed = new Set();
      symbols = symbolsForTab;
      render();
    },
    filterType: (value: string) => setFilter(type, value),
    filterKind: (value: SymbolKind) => setFilter(kind, value),
  };
}
