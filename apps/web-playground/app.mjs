import {
  PLAYGROUND_EXAMPLES,
  loadPlaygroundRuntime,
  renderUiNodeTree,
} from "./playground-core.mjs";

const byId = (id) => document.getElementById(id);

const statusEl = byId("pg-status");
const statusTextEl = byId("pg-status-text");
const themeButton = byId("pg-theme");
const exampleSelect = byId("pg-example");
const editorEl = byId("pg-editor");
const highlightEl = byId("pg-highlight");
const stdinEl = byId("pg-stdin");
const outputEl = byId("pg-output");
const outputEmptyEl = byId("pg-output-empty");
const outputBadgeEl = byId("pg-output-badge");
const diagnosticsEl = byId("pg-diagnostics");
const uiContainerEl = byId("pg-ui-container");
const uiSectionEl = byId("pg-ui-section");
const rustSectionEl = byId("pg-rust-section");
const rustCodeEl = byId("pg-rust-code");
const btnCopyRust = byId("pg-copy-rust");
const btnValidate = byId("pg-validate");
const btnFormat = byId("pg-format");
const btnRun = byId("pg-run");
const btnUi = byId("pg-run-ui");
const btnExportRust = byId("pg-export-rust");

let runtime = null;
let activeUiSession = null;

const THEME_STORAGE_KEY = "pg-theme";
const TOKEN_NAMES = ["comment", "string", "interpolation", "number", "keyword", "type", "builtin"];
const PORS_TOKEN_RE =
  /(?<comment>\/\/[^\n]*|\/\*[\s\S]*?\*\/)|(?<string>"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*')|(?<number>\b\d+(?:\.\d+)?\b)|(?<keyword>\b(?:funcao|retorne|se|senao|senao_se|seja|mutavel|enquanto|faca_enquanto|para|escolha|caso|registro|enum|verdadeiro|falso|nulo|e|ou|nao)\b)|(?<type>\b(?:inteiro|real|logico|cadeia|caracter|vetor|Resultado|Opcao)\b)|(?<builtin>\b(?:UI|Matematica|Texto|Tipos|Util|Calendario|Arquivos|Json|Internet|Graficos|Teclado|Mouse|Sons|escreva|escreval|leia|adicionar|remover|obter|tamanho)\b)/g;
const UI_START_RE = /\bfuncao\s+iniciar\s*\(/;

function setStatus(message, tone = "info") {
  statusTextEl.textContent = message;
  statusEl.dataset.tone = tone;
}

function applyTheme(theme, persist = true) {
  const resolved = theme === "dark" ? "dark" : "light";
  const usingDark = resolved === "dark";
  document.documentElement.dataset.theme = resolved;
  themeButton.textContent = usingDark ? "Tema: escuro" : "Tema: claro";
  themeButton.setAttribute("aria-label", usingDark ? "Ativar tema claro" : "Ativar tema escuro");
  themeButton.setAttribute("aria-pressed", String(usingDark));
  if (persist) {
    try {
      localStorage.setItem(THEME_STORAGE_KEY, resolved);
    } catch {
      // Ambientes com armazenamento bloqueado continuam usando o tema da sessão.
    }
  }
}

function appendToken(fragment, name, text) {
  if (!text) return;
  const span = document.createElement("span");
  span.className = `pg-tok-${name}`;
  span.textContent = text;
  fragment.append(span);
}

/** Índice do `}` que fecha a interpolação, ou -1 quando o trecho é literal. */
function interpolationEnd(literal, from) {
  const quote = literal[0];
  for (let i = from; i < literal.length - 1; i += 1) {
    const ch = literal[i];
    if (ch === "\\") {
      i += 1;
      continue;
    }
    if (ch === quote || ch === "{" || ch === "\n") return -1;
    if (ch === "}") return i > from ? i : -1;
  }
  return -1;
}

function appendStringTokens(fragment, literal) {
  let cursor = 0;
  for (let i = 1; i < literal.length - 1; ) {
    if (literal[i] === "\\") {
      i += 2;
      continue;
    }
    if (literal[i] === "{") {
      const end = interpolationEnd(literal, i + 1);
      if (end >= 0) {
        appendToken(fragment, "string", literal.slice(cursor, i));
        appendToken(fragment, "interpolation", literal.slice(i, end + 1));
        cursor = end + 1;
        i = end + 1;
        continue;
      }
    }
    i += 1;
  }
  appendToken(fragment, "string", literal.slice(cursor));
}

function renderHighlight(source) {
  const fragment = document.createDocumentFragment();
  let cursor = 0;
  for (const match of source.matchAll(PORS_TOKEN_RE)) {
    if (match.index > cursor) {
      fragment.append(document.createTextNode(source.slice(cursor, match.index)));
    }
    const token = TOKEN_NAMES.find((name) => match.groups?.[name]);
    if (token === "string") appendStringTokens(fragment, match[0]);
    else appendToken(fragment, token, match[0]);
    cursor = match.index + match[0].length;
  }
  fragment.append(document.createTextNode(source.slice(cursor)));
  highlightEl.replaceChildren(fragment);
  syncHighlightScroll();
}

function syncHighlightScroll() {
  highlightEl.scrollTop = editorEl.scrollTop;
  highlightEl.scrollLeft = editorEl.scrollLeft;
}

function setEditorValue(value) {
  editorEl.value = value;
  renderHighlight(value);
}

function syncOutput(text) {
  const value = text ?? "";
  outputEl.textContent = value;
  outputEmptyEl.hidden = value.length > 0;
}

function setControlsEnabled(enabled) {
  btnValidate.disabled = !enabled;
  btnFormat.disabled = !enabled;
  btnRun.disabled = !enabled;
  btnUi.disabled = !enabled;
  btnExportRust.disabled = !enabled;
}

function renderDiagnostics(diagnostics) {
  diagnosticsEl.replaceChildren();
  const count = diagnostics?.length ?? 0;
  outputBadgeEl.textContent = count > 0 ? `${count} diagnóstico${count === 1 ? "" : "s"}` : "Console";
  outputBadgeEl.dataset.tone = count > 0 ? "error" : "info";
  if (count === 0) {
    diagnosticsEl.hidden = true;
    return;
  }
  diagnosticsEl.hidden = false;
  for (const diag of diagnostics) {
    const li = document.createElement("li");
    li.className = "pg-diag-item";
    const code = document.createElement("strong");
    code.textContent = `[${diag.code}] `;
    const msg = document.createElement("span");
    msg.textContent = diag.message;
    li.append(code, msg);
    diagnosticsEl.append(li);
  }
}

function closeActiveUi() {
  if (runtime && activeUiSession !== null) {
    runtime.closeUi(activeUiSession);
    activeUiSession = null;
  }
  uiSectionEl.hidden = true;
  uiContainerEl.replaceChildren();
}

function loadExample(id) {
  const example = PLAYGROUND_EXAMPLES.find((item) => item.id === id) ?? PLAYGROUND_EXAMPLES[0];
  if (!example) return;
  closeActiveUi();
  setEditorValue(example.source);
  stdinEl.value = example.input;
  syncOutput("");
  renderDiagnostics([]);
  if (runtime) {
    if (example.mode === "ui") {
      runUiSession();
    } else {
      runConsole();
    }
  }
}

function runValidate() {
  if (!runtime) return;
  const result = runtime.validate(editorEl.value);
  renderDiagnostics(result.diagnostics);
  if (result.ok) {
    setStatus("Código .pors válido sem diagnósticos.", "ok");
  } else {
    setStatus(`Encontrados ${result.diagnostics.length} diagnóstico(s).`, "error");
  }
}

function runFormat() {
  if (!runtime) return;
  const result = runtime.format(editorEl.value);
  if (result.ok) {
    setEditorValue(result.formatted);
    renderDiagnostics([]);
    setStatus(result.changed ? "Código formatado." : "O código já está formatado.", "ok");
  } else {
    renderDiagnostics(result.diagnostics ?? []);
    setStatus(result.error, "error");
  }
}

function runConsole() {
  if (!runtime) return;
  closeActiveUi();
  const result = runtime.run(editorEl.value, stdinEl.value);
  renderDiagnostics(result.diagnostics ?? []);
  syncOutput(result.stdout ?? "");
  if (result.ok) {
    setStatus("Execução concluída na VM WebAssembly.", "ok");
  } else {
    setStatus(result.error, "error");
  }
}

function applyUiResult(result) {
  if (!result.ok) {
    renderDiagnostics(result.diagnostics ?? []);
    syncOutput(result.stdout ?? "");
    setStatus(result.error, "error");
    return;
  }
  activeUiSession = result.session;
  renderDiagnostics([]);
  syncOutput(result.stdout ?? "");
  uiSectionEl.hidden = false;
  renderUiNodeTree(document, uiContainerEl, result.tree, (event) => {
    if (!runtime || activeUiSession === null) return;
    applyUiResult(runtime.dispatchUi(activeUiSession, event));
  });
  setStatus(`Sessão visual MVU ativa (passo ${result.step}).`, "ok");
}

function runUiSession() {
  if (!runtime) return;
  // O playground abre com um exemplo de console. Sem um programa MVU carregado,
  // o botão apenas reportaria a ausência de "iniciar" e não mostraria prévia.
  if (!UI_START_RE.test(editorEl.value)) {
    const uiExample = PLAYGROUND_EXAMPLES.find((item) => item.mode === "ui");
    if (uiExample) {
      exampleSelect.value = uiExample.id;
      loadExample(uiExample.id);
      return;
    }
  }
  closeActiveUi();
  applyUiResult(runtime.startUi(editorEl.value));
}

function runExportRust(announce = true) {
  if (!runtime) return;
  const result = runtime.exportRust(editorEl.value);
  if (result.ok) {
    renderDiagnostics([]);
    rustCodeEl.textContent = result.code;
    rustSectionEl.hidden = false;
    if (announce) setStatus("Código Rust gerado em WebAssembly.", "ok");
  } else {
    renderDiagnostics(result.diagnostics ?? []);
    rustSectionEl.hidden = true;
    rustCodeEl.textContent = "";
    if (announce) setStatus(result.error, "error");
  }
}

for (const example of PLAYGROUND_EXAMPLES) {
  const option = document.createElement("option");
  option.value = example.id;
  option.textContent = example.title;
  exampleSelect.append(option);
}

applyTheme(document.documentElement.dataset.theme === "dark" ? "dark" : "light", false);
themeButton.addEventListener("click", () => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  applyTheme(next);
});

exampleSelect.addEventListener("change", () => {
  const rustWasOpen = !rustSectionEl.hidden && runtime !== null;
  loadExample(exampleSelect.value);
  // A troca de exemplo já atualiza o status com a execução do novo código.
  if (rustWasOpen) runExportRust(false);
});
btnValidate.addEventListener("click", runValidate);
btnFormat.addEventListener("click", runFormat);
btnRun.addEventListener("click", runConsole);
btnUi.addEventListener("click", runUiSession);
btnExportRust.addEventListener("click", () => runExportRust());
btnCopyRust.addEventListener("click", async () => {
  if (!rustCodeEl.textContent) return;
  try {
    await navigator.clipboard.writeText(rustCodeEl.textContent);
    setStatus("Código Rust copiado para a área de transferência.", "ok");
  } catch {
    setStatus("Não foi possível copiar automaticamente; selecione o código manualmente.", "error");
  }
});

editorEl.addEventListener("input", () => renderHighlight(editorEl.value));
editorEl.addEventListener("scroll", syncHighlightScroll);
editorEl.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
    event.preventDefault();
    const current = PLAYGROUND_EXAMPLES.find((item) => item.id === exampleSelect.value);
    if (current?.mode === "ui" || editorEl.value.includes("funcao visualizar(")) {
      runUiSession();
    } else {
      runConsole();
    }
  }
});

setControlsEnabled(false);
setEditorValue(PLAYGROUND_EXAMPLES[0].source);
stdinEl.value = PLAYGROUND_EXAMPLES[0].input;
syncOutput("");

loadPlaygroundRuntime("./portugol_wasm.wasm")
  .then((loaded) => {
    runtime = loaded;
    setControlsEnabled(true);
    setStatus("VM WebAssembly Portugol-RS pronta.", "ok");
    runConsole();
  })
  .catch((error) => {
    setStatus(
      `Módulo ./portugol_wasm.wasm indisponível (${error instanceof Error ? error.message : String(error)}). Execute scripts/build-web-playground.sh para gerar o bundle WASM.`,
      "error",
    );
  });
