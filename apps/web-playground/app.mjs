import {
  PLAYGROUND_EXAMPLES,
  loadPlaygroundRuntime,
  renderUiNodeTree,
} from "./playground-core.mjs";

const byId = (id) => document.getElementById(id);

const statusEl = byId("pg-status");
const exampleSelect = byId("pg-example");
const editorEl = byId("pg-editor");
const stdinEl = byId("pg-stdin");
const outputEl = byId("pg-output");
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

function setStatus(message, tone = "info") {
  statusEl.textContent = message;
  statusEl.dataset.tone = tone;
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
  if (!diagnostics || diagnostics.length === 0) {
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
  editorEl.value = example.source;
  stdinEl.value = example.input;
  outputEl.textContent = "";
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
    editorEl.value = result.formatted;
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
  outputEl.textContent = result.stdout ?? "";
  if (result.ok) {
    setStatus("Execução concluída na VM WebAssembly.", "ok");
  } else {
    setStatus(result.error, "error");
  }
}

function applyUiResult(result) {
  if (!result.ok) {
    renderDiagnostics(result.diagnostics ?? []);
    outputEl.textContent = result.stdout ?? "";
    setStatus(result.error, "error");
    return;
  }
  activeUiSession = result.session;
  renderDiagnostics([]);
  outputEl.textContent = result.stdout ?? "";
  uiSectionEl.hidden = false;
  renderUiNodeTree(document, uiContainerEl, result.tree, (event) => {
    if (!runtime || activeUiSession === null) return;
    applyUiResult(runtime.dispatchUi(activeUiSession, event));
  });
  setStatus(`Sessão visual MVU ativa (passo ${result.step}).`, "ok");
}

function runUiSession() {
  if (!runtime) return;
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
editorEl.value = PLAYGROUND_EXAMPLES[0].source;
stdinEl.value = PLAYGROUND_EXAMPLES[0].input;

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
