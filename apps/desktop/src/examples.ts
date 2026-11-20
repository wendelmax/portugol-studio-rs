import { icon } from "./icons";

const sources = import.meta.glob<string>(
  "../../../tests/corpus/exemplos/{entrada_saida,operacoes_aritmeticas,vetores_matrizes}/*.por",
  { eager: true, query: "?raw", import: "default" },
);
const indexes = import.meta.glob<string>(
  "../../../tests/corpus/exemplos/{entrada_saida,operacoes_aritmeticas,vetores_matrizes}/index.properties",
  { eager: true, query: "?raw", import: "default" },
);

const categories = {
  entrada_saida: "Entrada e saída",
  operacoes_aritmeticas: "Operações aritméticas",
  vetores_matrizes: "Vetores e matrizes",
};

function normalize(text: string): string {
  return text.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase("pt-BR");
}

export function mountExamples(openExample: (source: string, file: string) => void) {
  const search = document.querySelector<HTMLInputElement>("#example-search")!;
  const container = document.querySelector("#examples")!;
  const empty = document.querySelector<HTMLElement>("#examples-empty")!;

  const groups = Object.entries(categories).map(([directory, title]) => {
    const base = "../../../tests/corpus/exemplos/" + directory;
    const properties = new Map(
      [...indexes[base + "/index.properties"].matchAll(/^(item\d+\.\w+)\s*=\s*(.+)$/gm)]
        .map(([, key, value]) => [key, value.trim()]),
    );
    const group = document.createElement("details");
    const summary = document.createElement("summary");
    summary.append(icon("pequeno/folder_closed"), title);
    group.append(summary);
    group.open = directory === "entrada_saida";

    const entries = [...properties].filter(([key]) => key.endsWith(".file")).flatMap(([key, file]) => {
      const source = sources[base + "/" + file];
      if (source === undefined) return [];
      const prefix = key.slice(0, -".file".length);
      const name = properties.get(prefix + ".name") ?? file;
      const description = properties.get(prefix + ".description") ?? "";
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = name;
      button.title = description;
      button.onclick = () => openExample(source, file);
      group.append(button);
      return [{ button, searchable: normalize([title, name, file, description].join(" ")) }];
    });
    container.append(group);
    return { group, entries };
  });

  search.oninput = () => {
    const query = normalize(search.value.trim());
    for (const { group, entries } of groups) {
      for (const { button, searchable } of entries) button.hidden = !searchable.includes(query);
      group.hidden = entries.every(({ button }) => button.hidden);
      if (query && !group.hidden) group.open = true;
    }
    empty.hidden = groups.some(({ group }) => !group.hidden);
  };
}
