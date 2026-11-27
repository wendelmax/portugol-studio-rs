// Prévia do Rust gerado a partir do Portugol: tradução versionada por aba,
// editor somente leitura, diferenças entre traduções, gutter de mapeamento
// entre linhas e as ações copiar, salvar e desvincular.

import type { Extension } from "@codemirror/state";
import type { EditorView as EditorViewType } from "@codemirror/view";
import { EditorView } from "@codemirror/view";
import { ask, save as saveDialog } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { $ } from "../dom";
import { generatedRustDetachIsCurrent } from "../generated-rust-detach";
import { diffGeneratedRust, generatedRustDiffSummary } from "../generated-rust-diff";
import { createGeneratedRustEditorState, generatedRustMappings, type GeneratedRustMapping } from "../generated-rust-editor";
import type { LearningMode } from "../learning-mode";
import { acceptRustPreview, beginRustPreview, failRustPreview, generatedRustComparison } from "../rust-preview-state";
import { setStatus } from "../shell/status";
import { activeGeneratedMappings, mappedPreviewSegments, sourceMapLinks, type SourceMapLink } from "../source-map-ui";
import type { Tab } from "../state/tab";

type MappingState = "idle" | "generating" | "ready" | "stale" | "error";

/** Espera depois da última edição antes de pedir uma nova tradução. */
const EXPORT_DELAY_MS = 220;

export interface RustPreviewHost {
  /** Editor do código-fonte. */
  sourceView: EditorViewType;
  activeTab(): Tab | undefined;
  learningMode(): LearningMode;
  /** Volta ao trecho de origem de um trecho do Rust gerado. */
  goTo(offset: number): void;
  /** Extensões de cor e tema do editor do Rust gerado. */
  editorExtensions: Extension[];
  /** Chamado depois de cada atualização da prévia. */
  onRendered(): void;
  /** Abre o Rust desvinculado numa nova aba `.rs`. */
  openDetachedRust(code: string): void;
}

export function renderGeneratedRustDiff(container: HTMLElement, previous: string, current: string) {
  const diff = diffGeneratedRust(previous, current);
  const summary = document.createElement("p");
  summary.className = "rust-diff-summary";
  summary.setAttribute("role", "status");
  summary.textContent = generatedRustDiffSummary(diff);

  const table = document.createElement("table");
  table.className = "rust-diff-table";
  table.setAttribute("aria-label", "Diferenças entre a tradução Rust anterior e a atual");
  const head = document.createElement("thead");
  const headings = document.createElement("tr");
  for (const label of ["Versão anterior", "Versão atual"]) {
    const heading = document.createElement("th");
    heading.scope = "col";
    heading.textContent = label;
    headings.append(heading);
  }
  head.append(headings);
  const body = document.createElement("tbody");
  for (const row of diff.rows) {
    const line = document.createElement("tr");
    line.dataset.kind = row.kind;
    line.setAttribute("aria-label", row.kind === "removed" ? "Linha removida" : row.kind === "added" ? "Linha adicionada" : "Linha sem alteração");
    const createCell = (lineNumber: number | null, text: string, kind: "removed" | "added" | "context") => {
      const cell = document.createElement("td");
      const mark = document.createElement("span");
      mark.className = "rust-diff-mark";
      mark.textContent = kind === "removed" ? "−" : kind === "added" ? "+" : " ";
      mark.setAttribute("aria-hidden", "true");
      const number = document.createElement("span");
      number.className = "rust-diff-line-number";
      number.textContent = lineNumber === null ? "" : String(lineNumber);
      number.setAttribute("aria-hidden", "true");
      const code = document.createElement("code");
      code.textContent = text;
      cell.append(mark, number, code);
      return cell;
    };
    line.append(
      createCell(row.oldLine, row.kind === "added" ? "" : row.text, row.kind === "added" ? "context" : row.kind),
      createCell(row.newLine, row.kind === "removed" ? "" : row.text, row.kind === "removed" ? "context" : row.kind),
    );
    body.append(line);
  }
  table.append(head, body);
  container.replaceChildren(summary, table);
}

