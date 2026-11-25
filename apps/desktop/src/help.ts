import libraries from "./help-assets/bibliotecas.json";
import {
  PORS_HELP_DEFAULT_TOPIC,
  PORS_HELP_LIBRARIES,
  PORS_HELP_TOPICS,
  filterPorsHelp,
} from "./pors-help";

const html = import.meta.glob<string>("./help-assets/topicos/**/*.html", {
  eager: true, query: "?raw", import: "default",
});
const examples = import.meta.glob<string>("./help-assets/recursos/exemplos-pors/**/*.pors", {
  eager: true, query: "?raw", import: "default",
});
const images = import.meta.glob<string>("./help-assets/recursos/imagens/**/*.png", {
  eager: true, query: "?url", import: "default",
});

const parser = new DOMParser();
const allowedTags = new Set([
  "a", "b", "div", "h1", "h2", "hr", "img", "li", "p", "span", "table", "tbody", "td", "th", "thead", "tr", "ul",
]);

function normalize(value: string) {
  return value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase("pt-BR");
}

interface LibraryResource {
  name: string;
  signature: string;
  description: string;
  parameters?: { name: string; description: string }[];
  returns?: string;
}

function paragraph(text: string) {
  const element = document.createElement("p");
  element.textContent = text;
  return element;
}

