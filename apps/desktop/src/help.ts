import {
  PORS_HELP_DEFAULT_TOPIC,
  PORS_HELP_LIBRARIES,
  PORS_HELP_TOPICS,
  filterPorsHelp,
  type PorsHelpLibrary,
} from "./pors-help";
import { parsePorsSignature } from "./pors-stdlib-catalog";
import {
  buildSyntaxTopicTree,
  createHelpHistory,
  parseInlineCodeRuns,
  tokenizePorsCode,
  type HelpTreeNode,
} from "./help-navigation";

const html = import.meta.glob<string>("./help-assets/topicos/**/*.html", {
  eager: true, query: "?raw", import: "default",
});
const examples = import.meta.glob<string>("./help-assets/recursos/exemplos-pors/**/*.fer", {
  eager: true, query: "?raw", import: "default",
});
const images = import.meta.glob<string>("./help-assets/recursos/imagens/**/*.png", {
  eager: true, query: "?url", import: "default",
});

const parser = new DOMParser();
const allowedTags = new Set([
  "a", "b", "code", "div", "h1", "h2", "hr", "img", "li", "p", "span", "table", "tbody", "td", "th", "thead", "tr", "ul",
]);

function normalize(value: string) {
  return value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase("pt-BR");
}

function appendRichText(target: HTMLElement, text: string) {
  for (const run of parseInlineCodeRuns(text)) {
    if (run.code) {
      const code = document.createElement("code");
      code.textContent = run.text;
      target.append(code);
    } else {
      target.append(document.createTextNode(run.text));
    }
  }
}

function paragraph(text: string) {
  const element = document.createElement("p");
  appendRichText(element, text);
  return element;
}

function renderHighlightedCode(source: string): HTMLPreElement {
  const pre = document.createElement("pre");
  pre.className = "help-code";
  const code = document.createElement("code");
  for (const token of tokenizePorsCode(source.trimEnd())) {
    if (token.kind === "plain") {
      code.append(document.createTextNode(token.text));
    } else {
      const span = document.createElement("span");
      span.className = `hl-${token.kind}`;
      span.textContent = token.text;
      code.append(span);
    }
  }
  pre.append(code);
  return pre;
}

export interface MountHelpOptions {
  openExample: (source: string, file: string) => void;
  runExample?: (source: string, file: string) => void;
  openOwnershipLab?: () => void;
}