export function mountRustPreview(host: RustPreviewHost) {
  const view = host.sourceView;
  const activeTab = host.activeTab;
  const generatedRustEditor = new EditorView({
    parent: $("rust-output"),
    state: createGeneratedRustEditorState("", host.goTo, host.editorExtensions),
  });

  let rustExportTimer: number | undefined;
  let rustExportGeneration = 0;
  let rustMappingLinks: SourceMapLink[] = [];
  let rustGeneratedMappings: GeneratedRustMapping[] = [];
  let rustMappingState: MappingState = "idle";
  let comparisonBasis = 44;
  try {
    const savedBasis = Number(localStorage.getItem("portugol.rust-comparison-basis"));
    if (Number.isFinite(savedBasis) && savedBasis >= 25 && savedBasis <= 68) comparisonBasis = savedBasis;
  } catch { /* Preferências indisponíveis não impedem a comparação. */ }

  function setComparisonBasis(value: number) {
    comparisonBasis = Math.max(25, Math.min(68, value));
    const preview = $<HTMLElement>("rust-preview");
    preview.style.flexBasis = `${comparisonBasis}%`;
    const splitter = $<HTMLElement>("rust-comparison-splitter");
    splitter.setAttribute("aria-valuenow", String(Math.round(comparisonBasis)));
  }

  setComparisonBasis(comparisonBasis);

  const comparisonSplitter = $<HTMLElement>("rust-comparison-splitter");
  comparisonSplitter.addEventListener("pointerdown", (start) => {
    if (start.button !== 0) return;
    comparisonSplitter.setPointerCapture(start.pointerId);
    const origin = start.clientX;
    const initial = comparisonBasis;
    const width = Math.max(1, comparisonSplitter.parentElement?.clientWidth ?? window.innerWidth);
    const move = (event: PointerEvent) => setComparisonBasis(initial - (event.clientX - origin) / width * 100);
    const finish = () => {
      try { localStorage.setItem("portugol.rust-comparison-basis", String(comparisonBasis)); } catch { /* Ignore storage errors. */ }
      comparisonSplitter.removeEventListener("pointermove", move);
      comparisonSplitter.removeEventListener("pointerup", finish);
      comparisonSplitter.removeEventListener("pointercancel", finish);
      comparisonSplitter.removeEventListener("lostpointercapture", finish);
    };
    comparisonSplitter.addEventListener("pointermove", move);
    comparisonSplitter.addEventListener("pointerup", finish, { once: true });
    comparisonSplitter.addEventListener("pointercancel", finish, { once: true });
    comparisonSplitter.addEventListener("lostpointercapture", finish, { once: true });
  });
  comparisonSplitter.addEventListener("keydown", (event) => {
    let next: number | null = null;
    if (event.key === "ArrowLeft") next = comparisonBasis + 2;
    else if (event.key === "ArrowRight") next = comparisonBasis - 2;
    else if (event.key === "Home") next = 25;
    else if (event.key === "End") next = 68;
    if (next === null) return;
    event.preventDefault();
    setComparisonBasis(next);
    try { localStorage.setItem("portugol.rust-comparison-basis", String(comparisonBasis)); } catch { /* Ignore storage errors. */ }
  });

  function selectComparisonPane(pane: "source" | "generated") {
    const area = $<HTMLElement>("editor").parentElement!;
    area.dataset.comparisonPane = pane;
    $("rust-comparison-source-tab").setAttribute("aria-selected", String(pane === "source"));
    $("rust-comparison-generated-tab").setAttribute("aria-selected", String(pane === "generated"));
    $("rust-comparison-source-tab").tabIndex = pane === "source" ? 0 : -1;
    $("rust-comparison-generated-tab").tabIndex = pane === "generated" ? 0 : -1;
    requestAnimationFrame(() => {
      view.requestMeasure();
      generatedRustEditor.requestMeasure();
    });
  }
  const comparisonTabs = [
    { pane: "source" as const, button: $("rust-comparison-source-tab") },
    { pane: "generated" as const, button: $("rust-comparison-generated-tab") },
  ];
  comparisonTabs.forEach(({ pane, button }, index) => {
    button.addEventListener("click", () => selectComparisonPane(pane));
    button.addEventListener("keydown", (event) => {
      const next = event.key === "ArrowRight" ? (index + 1) % comparisonTabs.length
        : event.key === "ArrowLeft" ? (index + comparisonTabs.length - 1) % comparisonTabs.length
          : event.key === "Home" ? 0
            : event.key === "End" ? comparisonTabs.length - 1
              : -1;
      if (next < 0) return;
      event.preventDefault();
      selectComparisonPane(comparisonTabs[next].pane);
      comparisonTabs[next].button.focus();
    });
  });
  selectComparisonPane("source");

  /** Descarta a tradução pendente; respostas atrasadas são ignoradas. */
  function cancelRustExport() {
    rustExportGeneration++;
    if (rustExportTimer !== undefined) window.clearTimeout(rustExportTimer);
    rustExportTimer = undefined;
  }

  function scheduleRustExport() {
    const tab = activeTab();
    if (!tab || tab.language === "rust") {
      rustExportGeneration++;
      return;
    }
    const source = view.state.doc.toString();
    const tabId = tab.id;
    const generation = ++rustExportGeneration;
    if (rustExportTimer !== undefined) window.clearTimeout(rustExportTimer);
    tab.rustPreview = beginRustPreview(tab.rustPreview, source);
    renderRustPreview(tab);
    rustExportTimer = window.setTimeout(async () => {
      let result: api.RustExport;
      try {
        result = await api.exportPortugolRs(source);
      } catch (error) {
        if (generation !== rustExportGeneration || activeTab()?.id !== tabId || view.state.doc.toString() !== source) return;
        tab.rustPreview = failRustPreview(tab.rustPreview ?? { pending: false, stale: false }, "Falha ao solicitar a tradução: " + String(error));
        renderRustPreview(tab);
        return;
      }
      if (generation !== rustExportGeneration || activeTab()?.id !== tabId || view.state.doc.toString() !== source) return;
      tab.rustPreview = acceptRustPreview(tab.rustPreview ?? { pending: false, stale: false }, source, result);
      renderRustPreview(tab);
    }, EXPORT_DELAY_MS);
  }

  function renderRustPreview(tab: Tab) {
    if (activeTab()?.id !== tab.id) return;
    const state = tab.rustPreview;
    const status = $<HTMLElement>("rust-status");
    const copy = $<HTMLButtonElement>("rust-copy");
    const save = $<HTMLButtonElement>("rust-save");
    const exportCargo = $<HTMLButtonElement>("rust-export-cargo");
    const diffToggle = $<HTMLButtonElement>("rust-diff-toggle");
    const generatedOutput = $<HTMLElement>("rust-output");
    const diffOutput = $<HTMLElement>("rust-diff-output");
    const detach = $<HTMLButtonElement>("rust-detach");
    const artifact = state?.artifact;

    const stale = Boolean(state?.stale);
    const pending = Boolean(state?.pending);
    const comparison = generatedRustComparison(state);
    const canShowDiff = Boolean(comparison);
    if (!canShowDiff) tab.showRustDiff = false;
    const showDiff = Boolean(tab.showRustDiff && canShowDiff);
    diffToggle.disabled = !canShowDiff;
    diffToggle.textContent = showDiff ? "Voltar ao código atual" : "Mostrar diferenças";
    diffToggle.setAttribute("aria-pressed", String(showDiff));
    generatedOutput.hidden = showDiff;
    diffOutput.hidden = !showDiff;
    if (showDiff && comparison) renderGeneratedRustDiff(diffOutput, comparison.previous.code, comparison.current.code);

    const code = artifact?.code ?? "";
    rustGeneratedMappings = artifact && !stale
      ? mappedPreviewSegments(artifact.code, artifact.sourceMap).map((segment) => ({
        ...segment,
        quality: segment.quality === "approximate" ? "approximate" : "exact",
      }))
      : [];
    const generatedDocument = generatedRustEditor.state.doc;
    const mappingEffect = generatedRustMappings.of(rustGeneratedMappings);
    if (generatedDocument.toString() === code) generatedRustEditor.dispatch({ effects: mappingEffect });
    else generatedRustEditor.dispatch({
      changes: { from: 0, to: generatedDocument.length, insert: code },
      effects: mappingEffect,
    });

    rustMappingLinks = artifact && !state?.stale ? sourceMapLinks(artifact.sourceMap) : [];
    rustMappingState = state?.stale ? "stale" : state?.error && !artifact ? "error" : artifact ? "ready" : state?.pending ? "generating" : "idle";
    if (state?.stale) {
      status.textContent = state.pending
        ? "Tradução desatualizada — atualizando…"
        : `Tradução desatualizada. ${state.error ?? "Aguardando tradução da versão atual."}`;
    } else if (state?.error) {
      status.textContent = state.error;
    } else if (state?.pending && artifact) {
      status.textContent = "Atualizando tradução; exibindo versão da fonte atual.";
    } else if (state?.pending) {
      status.textContent = "Atualizando tradução…";
    } else if (artifact) {
      status.textContent = "Tradução disponível para copiar ou salvar. Clique em um trecho mapeado ou pressione F12 para voltar à origem.";
    } else {
      status.textContent = "Documento Rust independente; a prévia gerada não altera este arquivo.";
    }
    if (state?.error) status.dataset.error = "true";
    else delete status.dataset.error;
    if (state?.stale) status.dataset.stale = "true";
    else delete status.dataset.stale;
    copy.disabled = save.disabled = detach.disabled = !artifact || Boolean(pending || stale);
    exportCargo.disabled = copy.disabled;
    renderMappingGutter();
    host.onRendered();
  }

  function renderMappingGutter() {
    const gutter = $<HTMLElement>("rust-mapping-gutter");
    const preview = $<HTMLElement>("rust-preview");
    const comparisonTabs = $<HTMLElement>("rust-comparison-tabs");
    const splitter = $<HTMLElement>("rust-comparison-splitter");
    const tab = activeTab();
    const visible = Boolean(tab && tab.language !== "rust" && host.learningMode() === "bridge" && !preview.hidden);
    gutter.hidden = !visible;
    splitter.hidden = !visible;
    comparisonTabs.hidden = !visible;
    if (!visible) return;

    setComparisonBasis(comparisonBasis);

    const sourceLine = view.state.doc.lineAt(view.state.selection.main.head).number;
    const activeMappings = activeGeneratedMappings(rustGeneratedMappings, view.state.selection.main.head);
    generatedRustEditor.dispatch({ effects: generatedRustMappings.of(activeMappings) });
    const status = $<HTMLElement>("rust-mapping-status");
    const links = $<HTMLElement>("rust-mapping-links");
    links.replaceChildren();
    if (rustMappingState === "stale") {
      status.textContent = "Mapeamento desatualizado; aguarde a tradução atual.";
      return;
    }
    if (rustMappingState === "generating" || rustMappingState === "idle") {
      status.textContent = "Atualizando mapa…";
      return;
    }
    if (rustMappingState === "error") {
      status.textContent = "Mapeamento indisponível.";
      return;
    }

    const currentLineLinks = rustMappingLinks.filter((link) => link.sourceLine === sourceLine);
    if (!currentLineLinks.length) {
      status.textContent = `Linha ${sourceLine} sem correspondência direta.`;
      return;
    }
    status.textContent = `Linha ${sourceLine}`;
    for (const link of currentLineLinks) {
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.testid = `mapping-link-source-${link.sourceLine}-rust-${link.generatedLine}`;
      button.dataset.quality = link.quality;
      button.textContent = `L${link.sourceLine} ↔ L${link.generatedLine}`;
      button.title = link.quality === "approximate"
        ? `Localização aproximada: linha ${link.sourceLine} em Portugol para linha ${link.generatedLine} em Rust`
        : `Ir da linha ${link.sourceLine} em Portugol para a linha ${link.generatedLine} em Rust`;
      button.setAttribute("aria-label", button.title);
      button.onclick = () => focusRustMapping(link);
      links.append(button);
    }
  }

  function focusRustMapping(link: SourceMapLink) {
    const document = generatedRustEditor.state.doc;
    if (link.generatedLine < 1 || link.generatedLine > document.lines) return;
    const line = document.line(link.generatedLine);
    const position = Math.max(line.from, Math.min(line.to, link.generatedOffset));
    generatedRustEditor.dispatch({ selection: { anchor: position }, scrollIntoView: true });
    generatedRustEditor.focus();
  }

  $<HTMLButtonElement>("rust-copy").onclick = async () => {
    try {
      await navigator.clipboard.writeText(generatedRustEditor.state.doc.toString());
      setStatus("Código Rust copiado.", "ok");
    } catch (error) {
      setStatus(String(error), "error");
    }
  };
  $<HTMLButtonElement>("rust-save").onclick = async () => {
    const code = generatedRustEditor.state.doc.toString();
    try {
      const path = await saveDialog({ defaultPath: "programa.rs", filters: [{ name: "Rust", extensions: ["rs"] }] });
      if (!path) return;
      await api.writeFile(path, code);
      setStatus("Rust salvo em " + path + ".", "ok");
    } catch (error) {
      setStatus(String(error), "error");
    }
  };
  $<HTMLButtonElement>("rust-export-cargo").onclick = async () => {
    const code = generatedRustEditor.state.doc.toString();
    const tab = activeTab();
    const rawName = tab?.path?.split(/[\\/]/).pop() ?? tab?.exampleName ?? "programa_portugol";
    try {
      const exported = await api.exportCargoProject(rawName, code);
      if (!exported) return;
      setStatus(`Projeto Cargo "${exported.packageName}" exportado em ${exported.projectDir}.`, "ok");
    } catch (error) {
      setStatus(String(error), "error");
    }
  };
  $<HTMLButtonElement>("rust-diff-toggle").onclick = () => {
    const tab = activeTab();
    if (!tab || !tab.rustPreview?.previousArtifact) return;
    tab.showRustDiff = !tab.showRustDiff;
    renderRustPreview(tab);
  };
  $<HTMLButtonElement>("rust-detach").onclick = async () => {
    const sourceTab = activeTab();
    const previewState = sourceTab?.rustPreview;
    const artifact = previewState?.artifact;
    if (!sourceTab || sourceTab.language === "rust" || !artifact || previewState.pending || previewState.stale) return;

    const snapshot = { tab: sourceTab, document: view.state.doc, artifact };
    const accepted = await ask(
      "O código Rust será copiado para uma nova aba independente. Alterações futuras no Portugol não atualizarão essa cópia. A origem e a prévia vinculada continuarão preservadas. Deseja continuar?",
      { title: "Desvincular código Rust gerado", kind: "warning", okLabel: "Desvincular e editar", cancelLabel: "Cancelar" },
    );
    if (!accepted) return;

    const currentTab = activeTab();
    const currentPreview = currentTab?.rustPreview;
    if (!currentTab || !currentPreview?.artifact || !generatedRustDetachIsCurrent(snapshot, {
      tab: currentTab,
      document: view.state.doc,
      artifact: currentPreview.artifact,
      pending: Boolean(currentPreview.pending),
      stale: Boolean(currentPreview.stale),
    })) {
      if (currentTab === sourceTab) setStatus("A fonte ou a tradução mudou enquanto a confirmação estava aberta. Gere novamente antes de desvincular.", "error");
      return;
    }

    host.openDetachedRust(artifact.code);
  };

  return {
    /** Editor somente leitura do Rust gerado (usado ao trocar de tema). */
    generatedEditor: generatedRustEditor,
    schedule: scheduleRustExport,
    cancel: cancelRustExport,
    render: renderRustPreview,
    renderMappingGutter,
    /** Restaura a largura padrão da comparação lado a lado com Rust. */
    resetComparisonBasis() {
      try { localStorage.removeItem("portugol.rust-comparison-basis"); } catch { /* Ignore storage errors. */ }
      setComparisonBasis(44);
    },
  };
}
