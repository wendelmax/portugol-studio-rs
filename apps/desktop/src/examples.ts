import * as api from "./api";
import { icon } from "./icons";
import { buildPorsExampleCatalog, PORS_EXAMPLE_PREFIX } from "./pors-catalog";

const normalize = (text: string) => text.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase("pt-BR");

type Example = Extract<api.CatalogEntry, { kind: "file" }>;

export function mountExamples(openExample: (source: string, file: string, workPath: string) => void) {
  const search = document.querySelector<HTMLInputElement>("#example-search")!;
  const container = document.querySelector("#examples")!;
  const empty = document.querySelector<HTMLElement>("#examples-empty")!;
  const preview = document.querySelector<HTMLElement>("#home-preview")!;
  const title = document.querySelector<HTMLElement>("#example-title")!;
  const description = document.querySelector<HTMLElement>("#example-description")!;
  const image = document.querySelector<HTMLImageElement>("#example-image")!;
  const open = document.querySelector<HTMLButtonElement>("#example-open")!;
  const error = document.querySelector<HTMLElement>("#example-error")!;
  const expanded = new Set<string>();
  let catalog: api.CatalogEntry[] = [];
  const porsExamplesById = new Map<string, api.PorsExampleInfo>();
  let selected: Example | null = null;
  let imageUrl: string | null = null;

  function updateSelection() {
    for (const button of container.querySelectorAll<HTMLButtonElement>("button[data-example]")) {
      button.setAttribute("aria-current", String(button.dataset.example === selected?.file));
    }
  }

  function showExample(example: Example | null) {
    selected = example;
    preview.hidden = !example;
    document.querySelector(".home-brand")?.classList.toggle("has-preview", Boolean(example));
    updateSelection();
    if (imageUrl) URL.revokeObjectURL(imageUrl);
    imageUrl = null;
    image.removeAttribute("src");
    image.hidden = true;
    error.hidden = true;
    if (!example) return;
    title.textContent = example.name;
    description.textContent = example.description;
    description.hidden = !example.description;
  }

  function renderEntries(entries: api.CatalogEntry[], query: string, path = "", parentMatches = false): HTMLElement[] {
    return entries.flatMap<HTMLElement>((entry) => {
      const key = path + "/" + entry.name;
      const matches = parentMatches || normalize(entry.name).includes(query);
      if (entry.kind === "folder") {
        const children = renderEntries(entry.entries, query, key, matches);
        if (!children.length) return [];
        const group = document.createElement("details");
        const summary = document.createElement("summary");
        summary.append(icon("pequeno/folder_closed"), entry.name);
        group.append(summary, ...children);
        group.open = Boolean(query) || expanded.has(key);
        group.ontoggle = () => {
          if (query) return;
          if (group.open) expanded.add(key);
          else expanded.delete(key);
        };
        return [group];
      }
      if (!matches && !normalize(entry.file + " " + entry.description).includes(query)) return [];
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.example = entry.file;
      button.textContent = entry.name;
      button.title = entry.description;
      button.onclick = () => showExample(entry);
      button.ondblclick = () => open.click();
      return [button];
    });
  }

  function render() {
    const query = normalize(search.value.trim());
    const entries = renderEntries(catalog, query);
    container.replaceChildren(...entries);
    empty.hidden = Boolean(entries.length);
    empty.textContent = catalog.length ? "Nenhum exemplo encontrado." : "Não há exemplos no catálogo.";
    updateSelection();
  }

  open.onclick = async () => {
    const example = selected;
    if (!example) return;
    open.disabled = true;
    error.hidden = true;
    try {
      if (!example.file.startsWith(PORS_EXAMPLE_PREFIX)) {
        throw new Error(`Exemplo fora do catálogo Portugol-RS: ${example.file}`);
      }
      const id = example.file.slice(PORS_EXAMPLE_PREFIX.length);
      const porsExample = porsExamplesById.get(id);
      if (!porsExample) {
        throw new Error(`Exemplo Portugol-RS não encontrado: ${id}`);
      }
      openExample(porsExample.source, porsExample.file_name, "");
    } catch (failure) {
      error.textContent = String(failure);
      error.hidden = false;
    } finally {
      open.disabled = false;
    }
  };

  document.querySelector<HTMLButtonElement>("#example-back")!.onclick = () => showExample(null);
  void api.listPorsExamples().then((items) => {
    porsExamplesById.clear();
    for (const item of items) {
      porsExamplesById.set(item.id, item);
    }
    catalog = buildPorsExampleCatalog(items);
    search.disabled = false;
    search.oninput = render;
    render();
  }).catch((reason) => {
    empty.hidden = false;
    empty.textContent = `Não foi possível carregar os exemplos: ${reason}`;
  });
}