export function mountHelp(optionsOrOpen: MountHelpOptions | ((source: string, file: string) => void)) {
  const options: MountHelpOptions = typeof optionsOrOpen === "function"
    ? { openExample: optionsOrOpen }
    : optionsOrOpen;
  const search = document.querySelector<HTMLInputElement>("#help-search")!;
  const searchClear = document.querySelector<HTMLButtonElement>("#help-search-clear");
  const searchCount = document.querySelector<HTMLElement>("#help-search-count");
  const navigation = document.querySelector<HTMLElement>("#help-topics")!;
  const article = document.querySelector<HTMLElement>("#help-article")!;
  const empty = document.querySelector<HTMLElement>("#help-empty")!;
  const backButton = document.querySelector<HTMLButtonElement>("#help-history-back");
  const forwardButton = document.querySelector<HTMLButtonElement>("#help-history-forward");
  const homeTopicButton = document.querySelector<HTMLButtonElement>("#help-home-topic");
  const breadcrumbs = document.querySelector<HTMLElement>("#help-breadcrumbs");

  const topics = Object.entries(html).map(([file, text]) => {
    const id = file.slice("./help-assets/".length);
    const doc = parser.parseFromString(text, "text/html");
    return { id, title: doc.title, body: doc.body };
  });
  const topicById = new Map(topics.map((topic) => [topic.id, topic]));
  const syntaxTree = buildSyntaxTopicTree(topics);
  const history = createHelpHistory(PORS_HELP_DEFAULT_TOPIC);

  function selectedId() {
    return history.current();
  }

  function syncHistoryControls() {
    if (backButton) backButton.disabled = !history.canBack();
    if (forwardButton) forwardButton.disabled = !history.canForward();
  }

  function buildCodeBlock(titleText: string, source: string, fileName?: string, executable = false) {
    const block = document.createElement("section");
    block.className = "help-code-block";
    const header = document.createElement("div");
    header.className = "help-code-header";
    const title = document.createElement("h3");
    title.textContent = titleText;
    const actions = document.createElement("div");
    actions.className = "help-code-actions";

    const copyBtn = document.createElement("button");
    copyBtn.type = "button";
    copyBtn.textContent = "Copiar";
    copyBtn.onclick = () => {
      void navigator.clipboard?.writeText(source).then(() => {
        copyBtn.textContent = "Copiado!";
        window.setTimeout(() => { copyBtn.textContent = "Copiar"; }, 1400);
      }).catch(() => {});
    };
    actions.append(copyBtn);

    if (executable && fileName) {
      const openBtn = document.createElement("button");
      openBtn.type = "button";
      openBtn.textContent = "Abrir no editor (.fer)";
      openBtn.onclick = () => options.openExample(source, fileName);
      actions.append(openBtn);

      if (options.runExample) {
        const runBtn = document.createElement("button");
        runBtn.type = "button";
        runBtn.className = "primary";
        runBtn.textContent = "Abrir e executar";
        runBtn.onclick = () => options.runExample?.(source, fileName);
        actions.append(runBtn);
      }
    }

    header.append(title, actions);
    block.append(header, renderHighlightedCode(source));
    return block;
  }

  function choose(id: string, recordHistory = true) {
    if (recordHistory) history.visit(id);
    syncHistoryControls();
    renderArticle();
    renderBreadcrumbs();
    updateSelection();
    revealActiveInTree();
  }

  function navigateTo(id: string) {
    choose(id, true);
    article.focus();
  }

  function renderBreadcrumbs() {
    if (!breadcrumbs) return;
    breadcrumbs.replaceChildren();
    const current = selectedId();
    const crumbs: { label: string; id?: string }[] = [{ label: "Ajuda", id: PORS_HELP_DEFAULT_TOPIC }];

    const porsTopic = PORS_HELP_TOPICS.find(({ id }) => id === current);
    if (porsTopic) {
      if (porsTopic.id !== PORS_HELP_DEFAULT_TOPIC) {
        crumbs.push({ label: "Primeiros passos", id: PORS_HELP_DEFAULT_TOPIC });
        crumbs.push({ label: porsTopic.title });
      }
    } else if (current.startsWith("pors-lib:")) {
      const [libraryId, member] = current.split(".");
      const lib = PORS_HELP_LIBRARIES.find(({ id }) => id === libraryId);
      if (lib) {
        crumbs.push({ label: lib.group === "standard" ? "Bibliotecas padrão" : "UI e Multimídia" });
        const shortName = lib.name.replace(/^PORS:\s*/, "");
        if (member) {
          crumbs.push({ label: shortName, id: lib.id });
          crumbs.push({ label: member });
        } else {
          crumbs.push({ label: shortName });
        }
      }
    } else if (topicById.has(current)) {
      crumbs.push({ label: "Referência da sintaxe", id: "topicos/linguagem_portugol/index.html" });
      const parts = current.split("/");
      // Adiciona diretórios intermediários que possuam index.html
      for (let i = 3; i < parts.length - 1; i += 1) {
        const dirIndex = [...parts.slice(0, i + 1), "index.html"].join("/");
        const parent = topicById.get(dirIndex);
        if (parent && parent.id !== current) crumbs.push({ label: parent.title, id: parent.id });
      }
      const leaf = topicById.get(current)!;
      if (leaf.id !== "topicos/linguagem_portugol/index.html") {
        crumbs.push({ label: leaf.title });
      }
    }

    crumbs.forEach((crumb, idx) => {
      if (idx > 0) {
        const sep = document.createElement("span");
        sep.className = "help-crumb-sep";
        sep.setAttribute("aria-hidden", "true");
        sep.textContent = "›";
        breadcrumbs.append(sep);
      }
      if (crumb.id && idx < crumbs.length - 1) {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "help-crumb-btn";
        btn.textContent = crumb.label;
        btn.onclick = () => choose(crumb.id!);
        breadcrumbs.append(btn);
      } else {
        const span = document.createElement("span");
        span.className = "help-crumb-current";
        span.textContent = crumb.label;
        breadcrumbs.append(span);
      }
    });
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
      return buildCodeBlock(
        node.getAttribute("data-title") || "Exemplo",
        source,
        path.split("/").pop() ?? "exemplo.fer",
        node.getAttribute("data-type") === "exemplo",
      );
    }

    const element = document.createElement(node.localName);
    if (node.localName === "a") {
      const href = node.getAttribute("href");
      if (href && topicById.has(href)) {
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

  function renderMemberCard(item: PorsHelpLibrary["items"][number], libId: string) {
    const section = document.createElement("section");
    section.className = "help-member-card";
    section.dataset.memberSearch = normalize(`${item.name} ${item.signature} ${item.description}`);

    const headingRow = document.createElement("div");
    headingRow.className = "help-member-heading";
    const title = document.createElement("h3");
    const link = document.createElement("button");
    link.type = "button";
    link.className = "help-member-link";
    link.textContent = item.name;
    link.onclick = () => choose(`${libId}.${item.name}`);
    title.append(link);
    const badge = document.createElement("span");
    badge.className = `help-kind-badge help-kind-${item.kind}`;
    badge.textContent = item.kind === "constant" ? "constante" : "função";
    headingRow.append(title, badge);

    const sig = renderHighlightedCode(item.signature);
    sig.classList.add("help-signature");
    section.append(headingRow, sig, paragraph(item.description));

    const details = parsePorsSignature(item.signature);
    if (details.parameters.length > 0) {
      const paramTitle = document.createElement("h4");
      paramTitle.className = "help-sublabel";
      paramTitle.textContent = "Parâmetros";
      const dl = document.createElement("dl");
      dl.className = "help-params";
      for (const param of details.parameters) {
        const dt = document.createElement("dt");
        const code = document.createElement("code");
        code.textContent = param.name;
        dt.append(code);
        const dd = document.createElement("dd");
        const ty = document.createElement("code");
        ty.textContent = param.type;
        dd.append(ty);
        dl.append(dt, dd);
      }
      section.append(paramTitle, dl);
    }
    if (details.returns) {
      const ret = document.createElement("p");
      ret.className = "help-return";
      const strong = document.createElement("b");
      strong.textContent = item.kind === "constant" ? "Tipo: " : "Retorno: ";
      const code = document.createElement("code");
      code.textContent = details.returns;
      ret.append(strong, code);
      section.append(ret);
    }
    return section;
  }

  function renderLibrary(lib: PorsHelpLibrary, member?: string) {
    const shortName = lib.name.replace(/^PORS:\s*/, "");
    const header = document.createElement("h1");
    header.textContent = member ? `${shortName}.${member}` : lib.name;
    article.append(header);

    if (member) {
      const back = document.createElement("button");
      back.type = "button";
      back.className = "help-back-library";
      back.textContent = `← Ver todos os itens de ${shortName}`;
      back.onclick = () => choose(lib.id);
      article.append(back);
      const found = lib.items.find((item) => item.name === member);
      if (found) {
        article.append(renderMemberCard(found, lib.id));
      } else {
        article.append(paragraph(`Item \`${member}\` não encontrado em \`${shortName}\`.`));
      }
      return;
    }

    const constants = lib.items.filter((item) => item.kind === "constant");
    const functions = lib.items.filter((item) => item.kind === "function");

    const meta = document.createElement("p");
    meta.className = "help-lib-meta";
    meta.textContent = `${lib.group === "standard" ? "Biblioteca padrão" : "UI e Multimídia"} · ${functions.length} função(ões)`
      + (constants.length ? ` · ${constants.length} constante(s)` : "");
    article.append(meta, paragraph(lib.description));

    if (lib.example) {
      article.append(buildCodeBlock(lib.example.title, lib.example.source, lib.example.fileName, true));
    }

    const filterInput = document.createElement("input");
    filterInput.type = "search";
    filterInput.className = "help-lib-filter";
    filterInput.placeholder = `Filtrar membros de ${shortName}…`;
    filterInput.setAttribute("aria-label", `Filtrar membros de ${shortName}`);
    article.append(filterInput);

    const sectionsContainer = document.createElement("div");
    sectionsContainer.className = "help-lib-sections";

    const groups: { title: string; items: readonly PorsHelpLibrary["items"][number][] }[] = [];
    if (functions.length) groups.push({ title: `Funções (${functions.length})`, items: functions });
    if (constants.length) groups.push({ title: `Constantes (${constants.length})`, items: constants });

    const renderedGroups: { wrapper: HTMLElement; cards: HTMLElement[] }[] = [];
    for (const group of groups) {
      const wrapper = document.createElement("section");
      const h2 = document.createElement("h2");
      h2.textContent = group.title;
      wrapper.append(h2);
      const cards: HTMLElement[] = [];
      for (const item of group.items) {
        const card = renderMemberCard(item, lib.id);
        wrapper.append(card);
        cards.push(card);
      }
      sectionsContainer.append(wrapper);
      renderedGroups.push({ wrapper, cards });
    }

    const filterEmpty = document.createElement("p");
    filterEmpty.setAttribute("role", "status");
    filterEmpty.hidden = true;
    filterEmpty.textContent = "Nenhum membro correspondente neste módulo.";
    article.append(sectionsContainer, filterEmpty);

    filterInput.oninput = () => {
      const q = normalize(filterInput.value.trim());
      let visibleTotal = 0;
      for (const { wrapper, cards } of renderedGroups) {
        let groupVisible = 0;
        for (const card of cards) {
          const match = !q || (card.dataset.memberSearch ?? "").includes(q);
          card.hidden = !match;
          if (match) groupVisible += 1;
        }
        wrapper.hidden = groupVisible === 0;
        visibleTotal += groupVisible;
      }
      filterEmpty.hidden = visibleTotal > 0;
    };
  }

  function renderArticle() {
    article.replaceChildren();
    const current = selectedId();
    const porsTopic = PORS_HELP_TOPICS.find(({ id }) => id === current);
    if (porsTopic) {
      const header = document.createElement("h1");
      header.textContent = porsTopic.title;
      article.append(header, paragraph(porsTopic.summary));
      for (const text of porsTopic.paragraphs) {
        article.append(paragraph(text));
      }
      if (porsTopic.comparisonRows?.length) {
        const table = document.createElement("table");
        const thead = document.createElement("thead");
        const headRow = document.createElement("tr");
        for (const col of ["Conceito", "Ferro (.fer)", "Rust (.rs)"]) {
          const th = document.createElement("th");
          th.textContent = col;
          headRow.append(th);
        }
        thead.append(headRow);
        const tbody = document.createElement("tbody");
        for (const [concept, porsCode, rustCode] of porsTopic.comparisonRows) {
          const tr = document.createElement("tr");
          const tdConcept = document.createElement("td");
          tdConcept.textContent = concept;
          const tdPors = document.createElement("td");
          const codePors = document.createElement("code");
          codePors.textContent = porsCode;
          tdPors.append(codePors);
          const tdRust = document.createElement("td");
          const codeRust = document.createElement("code");
          codeRust.textContent = rustCode;
          tdRust.append(codeRust);
          tr.append(tdConcept, tdPors, tdRust);
          tbody.append(tr);
        }
        table.append(thead, tbody);
        article.append(table);
      }
      if (porsTopic.id === "pors:ponte-rust" && options.openOwnershipLab) {
        const labBtn = document.createElement("button");
        labBtn.type = "button";
        labBtn.className = "primary help-action-cta";
        labBtn.textContent = "Abrir Laboratório de Ownership";
        labBtn.onclick = () => options.openOwnershipLab?.();
        article.append(labBtn);
      }
      if (porsTopic.example) {
        article.append(buildCodeBlock(porsTopic.example.title, porsTopic.example.source, porsTopic.example.fileName, true));
      }
      refreshTheme();
      article.scrollTop = 0;
      return;
    }

    if (current.startsWith("pors-lib:")) {
      const [libraryId, libraryMember] = current.split(".");
      const porsLibrary = PORS_HELP_LIBRARIES.find(({ id }) => id === libraryId);
      if (porsLibrary) {
        renderLibrary(porsLibrary, libraryMember);
        refreshTheme();
        article.scrollTop = 0;
        return;
      }
    }

    const topic = topicById.get(current);
    if (topic) {
      for (const child of topic.body.childNodes) {
        const rendered = renderNode(child, topic.id);
        if (rendered) article.append(rendered);
      }
    }
    refreshTheme();
    article.scrollTop = 0;
  }

  function updateSelection() {
    const current = selectedId();
    for (const button of navigation.querySelectorAll<HTMLButtonElement>("button[data-topic]")) {
      button.setAttribute("aria-current", String(button.dataset.topic === current));
    }
  }

  function revealActiveInTree() {
    const current = selectedId();
    const active = navigation.querySelector<HTMLButtonElement>(`button[data-topic="${CSS.escape(current)}"]`);
    if (!active) return;
    let parent = active.parentElement;
    while (parent && parent !== navigation) {
      if (parent instanceof HTMLDetailsElement) parent.open = true;
      parent = parent.parentElement;
    }
    active.scrollIntoView?.({ block: "nearest" });
  }

  function topicButton(id: string, title: string, depth = 0) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = title;
    button.dataset.topic = id;
    button.style.paddingInlineStart = `${8 + depth * 12}px`;
    button.onclick = () => choose(id);
    return button;
  }

  function renderSyntaxNode(node: HelpTreeNode, depth: number): HTMLElement {
    if (!node.children.length) {
      return topicButton(node.id, node.title, depth);
    }
    const details = document.createElement("details");
    details.className = "help-tree-branch";
    const summary = document.createElement("summary");
    summary.className = "help-branch-summary";
    summary.style.paddingInlineStart = `${4 + depth * 10}px`;
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "help-branch-button";
    btn.textContent = node.title;
    btn.dataset.topic = node.id;
    btn.onclick = (event) => {
      event.stopPropagation();
      details.open = true;
      choose(node.id);
    };
    summary.append(btn);
    details.append(summary);
    const list = document.createElement("div");
    list.className = "help-branch-children";
    for (const child of node.children) {
      list.append(renderSyntaxNode(child, depth + 1));
    }
    details.append(list);
    return details;
  }

  function appendLibraryEntries(
    target: DocumentFragment,
    libs: readonly PorsHelpLibrary[],
    query: string,
  ): number {
    let matches = 0;
    for (const lib of libs) {
      const shortName = lib.name.replace(/^PORS:\s*/, "");
      const matchingItems = query
        ? lib.items.filter((item) => normalize(`${shortName} ${item.name} ${item.signature} ${item.description}`).includes(query))
        : lib.items;

      const details = document.createElement("details");
      details.className = "help-tree-branch";
      if (query && matchingItems.length > 0) details.open = true;

      const summary = document.createElement("summary");
      summary.className = "help-branch-summary";
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "help-branch-button";
      btn.textContent = `${shortName} (${lib.items.length})`;
      btn.dataset.topic = lib.id;
      btn.onclick = (event) => {
        event.stopPropagation();
        choose(lib.id);
      };
      summary.append(btn);
      details.append(summary);

      const list = document.createElement("div");
      list.className = "help-branch-children";
      for (const item of matchingItems) {
        list.append(topicButton(`${lib.id}.${item.name}`, item.name, 1));
      }
      details.append(list);
      target.append(details);
      matches += query ? Math.max(1, matchingItems.length) : 1;
    }
    return matches;
  }

  function buildGroup(label: string, entries: DocumentFragment, openByDefault: boolean): HTMLDetailsElement {
    const group = document.createElement("details");
    group.className = "help-group";
    group.open = openByDefault;
    const summary = document.createElement("summary");
    summary.className = "help-group-summary";
    const heading = document.createElement("h2");
    heading.textContent = label;
    summary.append(heading);
    const body = document.createElement("div");
    body.className = "help-group-body";
    body.append(entries);
    group.append(summary, body);
    return group;
  }

  function renderNavigation() {
    const rawQuery = search.value.trim();
    const query = normalize(rawQuery);
    if (searchClear) searchClear.hidden = rawQuery.length === 0;

    let totalMatches = 0;
    const porsMatches = filterPorsHelp(search.value);
    const porsTopics = document.createDocumentFragment();
    for (const topic of porsMatches.topics) {
      porsTopics.append(topicButton(topic.id, topic.title));
      totalMatches += 1;
    }

    const standardLibraries = document.createDocumentFragment();
    totalMatches += appendLibraryEntries(
      standardLibraries,
      porsMatches.libraries.filter((lib) => lib.group === "standard"),
      query,
    );

    const visualLibraries = document.createDocumentFragment();
    totalMatches += appendLibraryEntries(
      visualLibraries,
      porsMatches.libraries.filter((lib) => lib.group === "visual"),
      query,
    );

    const language = document.createDocumentFragment();
    if (query) {
      for (const { id, title, body } of topics) {
        if (!normalize(title + " " + body.textContent).includes(query)) continue;
        language.append(topicButton(id, title, 0));
        totalMatches += 1;
      }
    } else {
      for (const node of syntaxTree) {
        language.append(renderSyntaxNode(node, 0));
      }
    }

    navigation.replaceChildren();
    for (const [label, entriesList, defaultOpen] of [
      ["Primeiros passos (PORS)", porsTopics, true],
      ["Bibliotecas padrão (PORS)", standardLibraries, true],
      ["UI e Multimídia (PORS)", visualLibraries, Boolean(query)],
      ["Referência da sintaxe (.fer)", language, Boolean(query)],
    ] as const) {
      if (!entriesList.childNodes.length) continue;
      navigation.append(buildGroup(label, entriesList, defaultOpen || Boolean(query)));
    }

    if (searchCount) {
      searchCount.hidden = !query;
      if (query) {
        searchCount.textContent = totalMatches === 1 ? "1 resultado" : `${totalMatches} resultados`;
      }
    }
    empty.hidden = navigation.childNodes.length > 0;
    updateSelection();
    if (!query) revealActiveInTree();
  }

  function refreshTheme() {
    const theme = document.documentElement.dataset.theme;
    const dark = theme === "dark" || (!theme && window.matchMedia("(prefers-color-scheme: dark)").matches);
    for (const image of article.querySelectorAll<HTMLImageElement>("img[data-help-dark]")) {
      const src = dark ? image.dataset.helpDark : image.dataset.helpLight;
      if (src) image.src = src;
    }
  }

  if (backButton) {
    backButton.onclick = () => {
      const prev = history.back();
      if (prev) choose(prev, false);
    };
  }
  if (forwardButton) {
    forwardButton.onclick = () => {
      const next = history.forward();
      if (next) choose(next, false);
    };
  }
  if (homeTopicButton) {
    homeTopicButton.onclick = () => choose(PORS_HELP_DEFAULT_TOPIC, true);
  }
  if (searchClear) {
    searchClear.onclick = () => {
      search.value = "";
      renderNavigation();
      search.focus();
    };
  }

  search.oninput = renderNavigation;
  syncHistoryControls();
  renderNavigation();
  renderArticle();
  renderBreadcrumbs();

  return { refreshTheme, focusSearch: () => search.focus(), navigateTo };
}