export function mountHelp(openExample: (source: string, file: string) => void) {
  const search = document.querySelector<HTMLInputElement>("#help-search")!;
  const navigation = document.querySelector<HTMLElement>("#help-topics")!;
  const article = document.querySelector<HTMLElement>("#help-article")!;
  const empty = document.querySelector<HTMLElement>("#help-empty")!;
  const topics = Object.entries(html).map(([file, text]) => {
    const id = file.slice("./help-assets/".length);
    const doc = parser.parseFromString(text, "text/html");
    return { id, title: doc.title, body: doc.body };
  });
  let selected = PORS_HELP_DEFAULT_TOPIC;

  function choose(id: string) {
    selected = id;
    renderArticle();
    if (search.value) {
      search.value = "";
      renderNavigation();
    } else {
      updateSelection();
    }
  }

  function renderNode(node: Node, topic: string): Node | null {
    if (node.nodeType === Node.TEXT_NODE) return document.createTextNode(node.textContent ?? "");
    if (!(node instanceof Element) || !allowedTags.has(node.localName)) return null;

    const asset = (path: string) => decodeURI(new URL(path, "https://ajuda.invalid/" + topic).pathname.slice(1));
    const example = node.getAttribute("data-file");
    if (node.classList.contains("codigo-portugol") && example) {
      const path = asset(example);
      const source = examples["./help-assets/" + path];
      if (source === undefined) return null;
      const block = document.createElement("section");
      const title = document.createElement("h3");
      title.textContent = node.getAttribute("data-title") || "Exemplo";
      const code = document.createElement("pre");
      code.textContent = source.trimEnd();
      block.append(title, code);
      if (node.getAttribute("data-type") === "exemplo") {
        const button = document.createElement("button");
        button.textContent = "Abrir no editor (.pors)";
        button.onclick = () => openExample(source, path.split("/").pop() ?? "exemplo.pors");
        block.append(button);
      }
      return block;
    }

    const element = document.createElement(node.localName);
    if (node.localName === "a") {
      const href = node.getAttribute("href");
      if (href && topics.some(({ id }) => id === href)) {
        element.setAttribute("href", "#" + href);
        element.onclick = (event) => {
          event.preventDefault();
          choose(href);
          article.focus();
        };
      }
    } else if (node.localName === "img") {
      const src = node.getAttribute("src");
      if (!src) return null;
      const prefix = "./help-assets/";
      const image = element as HTMLImageElement;
      const dark = src.includes("${tema}");
      const darkPath = prefix + asset(src.replace("${tema}", "Dark"));
      const lightPath = prefix + asset(src.replace("${tema}", "Portugol"));
      const light = images[lightPath];
      if (!light) return null;
      image.alt = node.getAttribute("alt") ?? "";
      image.src = light;
      if (dark && images[darkPath]) {
        image.dataset.helpDark = images[darkPath];
        image.dataset.helpLight = light;
      }
    }
    for (const attribute of ["colspan", "rowspan"]) {
      const value = node.getAttribute(attribute);
      if (value && /^\d+$/.test(value)) element.setAttribute(attribute, value);
    }
    for (const child of node.childNodes) {
      const rendered = renderNode(child, topic);
      if (rendered) element.append(rendered);
    }
    return element;
  }

  function renderLibrary(name: string, description: string, resources: LibraryResource[], member?: string) {
    const header = document.createElement("h1");
    header.textContent = name;
    article.append(header, paragraph(description));
    for (const resource of resources) {
      if (member && member !== resource.name) continue;
      const section = document.createElement("section");
      const title = document.createElement("h2");
      title.textContent = resource.name;
      const signature = document.createElement("pre");
      signature.textContent = resource.signature;
      section.append(title, signature, paragraph(resource.description));
      if (resource.parameters) {
        const parameters = document.createElement("dl");
        for (const { name, description } of resource.parameters) {
          const parameter = document.createElement("dt");
          const details = document.createElement("dd");
          parameter.textContent = name;
          details.textContent = description;
          parameters.append(parameter, details);
        }
        section.append(parameters);
      }
      if (resource.returns) section.append(paragraph("Retorno — " + resource.returns));
      article.append(section);
    }
  }

  function renderArticle() {
    article.replaceChildren();
    const porsTopic = PORS_HELP_TOPICS.find(({ id }) => id === selected);
    if (porsTopic) {
      const header = document.createElement("h1");
      header.textContent = porsTopic.title;
      article.append(header, paragraph(porsTopic.summary));
      for (const text of porsTopic.paragraphs) {
        article.append(paragraph(text));
      }
      if (porsTopic.example) {
        const { title, fileName, source } = porsTopic.example;
        const block = document.createElement("section");
        const heading = document.createElement("h3");
        heading.textContent = title;
        const code = document.createElement("pre");
        code.textContent = source.trimEnd();
        const button = document.createElement("button");
        button.textContent = "Abrir no editor (.pors)";
        button.onclick = () => openExample(source, fileName);
        block.append(heading, code, button);
        article.append(block);
      }
      refreshTheme();
      article.scrollTop = 0;
      return;
    }
    const porsLibrary = PORS_HELP_LIBRARIES.find(({ id }) => id === selected);
    if (porsLibrary) {
      renderLibrary(
        porsLibrary.name,
        porsLibrary.description,
        porsLibrary.items.map((item) => ({
          name: item.name,
          signature: item.signature,
          description: item.description,
        })),
      );
      refreshTheme();
      article.scrollTop = 0;
      return;
    }
    const topic = topics.find(({ id }) => id === selected);
    if (topic) {
      for (const child of topic.body.childNodes) {
        const rendered = renderNode(child, topic.id);
        if (rendered) article.append(rendered);
      }
    } else {
      const [name, member] = selected.split(".");
      const library = libraries[name as keyof typeof libraries];
      if (library) renderLibrary(name, library.description, library.resources, member);
    }
    refreshTheme();
    article.scrollTop = 0;
  }

  function updateSelection() {
    for (const button of navigation.querySelectorAll("button[data-topic]")) {
      button.setAttribute("aria-current", String((button as HTMLButtonElement).dataset.topic === selected));
    }
  }

  function topicButton(id: string, title: string, depth = 0) {
    const button = document.createElement("button");
    button.textContent = title;
    button.dataset.topic = id;
    button.style.paddingInlineStart = `${8 + depth * 12}px`;
    button.onclick = () => choose(id);
    return button;
  }

  function renderNavigation() {
    const query = normalize(search.value.trim());
    const porsMatches = filterPorsHelp(search.value);
    const porsFragment = document.createDocumentFragment();
    for (const topic of porsMatches.topics) {
      porsFragment.append(topicButton(topic.id, topic.title));
    }
    for (const lib of porsMatches.libraries) {
      porsFragment.append(topicButton(lib.id, lib.name));
    }
    const language = document.createDocumentFragment();
    const entries = document.createDocumentFragment();
    for (const { id, title, body } of topics) {
      if (query && !normalize(title + " " + body.textContent).includes(query)) continue;
      language.append(topicButton(id, title, query ? 0 : id.split("/").length - 3));
    }
    for (const [name, { description, resources }] of Object.entries(libraries)) {
      if (!query || normalize(name + " " + description).includes(query)) {
        entries.append(topicButton(name, name));
      }
      if (!query) continue;
      for (const resource of resources) {
        if (normalize(name + " " + resource.name + " " + resource.description).includes(query)) {
          entries.append(topicButton(name + "." + resource.name, name + "." + resource.name, 1));
        }
      }
    }

    navigation.replaceChildren();
    for (const [label, entriesList] of [
      ["Portugol-RS (PORS)", porsFragment],
      ["Guia da linguagem (.pors)", language],
      ["Bibliotecas clássicas", entries],
    ] as const) {
      if (!entriesList.childNodes.length) continue;
      const heading = document.createElement("h2");
      heading.textContent = label;
      navigation.append(heading, entriesList);
    }
    empty.hidden = navigation.childNodes.length > 0;
    updateSelection();
  }

  function refreshTheme() {
    const theme = document.documentElement.dataset.theme;
    const dark = theme === "dark" || !theme && window.matchMedia("(prefers-color-scheme: dark)").matches;
    for (const image of article.querySelectorAll<HTMLImageElement>("img[data-help-dark]")) {
      const src = dark ? image.dataset.helpDark : image.dataset.helpLight;
      if (src) image.src = src;
    }
  }

  search.oninput = renderNavigation;
  renderNavigation();
  renderArticle();

  return { refreshTheme, focusSearch: () => search.focus() };
}
