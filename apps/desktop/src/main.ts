import { mountTips } from "./tips";
import { applyThemeColors, mountThemeEditor, themeProfile } from "./themes";
// IDE: abas de arquivos, editor CodeMirror, console e execução.
//
// Atalhos seguem o Portugol Studio original (TelaAtalhos.java):
// Shift+F6 executa, Shift+F7 interrompe, Ctrl+Q fecha a aba,
// Alt+←/→ troca de aba, Shift+Esc expande o editor.

import "./style.css";
import license from "../../../LICENSE?raw";
import { version } from "../package.json";

import { EditorState, Compartment, RangeSet, StateEffect, StateField, type Extension } from "@codemirror/state";
import {
  EditorView, GutterMarker, gutter, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter,
  drawSelection, highlightSpecialChars, Decoration, type DecorationSet,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab, isolateHistory } from "@codemirror/commands";
import {
  syntaxHighlighting, HighlightStyle, bracketMatching, indentOnInput, foldGutter, foldKeymap,
} from "@codemirror/language";
import { forceLinting, linter, lintGutter, setDiagnostics, type Diagnostic } from "@codemirror/lint";
import { openSearchPanel, highlightSelectionMatches } from "@codemirror/search";
import { autocompletion, closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { rust } from "@codemirror/lang-rust";
import { createGeneratedRustEditorState, generatedRustMappings, type GeneratedRustMapping } from "./generated-rust-editor";
import { generatedRustDetachIsCurrent } from "./generated-rust-detach";
import { diffGeneratedRust, generatedRustDiffSummary } from "./generated-rust-diff";
import { tags } from "@lezer/highlight";
import { open as openDialog, save as saveDialog, ask } from "@tauri-apps/plugin-dialog";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { renderDiagnostics } from "./diagnostics";
import { mappedPreviewSegments, sourceDiagnostic, sourceMapLinks, type SourceMapLink } from "./source-map-ui";
import { mountExamples } from "./examples";
import { mountHomeNewFileMenu } from "./home-new-file-menu";
import { mountExercises, readExercise, type ExerciseDocument } from "./exercises";
import {
  exerciseTestPanelState,
  exerciseTestStatus,
  isCurrentExerciseTestRequest,
  type ExerciseTestRequest,
  type ExerciseTestRun,
} from "./exercise-test-panel";
import { portugolFolding } from "./folding";
import { editorSearchKeys, findDialog } from "./find";
import { icon, mountIcons } from "./icons";
import { mountOutline, symbolKindLabels } from "./outline";
import { panelContentId, panelNameForKey, type PanelName } from "./panel-tabs";
import { calculateSidePaneMinimums, clampPaneSize, DEFAULT_PANE_LAYOUT, loadPaneLayout, normalizePaneLayout, resizePaneForKey, savePaneLayout, type PaneLayout } from "./pane-layout";
import { LIBRARIES, portugol, portugolRs, SNIPPETS } from "./portugol";
import { detectSourceLanguage, ensureSourceExtension, type SourceLanguage } from "./document-language";
import { semanticSymbolLanguage } from "./semantic-symbol-actions";
import { collectWorkspaceFiles, flattenWorkspaceEntries, workspaceFileKind, workspaceRenameFileName, workspaceSourceFileName, type WorkspaceEntry } from "./workspace-explorer";
import { workspaceExportStatus } from "./workspace-export-state";
import { canDebugSource, canFormatSource, canRunSource } from "./runtime-capabilities";
import { parseProcessArguments } from "./process-arguments";
import { hasCargoExecutionConsent, hasRustExecutionConsent, rememberCargoExecutionConsent, rememberRustExecutionConsent } from "./rust-execution-consent";
import { applyWorkspaceRenameToTabs, deleteWorkspaceFileFlow, prepareWorkspaceRename } from "./workspace-file-actions";
import { conceptsForMode, resolveConceptForMode } from "./learning-content";
import { learningPinForMode, setLearningPin, toggleLearningPin, type LearningPins } from "./learning-pins";
import { explainMappedConcept, learningContextAtSelection } from "./learning-context";
import { createLatestOnlyRustSourceChecker } from "./rust-source-check";
import { replayRunEvents, routeRunEvent as routeRunEventById, type PendingRunEvent } from "./run-event-router";
import { loadLearningMode, saveLearningMode, type LearningMode } from "./learning-mode";
import {
  getOwnershipScenario,
  getOwnershipSnapshot,
  ownershipScenarioIds,
  type OwnershipBindingState,
  type OwnershipScenarioId,
} from "./ownership-lab";
import { filterCommands, isDebugCommandAvailable, nextCommandIndex } from "./command-palette";
import { acceptRustPreview, beginRustPreview, failRustPreview, generatedRustComparison, markRustPreviewStale, type RustPreviewState } from "./rust-preview-state";
import { PORS_UI_EXAMPLES } from "./pors-ui-examples";
import { mountPorsUiPreviewPanel } from "./pors-ui-preview-panel";
import { mountUiPreview } from "./ui-preview-renderer";
import * as api from "./api";

const TEMPLATE = "programa\n{\n\tfuncao inicio()\n\t{\n\t\t\n\t}\n}\n";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;

function loadSavedPaneLayout(preferenceKey?: string): PaneLayout {
  try {
    return loadPaneLayout(localStorage, preferenceKey);
  } catch {
    return { ...DEFAULT_PANE_LAYOUT };
  }
}

let paneLayout = loadSavedPaneLayout();

function persistPaneLayout() {
  try {
    savePaneLayout(localStorage, paneLayout, activeTab()?.workspacePreferenceKey);
  } catch {
    // A configuração permanece ativa enquanto a janela estiver aberta.
  }
}

// ---------------------------------------------------------------------------
// Editor
// ---------------------------------------------------------------------------

interface Tab {
  id: number;
  path: string | null;
  language: SourceLanguage;
  exampleName?: string;
  examplePath?: string;
  exercise?: ExerciseDocument;
  exerciseTestRun?: ExerciseTestRun;
  exerciseTestGeneration?: number;
  workspaceId?: string;
  workspacePath?: string;
  workspacePreferenceKey?: string;
  cargoProject?: boolean;
  cargoTarget?: api.CargoRunTarget;
  workspaceExercisePath?: string;
  state: EditorState;
  rustPreview?: RustPreviewState;
  learningPins?: LearningPins;
  showRustDiff?: boolean;
  /** Texto da última gravação, para saber se há alterações. */
  saved: string;
  diagnostics: api.DiagnosticInfo[];
  runtimeCompileDiagnostics?: api.DiagnosticInfo[];
  symbols: api.SymbolInfo[];
}

// Tema: "system" segue o sistema operacional; a escolha fica salva.
const THEME_KEY = "portugol.theme";
const theme = new Compartment();
const sourceLanguage = new Compartment();
const portugolTools = new Compartment();
const generatedRustTheme = new Compartment();
const darkQuery = window.matchMedia("(prefers-color-scheme: dark)");

function themeChoice(): string {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    if (saved === "light" || saved === "dark" || saved === "system" || (saved && themeProfile(saved))) return saved;
  } catch {
    // Sem armazenamento local: fica no padrão.
  }
  return "preset:portugol-studio-next";
}

let appliedTheme = themeChoice();
const isDark = () => {
  const choice = themeProfile(appliedTheme)?.base ?? appliedTheme;
  return choice === "dark" || (choice === "system" && darkQuery.matches);
};
const editorColors = syntaxHighlighting(HighlightStyle.define([
  { tag: tags.keyword, color: "var(--syntax-keyword)" },
  { tag: [tags.typeName, tags.namespace], color: "var(--syntax-type)" },
  { tag: tags.number, color: "var(--syntax-number)" },
  { tag: tags.bool, color: "var(--syntax-logical)" },
  { tag: tags.function(tags.variableName), color: "var(--syntax-function)" },
  { tag: tags.standard(tags.function(tags.variableName)), color: "var(--syntax-function)" },
  { tag: tags.string, color: "var(--syntax-string)" },
  { tag: tags.comment, color: "var(--syntax-comment)" },
  { tag: tags.operator, color: "var(--syntax-operator)" },
]));
const centerKey = "portugol.centerSource";
let centerActive = false;
try {
  centerActive = localStorage.getItem(centerKey) === "true";
} catch {
  // A opção também funciona sem armazenamento.
}

const darkEditor = EditorView.theme({}, { dark: true });
const editorTheme = (): Extension => isDark() ? darkEditor : [];

/** Destaque da linha onde ocorreu o erro de execução. */
const setRuntimeError = StateEffect.define<{ from: number; to: number } | null>();
const runtimeError = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setRuntimeError)) {
        if (!effect.value) return Decoration.none;
        const line = tr.state.doc.lineAt(Math.min(effect.value.from, tr.state.doc.length));
        return Decoration.set([Decoration.line({ class: "cm-runtime-error" }).range(line.from)]);
      }
    }
    return tr.docChanged ? Decoration.none : value.map(tr.changes);
  },
  provide: (field) => EditorView.decorations.from(field),
});

/** Offsets dos pontos de parada; acompanham as edições do documento. */
const toggleBreakpoint = StateEffect.define<number>();
const breakpoints = StateField.define<Set<number>>({
  create: () => new Set<number>(),
  update(value, tr) {
    let next = value;
    for (const effect of tr.effects) {
      if (effect.is(toggleBreakpoint)) {
        next = new Set(next);
        if (next.has(effect.value)) next.delete(effect.value);
        else next.add(effect.value);
      }
    }
    if (!tr.docChanged) return next;
    const mapped = new Set<number>();
    for (const pos of next) mapped.add(tr.changes.mapPos(pos, -1));
    return mapped;
  },
});

const breakpointMarker = new (class extends GutterMarker {
  toDOM() {
    const dot = icon("pequeno/bug");
    dot.classList.add("cm-breakpoint");
    dot.title = "Ponto de parada — clique para remover";
    return dot;
  }
})();

const breakpointGutter = gutter({
  class: "cm-breakpoint-gutter",
  markers: (view) => {
    const positions = [...view.state.field(breakpoints)]
      .map((pos) => view.state.doc.lineAt(Math.min(pos, view.state.doc.length)).from)
      .sort((a, b) => a - b);
    return RangeSet.of(positions.map((pos) => breakpointMarker.range(pos)));
  },
  initialSpacer: () => breakpointMarker,
  domEventHandlers: {
    mousedown(view, line, event) {
      if ((event as MouseEvent).button !== 0) return false;
      view.dispatch({ effects: toggleBreakpoint.of(line.from) });
      syncBreakpoints(view.state);
      return true;
    },
  },
});

/** Destaque da linha onde a VM está parada no depurador. */
const setDebugLine = StateEffect.define<number | null>();
const debugLine = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setDebugLine)) {
        if (effect.value === null) return Decoration.none;
        const line = tr.state.doc.line(Math.max(1, Math.min(effect.value, tr.state.doc.lines)));
        return Decoration.set([Decoration.line({ class: "cm-debug-line" }).range(line.from)]);
      }
    }
    return tr.docChanged ? Decoration.none : value.map(tr.changes);
  },
  provide: (field) => EditorView.decorations.from(field),
});

function toCodeMirror(info: api.DiagnosticInfo, length: number): Diagnostic {
  const from = Math.min(info.start.offset, length);
  const to = Math.min(Math.max(info.end.offset, from + 1), length);
  return {
    from,
    to: Math.max(to, from),
    severity: info.severity,
    source: info.code,
    message: info.help ? `${info.message}\nAjuda: ${info.help}` : info.message,
  };
}

const portugolLinter = linter(
  async (view) => {
    const doc = view.state.doc;
    const tab = activeTab();
    if (tab?.language === "rust") {
      tab.diagnostics = [];
      tab.symbols = [];
      renderMessages();
      outline.show([]);
      updateLibraryButtons([]);
      return [];
    }
    const language = tab?.language ?? "portugol";
    const inspection = language === "portugol-rs"
      ? await api.inspectPortugolRs(doc.toString())
      : await api.inspect(doc.toString());
    const { diagnostics, symbols } = inspection;
    if (tab && tab === activeTab() && view.state.doc === doc) {
      tab.diagnostics = diagnostics;
      tab.symbols = symbols;
      renderMessages();
      outline.show(symbols);
      updateLibraryButtons(symbols);
    }
    return [...diagnostics, ...(tab?.runtimeCompileDiagnostics ?? [])].map((d) => toCodeMirror(d, doc.length));
  },
  { delay: 350 },
);

const latestRustSourceCheck = createLatestOnlyRustSourceChecker(api.checkRustSource);
const rustSourceLinter = linter(
  async (editor) => {
    const doc = editor.state.doc;
    const tab = activeTab();
    if (tab?.language !== "rust" || tab.cargoProject === true || rustcAvailable !== true) {
      if (tab?.language === "rust") {
        tab.diagnostics = [];
        renderMessages();
      }
      return [];
    }

    const checked = await latestRustSourceCheck(doc.toString());
    if (activeTab() !== tab || editor.state.doc !== doc || checked.source !== doc.toString()) return [];
    const diagnostics: api.DiagnosticInfo[] = checked.error
      ? [{
        severity: "warning",
        code: "rustc.check",
        message: `Não foi possível analisar este arquivo Rust: ${checked.error}`,
        help: null,
        start: { offset: 0, line: 1, column: 1 },
        end: { offset: 0, line: 1, column: 1 },
      }]
      : checked.diagnostics.flatMap((diagnostic) => {
        if (diagnostic.severity !== "error" && diagnostic.severity !== "warning") return [];
        return [{
          severity: diagnostic.severity,
          code: diagnostic.code ?? "rustc",
          message: diagnostic.message,
          help: diagnostic.rendered,
          start: diagnostic.start,
          end: diagnostic.end,
        }];
      });
    tab.diagnostics = diagnostics;
    renderMessages();
    return [...diagnostics, ...(tab.runtimeCompileDiagnostics ?? [])]
      .map((diagnostic) => toCodeMirror(diagnostic, doc.length));
  },
  { delay: 500 },
);

function extensions(language: SourceLanguage): Extension[] {
  const isRust = language === "rust";
  const isPortugol = !isRust;
  const tools: Extension[] = isPortugol
    ? [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]
    : [rustSourceLinter];
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightSpecialChars(),
    history(),
    drawSelection(),
    indentOnInput(),
    bracketMatching(),
    closeBrackets(),
    autocompletion(),
    highlightActiveLine(),
    highlightSelectionMatches(),
    findDialog,
    EditorState.tabSize.of(4),
    sourceLanguage.of(isRust ? rust() : language === "portugol-rs" ? portugolRs() : portugol()),
    portugolTools.of(tools),
    foldGutter({ markerDOM: (open) => {
      const marker = document.createElement("span");
      marker.textContent = open ? "−" : "+";
      marker.title = open ? "Recolher bloco" : "Expandir bloco";
      marker.setAttribute("aria-label", marker.title);
      return marker;
    }}),
    editorColors,
    theme.of(editorTheme()),
    lintGutter(),
    EditorView.domEventHandlers({
      click(event, editor) {
        const mouse = event as MouseEvent;
        if (activeTab()?.language === "rust") return false;
        if (mouse.button !== 0 || (!mouse.ctrlKey && !mouse.metaKey)) return false;
        const offset = editor.posAtCoords({ x: mouse.clientX, y: mouse.clientY });
        if (offset === null) return false;
        void goToDeclarationAt(offset, true);
        return false;
      },
    }),
    keymap.of([
      { key: "F12", run: () => { void goToDeclarationAt(view.state.selection.main.head); return true; } },
      ...closeBracketsKeymap, ...defaultKeymap, ...editorSearchKeys,
      ...historyKeymap, ...completionKeymap, ...foldKeymap, indentWithTab,
    ]),
    EditorView.updateListener.of((update) => {
      if (update.docChanged && activeTab()?.runtimeCompileDiagnostics?.length) {
        activeTab()!.runtimeCompileDiagnostics = [];
        renderMessages();
      }
      if (update.docChanged) {
        const tab = activeTab();
        if (tab?.exercise) {
          tab.exerciseTestGeneration = (tab.exerciseTestGeneration ?? 0) + 1;
          tab.exerciseTestRun = undefined;
          renderExerciseTests();
        }
        renderTabs();
      }
      if (update.docChanged && activeTab()?.language !== "rust") {
        const tab = activeTab();
        if (tab?.rustPreview) {
          tab.rustPreview = markRustPreviewStale(tab.rustPreview, update.state.doc.toString());
        }
        if (learningMode === "bridge") scheduleRustExport();
      }
      if (update.selectionSet) learningConceptManuallySelected = false;
      if ((update.selectionSet || update.docChanged) && !$<HTMLElement>("learning-panel").hidden) renderLearningPanel();
      if (update.selectionSet || update.docChanged) renderCursor();
      if (update.selectionSet || update.docChanged) renderMappingGutter();
      if (centerActive && update.selectionSet) {
        queueMicrotask(() => {
          if (centerActive && view.state === update.state) centerSelection();
        });
      }
    }),
  ];
}

const view = new EditorView({ parent: $("editor") });
const generatedRustEditor = new EditorView({
  parent: $("rust-output"),
  state: createGeneratedRustEditorState("", goTo, [editorColors, generatedRustTheme.of(editorTheme())]),
});

function applyTheme(choice: string) {
  appliedTheme = choice;
  try {
    localStorage.setItem(THEME_KEY, choice);
  } catch {
    // Ignora: a escolha vale só para esta sessão.
  }
  if (choice === "system") delete document.documentElement.dataset.theme;
  else document.documentElement.dataset.theme = themeProfile(choice)?.base ?? choice;
  applyThemeColors(document.documentElement, themeProfile(choice)?.colors);
  view.dispatch({ effects: theme.reconfigure(editorTheme()) });
  generatedRustEditor.dispatch({ effects: generatedRustTheme.reconfigure(editorTheme()) });
  help?.refreshTheme();
}

darkQuery.addEventListener("change", () => {
  view.dispatch({ effects: theme.reconfigure(editorTheme()) });
  generatedRustEditor.dispatch({ effects: generatedRustTheme.reconfigure(editorTheme()) });
  help?.refreshTheme();
});

// ---------------------------------------------------------------------------
// Abas
// ---------------------------------------------------------------------------

let tabs: Tab[] = [];
let workspaceState: api.WorkspaceListing | null = null;
let workspaceLoading = false;
let workspaceRequest = 0;
let selectedWorkspacePath: string | null = null;
const expandedWorkspaceFolders = new Set<string>();
const homeId = 0;
const helpId = -1;
let activeId = homeId;
let helpOpen = false;
let nextTabId = 1;
let rustExportTimer: number | undefined;
let rustExportGeneration = 0;
let rustMappingLinks: SourceMapLink[] = [];
let rustMappingState: "idle" | "generating" | "ready" | "stale" | "error" = "idle";
let definitionNavigationGeneration = 0;
let learningMode: LearningMode = loadLearningMode();
let selectedLearningConceptId = "variables";
let learningConceptManuallySelected = false;
let learningPracticeBusy = false;
let ownershipScenarioId: OwnershipScenarioId = "move";
let ownershipStepIndex = 0;

const activeTab = () => tabs.find((t) => t.id === activeId);

function tabName(tab: Tab): string {
  if (!tab.path) return tab.exampleName ?? "Sem título";
  return tab.path.split(/[\\/]/).pop() ?? tab.path;
}

function isDirty(tab: Tab): boolean {
  const text = tab.id === activeId ? view.state.doc.toString() : tab.state.doc.toString();
  return text !== tab.saved;
}

const NEW_PROGRAM = TEMPLATE.replace("\t\t\n", '\t\tescreva("Olá Mundo")\n');
const PORS_TEMPLATE = "funcao inicio() {\n    seja mutavel idade: inteiro = 16;\n    idade += 1;\n    escreva(idade);\n}\n";

function newTab(text = NEW_PROGRAM, path: string | null = null, exampleName?: string, examplePath?: string, exercise?: ExerciseDocument, explicitLanguage?: SourceLanguage, workspaceFile?: { id: string; path: string; preferenceKey: string }): Tab {
  const language = explicitLanguage ?? detectSourceLanguage(path ?? "programa.por");
  const tab: Tab = {
    id: nextTabId++,
    path,
    language,
    exampleName,
    examplePath,
    exercise,
    workspaceId: workspaceFile?.id,
    workspacePath: workspaceFile?.path,
    workspacePreferenceKey: workspaceFile?.preferenceKey,
    state: EditorState.create({ doc: text, extensions: extensions(language) }),
    saved: text,
    diagnostics: [],
    symbols: [],
  };
  tabs.push(tab);
  activate(tab.id);
  return tab;
}

function activate(id: number) {
  const current = activeTab();
  if (current) current.state = view.state;
  const tab = tabs.find((t) => t.id === id);
  if (!tab && id !== homeId && id !== helpId) return;
  activeId = id;
  if (id === helpId) helpOpen = true;
  $("home").hidden = id !== homeId;
  $("help").hidden = id !== helpId;
  $("code").hidden = !tab;
  $("editor-exercise").hidden = !tab?.exercise;
  if (!tab) {
    rustExportGeneration++;
    renderTabs();
    renderExerciseTests();
    $("cursor").textContent = "";
    if (id === helpId) void openHelp();
    else $("home").focus();
    return;
  }
  const workspacePath = tab.workspacePath ?? tab.workspaceExercisePath;
  if (tab.workspaceId === workspaceState?.id && workspacePath) {
    selectedWorkspacePath = workspacePath;
    renderWorkspaceExplorer();
  }
  view.setState(tab.state);
  paneLayout = loadSavedPaneLayout(tab.workspacePreferenceKey);
  applyLearningMode(loadLearningMode(undefined, tab.workspacePreferenceKey), false);
  view.dispatch({ effects: [
    theme.reconfigure(editorTheme()),
    sourceLanguage.reconfigure(tab.language === "rust" ? rust() : tab.language === "portugol-rs" ? portugolRs() : portugol()),
    portugolTools.reconfigure(tab.language === "rust" ? [rustSourceLinter] : [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]),
    setDebugLine.of(tab.id === runTabId ? paused?.position.line ?? null : null),
  ] });
  applyBottomPaneSize();
  if (!explorer.hidden) applyExplorerPaneSize();
  if (!utilities.hidden) applyUtilitiesPaneSize();
  if (tab.language === "rust") view.dispatch(setDiagnostics(view.state, []));
  renderTabs();
  renderExerciseTests();
  $<HTMLElement>("rust-preview").hidden = learningMode !== "bridge" || tab.language === "rust";
  if (tab.language !== "rust") renderRustPreview(tab);
  renderMappingGutter();
  renderMessages();
  outline.show(tab.symbols, current !== tab);
  updateLibraryButtons(tab.symbols);
  renderCursor();
  if (centerActive) centerSelection();
  if (learningMode === "bridge" && tab.language !== "rust") scheduleRustExport();
  if (tab.language === "rust") {
    rustExportGeneration++;
  }
  syncLanguageControls(tab);
  view.focus();
}

function syncLanguageControls(tab: Tab) {
  const semanticLanguage = semanticSymbolLanguage(tab.language);
  const pors = semanticLanguage === "portugol-rs";
  const rustFile = tab.language === "rust";
  const busy = runId !== null || pendingEvents !== null || porsRunPending;
  for (const id of ["btn-run", "btn-debug", "btn-debug-pause", "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out"]) {
    const button = $<HTMLButtonElement>(id);
    const isRun = id === "btn-run";
    const cargoDetectionPending = rustFile && Boolean(tab.workspaceId && tab.workspacePath) && tab.cargoProject === undefined;
    const runtimeUnavailable = isRun
      ? !canRunSource(tab.language, rustcAvailable) || cargoDetectionPending || (rustFile && tab.cargoProject === true && cargoAvailable === false)
      : !canDebugSource(tab.language);
    const stateDisabled = id === "btn-debug-pause"
      ? runId === null || paused !== null
      : id === "btn-run" || id === "btn-debug"
        ? busy && paused === null
        : paused === null;
    button.disabled = runtimeUnavailable || stateDisabled;
    button.dataset.defaultTitle ??= button.title;
    const unavailableReason = cargoDetectionPending
      ? "Verificando se este arquivo pertence a um projeto Cargo."
      : rustFile && rustcAvailable !== true
      ? "Instale a toolchain Rust para executar arquivos .rs."
      : rustFile && tab.cargoProject === true && cargoAvailable === false
        ? "Instale Cargo para executar o projeto Rust."
      : rustFile && !isRun
        ? "Depuração Rust ainda não está disponível."
      : pors && isRun && rustcAvailable !== true
        ? "Instale a toolchain Rust para executar arquivos .pors."
          : "";
    if (unavailableReason) button.title = unavailableReason;
    else if (pors && isRun) button.title = "Compilar e executar o subset suportado com rustc";
    else if (rustFile && isRun && tab.cargoProject === true) button.title = "Compilar e executar o projeto Cargo (scripts e dependências executam localmente)";
    else if (rustFile && isRun) button.title = "Compilar e executar Rust 2021 com rustc (execução sem sandbox)";
    else button.title = button.dataset.defaultTitle;
    button.setAttribute("aria-description", unavailableReason);
  }
  const args = $<HTMLInputElement>("args");
  args.disabled = pors;
  args.title = pors
    ? "Parâmetros ainda não são suportados em Portugol-RS."
    : rustFile
      ? "Espaços separam itens; aspas duplas agrupam. O programa recebe argv, sem shell."
      : "";
  const format = $<HTMLButtonElement>("editor-format");
  format.disabled = !canFormatSource(tab.language, rustfmtAvailable);
  format.title = rustFile && rustfmtAvailable !== true
    ? "Instale o componente rustfmt com `rustup component add rustfmt`."
    : "Formatar código-fonte";
  for (const [name, button] of libraryButtons) {
    button.disabled = pors || rustFile || tab.symbols.some((symbol) => symbol.kind === "library" && symbol.name.split(" → ", 1)[0] === name);
    button.title = rustFile ? "Bibliotecas Portugol não se aplicam a Rust." : pors ? "Bibliotecas clássicas não se aplicam ao Portugol-RS." : `Incluir biblioteca ${name}`;
  }
  for (const id of ["editor-definition", "editor-rename"]) {
    const button = $<HTMLButtonElement>(id);
    button.disabled = rustFile;
    button.setAttribute("aria-description", rustFile ? "Navegação e renomeação de símbolos Rust ainda não estão implementadas." : "");
  }
}

async function closeTab(id: number): Promise<boolean> {
  if (id === helpId) {
    helpOpen = false;
    if (activeId === helpId) activate(tabs[tabs.length - 1]?.id ?? homeId);
    else renderTabs();
    return true;
  }
  const tab = tabs.find((t) => t.id === id);
  if (!tab) return true;
  if (isDirty(tab)) {
    const discard = await ask(`"${tabName(tab)}" tem alterações não salvas. Fechar mesmo assim?`, {
      title: "Fechar aba", kind: "warning", okLabel: "Fechar sem salvar", cancelLabel: "Cancelar",
    });
    if (!discard) return false;
  }
  const index = tabs.indexOf(tab);
  tabs.splice(index, 1);
  if (id === activeId) activate(tabs[Math.max(0, index - 1)]?.id ?? 0);
  else renderTabs();
  return true;
}

function renderTabs() {
  const bar = $("tabs");
  const home = document.createElement("button");
  home.className = "tab home-tab" + (activeId === 0 ? " active" : "");
  home.append(icon("pequeno/light_pix"), "Portugol Studio");
  home.title = "Tela inicial (Alt+Home)";
  home.setAttribute("aria-current", String(activeId === 0));
  home.onclick = () => activate(0);
  const add = document.createElement("button");
  add.className = "tab add-tab";
  add.textContent = "+";
  add.title = "Novo arquivo (Ctrl+N)";
  add.setAttribute("aria-label", "Novo arquivo");
  add.onclick = () => newTab();
  const helpTab = document.createElement("div");
  helpTab.className = "file-tab tab" + (activeId === helpId ? " active" : "");
  const helpButton = document.createElement("button");
  helpButton.textContent = "Ajuda";
  helpButton.setAttribute("aria-current", String(activeId === helpId));
  helpButton.onclick = () => activate(helpId);
  const closeHelp = document.createElement("button");
  closeHelp.className = "close";
  closeHelp.textContent = "×";
  closeHelp.setAttribute("aria-label", "Fechar ajuda");
  closeHelp.onclick = () => void closeTab(helpId);
  helpTab.append(helpButton, closeHelp);
  bar.replaceChildren(
    home,
    ...(helpOpen ? [helpTab] : []),
    ...tabs.map((tab) => {
      const el = document.createElement("div");
      el.className = "file-tab tab" + (tab.id === activeId ? " active" : "");
      el.title = tab.path ?? "Arquivo ainda não salvo";
      const name = document.createElement("button");
      name.setAttribute("aria-current", String(tab.id === activeId));
      name.textContent = tabName(tab);
      if (isDirty(tab)) name.className = "dirty";
      const close = document.createElement("button");
      close.className = "close";
      close.textContent = "×";
      close.title = "Fechar (Ctrl+Q)";
      close.onclick = (e) => {
        e.stopPropagation();
        void closeTab(tab.id);
      };
      name.prepend(icon("pequeno/light_pix"));
      el.append(name, close);
      name.onclick = () => activate(tab.id);
      el.onauxclick = (e) => e.button === 1 && void closeTab(tab.id);
      return el;
    }),
    add,
  );
  const tab = activeTab();
  const title = tab ? tabName(tab) : activeId === helpId ? "Ajuda" : "";
  document.title = title ? `${title} — Portugol Studio` : "Portugol Studio";
}

function renderCursor() {
  if (!activeTab()) return;
  const head = view.state.selection.main.head;
  const line = view.state.doc.lineAt(head);
  $("cursor").textContent = `Linha ${line.number}, coluna ${head - line.from + 1}`;
}

// ---------------------------------------------------------------------------
// Arquivos
// ---------------------------------------------------------------------------

const FILTERS = [{ name: "Programas", extensions: ["por", "pors", "rs"] }];

let workspaceError: string | null = null;
let workspaceFileCreationPending = false;
let workspaceExportPending = false;
let workspaceRenameTarget: Extract<WorkspaceEntry, { kind: "file" }> | null = null;
let workspaceRenamePending = false;
let workspaceDeleteTarget: Extract<WorkspaceEntry, { kind: "file" }> | null = null;
let workspaceDeletePending = false;

function setWorkspaceExplorerVisible(visible: boolean) {
  $("workspace-explorer").hidden = !visible;
  $("workspace-explorer-splitter").hidden = !visible;
  $("btn-explorer").setAttribute("aria-expanded", String(visible));
  if (visible) {
    applyExplorerPaneSize();
    if (!utilities.hidden) applyUtilitiesPaneSize();
  } else if (!utilities.hidden) applyUtilitiesPaneSize();
}

function renderWorkspaceExplorer() {
  $("workspace-explorer").setAttribute("aria-busy", String(workspaceLoading));
  const tree = $<HTMLUListElement>("workspace-tree");
  const message = $("workspace-tree-state");
  const name = $("workspace-name");
  const openButtons = [$<HTMLButtonElement>("home-open-folder"), $<HTMLButtonElement>("workspace-open-folder")];
  openButtons.forEach((button) => { button.disabled = workspaceLoading; });
  $<HTMLButtonElement>("workspace-export").disabled = !workspaceState || workspaceLoading || workspaceExportPending;
  $<HTMLButtonElement>("workspace-new-file").disabled = !workspaceState || workspaceLoading;
  name.hidden = !workspaceState;
  name.textContent = workspaceState?.name ?? "";

  const rows = flattenWorkspaceEntries(workspaceState?.entries ?? [], expandedWorkspaceFolders);
  tree.setAttribute("aria-label", workspaceState ? `Arquivos em ${workspaceState.name}` : "Arquivos do projeto");
  const items = rows.map(({ entry, depth, position, setSize }) => {
    const item = document.createElement("li");
    item.setAttribute("role", "none");
    const button = document.createElement("button");
    const isFolder = entry.kind === "folder";
    const isExpanded = isFolder && expandedWorkspaceFolders.has(entry.path);
    const marker = document.createElement("span");
    marker.className = "workspace-tree-marker";
    marker.setAttribute("aria-hidden", "true");
    marker.textContent = isFolder ? (isExpanded ? "▾" : "▸") : workspaceFileKind(entry.path) === "rust" ? "R" : workspaceFileKind(entry.path) === "portugol-rs" ? "RS" : workspaceFileKind(entry.path) === "exercise" ? "EX" : "POR";
    const label = document.createElement("span");
    label.className = "workspace-tree-label";
    label.textContent = entry.name;
    button.type = "button";
    button.className = `workspace-tree-item${isFolder ? " is-folder" : " is-file"}`;
    button.dataset.path = entry.path;
    button.dataset.depth = String(depth);
    button.setAttribute("role", "treeitem");
    button.setAttribute("aria-level", String(depth + 1));
    button.setAttribute("aria-posinset", String(position));
    button.setAttribute("aria-setsize", String(setSize));
    button.setAttribute("aria-selected", String(!isFolder && selectedWorkspacePath === entry.path));
    button.style.paddingInlineStart = "8px";
    if (isFolder) {
      button.setAttribute("aria-expanded", String(isExpanded));
      button.setAttribute("aria-label", `${entry.name}, pasta${isExpanded ? ", expandida" : ", recolhida"}`);
      button.onclick = () => {
        if (expandedWorkspaceFolders.has(entry.path)) expandedWorkspaceFolders.delete(entry.path);
        else expandedWorkspaceFolders.add(entry.path);
        renderWorkspaceExplorer();
        tree.querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(entry.path)}"]`)?.focus();
      };
    } else {
      button.setAttribute("aria-label", `${entry.name}, arquivo`);
      button.onclick = () => {
        selectedWorkspacePath = entry.path;
        renderWorkspaceExplorer();
        void openWorkspaceFile(entry).catch((error) => {
          workspaceError = String(error);
          renderWorkspaceExplorer();
          setStatus(String(error), "error");
        });
      };
    }
    button.append(marker, label);
    const row = document.createElement("div");
    row.className = "workspace-tree-row";
    row.style.paddingInlineStart = `${depth * 16}px`;
    row.append(button);
    if (!isFolder) {
      const actions = document.createElement("div");
      actions.className = "workspace-file-actions";
      const renameButton = document.createElement("button");
      renameButton.type = "button";
      renameButton.textContent = "✎";
      renameButton.title = `Renomear ${entry.name}`;
      renameButton.setAttribute("aria-label", `Renomear ${entry.name}`);
      renameButton.dataset.action = "workspace-rename-file";
      renameButton.onclick = (event) => {
        event.stopPropagation();
        openWorkspaceRenameDialog(entry);
      };
      actions.append(renameButton);
      const deleteButton = document.createElement("button");
      deleteButton.type = "button";
      deleteButton.textContent = "🗑";
      deleteButton.title = `Excluir ${entry.name}`;
      deleteButton.setAttribute("aria-label", `Excluir ${entry.name}`);
      deleteButton.dataset.action = "workspace-delete-file";
      deleteButton.onclick = (event) => {
        event.stopPropagation();
        openWorkspaceDeleteDialog(entry);
      };
      actions.append(deleteButton);
      actions.dataset.testid = "file-actions";
      row.append(actions);
    }
    item.append(row);
    return item;
  });
  tree.replaceChildren(...items);
  tree.hidden = rows.length === 0;

  if (workspaceLoading) {
    message.textContent = "Abrindo pasta…";
    message.hidden = false;
  } else if (workspaceError) {
    message.textContent = workspaceError;
    message.hidden = false;
  } else if (!workspaceState) {
    message.textContent = "Abra uma pasta para navegar pelos arquivos do projeto.";
    message.hidden = false;
  } else if (rows.length === 0) {
    message.textContent = "Esta pasta não contém arquivos .por, .pors, .rs ou .pex.";
    message.hidden = false;
  } else {
    message.textContent = "";
    message.hidden = true;
  }
}

async function openWorkspaceFolder() {
  if (workspaceLoading) return;
  const request = ++workspaceRequest;
  workspaceLoading = true;
  workspaceError = null;
  renderWorkspaceExplorer();
  try {
    const selected = await api.openWorkspace();
    if (request !== workspaceRequest || !selected) return;
    workspaceState = selected;
    selectedWorkspacePath = null;
    expandedWorkspaceFolders.clear();
    workspaceError = null;
    setWorkspaceExplorerVisible(true);
    if (!activeTab()) {
      const previousTab = tabs[tabs.length - 1];
      if (previousTab) activate(previousTab.id);
      else newTab();
    }
    renderWorkspaceExplorer();
    setStatus(`Pasta aberta: ${selected.name}`, "ok");
  } catch (error) {
    if (request === workspaceRequest) {
      workspaceError = String(error);
      renderWorkspaceExplorer();
      setStatus(String(error), "error");
    }
  } finally {
    if (request === workspaceRequest) {
      workspaceLoading = false;
      renderWorkspaceExplorer();
    }
  }
}

async function restoreWorkspaceOnStartup() {
  const request = ++workspaceRequest;
  try {
    const restored = await api.restoreWorkspace();
    if (request !== workspaceRequest || !restored) return;
    workspaceState = restored;
    selectedWorkspacePath = null;
    expandedWorkspaceFolders.clear();
    setWorkspaceExplorerVisible(true);
    renderWorkspaceExplorer();
    setStatus(`Última pasta reaberta: ${restored.name}`, "ok");
  } catch (error) {
    if (request === workspaceRequest) setStatus(String(error), "error");
  }
}

async function openWorkspaceFile(entry: Extract<WorkspaceEntry, { kind: "file" }>) {
  const selected = workspaceState;
  if (!selected) return;
  const kind = workspaceFileKind(entry.path);
  if (!kind) return;
  if (kind === "exercise") {
    const existingExercise = tabs.find((tab) =>
      tab.workspaceId === selected.id && tab.workspaceExercisePath === entry.path,
    );
    if (existingExercise) {
      activate(existingExercise.id);
      return;
    }
    const loaded = await api.readWorkspaceFile(selected.id, entry.path);
    if (workspaceState?.id !== selected.id) return;
    const exercise = await api.inspectExercise(loaded.source);
    if (workspaceState?.id !== selected.id) return;
    openExercise({ exercise, name: loaded.path.split("/").pop() ?? entry.name, contents: loaded.source }, {
      id: selected.id,
      path: loaded.relativePath,
      preferenceKey: selected.preferenceKey,
      exercise: true,
    });
    setStatus(`Exercício aberto: ${entry.path}`, "ok");
    return;
  }

  const existing = tabs.find((tab) => tab.workspaceId === selected.id && tab.workspacePath === entry.path);
  if (existing) {
    activate(existing.id);
    return;
  }
  const loaded = await api.readWorkspaceFile(selected.id, entry.path);
  if (workspaceState?.id !== selected.id) return;
  const current = activeTab();
  if (current && !current.path && !current.exampleName && !isDirty(current) && tabs.length === 1) tabs = [];
  const tab = newTab(loaded.source, loaded.path, undefined, undefined, undefined, undefined, {
    id: selected.id,
    path: loaded.relativePath,
    preferenceKey: selected.preferenceKey,
  });
  tab.workspacePath = loaded.relativePath;
  if (tab.language === "rust") {
    void api.workspaceCargoProjectAvailable(selected.id, loaded.relativePath).then((available) => {
      tab.cargoProject = available;
      if (activeTab() === tab) {
        syncLanguageControls(tab);
        forceLinting(view);
      }
    }).catch((error: unknown) => {
      tab.cargoProject = false;
      console.error("Não foi possível verificar Cargo.toml:", error);
      if (activeTab() === tab) syncLanguageControls(tab);
    });
  }
  rememberFile(loaded.path);
  setStatus(`Aberto: ${entry.path}`, "ok");
}

function showWorkspaceCreateError(message: string | null) {
  const error = $("workspace-create-error");
  error.textContent = message ?? "";
  error.hidden = !message;
  $("workspace-create-name").setAttribute("aria-invalid", String(Boolean(message)));
}

function showWorkspaceRenameError(message: string | null) {
  const error = $("workspace-rename-error");
  error.textContent = message ?? "";
  error.hidden = !message;
  $("workspace-rename-name").setAttribute("aria-invalid", String(Boolean(message)));
}

function openWorkspaceRenameDialog(entry: Extract<WorkspaceEntry, { kind: "file" }>) {
  if (!workspaceState || workspaceRenamePending) return;
  workspaceRenameTarget = entry;
  const input = $<HTMLInputElement>("workspace-rename-name");
  showWorkspaceRenameError(null);
  $<HTMLFormElement>("workspace-rename-form").reset();
  input.value = entry.name;
  $<HTMLDialogElement>("workspace-rename-dialog").showModal();
  const dot = entry.name.lastIndexOf(".");
  input.setSelectionRange(0, dot > 0 ? dot : entry.name.length);
  input.focus();
}

function replaceRecentFile(oldPath: string, newPath: string) {
  recent = [newPath, ...recent.filter((path) => path !== oldPath && path !== newPath)].slice(0, 6);
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recent));
  } catch {
    // A lista em memória permanece disponível.
  }
  renderRecent();
}

function forgetRecentFile(path: string) {
  recent = recent.filter((entry) => entry !== path);
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recent));
  } catch {
    // A lista em memória permanece disponível.
  }
  renderRecent();
}

async function submitWorkspaceRenameDialog(event: SubmitEvent) {
  event.preventDefault();
  const workspace = workspaceState;
  const original = workspaceRenameTarget;
  if (!workspace || !original) return;

  const input = $<HTMLInputElement>("workspace-rename-name");
  let newName: string;
  try {
    newName = workspaceRenameFileName(input.value, original.path);
  } catch (error) {
    showWorkspaceRenameError(String(error).replace(/^Error: /, ""));
    input.focus();
    return;
  }
  if (newName === original.name) {
    $<HTMLDialogElement>("workspace-rename-dialog").close();
    workspaceRenameTarget = null;
    return;
  }

  const submit = $<HTMLButtonElement>("workspace-rename-submit");
  const cancel = $<HTMLButtonElement>("workspace-rename-cancel");
  workspaceRenamePending = true;
  submit.disabled = true;
  cancel.disabled = true;
  showWorkspaceRenameError(null);
  try {
    const openTabs = tabs.filter((tab) => tab.workspaceId === workspace.id &&
      (tab.workspacePath === original.path || tab.workspaceExercisePath === original.path));
    const previousActive = activeId;
    const preparation = await prepareWorkspaceRename({
      tabs: openTabs,
      sourcePath: original.path,
      isDirty,
      confirmSave: () => ask(`"${original.name}" tem alterações não salvas. Salvar antes de renomear?`, {
        title: "Renomear arquivo",
        kind: "warning",
        okLabel: "Salvar e renomear",
        cancelLabel: "Cancelar",
      }),
      saveTab: async (tab) => {
        activate(tab.id);
        try {
          return await save();
        } catch {
          return false;
        }
      },
    });
    activate(previousActive);
    if (preparation === "cancelled") return;
    if (preparation === "save-failed") {
      showWorkspaceRenameError("Não foi possível salvar. O arquivo não foi renomeado.");
      return;
    }

    const renamed = await api.renameWorkspaceFile(workspace.id, original.path, newName);
    if (workspaceState?.id !== workspace.id) return;
    for (const oldPath of applyWorkspaceRenameToTabs(openTabs, original.path, renamed.file)) {
      replaceRecentFile(oldPath, renamed.file.path);
    }
    workspaceState = renamed.listing;
    workspaceError = null;
    selectedWorkspacePath = renamed.file.relativePath;
    renderWorkspaceExplorer();
    renderTabs();
    $<HTMLDialogElement>("workspace-rename-dialog").close();
    $<HTMLFormElement>("workspace-rename-form").reset();
    workspaceRenameTarget = null;
    rememberFile(renamed.file.path);
    setStatus(`Renomeado: ${newName}`, "ok");
    queueMicrotask(() => $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(renamed.file.relativePath)}"]`)?.focus());
  } catch (error) {
    const message = String(error).replace(/^Error: /, "");
    showWorkspaceRenameError(message);
    input.focus();
  } finally {
    workspaceRenamePending = false;
    submit.disabled = false;
    cancel.disabled = false;
  }
}

function showWorkspaceDeleteError(message: string | null) {
  const error = $("workspace-delete-error");
  error.textContent = message ?? "";
  error.hidden = !message;
}

function updateWorkspaceDeleteDialog(entry: Extract<WorkspaceEntry, { kind: "file" }>, workspaceId: string) {
  const dirty = tabs.some((tab) => tab.workspaceId === workspaceId &&
    (tab.workspacePath === entry.path || tab.workspaceExercisePath === entry.path) && isDirty(tab));
  $("workspace-delete-title").textContent = `Excluir ${entry.name}?`;
  const warning = $("workspace-delete-warning");
  warning.textContent = dirty
    ? "Há alterações não salvas em abas abertas. Salvar mantém o arquivo; excluir com descarte fecha essas abas."
    : "";
  warning.hidden = !dirty;
  $<HTMLButtonElement>("workspace-delete-save").hidden = !dirty;
  $<HTMLButtonElement>("workspace-delete-submit").textContent = dirty
    ? "Excluir e descartar alterações"
    : "Excluir arquivo";
}

function openWorkspaceDeleteDialog(entry: Extract<WorkspaceEntry, { kind: "file" }>) {
  if (!workspaceState || workspaceDeletePending) return;
  workspaceDeleteTarget = entry;
  showWorkspaceDeleteError(null);
  updateWorkspaceDeleteDialog(entry, workspaceState.id);
  $<HTMLDialogElement>("workspace-delete-dialog").showModal();
  $("workspace-delete-cancel").focus();
}

async function chooseWorkspaceDeleteAction(choice: "save" | "delete" | "discard") {
  const workspace = workspaceState;
  const target = workspaceDeleteTarget;
  if (!workspace || !target || workspaceDeletePending) return;

  const buttons = [
    $<HTMLButtonElement>("workspace-delete-save"),
    $<HTMLButtonElement>("workspace-delete-submit"),
    $<HTMLButtonElement>("workspace-delete-cancel"),
  ];
  workspaceDeletePending = true;
  buttons.forEach((button) => { button.disabled = true; });
  showWorkspaceDeleteError(null);
  try {
    const previousActive = activeId;
    const result = await deleteWorkspaceFileFlow({
      tabs,
      workspaceId: workspace.id,
      filePath: target.path,
      choice,
      isDirty,
      saveTab: async (tab) => {
        activate(tab.id);
        try {
          return await save();
        } catch {
          return false;
        }
      },
      deleteFile: () => api.deleteWorkspaceFile(workspace.id, target.path),
    });
    if (choice === "save") activate(previousActive);
    if (result.status === "cancelled") return;
    if (result.status === "save-failed") {
      showWorkspaceDeleteError("Não foi possível salvar todas as alterações. O arquivo foi mantido.");
      updateWorkspaceDeleteDialog(target, workspace.id);
      return;
    }
    if (result.status === "needs-decision") {
      showWorkspaceDeleteError("Há alterações não salvas. Escolha salvar ou descartar antes de excluir.");
      updateWorkspaceDeleteDialog(target, workspace.id);
      return;
    }
    if (result.status === "saved") {
      $<HTMLDialogElement>("workspace-delete-dialog").close();
      workspaceDeleteTarget = null;
      setStatus(`Alterações salvas; arquivo mantido: ${target.name}`, "ok");
      return;
    }

    if (workspaceState?.id !== workspace.id) return;
    const deleted = result.deletion;
    workspaceState = deleted.listing;
    workspaceError = null;
    if (selectedWorkspacePath === target.path) selectedWorkspacePath = null;
    forgetRecentFile(deleted.file.path);
    tabs = result.remainingTabs;
    const activeWasDeleted = result.deletedTabs.some((tab) => tab.id === activeId);
    renderWorkspaceExplorer();
    if (activeWasDeleted) activate(tabs[tabs.length - 1]?.id ?? homeId);
    else renderTabs();
    $<HTMLDialogElement>("workspace-delete-dialog").close();
    workspaceDeleteTarget = null;
    setStatus(`Arquivo excluído: ${target.name}`, "ok");
    queueMicrotask(() => {
      const first = $("workspace-tree").querySelector<HTMLButtonElement>("[role=treeitem]");
      if (first) first.focus();
      else $("workspace-new-file").focus();
    });
  } catch (error) {
    const message = String(error).replace(/^Error: /, "");
    showWorkspaceDeleteError(message);
    setStatus(message, "error");
  } finally {
    workspaceDeletePending = false;
    buttons.forEach((button) => { button.disabled = false; });
  }
}

function openWorkspaceCreateDialog() {
  if (!workspaceState) return;
  const dialog = $<HTMLDialogElement>("workspace-create-dialog");
  showWorkspaceCreateError(null);
  $<HTMLFormElement>("workspace-create-form").reset();
  dialog.showModal();
  $("workspace-create-name").focus();
}

function updateWorkspaceExportSelection() {
  const selectedCount = document.querySelectorAll<HTMLInputElement>("#workspace-export-files input:checked").length;
  document.querySelectorAll<HTMLInputElement>("#workspace-export-files input").forEach((input) => {
    input.disabled = workspaceExportPending;
  });
  const submit = $<HTMLButtonElement>("workspace-export-submit");
  submit.disabled = workspaceExportPending || selectedCount === 0;
  $<HTMLButtonElement>("workspace-export-select-all").disabled = workspaceExportPending;
  $<HTMLButtonElement>("workspace-export-select-none").disabled = workspaceExportPending;
  $<HTMLButtonElement>("workspace-export-cancel").disabled = workspaceExportPending;
  if (!workspaceExportPending) $("workspace-export-status").textContent = workspaceExportStatus(selectedCount);
}

function renderWorkspaceExportSelection() {
  const files = collectWorkspaceFiles(workspaceState?.entries ?? []);
  const list = $("workspace-export-files");
  list.replaceChildren();
  for (const [index, entry] of files.entries()) {
    const label = document.createElement("label");
    label.className = "workspace-export-file";
    const checkbox = document.createElement("input");
    checkbox.type = "checkbox";
    checkbox.value = entry.path;
    checkbox.checked = true;
    checkbox.id = `workspace-export-file-${index}`;
    checkbox.addEventListener("change", updateWorkspaceExportSelection);
    const name = document.createElement("span");
    name.textContent = entry.path;
    label.htmlFor = checkbox.id;
    label.append(checkbox, name);
    list.append(label);
  }
  $("workspace-export-empty").hidden = files.length > 0;
  $<HTMLButtonElement>("workspace-export-select-all").hidden = files.length === 0;
  $<HTMLButtonElement>("workspace-export-select-none").hidden = files.length === 0;
  updateWorkspaceExportSelection();
}

function openWorkspaceExportDialog() {
  if (!workspaceState || workspaceExportPending) return;
  $("workspace-export-error").hidden = true;
  $("workspace-export-error").textContent = "";
  renderWorkspaceExportSelection();
  const dialog = $<HTMLDialogElement>("workspace-export-dialog");
  dialog.showModal();
  document.querySelector<HTMLInputElement>("#workspace-export-files input")?.focus();
}

async function submitWorkspaceExport() {
  const workspace = workspaceState;
  if (!workspace || workspaceExportPending) return;
  const relativePaths = [...document.querySelectorAll<HTMLInputElement>("#workspace-export-files input:checked")]
    .map((input) => input.value);
  const error = $("workspace-export-error");
  error.hidden = true;
  error.textContent = "";
  if (relativePaths.length === 0) {
    error.textContent = "Selecione ao menos um arquivo para exportar.";
    error.hidden = false;
    return;
  }

  workspaceExportPending = true;
  $("workspace-export-dialog").setAttribute("aria-busy", "true");
  $("workspace-export-status").textContent = "Escolha uma pasta de destino no seletor do sistema…";
  updateWorkspaceExportSelection();
  renderWorkspaceExplorer();
  let cancelled = false;
  try {
    const result = await api.exportWorkspaceFiles(workspace.id, relativePaths);
    if (!result) {
      cancelled = true;
      return;
    }
    $<HTMLDialogElement>("workspace-export-dialog").close();
    setStatus(`${result.fileCount} ${result.fileCount === 1 ? "arquivo exportado" : "arquivos exportados"} para a pasta “${result.projectName}”.`, "ok");
  } catch (message) {
    error.textContent = typeof message === "string" ? message : String(message);
    error.hidden = false;
  } finally {
    workspaceExportPending = false;
    $("workspace-export-dialog").removeAttribute("aria-busy");
    updateWorkspaceExportSelection();
    if (cancelled) {
      const selectedCount = document.querySelectorAll<HTMLInputElement>("#workspace-export-files input:checked").length;
      $("workspace-export-status").textContent = workspaceExportStatus(selectedCount, "cancelled");
    }
    renderWorkspaceExplorer();
  }
}

async function submitWorkspaceCreateDialog(event: SubmitEvent) {
  event.preventDefault();
  const selected = workspaceState;
  if (!selected) return;

  const name = $<HTMLInputElement>("workspace-create-name");
  const language = $<HTMLSelectElement>("workspace-create-language").value as "por" | "pors" | "rs";
  let fileName: string;
  try {
    fileName = workspaceSourceFileName(name.value, language);
  } catch (error) {
    showWorkspaceCreateError(String(error).replace(/^Error: /, ""));
    name.focus();
    return;
  }

  const submit = $<HTMLButtonElement>("workspace-create-submit");
  const cancel = $<HTMLButtonElement>("workspace-create-cancel");
  workspaceFileCreationPending = true;
  submit.disabled = true;
  cancel.disabled = true;
  showWorkspaceCreateError(null);
  try {
    const refreshed = await api.createWorkspaceFile(selected.id, fileName);
    if (workspaceState?.id !== selected.id) return;
    workspaceState = refreshed;
    workspaceError = null;
    selectedWorkspacePath = fileName;
    setWorkspaceExplorerVisible(true);
    renderWorkspaceExplorer();
    $<HTMLDialogElement>("workspace-create-dialog").close();
    $<HTMLFormElement>("workspace-create-form").reset();
    const created = refreshed.entries.find((entry): entry is Extract<WorkspaceEntry, { kind: "file" }> =>
      entry.kind === "file" && entry.path === fileName,
    );
    if (created) await openWorkspaceFile(created);
    else setStatus(`Arquivo criado: ${fileName}`, "ok");
  } catch (error) {
    const message = String(error).replace(/^Error: /, "");
    if ($<HTMLDialogElement>("workspace-create-dialog").open) {
      showWorkspaceCreateError(message);
      name.focus();
    } else {
      workspaceError = message;
      renderWorkspaceExplorer();
      setStatus(message, "error");
    }
  } finally {
    workspaceFileCreationPending = false;
    submit.disabled = false;
    cancel.disabled = false;
  }
}

$("home-open-folder").onclick = () => void openWorkspaceFolder();
$("workspace-open-folder").onclick = () => void openWorkspaceFolder();
$("workspace-export").onclick = openWorkspaceExportDialog;
$("workspace-export-select-all").onclick = () => {
  document.querySelectorAll<HTMLInputElement>("#workspace-export-files input").forEach((input) => { input.checked = true; });
  updateWorkspaceExportSelection();
};
$("workspace-export-select-none").onclick = () => {
  document.querySelectorAll<HTMLInputElement>("#workspace-export-files input").forEach((input) => { input.checked = false; });
  updateWorkspaceExportSelection();
};
$("workspace-export-submit").onclick = () => void submitWorkspaceExport();
$<HTMLButtonElement>("workspace-export-cancel").onclick = () => $<HTMLDialogElement>("workspace-export-dialog").close();
$<HTMLDialogElement>("workspace-export-dialog").addEventListener("cancel", (event) => {
  if (workspaceExportPending) event.preventDefault();
});
$("workspace-new-file").onclick = openWorkspaceCreateDialog;
$("workspace-create-form").addEventListener("submit", (event) => void submitWorkspaceCreateDialog(event as SubmitEvent));
$("workspace-create-dialog").addEventListener("cancel", (event) => {
  if (workspaceFileCreationPending) event.preventDefault();
});
$("workspace-create-cancel").onclick = () => {
  $<HTMLDialogElement>("workspace-create-dialog").close();
  $<HTMLFormElement>("workspace-create-form").reset();
  showWorkspaceCreateError(null);
};
$("workspace-rename-form").addEventListener("submit", (event) => void submitWorkspaceRenameDialog(event as SubmitEvent));
$("workspace-rename-dialog").addEventListener("cancel", (event) => {
  if (workspaceRenamePending) event.preventDefault();
});
$("workspace-rename-cancel").onclick = () => {
  $<HTMLDialogElement>("workspace-rename-dialog").close();
  $<HTMLFormElement>("workspace-rename-form").reset();
  workspaceRenameTarget = null;
  showWorkspaceRenameError(null);
};
$<HTMLButtonElement>("workspace-delete-save").onclick = () => void chooseWorkspaceDeleteAction("save");
$<HTMLButtonElement>("workspace-delete-submit").onclick = () => {
  const choice = tabs.some((tab) => tab.workspaceId === workspaceState?.id &&
    (tab.workspacePath === workspaceDeleteTarget?.path || tab.workspaceExercisePath === workspaceDeleteTarget?.path) && isDirty(tab))
    ? "discard"
    : "delete";
  void chooseWorkspaceDeleteAction(choice);
};
$("workspace-delete-dialog").addEventListener("cancel", (event) => {
  if (workspaceDeletePending) event.preventDefault();
  else workspaceDeleteTarget = null;
});
$<HTMLButtonElement>("workspace-delete-cancel").onclick = () => {
  if (workspaceDeletePending) return;
  $<HTMLDialogElement>("workspace-delete-dialog").close();
  workspaceDeleteTarget = null;
  showWorkspaceDeleteError(null);
};
$("btn-explorer").onclick = () => {
  if (!workspaceState) {
    void openWorkspaceFolder();
    return;
  }
  setWorkspaceExplorerVisible(Boolean($("workspace-explorer").hidden));
};

$("workspace-tree").addEventListener("keydown", (event) => {
  if (!(event instanceof KeyboardEvent)) return;
  const current = event.target instanceof HTMLButtonElement ? event.target : null;
  if (!current?.matches("[role=treeitem]")) return;
  const items = [...$("workspace-tree").querySelectorAll<HTMLButtonElement>("[role=treeitem]")];
  const index = items.indexOf(current);
  const focus = (item?: HTMLButtonElement) => {
    if (item) {
      event.preventDefault();
      item.focus();
    }
  };
  if (event.key === "ArrowDown") focus(items[index + 1]);
  else if (event.key === "ArrowUp") focus(items[index - 1]);
  else if (event.key === "Home") focus(items[0]);
  else if (event.key === "End") focus(items[items.length - 1]);
  else if (event.key === "ArrowRight" && current.classList.contains("is-folder")) {
    const path = current.dataset.path!;
    if (!expandedWorkspaceFolders.has(path)) {
      event.preventDefault();
      expandedWorkspaceFolders.add(path);
      renderWorkspaceExplorer();
      $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(path)}"]`)?.focus();
    } else if (items[index + 1]?.dataset.depth === String(Number(current.dataset.depth) + 1)) {
      focus(items[index + 1]);
    }
  } else if (event.key === "ArrowLeft") {
    const path = current.dataset.path!;
    if (current.classList.contains("is-folder") && expandedWorkspaceFolders.has(path)) {
      event.preventDefault();
      expandedWorkspaceFolders.delete(path);
      renderWorkspaceExplorer();
      $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(path)}"]`)?.focus();
    } else {
      const depth = Number(current.dataset.depth);
      focus(items.slice(0, index).reverse().find((item) => Number(item.dataset.depth) === depth - 1));
    }
  }
});

renderWorkspaceExplorer();
void restoreWorkspaceOnStartup();

const RECENT_KEY = "portugol.recent";

function loadRecent(): string[] {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(RECENT_KEY) ?? "[]");
    if (Array.isArray(saved)) return saved.filter((path): path is string => typeof path === "string").slice(0, 6);
  } catch {
    // A lista continua funcionando nesta sessão quando o armazenamento está indisponível.
  }
  return [];
}

let recent = loadRecent();

function renderRecent() {
  $("recent").hidden = recent.length === 0;
  $("recent-files").replaceChildren(...recent.map((path) => {
    const button = document.createElement("button");
    const name = document.createElement("span");
    name.textContent = path.split(/[\\/]/).pop() ?? path;
    button.append(icon("grande/file"), name);
    button.title = path;
    button.onclick = () => void openFile(path).catch((error) => setStatus(String(error), "error"));
    return button;
  }));
}

function rememberFile(path: string) {
  recent = [path, ...recent.filter((entry) => entry !== path)].slice(0, 6);
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recent));
  } catch {
    // A lista em memória permanece disponível.
  }
  renderRecent();
}

async function openFile(path: string) {
  const existing = tabs.find((tab) => tab.path === path);
  if (existing) {
    activate(existing.id);
  } else {
    const text = await api.readFile(path);
    const current = activeTab();
    if (current && !current.path && !current.exampleName && !isDirty(current) && tabs.length === 1) {
      tabs = [];
    }
    newTab(text, path);
  }
  rememberFile(path);
}

async function openFiles() {
  try {
    const selected = await openDialog({ multiple: true, filters: FILTERS });
    if (!selected) return;
    for (const path of Array.isArray(selected) ? selected : [selected]) {
      await openFile(path);
    }
  } catch (error) {
    setStatus(String(error), "error");
  }
}

async function save(as = false): Promise<boolean> {
  const tab = activeTab();
  if (!tab) return false;
  let path = tab.path;
  let workspaceFile = Boolean(tab.workspaceId && tab.workspacePath);
  if (!path || as) {
    const extension = tab.language === "portugol-rs" ? "pors" : tab.language === "rust" ? "rs" : "por";
    const exerciseName = tab.exercise ? tab.exercise.name.replace(/\.pex$/i, "") + ".por" : undefined;
    const filters = tab.language === "rust" ? [{ name: "Rust", extensions: ["rs"] }] : FILTERS;
    path = await saveDialog({ filters, defaultPath: tab.path ?? exerciseName ?? tab.exampleName ?? `programa.${extension}` });
    if (!path) return false;
    path = ensureSourceExtension(path, tab.language);
    workspaceFile = false;
  }
  const text = view.state.doc.toString();
  try {
    if (workspaceFile && tab.workspaceId && tab.workspacePath) {
      await api.writeWorkspaceFile(tab.workspaceId, tab.workspacePath, text);
    } else {
      await api.writeFile(path, text);
    }
  } catch (error) {
    setStatus(String(error), "error");
    return false;
  }
  tab.path = path;
  if (!workspaceFile) {
    tab.workspaceId = undefined;
    tab.workspacePath = undefined;
    tab.workspaceExercisePath = undefined;
  }
  const previousLanguage = tab.language;
  tab.language = detectSourceLanguage(path);
  const languageChanged = tab.language !== previousLanguage;
  if (languageChanged && tab.rustPreview?.artifact) {
    tab.rustPreview = { ...tab.rustPreview, pending: false, stale: true, error: undefined };
  }
  if (tab.id === activeId) {
    view.dispatch({ effects: [
      sourceLanguage.reconfigure(tab.language === "rust" ? rust() : tab.language === "portugol-rs" ? portugolRs() : portugol()),
      portugolTools.reconfigure(tab.language === "rust" ? [rustSourceLinter] : [portugolFolding, portugolLinter, runtimeError, breakpoints, breakpointGutter, debugLine]),
    ] });
    syncLanguageControls(tab);
    if (languageChanged) {
      rustExportGeneration++;
      if (rustExportTimer !== undefined) window.clearTimeout(rustExportTimer);
      rustExportTimer = undefined;
      $<HTMLElement>("rust-preview").hidden = learningMode !== "bridge" || tab.language === "rust";
      renderRustPreview(tab);
      if (learningMode === "bridge" && tab.language !== "rust") scheduleRustExport();
      else renderMappingGutter();
    }
  }
  if (tab.language === "rust") {
    if (tab.id === activeId) view.dispatch(setDiagnostics(view.state, []));
    tab.diagnostics = [];
    tab.symbols = [];
    renderMessages();
    outline.show([]);
    updateLibraryButtons([]);
  }
  tab.saved = text;
  rememberFile(path);
  renderTabs();
  setStatus(`Salvo em ${path}`, "ok");
  return true;
}

// ---------------------------------------------------------------------------
// Execução e console
// ---------------------------------------------------------------------------

let runId: number | null = null;
let runTabId: number | null = null;
let pendingEvents: PendingRunEvent[] | null = null;
let rustcAvailable: boolean | null = null;
let cargoAvailable: boolean | null = null;
let rustfmtAvailable: boolean | null = null;
let porsRunPending = false;
let paused: api.DebugInfo | null = null;
const output = $("console-output");
const consoleBox = $("console");
const inputForm = $<HTMLFormElement>("console-form");
const inputField = $<HTMLInputElement>("console-input");

function setStatus(text: string, kind: "" | "ok" | "error" = "") {
  const status = $("status");
  status.textContent = text;
  status.className = kind;
}

function appendConsole(text: string, className?: string) {
  const atBottom = consoleBox.scrollTop + consoleBox.clientHeight >= consoleBox.scrollHeight - 4;
  if (className) {
    const span = document.createElement("span");
    span.className = className;
    span.textContent = text;
    output.append(span);
  } else {
    output.append(text);
  }
  if (atBottom) consoleBox.scrollTop = consoleBox.scrollHeight;
}

function setRunning(running: boolean) {
  const language = activeTab()?.language;
  const canRun = language !== undefined && canRunSource(language, rustcAvailable);
  const canDebug = language !== undefined && canDebugSource(language);
  const isBusy = running && paused === null;
  $<HTMLButtonElement>("btn-run").disabled = !canRun || isBusy;
  $<HTMLButtonElement>("btn-debug").disabled = !canDebug || isBusy;
  $<HTMLButtonElement>("btn-stop").disabled = !running || runId === null;
  $<HTMLButtonElement>("btn-debug-pause").disabled = !canDebug || !running || paused !== null;
  for (const id of [
    "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out",
  ]) {
    $<HTMLButtonElement>(id).disabled = !canDebug || paused === null;
  }
  if (!running) inputForm.hidden = true;
  $("debug-vars").hidden = paused === null || paused.variables.length === 0;
  $("inspector-empty").hidden = !$("debug-vars").hidden;
  $("inspector-empty").textContent = paused
    ? "Nenhuma variável neste ponto."
    : "Pause a execução para inspecionar as variáveis.";
}

/** Linhas (base 1) dos pontos de parada no estado do editor. */
function breakpointLines(state: EditorState): number[] {
  return [...state.field(breakpoints)]
    .map((pos) => state.doc.lineAt(Math.min(pos, state.doc.length)).number)
    .sort((a, b) => a - b);
}

/** Pontos de parada podem mudar enquanto o programa está em execução. */
function syncBreakpoints(state: EditorState) {
  if (runId === null || activeId !== runTabId) return;
  void api.debugSetBreakpoints(runId, state.doc.toString(), breakpointLines(state)).catch(() => {});
}

function setPaused(info: api.DebugInfo | null) {
  paused = info;
  if (activeId === runTabId) view.dispatch({ effects: setDebugLine.of(info?.position.line ?? null) });
  setRunning(runId !== null);
  if (info) {
    renderDebug(info);
    showUtilities(true);
    showPanel("debug");
    if (activeId === runTabId) goTo(info.position.offset);
    const activeRunTab = tabs.find(({ id }) => id === runTabId);
    setStatus(activeRunTab?.language === "portugol-rs"
      ? `VM Portugol-RS pausada na linha ${info.position.line}.`
      : `Pausado na linha ${info.position.line}.`);
  }
}

async function debugCommand(command: (id: number) => Promise<void>) {
  if (runId === null || paused === null) return;
  const id = runId;
  view.dispatch({ effects: setDebugLine.of(null) });
  paused = null;
  setRunning(true);
  setStatus(activeTab()?.language === "portugol-rs" ? "VM Portugol-RS executando…" : "Executando…");
  try {
    await command(id);
  } catch {
    // A execução pode ter terminado enquanto o comando era enviado.
  }
}

function renderDebug(info: api.DebugInfo) {
  $("debug-stack").replaceChildren(...info.stack.map((frame, index) => {
    const li = document.createElement("li");
    li.textContent = `${frame.function} — linha ${frame.position.line}:${frame.position.column}`;
    if (index === 0) li.className = "current";
    li.onclick = () => {
      if (runTabId === null || !tabs.some(({ id }) => id === runTabId)) return;
      if (activeId !== runTabId) activate(runTabId);
      goTo(frame.position.offset);
    };
    return li;
  }));
  const body = document.querySelector<HTMLTableSectionElement>("#debug-vars tbody")!;
  body.replaceChildren(...info.variables.map((variable) => {
    const row = document.createElement("tr");
    for (const text of [variable.name, variable.ty, variable.value, variable.scope]) {
      const cell = document.createElement("td");
      cell.textContent = text;
      row.append(cell);
    }
    return row;
  }));
}

function parseArgs(text: string): string[] {
  // Aspas agrupam: `"João Silva" 19` -> ["João Silva", "19"].
  return [...text.matchAll(/"([^"]*)"|(\S+)/g)].map((m) => m[1] ?? m[2]);
}

function chooseCargoTarget(targets: api.CargoRunTarget[]): Promise<api.CargoRunTarget | null> {
  const dialog = $<HTMLDialogElement>("cargo-target-dialog");
  const select = $<HTMLSelectElement>("cargo-target-select");
  select.replaceChildren(...targets.map((target, index) => {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = `${target.packageName} — ${target.targetName}`;
    return option;
  }));
  $<HTMLButtonElement>("cargo-target-cancel").onclick = () => dialog.close("cancel");
  dialog.returnValue = "";
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => {
      const index = Number(select.value);
      resolve(dialog.returnValue === "run" && Number.isInteger(index) ? targets[index] ?? null : null);
    }, { once: true });
    dialog.showModal();
    select.focus();
  });
}

async function runProgram(pauseOnStart = false) {
  if (runId !== null || pendingEvents !== null || porsRunPending) return;
  const tab = activeTab();
  if (!tab) return;
  if (tab.language === "rust") {
    if (tab.workspaceId && tab.workspacePath && tab.cargoProject === undefined) {
      setStatus("Verificando se este arquivo pertence a um projeto Cargo.");
      return;
    }
    if (pauseOnStart) {
      setStatus("Depuração de arquivos Rust ainda não está disponível.");
      return;
    }
    if (tab.cargoProject === true && cargoAvailable !== true) {
      setStatus("Instale Cargo para executar o projeto Rust.", "error");
      return;
    }
    if (rustcAvailable !== true) {
      setStatus("Instale a toolchain Rust para executar arquivos .rs.", "error");
      return;
    }
    let rustArgs: string[];
    try {
      rustArgs = parseProcessArguments($<HTMLInputElement>("args").value);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Argumentos inválidos.", "error");
      return;
    }
    if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
      const dirtyFiles = tabs.filter((candidate) =>
        candidate.workspaceId === tab.workspaceId && candidate.language === "rust" && candidate !== tab && isDirty(candidate),
      );
      if (dirtyFiles.length > 0) {
        setStatus("Salve os outros arquivos alterados deste projeto antes de executar Cargo.", "error");
        return;
      }
      if (isDirty(tab) && !(await save())) {
        setStatus("Salve o arquivo Rust antes de executar o projeto Cargo.", "error");
        return;
      }
    }
    if (activeTab() !== tab) return;
    const documentState = view.state.doc;
    const source = documentState.toString();
    porsRunPending = true;
    runTabId = tab.id;
    syncLanguageControls(tab);
    let cargoTarget: api.CargoRunTarget | null = null;
    try {
      if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
        setStatus("Identificando os binários do projeto Cargo…");
        const discoveryId = await api.registerRustSourceRun();
        runId = discoveryId;
        setRunning(true);
        let targetInfo: api.CargoRunTargets;
        try {
          targetInfo = await api.cargoProjectRunTargets(discoveryId, tab.workspaceId, tab.workspacePath, rustArgs);
        } finally {
          if (runId === discoveryId) runId = null;
          setRunning(false);
        }
        if (activeTab() !== tab || view.state.doc !== documentState) return;
        if (targetInfo.targets.length === 0) {
          setStatus("Este projeto Cargo não contém um binário para executar.", "error");
          return;
        }
        cargoTarget = targetInfo.targets.find((target) =>
          target.packageName === tab.cargoTarget?.packageName && target.targetName === tab.cargoTarget?.targetName,
        ) ?? targetInfo.defaultTarget ?? (targetInfo.targets.length === 1 ? targetInfo.targets[0] : null);
        if (!cargoTarget) {
          cargoTarget = await chooseCargoTarget(targetInfo.targets);
          if (!cargoTarget) {
            if (activeTab() === tab) setStatus("Execução Cargo cancelada.");
            return;
          }
        }
        tab.cargoTarget = cargoTarget;
      }
      const cargoConsent = tab.cargoProject === true;
      const hasConsent = cargoConsent
        ? hasCargoExecutionConsent(sessionStorage)
        : hasRustExecutionConsent(sessionStorage);
      if (!hasConsent) {
        const consentText = tab.cargoProject === true
          ? "Cargo pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta execução não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?"
          : "Arquivos Rust compilados e executados podem acessar arquivos, processos e recursos permitidos à sua conta. Esta execução não possui sandbox. Execute apenas código em que você confia. Deseja continuar?";
        const accepted = await ask(
          consentText,
          { title: tab.cargoProject === true ? "Executar projeto Cargo sem sandbox" : "Executar código Rust sem sandbox", kind: "warning", okLabel: "Executar Rust", cancelLabel: "Cancelar" },
        );
        if (!accepted) {
          if (activeTab() === tab) setStatus("Execução Rust cancelada.");
          return;
        }
        if (cargoConsent) rememberCargoExecutionConsent(sessionStorage, true);
        else rememberRustExecutionConsent(sessionStorage, true);
      }
      if (activeTab() !== tab || view.state.doc !== documentState) return;
      const id = await api.registerRustSourceRun();
      runId = id;
      view.dispatch({ effects: setRuntimeError.of(null) });
      view.dispatch({ effects: setDebugLine.of(null) });
      paused = null;
      showPanel("console");
      output.replaceChildren();
      setRunning(true);
      if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
        setStatus("Compilando o projeto Cargo…");
        const result = await api.runCargoProject(id, tab.workspaceId, tab.workspacePath, cargoTarget, rustArgs);
        if (activeTab() !== tab || view.state.doc !== documentState) return;
        if (result.stdout) appendConsole(result.stdout);
        if (result.stderr) appendConsole(result.stderr, "error");
        if (result.status === "executed") {
          setStatus(result.exitCode === 0 ? "Projeto Cargo finalizado." : `Programa Rust terminou com código ${result.exitCode ?? "desconhecido"}.`, result.exitCode === 0 ? "ok" : "error");
        } else if (result.status === "cancelled") {
          setStatus(`Execução Cargo cancelada durante a ${result.phase === "compilation" ? "compilação" : "execução"}.`);
        } else if (result.status === "build_limit" || result.status === "runtime_limit") {
          const reason = result.reason === "time" ? (result.phase === "compilation" ? "20 segundos de compilação" : "3 segundos de execução") : "limite de saída de 1 MiB";
          setStatus(`Projeto Cargo interrompido: ${reason}.`, "error");
        } else {
          setStatus(result.phase === "compilation" ? "Cargo não conseguiu compilar o projeto." : "O programa Cargo terminou com erro.", "error");
        }
        return;
      }
      setStatus("Compilando e executando Rust 2021 com rustc…");
      const result = await api.runRustSource(id, source, rustArgs);
      if (activeTab() !== tab || view.state.doc !== documentState) return;
      if (result.status === "cancelled") {
        if (result.stdout) appendConsole(result.stdout);
        if (result.stderr) appendConsole(result.stderr, "error");
        const phase = result.phase === "compilation" ? "compilação" : "execução";
        setStatus(`Execução Rust cancelada durante a ${phase}.`);
      } else if (result.status === "executed") {
        if (result.stdout) appendConsole(result.stdout);
        if (result.stderr) appendConsole(result.stderr, "error");
        setStatus(
          result.exitCode === 0
            ? "Programa Rust finalizado com rustc."
            : `Programa Rust terminou com código ${result.exitCode ?? "desconhecido"}.`,
          result.exitCode === 0 ? "ok" : "error",
        );
      } else if (result.status === "runtime_limit") {
        if (result.stdout) appendConsole(result.stdout);
        if (result.stderr) appendConsole(result.stderr, "error");
        const reason = result.reason === "time" ? "tempo máximo de 3 segundos" : "limite de saída de 1 MiB";
        setStatus(`Execução Rust interrompida: ${reason}.`, "error");
      } else if (result.status === "compile_limit") {
        if (result.stderr) appendConsole(result.stderr, "error");
        const reason = result.reason === "time" ? "tempo máximo de compilação de 20 segundos" : "limite de saída de compilação de 1 MiB";
        setStatus(`Compilação Rust interrompida: ${reason}.`, "error");
      } else {
        const diagnostics: api.DiagnosticInfo[] = result.diagnostics.map((item) => ({
          severity: item.severity === "warning" ? "warning" : "error",
          code: item.code ?? "rustc",
          message: item.message,
          help: item.rendered,
          start: item.start,
          end: item.end,
        }));
        tab.runtimeCompileDiagnostics = diagnostics;
        if (result.stderr) appendConsole(result.stderr, "error");
        if (diagnostics.length) {
          renderMessages();
          view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
          showPanel("messages");
          setStatus("rustc encontrou erros no arquivo Rust.", "error");
        } else {
          setStatus("rustc encontrou erros; veja o detalhe no console.", "error");
        }
      }
    } catch (error) {
      if (activeTab() === tab && view.state.doc === documentState) {
        const message = String(error);
        if (message.includes("Descoberta de alvos Cargo cancelada")) {
          setStatus("Execução Cargo cancelada.");
        } else {
          appendConsole(message + "\n", "error");
          setStatus("Não foi possível compilar ou executar o arquivo Rust.", "error");
        }
      }
    } finally {
      runId = null;
      porsRunPending = false;
      runTabId = null;
      setRunning(runId !== null);
      const current = activeTab();
      if (current) syncLanguageControls(current);
    }
    return;
  }
  if (tab.language === "portugol-rs") {
    const documentState = view.state.doc;
    const source = documentState.toString();
    view.dispatch({ effects: setRuntimeError.of(null) });
    view.dispatch({ effects: setDebugLine.of(null) });
    paused = null;
    showPanel("console");
    output.replaceChildren();
    inputForm.hidden = true;
    if (pauseOnStart) {
      const events: PendingRunEvent[] = [];
      pendingEvents = events;
      runTabId = tab.id;
      setRunning(true);
      try {
        await runListeners;
        const id = await api.debugPortugolRs(source, breakpointLines(view.state));
        runId = id;
        setRunning(true);
        setStatus("Depurando Portugol-RS pela VM.");
        replayRunEvents(events, id);
      } catch (errors) {
        runTabId = null;
        setRunning(false);
        if (Array.isArray(errors)) {
          tab.diagnostics = errors;
          renderMessages();
          showPanel("messages");
          setStatus("O Portugol-RS contém erros e não pode ser depurado.", "error");
        } else {
          setStatus(String(errors), "error");
        }
      } finally {
        pendingEvents = null;
      }
      return;
    }
    if (rustcAvailable !== true) {
      setStatus("Instale a toolchain Rust para executar arquivos .pors.", "error");
      return;
    }
    porsRunPending = true;
    runTabId = tab.id;
    setRunning(true);
    setStatus("Compilando e executando com rustc…");
    let porsRunId: number | null = null;
    try {
      porsRunId = await api.registerPortugolRsRun();
      runId = porsRunId;
      setRunning(true);
      await runListeners;
      const result = await api.runPortugolRs(porsRunId, source);
      if (activeTab() !== tab || view.state.doc !== documentState) return;
      if (result.status === "cancelled") {
        if (result.stdout) appendConsole(result.stdout);
        if (result.stderr) appendConsole(result.stderr, "error");
        const phase = result.phase === "compilation" ? "compilação" : "execução";
        setStatus(`Execução Portugol-RS cancelada durante a ${phase}.`);
      } else if (result.status === "executed") {
        if (result.stdout) appendConsole(result.stdout);
        if (result.stderr) appendConsole(result.stderr, "error");
        setStatus(
          result.exitCode === 0
            ? "Programa Portugol-RS finalizado com rustc."
            : `Programa Portugol-RS terminou com código ${result.exitCode ?? "desconhecido"}.`,
          result.exitCode === 0 ? "ok" : "error",
        );
      } else if (result.status === "compile_limit") {
        if (result.stderr) appendConsole(result.stderr, "error");
        const reason = result.reason === "time" ? "tempo máximo de compilação de 20 segundos" : "limite de saída de compilação de 1 MiB";
        setStatus(`Compilação Portugol-RS interrompida: ${reason}.`, "error");
      } else if (result.status === "runtime_limit") {
        if (result.stdout) appendConsole(result.stdout);
        if (result.stderr) appendConsole(result.stderr, "error");
        const descriptions = {
          time: "tempo máximo de 3 segundos",
          output: "limite de saída de 1 MiB",
          string_memory: "limite de memória para cadeias de 16 MiB",
        } as const;
        setStatus(`Execução Portugol-RS interrompida: ${descriptions[result.reason]}.`, "error");
      } else {
        const diagnostics = result.diagnostics.map(sourceDiagnostic).filter((item): item is api.DiagnosticInfo => item !== null);
        tab.runtimeCompileDiagnostics = diagnostics;
        if (result.stderr) appendConsole(result.stderr, "error");
        if (diagnostics.length) {
          renderMessages();
          view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
          showPanel("messages");
          setStatus("rustc encontrou erros no Rust gerado. Selecione um diagnóstico para voltar à origem.", "error");
        } else {
          setStatus("rustc encontrou um erro no código gerado; veja o detalhe no console.", "error");
        }
      }
    } catch (error) {
      if (activeTab() === tab && view.state.doc === documentState) {
        appendConsole(String(error) + "\n", "error");
        setStatus("Não foi possível compilar ou executar o Portugol-RS.", "error");
      }
    } finally {
      inputForm.hidden = true;
      if (runId === porsRunId) runId = null;
      porsRunPending = false;
      runTabId = null;
      setRunning(runId !== null);
    }
    return;
  }
  view.dispatch({ effects: setRuntimeError.of(null) });
  view.dispatch({ effects: setDebugLine.of(null) });
  paused = null;
  showPanel("console");
  output.replaceChildren();
  const source = view.state.doc.toString();
  const args = parseArgs($<HTMLInputElement>("args").value);
  const lines = breakpointLines(view.state);
  const events: PendingRunEvent[] = [];
  pendingEvents = events;
  runTabId = tab.id;
  $<HTMLButtonElement>("btn-run").disabled = true;
  $<HTMLButtonElement>("btn-debug").disabled = true;
  try {
    await runListeners;
    const id = await api.run(source, args, tab.path ?? tab.examplePath ?? null, lines, pauseOnStart);
    runId = id;
    setRunning(true);
    setStatus("Executando…");
    replayRunEvents(events, id);
  } catch (errors) {
    runTabId = null;
    setRunning(false);
    if (Array.isArray(errors)) {
      tab.diagnostics = errors;
      renderMessages();
      showPanel("messages");
      setStatus("O programa contém erros e não pode ser executado.", "error");
    } else {
      setStatus(String(errors), "error");
    }
  } finally {
    pendingEvents = null;
  }
}

function runOrContinue() {
  return paused ? debugCommand(api.debugContinue) : runProgram();
}

function stepOrStart() {
  return paused ? debugCommand(api.debugStepOver) : runProgram(true);
}

async function stopProgram() {
  if (runId === null) return;
  const id = runId;
  try {
    const accepted = await api.stop(id);
    if (accepted && runId === id && activeId === runTabId) {
      setStatus("Interrupção solicitada…");
    }
  } catch (error) {
    if (runId === id && activeId === runTabId) setStatus(`Não foi possível interromper: ${String(error)}`, "error");
  }
}

async function checkCargoProject() {
  if (runId !== null || pendingEvents !== null || porsRunPending) return;
  const tab = activeTab();
  if (!tab || tab.language !== "rust") return;
  if (tab.workspaceId && tab.workspacePath && tab.cargoProject === undefined) {
    setStatus("Verificando se este arquivo pertence a um projeto Cargo.");
    return;
  }
  if (tab.cargoProject !== true || !tab.workspaceId || !tab.workspacePath) {
    setStatus("Abra um arquivo Rust pelo explorador dentro de um projeto Cargo.", "error");
    return;
  }
  if (cargoAvailable !== true) {
    setStatus("Instale Cargo para analisar o projeto Rust.", "error");
    return;
  }
  const dirtyFiles = tabs.filter((candidate) =>
    candidate.workspaceId === tab.workspaceId && candidate.language === "rust" && candidate !== tab && isDirty(candidate),
  );
  if (dirtyFiles.length > 0) {
    setStatus("Salve os outros arquivos Rust alterados deste workspace antes da análise Cargo.", "error");
    return;
  }
  if (isDirty(tab) && !(await save())) {
    setStatus("Salve o arquivo Rust antes de analisar o projeto Cargo.", "error");
    return;
  }
  if (activeTab() !== tab) return;
  const documentState = view.state.doc;
  let registeredId: number | null = null;
  porsRunPending = true;
  runTabId = tab.id;
  syncLanguageControls(tab);
  try {
    if (!hasCargoExecutionConsent(sessionStorage)) {
      const accepted = await ask(
        "cargo check pode acessar a rede e executar scripts de build, macros procedurais e dependências no seu usuário local. Esta análise não possui sandbox. Execute apenas projetos em que você confia. Deseja continuar?",
        { title: "Analisar projeto Cargo sem sandbox", kind: "warning", okLabel: "Analisar Cargo", cancelLabel: "Cancelar" },
      );
      if (!accepted) {
        if (activeTab() === tab) setStatus("Análise Cargo cancelada.");
        return;
      }
      rememberCargoExecutionConsent(sessionStorage, true);
    }
    if (activeTab() !== tab || view.state.doc !== documentState) return;
    registeredId = await api.registerRustSourceRun();
    runId = registeredId;
    setRunning(true);
    view.dispatch({ effects: setRuntimeError.of(null) });
    output.replaceChildren();
    showPanel("messages");
    setStatus("Analisando projeto com cargo check…");
    const result = await api.checkCargoProject(registeredId, tab.workspaceId, tab.workspacePath);
    if (activeTab() !== tab || view.state.doc !== documentState) return;
    const diagnostics: api.DiagnosticInfo[] = result.diagnostics.map((item) => ({
      severity: item.severity === "warning" ? "warning" : "error",
      code: item.code ?? "cargo",
      message: item.message,
      help: item.rendered,
      start: item.start,
      end: item.end,
    }));
    tab.diagnostics = diagnostics;
    renderMessages();
    view.dispatch(setDiagnostics(view.state, diagnostics.map((item) => toCodeMirror(item, view.state.doc.length))));
    if (result.stderr) appendConsole(result.stderr, result.status === "check_error" ? "error" : "");
    if (result.status === "cancelled") {
      setStatus("Análise Cargo cancelada.");
    } else if (result.status === "check_limit") {
      const reason = result.reason === "time" ? "20 segundos" : "limite de saída de 1 MiB";
      setStatus(`Análise Cargo interrompida: ${reason}.`, "error");
    } else if (result.status === "check_error") {
      showPanel(diagnostics.length ? "messages" : "console");
      setStatus(diagnostics.length
        ? "Cargo encontrou diagnósticos no arquivo aberto; confira o Console para detalhes do projeto."
        : "Cargo check terminou com erros em outros arquivos ou no manifesto.", "error");
    } else {
      showPanel(diagnostics.length ? "messages" : result.stderr ? "console" : "messages");
      const warnings = diagnostics.filter((item) => item.severity === "warning").length;
      setStatus(diagnostics.length ? `Análise concluída: ${warnings} aviso(s) no arquivo aberto.` : "Análise Cargo concluída sem diagnósticos no arquivo aberto.", "ok");
    }
  } catch (error) {
    if (activeTab() === tab && view.state.doc === documentState) {
      appendConsole(`${String(error)}\n`, "error");
      showPanel("console");
      setStatus("Não foi possível analisar o projeto Cargo.", "error");
    }
  } finally {
    if (registeredId !== null && runId === registeredId) runId = null;
    porsRunPending = false;
    runTabId = null;
    setRunning(false);
    const current = activeTab();
    if (current) syncLanguageControls(current);
  }
}

const ownershipStateLabels: Record<OwnershipBindingState, string> = {
  owner: "Dono",
  "shared-reference": "Empréstimo compartilhado (&T)",
  "mutable-reference": "Empréstimo mutável exclusivo (&mut T)",
  moved: "Movido; valor indisponível",
  expired: "Empréstimo encerrado",
  dropped: "Liberado ao sair do escopo",
};

function renderOwnershipLab() {
  const scenario = getOwnershipScenario(ownershipScenarioId);
  const snapshot = getOwnershipSnapshot(ownershipScenarioId, ownershipStepIndex);
  if (!scenario || !snapshot) return;

  const picker = $<HTMLSelectElement>("ownership-scenario");
  picker.replaceChildren(...ownershipScenarioIds().map((id) => {
    const option = document.createElement("option");
    const item = getOwnershipScenario(id);
    option.value = id;
    option.textContent = item?.title ?? id;
    return option;
  }));
  picker.value = scenario.id;

  const code = $<HTMLElement>("ownership-source");
  code.replaceChildren(...scenario.source.split("\n").map((text, index) => {
    const line = document.createElement("span");
    line.className = "ownership-code-line";
    line.dataset.line = String(index + 1);
    if (index + 1 === snapshot.currentLine) {
      line.dataset.current = "true";
      line.setAttribute("aria-current", "location");
    }
    line.textContent = text || " ";
    return line;
  }));

  const bindings = $<HTMLUListElement>("ownership-bindings");
  bindings.replaceChildren(...snapshot.bindings.map((binding) => {
    const item = document.createElement("li");
    item.className = "ownership-binding";
    item.dataset.state = binding.state;

    const heading = document.createElement("div");
    heading.className = "ownership-binding-heading";
    const name = document.createElement("code");
    name.textContent = binding.name;
    const type = document.createElement("span");
    type.textContent = binding.type;
    const state = document.createElement("span");
    state.className = "ownership-state";
    state.textContent = ownershipStateLabels[binding.state];
    heading.append(name, type, state);

    const description = document.createElement("p");
    description.textContent = binding.description;
    item.append(heading, description);
    if (binding.target) {
      const relation = document.createElement("p");
      relation.className = "ownership-relation";
      relation.textContent = binding.state === "owner"
        ? `Controla: ${binding.target}`
        : `Empresta de: ${binding.target}`;
      item.append(relation);
    }
    if (binding.value !== undefined) {
      const value = document.createElement("p");
      value.className = "ownership-value";
      value.textContent = `Valor: ${binding.value}`;
      item.append(value);
    }
    return item;
  }));

  const heap = $<HTMLUListElement>("ownership-heap");
  heap.replaceChildren(...snapshot.heap.map((block) => {
    const item = document.createElement("li");
    item.className = "ownership-heap-block";
    item.dataset.state = block.state;
    const title = document.createElement("strong");
    title.textContent = block.id;
    const value = document.createElement("p");
    value.textContent = block.state === "allocated" ? `Conteúdo UTF-8: ${block.value}` : `Conteúdo liberado: ${block.value}`;
    const allocation = document.createElement("p");
    allocation.textContent = block.state === "allocated"
      ? `Alocação ativa; owner: ${block.owner ?? "não identificado"}.`
      : "Alocação liberada quando o owner saiu do escopo.";
    item.append(title, value, allocation);
    return item;
  }));

  $("ownership-step-title").textContent = snapshot.title;
  $("ownership-explanation").textContent = snapshot.explanation;
  $("ownership-step-status").textContent = `Passo ${ownershipStepIndex + 1} de ${scenario.snapshots.length}: ${snapshot.title}`;
  $("ownership-provenance").textContent = snapshot.provenance === "verified-simulation"
    ? "Simulação didática — não é execução nem trace de memória Rust."
    : "Proveniência indisponível.";
  const diagnostic = $<HTMLElement>("ownership-diagnostic");
  diagnostic.hidden = !snapshot.diagnostic;
  diagnostic.textContent = snapshot.diagnostic ?? "";

  $<HTMLButtonElement>("ownership-previous").disabled = ownershipStepIndex === 0;
  $<HTMLButtonElement>("ownership-next").disabled = ownershipStepIndex >= scenario.snapshots.length - 1;
  $<HTMLButtonElement>("ownership-restart").disabled = ownershipStepIndex === 0;
}

function renderLearningPanel() {
  const concepts = conceptsForMode(learningMode);
  const tab = activeTab();
  const pinnedConceptId = learningPinForMode(tab?.learningPins, learningMode, concepts.map(({ id }) => id));
  const source = view.state.doc.toString();
  const context = !pinnedConceptId && learningMode === "bridge" && !learningConceptManuallySelected && tab?.language !== "rust"
    ? learningContextAtSelection({
      source,
      selectionFrom: view.state.selection.main.from,
      selectionTo: view.state.selection.main.to,
      artifact: tab?.rustPreview?.artifact,
      stale: Boolean(tab?.rustPreview?.stale || tab?.rustPreview?.pending),
    })
    : null;
  const concept = resolveConceptForMode(learningMode, pinnedConceptId ?? context?.conceptId ?? selectedLearningConceptId);

  const picker = $<HTMLSelectElement>("learning-concept");
  picker.replaceChildren(...concepts.map((item) => {
    const option = document.createElement("option");
    option.value = item.id;
    option.textContent = item.title;
    return option;
  }));
  picker.value = concept.id;
  const pinButton = $<HTMLButtonElement>("learning-pin");
  const topicPinned = pinnedConceptId === concept.id;
  pinButton.disabled = !tab;
  pinButton.dataset.conceptId = concept.id;
  pinButton.setAttribute("aria-pressed", String(topicPinned));
  pinButton.textContent = topicPinned ? "Desafixar tópico" : "Fixar tópico";
  const practiceButton = $<HTMLButtonElement>("learning-practice-open");
  practiceButton.hidden = concept.practiceExerciseId === undefined;
  practiceButton.disabled = !tab || learningPracticeBusy;
  practiceButton.dataset.exerciseId = concept.practiceExerciseId === undefined ? "" : String(concept.practiceExerciseId);
  const active = activeTab();
  $("learning-mode-description").textContent = learningMode === "bridge"
    ? "Ponte didática: compare os conceitos de Portugol com exemplos Rust editoriais."
    : learningMode === "rust"
      ? active?.language === "rust"
        ? "Nível Rust: o arquivo .rs ativo é editado e executado com a toolchain Rust instalada."
        : "Nível Rust: abra ou crie um arquivo .rs para programar em Rust. O arquivo atual permanece intacto."
      : "Fundamentos de lógica e programação em Portugol.";
  $("learning-title").textContent = concept.title;
  $("learning-provenance").textContent = topicPinned
    ? "Tópico fixado — escolha manual nesta aba"
    : context
      ? "Relacionado à seleção por mapeamento exato da tradução atual"
      : "Conceito selecionado manualmente";
  $("learning-definition").textContent = context
    ? `${concept.definition} ${explainMappedConcept(context, tab?.language === "portugol-rs" ? "portugol-rs" : "portugol")}`
    : concept.definition;
  $("learning-what-changed").textContent = concept.whatChanged;
  $("learning-why-it-matters").textContent = concept.whyItMatters;
  $("learning-practice").textContent = concept.practice;
  const rustMode = learningMode === "rust";
  $("learning-portugol-example").hidden = rustMode;
  if (!rustMode) {
    $("learning-portugol-caption").textContent = context ? "Trecho selecionado na fonte" : concept.portugolExample.caption;
    $("learning-portugol-code").textContent = context ? context.sourceSnippet : concept.portugolExample.code;
  }

  const rustSection = $<HTMLElement>("learning-rust-example");
  const showRustExample = rustMode || (learningMode === "bridge" && (context !== null || concept.rustExample !== undefined));
  rustSection.hidden = !showRustExample;
  if (showRustExample) {
    $("learning-rust-heading").textContent = rustMode ? "Rust — exemplo" : "Rust — comparação didática";
    $("learning-rust-caption").textContent = context ? "Rust gerado para este trecho" : concept.rustExample!.caption;
    $("learning-rust-code").textContent = context ? context.rustSnippet : concept.rustExample!.code;
  }

  const ownershipLab = $<HTMLElement>("ownership-lab");
  const showOwnershipLab = learningMode !== "beginner" && concept.id === "ownership";
  ownershipLab.hidden = !showOwnershipLab;
  if (showOwnershipLab) renderOwnershipLab();
}

const bottomPanelNames: PanelName[] = ["console", "tests", "messages", "debug"];

function setInspectorView(learning: boolean) {
  $("learning-panel").hidden = !learning;
  for (const id of ["utilities-insert", "outline", "variable-inspector"]) {
    $<HTMLElement>(id).hidden = learning;
  }
}

function showPanel(which: PanelName) {
  const learningActive = which === "learning";
  setInspectorView(learningActive);
  if (learningActive) {
    showUtilities(true);
    renderLearningPanel();
    return;
  }
  for (const name of bottomPanelNames) {
    $(`tab-${name}`).classList.toggle("active", which === name);
    $(`tab-${name}`).setAttribute("aria-selected", String(which === name));
    $(panelContentId(name)).hidden = which !== name;
  }
  $("console-tools").hidden = which !== "console";
  if (which === "tests") renderExerciseTests();
}

document.querySelector<HTMLElement>(".panel-tabs")?.addEventListener("keydown", (event) => {
  const target = event.target;
  if (!(target instanceof HTMLElement)) return;
  const current = bottomPanelNames.find((name) => target.id === `tab-${name}`);
  if (!current) return;
  const next = panelNameForKey(bottomPanelNames, current, event.key);
  if (!next) return;
  event.preventDefault();
  showPanel(next);
  $<HTMLButtonElement>(`tab-${next}`).focus();
});

function goTo(offset: number) {
  const position = Math.min(offset, view.state.doc.length);
  view.dispatch({ selection: { anchor: position }, scrollIntoView: true });
  view.focus();
}

function renderMessages() {
  const tab = activeTab();
  renderDiagnostics([...(tab?.diagnostics ?? []), ...(tab?.runtimeCompileDiagnostics ?? [])], view.state.doc.lines, goTo);
}

function renderExerciseTests() {
  const tab = activeTab();
  const state = exerciseTestPanelState(tab?.exercise?.exercise, view.state.doc, tab?.exerciseTestRun);
  $("tests-title").textContent = tab?.exercise ? `Testes — ${tabName(tab)}` : "Testes do projeto";
  const status = $("tests-status");
  const nextStatus = exerciseTestStatus(state);
  if (status.textContent !== nextStatus) status.textContent = nextStatus;
  const run = $<HTMLButtonElement>("tests-run");
  run.hidden = state.kind === "empty" || state.kind === "no-cases";
  run.disabled = state.kind === "running";
  run.textContent = state.kind === "running" ? "Executando…" : "Executar casos";
  const failures = $<HTMLUListElement>("tests-results");
  failures.replaceChildren();
  if (state.kind === "completed") {
    for (const failure of state.result.failures) {
      const item = document.createElement("li");
      item.textContent = failure;
      failures.append(item);
    }
  }
  failures.hidden = state.kind !== "completed" || state.result.failures.length === 0;
}

async function runExerciseTests() {
  const tab = activeTab();
  if (!tab?.exercise || tab.exercise.exercise.cases <= 0) return;
  const documentState = view.state.doc;
  const request: ExerciseTestRequest = {
    tabId: tab.id,
    document: documentState,
    generation: (tab.exerciseTestGeneration ?? 0) + 1,
  };
  tab.exerciseTestGeneration = request.generation;
  tab.exerciseTestRun = { source: documentState, status: "running" };
  renderExerciseTests();
  const isCurrent = () => {
    const currentDocument = tab.id === activeId ? view.state.doc : tab.state.doc;
    return isCurrentExerciseTestRequest(request, {
      tabId: tab.id,
      document: currentDocument,
      generation: tab.exerciseTestGeneration ?? 0,
    }, tabs.includes(tab));
  };
  try {
    const grade = await exercises.evaluate(tab.exercise, documentState.toString(), isCurrent);
    if (!isCurrent()) return;
    if (!grade.current) {
      tab.exerciseTestRun = {
        source: documentState,
        status: "error",
        message: "Esta avaliação foi substituída por outra. Execute os casos novamente.",
      };
      return;
    }
    tab.exerciseTestRun = {
      source: documentState,
      status: "completed",
      result: {
        total: grade.evaluation.total,
        passed: grade.evaluation.passed,
        failures: grade.evaluation.failures,
        completed: grade.progress.completed,
      },
    };
  } catch (message) {
    if (isCurrent()) {
      tab.exerciseTestRun = {
        source: documentState,
        status: "error",
        message: typeof message === "string" ? message : "Não foi possível executar os casos do exercício.",
      };
    }
  } finally {
    renderExerciseTests();
  }
}

function routeRunEvent(id: number, handle: () => void) {
  routeRunEventById(id, runId, pendingEvents, handle);
}

const runListeners = api.listenRun({
  output(id, text) {
    routeRunEvent(id, () => appendConsole(text));
  },
  clear(id) {
    routeRunEvent(id, () => output.replaceChildren());
  },
  input(id, expected) {
    routeRunEvent(id, () => {
      $("console-hint").textContent = `leia (${expected}):`;
      inputForm.hidden = false;
      inputField.value = "";
      if (activeTab()) inputField.focus();
      consoleBox.scrollTop = consoleBox.scrollHeight;
    });
  },
  paused(id, info) {
    routeRunEvent(id, () => setPaused(info));
  },
  finished(id, outcome) {
    routeRunEvent(id, () => {
      const isRunTab = activeId === runTabId;
      runId = null;
      runTabId = null;
      paused = null;
      if (isRunTab) view.dispatch({ effects: setDebugLine.of(null) });
      setRunning(false);
      if (outcome.status === "finished") {
        appendConsole("\nPrograma finalizado.\n", "info");
        setStatus("Programa finalizado.", "ok");
      } else if (outcome.status === "stopped") {
        appendConsole("\nPrograma interrompido.\n", "info");
        setStatus("Programa interrompido.");
      } else if (outcome.status === "runtime_limit") {
        const descriptions = {
          time: "tempo ativo de 3 segundos",
          output: "limite de saída de 1 MiB",
          string_memory: "limite cumulativo de cadeias de 16 MiB",
        } as const;
        const message = `VM Portugol-RS interrompida: ${descriptions[outcome.reason]}.`;
        appendConsole(`\n${message}\n`, "error");
        setStatus(message, "error");
      } else {
        const { start, message, trace } = outcome;
        let text = `\nErro na linha ${start.line}, coluna ${start.column}: ${message}\n`;
        for (const frame of trace.slice(1, -1)) {
          text += `  chamado em ${frame.function} (linha ${frame.line})\n`;
        }
        appendConsole(text, "error");
        setStatus(`Erro de execução na linha ${start.line}.`, "error");
        if (isRunTab) {
          view.dispatch({ effects: setRuntimeError.of({ from: start.offset, to: outcome.end.offset }) });
          goTo(start.offset);
        }
      }
    });
  },
});

inputForm.addEventListener("submit", (e) => {
  e.preventDefault();
  if (runId === null) return;
  const line = inputField.value;
  appendConsole(line + "\n", "echo");
  inputForm.hidden = true;
  void api.sendInput(runId, line);
});

// ---------------------------------------------------------------------------
// Divisória redimensionável e modo editor expandido
// ---------------------------------------------------------------------------

const panel = document.querySelector<HTMLElement>(".panel")!;
const bottomSplitter = $("splitter");
const explorer = $("workspace-explorer");
const explorerSplitter = $("workspace-explorer-splitter");

function explorerPaneBounds() {
  const available = Math.max(1, $("code").clientWidth || window.innerWidth);
  const sideMinimums = calculateSidePaneMinimums(
    available,
    paneChromeWidth(),
    explorerSplitter.offsetWidth || 9,
    utilitiesSplitter.offsetWidth || 15,
    !explorer.hidden,
    !utilities.hidden,
  );
  const minimum = sideMinimums.explorer;
  const utilitiesWidth = utilities.hidden ? 0 : utilities.getBoundingClientRect().width;
  const maximum = Math.max(minimum, Math.min(420, available * 0.42, available - paneChromeWidth()
    - (explorer.hidden ? 0 : explorerSplitter.offsetWidth)
    - (utilities.hidden ? 0 : utilitiesSplitter.offsetWidth) - utilitiesWidth));
  return { minimum, maximum, available };
}

function bottomPaneBounds() {
  const available = Math.max(280, panel.parentElement?.clientHeight || window.innerHeight - 100);
  const minimum = 90;
  const maximum = Math.max(minimum, Math.min(available - 90, available * 0.65));
  return { minimum, maximum, available };
}

function utilitiesPaneBounds() {
  const available = Math.max(1, $("code").clientWidth || window.innerWidth);
  const sideMinimums = calculateSidePaneMinimums(
    available,
    paneChromeWidth(),
    explorerSplitter.offsetWidth || 9,
    utilitiesSplitter.offsetWidth || 15,
    !explorer.hidden,
    !utilities.hidden,
  );
  const minimum = sideMinimums.utilities;
  const explorerWidth = explorer.hidden ? 0 : explorer.getBoundingClientRect().width;
  const maximum = Math.max(minimum, Math.min(available * 0.55, 720, available - paneChromeWidth()
    - (explorer.hidden ? 0 : explorerSplitter.offsetWidth)
    - (utilities.hidden ? 0 : utilitiesSplitter.offsetWidth) - explorerWidth));
  return { minimum, maximum, available };
}

function paneChromeWidth() {
  const toolbar = $("code").querySelector<HTMLElement>(".code-toolbar")?.getBoundingClientRect().width ?? 56;
  const editorMinimum = $("rust-preview").hidden ? 100 : 360;
  return toolbar + editorMinimum;
}

function setSeparatorValue(separator: HTMLElement, value: number, bounds: { minimum: number; maximum: number }) {
  separator.setAttribute("aria-valuemin", String(Math.round(bounds.minimum)));
  separator.setAttribute("aria-valuemax", String(Math.round(bounds.maximum)));
  separator.setAttribute("aria-valuenow", String(Math.round(value)));
}

function applyExplorerPaneSize(width = paneLayout.explorer * explorerPaneBounds().available) {
  const bounds = explorerPaneBounds();
  const size = clampPaneSize(width, bounds.minimum, bounds.maximum);
  explorer.style.minWidth = `${bounds.minimum}px`;
  explorer.style.flexBasis = `${size}px`;
  setSeparatorValue(explorerSplitter, size, bounds);
  return { size, ...bounds };
}

function applyBottomPaneSize(height = paneLayout.bottom * bottomPaneBounds().available) {
  const bounds = bottomPaneBounds();
  const size = clampPaneSize(height, bounds.minimum, bounds.maximum);
  panel.style.flex = `0 0 ${size}px`;
  setSeparatorValue(bottomSplitter, size, bounds);
  return { size, ...bounds };
}

function applyUtilitiesPaneSize(width = paneLayout.utilities * utilitiesPaneBounds().available) {
  const bounds = utilitiesPaneBounds();
  const size = clampPaneSize(width, bounds.minimum, bounds.maximum);
  utilities.style.minWidth = `${bounds.minimum}px`;
  utilities.style.flex = `0 0 ${size}px`;
  setSeparatorValue(utilitiesSplitter, size, bounds);
  return { size, ...bounds };
}

function rememberPaneSize(key: keyof PaneLayout, size: number, available: number) {
  paneLayout = normalizePaneLayout({ ...paneLayout, [key]: size / available });
  persistPaneLayout();
}

function installPointerResize(
  separator: HTMLElement,
  orientation: "horizontal" | "vertical",
  readSize: () => number,
  applySize: (size: number) => { size: number; minimum: number; maximum: number; available: number },
  onCommit: (size: number, available: number) => void,
  deltaSign: 1 | -1,
) {
  separator.addEventListener("pointerdown", (start) => {
    if (start.button !== 0) return;
    separator.setPointerCapture(start.pointerId);
    const origin = orientation === "horizontal" ? start.clientY : start.clientX;
    const initial = readSize();
    const move = (event: PointerEvent) => {
      const coordinate = orientation === "horizontal" ? event.clientY : event.clientX;
      applySize(initial + (coordinate - origin) * deltaSign);
    };
    const finish = () => {
      const bounds = applySize(readSize());
      onCommit(bounds.size, bounds.available);
      separator.removeEventListener("pointermove", move);
      separator.removeEventListener("pointerup", finish);
      separator.removeEventListener("pointercancel", finish);
      separator.removeEventListener("lostpointercapture", finish);
    };
    separator.addEventListener("pointermove", move);
    separator.addEventListener("pointerup", finish, { once: true });
    separator.addEventListener("pointercancel", finish, { once: true });
    separator.addEventListener("lostpointercapture", finish, { once: true });
  });
}

function installKeyboardResize(
  separator: HTMLElement,
  orientation: "horizontal" | "vertical",
  readSize: () => number,
  applySize: (size: number) => { size: number; minimum: number; maximum: number; available: number },
  onCommit: (size: number, available: number) => void,
  direction: 1 | -1,
) {
  separator.addEventListener("keydown", (event) => {
    const bounds = applySize(readSize());
    const next = resizePaneForKey(event.key, orientation, direction, bounds.size, bounds);
    if (next === null) return;
    event.preventDefault();
    const updated = applySize(next);
    onCommit(updated.size, updated.available);
  });
}

function installPaneResize(
  separator: HTMLElement,
  orientation: "horizontal" | "vertical",
  readSize: () => number,
  applySize: (size: number) => { size: number; minimum: number; maximum: number; available: number },
  key: keyof PaneLayout,
  direction: 1 | -1,
) {
  const commit = (size: number, available: number) => rememberPaneSize(key, size, available);
  installPointerResize(separator, orientation, readSize, applySize, commit, direction);
  installKeyboardResize(separator, orientation, readSize, applySize, commit, direction);
}

installPaneResize(explorerSplitter, "vertical", () => explorer.getBoundingClientRect().width, applyExplorerPaneSize, "explorer", 1);
installPaneResize(bottomSplitter, "horizontal", () => panel.getBoundingClientRect().height, applyBottomPaneSize, "bottom", -1);

const utilities = $("utilities");
const utilitiesSplitter = $("utilities-splitter");
$("learning-inspector-slot").append($("learning-panel"));
const outline = mountOutline(goTo, renameAt);

const libraryButtons = new Map<string, HTMLButtonElement>();
for (const name of LIBRARIES) {
  const button = document.createElement("button");
  button.textContent = name;
  button.title = `Incluir biblioteca ${name}`;
  button.onclick = async () => {
    const tab = activeTab();
    if (!tab || (tab.id === runTabId && runId !== null)) return;
    if (tab.language === "portugol-rs") {
      setStatus("Bibliotecas clássicas ainda não estão disponíveis no Portugol-RS.");
      return;
    }
    const doc = view.state.doc;
    try {
      const change = await api.includeLibrary(doc.toString(), name);
      if (activeTab() !== tab || view.state.doc !== doc) return;
      if (change) view.dispatch({ changes: change, annotations: isolateHistory.of("full") });
      button.disabled = true;
      view.focus();
    } catch (message) {
      if (activeTab() === tab) setStatus(typeof message === "string" ? message : "Não foi possível incluir a biblioteca.", "error");
    }
  };
  $("utilities-libraries").append(button);
  libraryButtons.set(name, button);
}

function updateLibraryButtons(symbols: api.SymbolInfo[]) {
  const included = new Set(symbols.filter((symbol) => symbol.kind === "library")
    .map(({ name }) => name.split(" → ", 1)[0]));
  const unsupported = activeTab()?.language !== "portugol";
  for (const [name, button] of libraryButtons) button.disabled = unsupported || included.has(name);
}

for (const snippet of SNIPPETS) {
  const button = document.createElement("button");
  button.textContent = snippet.label;
  button.title = snippet.detail ?? snippet.label;
  button.onclick = () => {
    const tab = activeTab();
    if (!tab || tab.language === "rust" || (tab.id === runTabId && runId !== null)) return;
    if (snippet.label === "programa") {
      newTab();
      return;
    }
    const { from, to } = view.state.selection.main;
    if (typeof snippet.apply === "function") snippet.apply(view, snippet, from, to);
    view.focus();
  };
  $("utilities-models").append(button);
}

for (const selected of ["libraries", "models"]) {
  $(`utilities-${selected}-tab`).onclick = () => {
    for (const name of ["libraries", "models"]) {
      $(`utilities-${name}-tab`).setAttribute("aria-selected", String(name === selected));
      $(`utilities-${name}`).hidden = name !== selected;
    }
  };
}

function showUtilities(visible: boolean) {
  if (visible && $("code").classList.contains("expanded")) toggleExpanded();
  utilities.hidden = utilitiesSplitter.hidden = !visible;
  $("btn-utilities").setAttribute("aria-expanded", String(visible));
  if (visible) {
    if (!explorer.hidden) applyExplorerPaneSize();
    applyUtilitiesPaneSize();
  } else if (!explorer.hidden) applyExplorerPaneSize();
}

function toggleUtilities() {
  if (activeTab()) showUtilities(utilities.hidden === true);
}

installPaneResize(utilitiesSplitter, "vertical", () => utilities.getBoundingClientRect().width, applyUtilitiesPaneSize, "utilities", -1);
applyBottomPaneSize();
window.addEventListener("resize", () => {
  applyBottomPaneSize();
  if (!explorer.hidden) applyExplorerPaneSize();
  if (!utilities.hidden) applyUtilitiesPaneSize();
});

function toggleExpanded() {
  if (!activeTab()) return;
  const hidden = !panel.hidden;
  panel.hidden = hidden;
  $("splitter").hidden = hidden;
  $("code").classList.toggle("expanded", hidden);
  $("btn-unexpand").hidden = !hidden;
}

// ---------------------------------------------------------------------------
// Botões e atalhos
// ---------------------------------------------------------------------------

$("home-new").onclick = () => newTab();
mountHomeNewFileMenu($<HTMLDetailsElement>("home-new-menu"), {
  "portugol-rs": () => newTab(PORS_TEMPLATE, null, undefined, undefined, undefined, "portugol-rs"),
  rust: () => newTab("fn main() {\n    \n}\n", null, undefined, undefined, undefined, "rust"),
});
$("home-open").onclick = () => void openFiles();
$("btn-open").onclick = () => void openFiles();
const editorOptions = $<HTMLDetailsElement>("editor-options");
const consoleOptions = $<HTMLDetailsElement>("console-tools");

const outlineOptions = $<HTMLDetailsElement>("outline-options");
const optionMenus = [editorOptions, consoleOptions, outlineOptions];
const centerButton = $("editor-center");
centerButton.setAttribute("aria-pressed", String(centerActive));

function centerSelection() {
  view.dispatch({ effects: EditorView.scrollIntoView(view.state.selection.main.head, { y: "center" }) });
}

function toggleCenterSource() {
  if (!activeTab()) return;
  centerActive = !centerActive;
  centerButton.setAttribute("aria-pressed", String(centerActive));
  try {
    localStorage.setItem(centerKey, String(centerActive));
  } catch {
    // A preferência permanece válida nesta sessão.
  }
  if (centerActive) centerSelection();
  view.focus();
}

async function renameAt(offset: number) {
  const tab = activeTab();
  if (!tab || tab.language === "rust" || (runTabId === tab.id && runId !== null)) return;
  const doc = view.state.doc;
  const dialog = $<HTMLDialogElement>("rename");
  try {
    const target = semanticSymbolLanguage(tab.language) === "portugol-rs"
      ? await api.renameTargetPortugolRs(doc.toString(), offset)
      : await api.renameTarget(doc.toString(), offset);
    if (activeTab() !== tab || view.state.doc !== doc || document.querySelector("dialog[open]")) return;

    const form = $<HTMLFormElement>("rename-form");
    const input = $<HTMLInputElement>("rename-input");
    const error = $("rename-error");
    const submit = $<HTMLButtonElement>("rename-submit");
    $("rename-description").textContent = `${target.name} · ${target.occurrences} ${target.occurrences === 1 ? "ocorrência" : "ocorrências"}`;
    input.value = target.name;
    error.hidden = true;
    $("rename-cancel").onclick = () => dialog.close();
    form.onsubmit = async (event) => {
      event.preventDefault();
      error.hidden = true;
      submit.disabled = true;
      try {
        const changes = semanticSymbolLanguage(tab.language) === "portugol-rs"
          ? await api.renameSymbolPortugolRs(doc.toString(), offset, input.value.trim())
          : await api.renameSymbol(doc.toString(), offset, input.value.trim());
        if (!dialog.open || activeTab() !== tab || view.state.doc !== doc) return;
        if (changes.length) view.dispatch({ changes, annotations: isolateHistory.of("full"), userEvent: "input.rename" });
        dialog.close();
        view.focus();
      } catch (message) {
        error.textContent = typeof message === "string" ? message : "Não foi possível renomear o símbolo.";
        error.hidden = false;
      } finally {
        submit.disabled = false;
      }
    };
    dialog.showModal();
    input.select();
  } catch (message) {
    if (activeTab() === tab && view.state.doc === doc) {
      setStatus(typeof message === "string" ? message : "Não foi possível localizar o símbolo.", "error");
    }
  }
}

async function goToDeclarationAt(offset: number, quiet = false) {
  const tab = activeTab();
  if (!tab) return;
  if (tab.language === "rust") {
    if (!quiet) setStatus("Navegação de símbolos Rust ainda não está configurada.");
    return;
  }
  const doc = view.state.doc;
  const request = ++definitionNavigationGeneration;
  try {
    const location = semanticSymbolLanguage(tab.language) === "portugol-rs"
      ? await api.resolveSymbolPortugolRs(doc.toString(), offset)
      : await api.resolveSymbol(doc.toString(), offset);
    if (request !== definitionNavigationGeneration || activeTab() !== tab || view.state.doc !== doc) return;
    view.dispatch({
      selection: { anchor: location.start.offset, head: location.end.offset },
      effects: EditorView.scrollIntoView(location.start.offset, { y: "center" }),
    });
    view.focus();
  } catch (error) {
    if (!quiet && request === definitionNavigationGeneration && activeTab() === tab && view.state.doc === doc) {
      setStatus(typeof error === "string" ? error : "Não foi possível ir à declaração.", "error");
    }
  }
}

function renameSelection() {
  return renameAt(view.state.selection.main.from);
}

function goToDeclarationSelection() {
  return goToDeclarationAt(view.state.selection.main.head);
}

async function formatCode() {
  const tab = activeTab();
  if (!tab || (runTabId === tab.id && runId !== null)) return;
  if (tab.language === "rust" && rustfmtAvailable !== true) {
    setStatus("Instale o componente rustfmt com `rustup component add rustfmt`.", "error");
    return;
  }
  const doc = view.state.doc;
  try {
    const source = doc.toString();
    const changes = tab.language === "rust"
      ? await api.formatRustSource(source).then((formatted) =>
        formatted === source ? [] : [{ from: 0, to: doc.length, insert: formatted }])
      : tab.language === "portugol-rs"
        ? await api.formatPortugolRs(source)
        : await api.formatSource(source);
    if (activeTab() !== tab || view.state.doc !== doc) return;
    if (!changes) {
      setStatus("Não foi possível formatar. Confira a sintaxe do programa.", "error");
      return;
    }
    if (changes.length) view.dispatch({ changes, annotations: isolateHistory.of("full"), userEvent: "input.format" });
    setStatus(changes.length ? "Código formatado." : "O código já está formatado.", "ok");
    view.focus();
  } catch (error) {
    if (activeTab() === tab && view.state.doc === doc) {
      setStatus(error instanceof Error ? error.message : String(error), "error");
    }
  }
}

function editorMenuAction(id: string, action: () => unknown) {
  $(id).onclick = () => {
    editorOptions.open = false;
    action();
  };
}
function openExercise(exerciseDocument: ExerciseDocument, workspaceExercise?: { id: string; path: string; preferenceKey: string; exercise: true }) {
  if (workspaceExercise) {
    const existing = tabs.find((tab) =>
      tab.workspaceId === workspaceExercise.id && tab.workspaceExercisePath === workspaceExercise.path,
    );
    if (existing) {
      activate(existing.id);
      return;
    }
  }
  const source = `/*
${exerciseDocument.exercise.statement.replaceAll("*/", "* /")}
*/

${TEMPLATE}`;
  const current = activeTab();
  if (current && !current.path && !current.exampleName && !isDirty(current) && tabs.length === 1) tabs = [];
  const tab = newTab(source, null, exerciseDocument.name, undefined, exerciseDocument);
  if (workspaceExercise) {
    tab.workspaceId = workspaceExercise.id;
    tab.workspaceExercisePath = workspaceExercise.path;
    tab.workspacePreferenceKey = workspaceExercise.preferenceKey;
    selectedWorkspacePath = workspaceExercise.path;
    renderWorkspaceExplorer();
    activate(tab.id);
  }
}
const exercises = mountExercises(openExercise);
$("home-exercises").onclick = exercises.browse;
$("tests-run").onclick = () => void runExerciseTests();
editorMenuAction("editor-exercise", () => {
  const tab = activeTab();
  const exercise = tab?.exercise;
  if (!tab || !exercise) return;
  const documentState = view.state.doc;
  const isCurrent = () => tabs.includes(tab)
    && (tab.id === activeId ? view.state.doc : tab.state.doc) === documentState;
  void exercises.grade(exercise, documentState.toString(), isCurrent);
});
editorMenuAction("editor-search", () => openSearchPanel(view));
editorMenuAction("editor-definition", goToDeclarationSelection);
editorMenuAction("editor-rename", renameSelection);
editorMenuAction("editor-format", formatCode);
editorMenuAction("editor-center", toggleCenterSource);
editorMenuAction("editor-expand", toggleExpanded);
editorMenuAction("btn-utilities", toggleUtilities);
$("btn-unexpand").onclick = toggleExpanded;

document.addEventListener("click", (event) => {
  if (!(event.target instanceof Node)) return;
  for (const options of optionMenus) {
    if (options.open && !options.contains(event.target)) options.open = false;
  }
});
$("utilities-close").onclick = () => {
  showUtilities(false);
  view.focus();
};
$("btn-save").onclick = () => void save();
$("btn-save-as").onclick = () => void save(true);
$("btn-run").onclick = () => void runOrContinue();
function applyLearningMode(mode: LearningMode, persist = true) {
  const availableMode = mode === "rust" && rustcAvailable !== true ? "beginner" : mode;
  learningMode = availableMode;
  const selector = $<HTMLSelectElement>("learning-mode");
  selector.value = availableMode;
  if (persist) saveLearningMode(availableMode, undefined, activeTab()?.workspacePreferenceKey);
  if (!$<HTMLElement>("learning-panel").hidden) renderLearningPanel();

  const preview = $<HTMLElement>("rust-preview");
  const bridgeEnabled = availableMode === "bridge" && activeTab()?.language !== "rust";
  preview.hidden = !bridgeEnabled;
  renderMappingGutter();
  $<HTMLButtonElement>("btn-rust").setAttribute("aria-pressed", String(bridgeEnabled));
  if (bridgeEnabled) {
    if (activeTab()) scheduleRustExport();
  } else {
    rustExportGeneration++;
    if (rustExportTimer !== undefined) window.clearTimeout(rustExportTimer);
    rustExportTimer = undefined;
  }
}

$<HTMLSelectElement>("learning-mode").onchange = (event) => {
  const selected = (event.currentTarget as HTMLSelectElement).value;
  if (selected === "beginner" || selected === "bridge" || selected === "rust") applyLearningMode(selected);
};
applyLearningMode(learningMode, false);
$<HTMLButtonElement>("btn-rust").onclick = () => {
  applyLearningMode(learningMode === "bridge" ? "beginner" : "bridge");
};
$<HTMLButtonElement>("rust-close").onclick = () => {
  applyLearningMode("beginner");
  view.focus();
};

mountPorsUiPreviewPanel({
  elements: {
    panel: $<HTMLElement>("pors-ui-preview"),
    toggleButton: $<HTMLButtonElement>("btn-ui-preview"),
    runButton: $<HTMLButtonElement>("pors-ui-run"),
    restartButton: $<HTMLButtonElement>("pors-ui-restart"),
    cancelButton: $<HTMLButtonElement>("pors-ui-cancel"),
    closeButton: $<HTMLButtonElement>("pors-ui-close"),
    exampleSelect: $<HTMLSelectElement>("pors-ui-example-select"),
    loadExampleButton: $<HTMLButtonElement>("pors-ui-load-example"),
    status: $<HTMLElement>("pors-ui-status"),
    canvas: $<HTMLElement>("pors-ui-canvas"),
  },
  backend: {
    startUi: (source, budget) => api.porsUiStart(source, budget),
    dispatchUi: (sessionId, event) => api.porsUiDispatch(sessionId, event),
    cancelUi: (sessionId) => api.porsUiCancel(sessionId),
    closeUi: (sessionId) => api.porsUiClose(sessionId),
  },
  mount: mountUiPreview,
  examples: PORS_UI_EXAMPLES,
  getActiveSource: () => {
    const tab = activeTab();
    if (!tab) return null;
    return {
      source: view.state.doc.toString(),
      isPors: tab.language === "portugol-rs",
    };
  },
  onLoadExample: (example) => {
    newTab(example.source, null, example.fileName, undefined, undefined, "portugol-rs");
  },
  onCloseFocus: () => view.focus(),
});
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

  const tab = newTab(artifact.code, null, undefined, undefined, undefined, "rust");
  setStatus("Rust copiado para uma nova aba .rs independente. A origem e a prévia vinculada permanecem intactas.", "ok");
  view.focus();
  renderTabs();
  syncLanguageControls(tab);
};
$("btn-debug").onclick = () => void stepOrStart();
$("btn-stop").onclick = () => void stopProgram();
$("btn-debug-pause").onclick = () => {
  if (activeTab()?.language !== "portugol") return;
  if (runId !== null) void api.debugPause(runId);
};
$("btn-debug-continue").onclick = () => void debugCommand(api.debugContinue);
$("btn-debug-step-over").onclick = () => void debugCommand(api.debugStepOver);
$("btn-debug-step-into").onclick = () => void debugCommand(api.debugStepInto);
$("btn-debug-step-out").onclick = () => void debugCommand(api.debugStepOut);
$("btn-clear").onclick = () => output.replaceChildren();
$("btn-copy").onclick = async () => {
  try {
    await navigator.clipboard.writeText(output.textContent ?? "");
    setStatus("Saída copiada.", "ok");
  } catch (error) {
    setStatus(String(error), "error");
  }
};
$("tab-console").onclick = () => showPanel("console");
$("tab-tests").onclick = () => showPanel("tests");
$("tab-messages").onclick = () => showPanel("messages");
$("tab-debug").onclick = () => showPanel("debug");
document.querySelectorAll<HTMLButtonElement>("[data-activity]").forEach((button) => {
  button.onclick = () => {
    document.querySelectorAll<HTMLButtonElement>("[data-activity]").forEach((activity) => {
      activity.setAttribute("aria-pressed", String(activity === button));
    });
    if (button.dataset.activity !== "learn") setInspectorView(false);
    switch (button.dataset.activity) {
      case "project":
        $("btn-explorer").click();
        button.setAttribute("aria-pressed", $("btn-explorer").getAttribute("aria-expanded") ?? "false");
        break;
      case "learn":
        showPanel("learning");
        break;
      case "examples":
        activate(homeId);
        break;
      case "debug":
        showUtilities(true);
        showPanel("debug");
        break;
      case "settings":
        $("editor-options").setAttribute("open", "");
        $<HTMLSelectElement>("theme").focus();
        break;
    }
  };
});
$<HTMLSelectElement>("learning-concept").onchange = (event) => {
  selectedLearningConceptId = (event.currentTarget as HTMLSelectElement).value;
  const tab = activeTab();
  const availableConceptIds = conceptsForMode(learningMode).map(({ id }) => id);
  if (tab && learningPinForMode(tab.learningPins, learningMode, availableConceptIds)) {
    tab.learningPins = setLearningPin(tab.learningPins, learningMode, selectedLearningConceptId);
  }
  learningConceptManuallySelected = true;
  renderLearningPanel();
};
$<HTMLButtonElement>("learning-pin").onclick = (event) => {
  const tab = activeTab();
  if (!tab) return;
  const button = event.currentTarget as HTMLButtonElement;
  tab.learningPins = toggleLearningPin(tab.learningPins, learningMode, button.dataset.conceptId ?? selectedLearningConceptId);
  learningConceptManuallySelected = false;
  renderLearningPanel();
};
$<HTMLButtonElement>("learning-practice-open").onclick = async (event) => {
  if (learningPracticeBusy) return;
  const button = event.currentTarget as HTMLButtonElement;
  const exerciseId = Number(button.dataset.exerciseId);
  if (!Number.isSafeInteger(exerciseId) || exerciseId < 1 || !activeTab()) return;

  learningPracticeBusy = true;
  button.disabled = true;
  const status = $<HTMLElement>("learning-practice-status");
  status.textContent = `Abrindo exercício ${exerciseId}…`;
  try {
    await exercises.openById(exerciseId);
    status.textContent = `Exercício ${exerciseId} aberto no editor. A abertura não registra progresso.`;
  } catch {
    status.textContent = "Não foi possível abrir o exercício. Tente novamente ou use a lista de exercícios.";
  } finally {
    learningPracticeBusy = false;
    renderLearningPanel();
  }
};
$<HTMLSelectElement>("ownership-scenario").onchange = (event) => {
  const selected = (event.currentTarget as HTMLSelectElement).value;
  if (ownershipScenarioIds().includes(selected as OwnershipScenarioId)) ownershipScenarioId = selected as OwnershipScenarioId;
  ownershipStepIndex = 0;
  renderOwnershipLab();
};
$<HTMLButtonElement>("ownership-previous").onclick = () => {
  ownershipStepIndex = Math.max(0, ownershipStepIndex - 1);
  renderOwnershipLab();
};
$<HTMLButtonElement>("ownership-next").onclick = () => {
  const scenario = getOwnershipScenario(ownershipScenarioId);
  ownershipStepIndex = Math.min(scenario ? scenario.snapshots.length - 1 : 0, ownershipStepIndex + 1);
  renderOwnershipLab();
};
$<HTMLButtonElement>("ownership-restart").onclick = () => {
  ownershipStepIndex = 0;
  renderOwnershipLab();
};

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
      result = tab.language === "portugol-rs"
        ? await api.exportPortugolRs(source)
        : await api.exportRust(source);
    } catch (error) {
      if (generation !== rustExportGeneration || activeTab()?.id !== tabId || view.state.doc.toString() !== source) return;
      tab.rustPreview = failRustPreview(tab.rustPreview ?? { pending: false, stale: false }, "Falha ao solicitar a tradução: " + String(error));
      renderRustPreview(tab);
      return;
    }
    if (generation !== rustExportGeneration || activeTab()?.id !== tabId || view.state.doc.toString() !== source) return;
    tab.rustPreview = acceptRustPreview(tab.rustPreview ?? { pending: false, stale: false }, source, result);
    renderRustPreview(tab);
  }, 220);
}

function renderRustPreview(tab: Tab) {
  if (activeTab()?.id !== tab.id) return;
  const state = tab.rustPreview;
  const status = $<HTMLElement>("rust-status");
  const copy = $<HTMLButtonElement>("rust-copy");
  const save = $<HTMLButtonElement>("rust-save");
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
  const mappings: GeneratedRustMapping[] = artifact && !stale
    ? mappedPreviewSegments(artifact.code, artifact.sourceMap).map((segment) => ({
      ...segment,
      quality: segment.quality === "approximate" ? "approximate" : "exact",
    }))
    : [];
  const generatedDocument = generatedRustEditor.state.doc;
  const mappingEffect = generatedRustMappings.of(mappings);
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
  renderMappingGutter();
  if (!$<HTMLElement>("learning-panel").hidden) renderLearningPanel();
}

function renderGeneratedRustDiff(container: HTMLElement, previous: string, current: string) {
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

function renderMappingGutter() {
  const gutter = $<HTMLElement>("rust-mapping-gutter");
  const preview = $<HTMLElement>("rust-preview");
  const tab = activeTab();
  const visible = Boolean(tab && tab.language !== "rust" && learningMode === "bridge" && !preview.hidden);
  gutter.hidden = !visible;
  if (!visible) return;

  const sourceLine = view.state.doc.lineAt(view.state.selection.main.head).number;
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

function cycleTab(step: number) {
  const ids = [homeId, ...(helpOpen ? [helpId] : []), ...tabs.map((tab) => tab.id)];
  activate(ids[(ids.indexOf(activeId) + step + ids.length) % ids.length]);
}

const shortcutDialog = $<HTMLDialogElement>("shortcuts");

const commandPalette = $<HTMLDialogElement>("command-palette");
const commandSearch = $<HTMLInputElement>("command-search");
const commandResults = $("command-results");
const commandEmpty = $("command-empty");
let paletteReturnFocus: HTMLElement | null = null;
let paletteCommands: typeof shortcuts = [];
let selectedCommandIndex = -1;

const tabCommands = new Set([
  "Executar", "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução",
  "Entrar na função", "Sair da função", "Renomear símbolo", "Formatar código-fonte", "Analisar projeto Cargo",
  "Salvar", "Salvar como", "Parâmetros de execução", "Fechar aba", "Fechar todas as abas", "Localizar e substituir",
  "Buscar na estrutura", "Filtrar variável", "Filtrar vetor", "Filtrar matriz", "Filtrar função",
  "Filtrar constante", "Mostrar todos os tipos", "Filtrar tipo inteiro", "Filtrar tipo caracter",
  "Filtrar tipo real", "Filtrar tipo cadeia", "Filtrar tipo logico", "Filtrar tipo vazio",
]);
const rustUnsupportedCommands = new Set([
  "Executar", "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução",
  "Entrar na função", "Sair da função", "Renomear símbolo", "Formatar código-fonte",
  "Buscar na estrutura", "Filtrar variável", "Filtrar vetor", "Filtrar matriz", "Filtrar função",
  "Filtrar constante", "Mostrar todos os tipos", "Filtrar tipo inteiro", "Filtrar tipo caracter",
  "Filtrar tipo real", "Filtrar tipo cadeia", "Filtrar tipo logico", "Filtrar tipo vazio",
]);
const debugCommands = new Set([
  "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução", "Entrar na função", "Sair da função",
]);

function availablePaletteCommands() {
  const tab = activeTab();
  return shortcuts.filter(({ command }) => {
    if (command === "Paleta de comandos") return false;
    if (command === "Sobre (fora do editor); ir à declaração (no editor)") return false;
    if (tabCommands.has(command) && !tab) return false;
    if (command === "Executar" && tab?.language === "portugol-rs" && rustcAvailable !== true) return false;
    if (command === "Analisar projeto Cargo") return tab?.language === "rust" && tab.cargoProject === true && cargoAvailable === true && runId === null && !porsRunPending;
    if (command === "Parâmetros de execução" && tab?.language === "portugol-rs") return false;
    if (tab?.language === "rust" && rustUnsupportedCommands.has(command)) return false;
    if (debugCommands.has(command) && !isDebugCommandAvailable(command, tab?.language ?? null, runId !== null, paused !== null)) return false;
    return true;
  });
}

function renderCommandPalette() {
  paletteCommands = filterCommands(availablePaletteCommands(), commandSearch.value);
  selectedCommandIndex = paletteCommands.length > 0 ? Math.min(Math.max(selectedCommandIndex, 0), paletteCommands.length - 1) : -1;
  commandResults.replaceChildren();
  commandEmpty.hidden = paletteCommands.length > 0;
  commandSearch.setAttribute("aria-expanded", "true");
  for (const [index, item] of paletteCommands.entries()) {
    const option = document.createElement("button");
    option.type = "button";
    option.className = "command-option";
    option.id = `command-option-${index}`;
    option.setAttribute("role", "option");
    option.setAttribute("aria-selected", String(index === selectedCommandIndex));
    const name = document.createElement("span");
    name.textContent = item.command;
    const shortcut = document.createElement("kbd");
    shortcut.textContent = item.label;
    option.append(name, shortcut);
    option.addEventListener("mouseenter", () => {
      selectedCommandIndex = index;
      updateSelectedPaletteCommand();
    });
    option.addEventListener("click", () => executePaletteCommand(index));
    commandResults.append(option);
  }
  updateSelectedPaletteCommand();
}

function updateSelectedPaletteCommand() {
  for (const [index, option] of [...commandResults.querySelectorAll<HTMLElement>("[role=option]")].entries()) {
    option.setAttribute("aria-selected", String(index === selectedCommandIndex));
  }
  const id = selectedCommandIndex >= 0 ? `command-option-${selectedCommandIndex}` : "";
  if (id) commandSearch.setAttribute("aria-activedescendant", id);
  else commandSearch.removeAttribute("aria-activedescendant");
}

function executePaletteCommand(index: number) {
  const item = paletteCommands[index];
  if (!item) return;
  if (!availablePaletteCommands().includes(item)) {
    renderCommandPalette();
    return;
  }
  commandPalette.close();
  void item.action();
}

function showCommandPalette() {
  if (commandPalette.open) return;
  paletteReturnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  commandSearch.value = "";
  selectedCommandIndex = 0;
  renderCommandPalette();
  commandPalette.showModal();
  commandSearch.focus();
}

commandSearch.addEventListener("input", () => {
  selectedCommandIndex = 0;
  renderCommandPalette();
});
commandSearch.addEventListener("keydown", (event) => {
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    selectedCommandIndex = nextCommandIndex(paletteCommands.length, selectedCommandIndex, event.key);
    updateSelectedPaletteCommand();
  } else if (event.key === "Enter") {
    event.preventDefault();
    executePaletteCommand(selectedCommandIndex);
  }
});
commandPalette.addEventListener("close", () => {
  const focusTarget = paletteReturnFocus;
  paletteReturnFocus = null;
  if (focusTarget?.isConnected && !focusTarget.hasAttribute("disabled")) focusTarget.focus();
});

function showShortcuts() {
  if (!shortcutDialog.open) shortcutDialog.showModal();
}

const shortcuts: {
  label: string;
  command: string;
  matches: (event: KeyboardEvent) => boolean;
  action: () => unknown;
}[] = [
  { label: "Shift+F6 / Ctrl+Enter", command: "Executar", matches: (e) =>
    (e.shiftKey && e.key === "F6") || ((e.ctrlKey || e.metaKey) && e.key === "Enter"), action: runOrContinue },
  { label: "Shift+F5", command: "Depurar", matches: (e) => e.shiftKey && e.key === "F5",
    action: stepOrStart },
  { label: "Shift+F7", command: "Interromper", matches: (e) => e.shiftKey && e.key === "F7", action: stopProgram },
  { label: "F9", command: "Pausar", matches: (e) => !e.shiftKey && e.key === "F9",
    action: () => { if (runId !== null) void api.debugPause(runId); } },
  { label: "F5", command: "Continuar", matches: (e) => !e.shiftKey && e.key === "F5",
    action: () => debugCommand(api.debugContinue) },
  { label: "Shift+F9 / F10", command: "Próxima instrução",
    matches: (e) => (e.shiftKey && e.key === "F9") || e.key === "F10", action: () => debugCommand(api.debugStepOver) },
  { label: "Alt+F11", command: "Entrar na função", matches: (e) => e.altKey && !e.shiftKey && e.key === "F11",
    action: () => debugCommand(api.debugStepInto) },
  { label: "Shift+F11", command: "Sair da função", matches: (e) => e.shiftKey && e.key === "F11",
    action: () => debugCommand(api.debugStepOut) },
  { label: "F1", command: "Ajuda", matches: (e) => e.key === "F1", action: () => activate(helpId) },
  { label: "Ctrl+R", command: "Renomear símbolo",
    matches: (e) => (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "KeyR", action: renameSelection },
  { label: "Ctrl+Alt+F", command: "Formatar código-fonte",
    matches: (e) => (e.ctrlKey || e.metaKey) && e.altKey && !e.shiftKey && e.code === "KeyF", action: formatCode },
  { label: "via paleta de comandos", command: "Analisar projeto Cargo", matches: () => false, action: checkCargoProject },
  { label: "F3", command: "Dicas de interface", matches: (e) => e.key === "F3", action: () => tips.open() },
  { label: "F12", command: "Sobre (fora do editor); ir à declaração (no editor)", matches: (e) => e.key === "F12", action: () =>
    $<HTMLDialogElement>("about").showModal() },
  { label: "F11", command: "Atalhos de teclado", matches: (e) => !e.altKey && !e.shiftKey && e.key === "F11",
    action: showShortcuts },
  { label: "Alt+Home", command: "Tela inicial", matches: (e) => e.altKey && e.key === "Home", action: () => activate(0) },
  { label: "Alt+←", command: "Aba anterior", matches: (e) => e.altKey && e.key === "ArrowLeft", action: () => cycleTab(-1) },
  { label: "Alt+→", command: "Próxima aba", matches: (e) => e.altKey && e.key === "ArrowRight", action: () => cycleTab(1) },
  { label: "Shift+Pause", command: "Ativar/desativar centralização",
    matches: (e) => e.shiftKey && e.key === "Pause", action: toggleCenterSource },
  { label: "Shift+Esc", command: "Expandir/restaurar editor",
    matches: (e) => e.shiftKey && e.key === "Escape", action: toggleExpanded },
  { label: "Ctrl+Shift+P", command: "Paleta de comandos", matches: (e) =>
    (e.ctrlKey || e.metaKey) && e.shiftKey && !e.altKey && e.code === "KeyP", action: showCommandPalette },
];

function fileShortcut(key: string, shift: boolean, command: string, action: () => unknown) {
  shortcuts.push({
    label: shift ? `Ctrl+Shift+${key}` : `Ctrl+${key}`,
    command,
    matches: (e) => (e.ctrlKey || e.metaKey) && e.shiftKey === shift && e.key.toUpperCase() === key,
    action,
  });
}

fileShortcut("N", false, "Novo arquivo", () => newTab());
fileShortcut("O", false, "Abrir arquivo", openFiles);
fileShortcut("E", true, "Mostrar ou ocultar o explorador", () => {
  if (!workspaceState) void openWorkspaceFolder();
  else setWorkspaceExplorerVisible(Boolean($("workspace-explorer").hidden));
});
fileShortcut("S", false, "Salvar", () => save());
fileShortcut("S", true, "Salvar como", () => save(true));
fileShortcut("E", false, "Parâmetros de execução", () => {
  if (!activeTab()) return;
  editorOptions.open = true;
  $("args").focus();
});
fileShortcut("B", false, "Painel Utilitários", toggleUtilities);
fileShortcut("L", false, "Buscar na estrutura", () => {
  if (!activeTab()) return;
  showUtilities(true);
  outline.focus();
});
fileShortcut("Q", false, "Fechar aba", () => closeTab(activeId));
fileShortcut("Q", true, "Fechar todas as abas", async () => {
  for (const tab of [...tabs]) if (!(await closeTab(tab.id))) return;
  if (helpOpen) await closeTab(helpId);
});
shortcuts.push({ label: "Ctrl+F", command: "Localizar e substituir", matches: (e) =>
  (e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.code === "KeyF", action: () => openSearchPanel(view) });

for (const [key, ty] of Object.entries({ I: "inteiro", C: "caracter", R: "real", A: "cadeia", L: "logico", V: "vazio", T: "" })) {
  shortcuts.push({
    label: `Alt+Shift+${key}`, command: ty ? `Filtrar tipo ${ty}` : "Mostrar todos os tipos",
    matches: (e) => e.altKey && e.shiftKey && e.code === `Key${key}`,
    action: () => { if (activeTab()) { showUtilities(true); outline.filterType(ty); } },
  });
}

const symbolShortcuts = { A: "variable", E: "array", M: "matrix", F: "function", T: "constant" } as const;
for (const [key, kind] of Object.entries(symbolShortcuts)) {
  fileShortcut(key, true, `Filtrar ${symbolKindLabels[kind].toLowerCase()}`, () => {
    if (!activeTab()) return;
    showUtilities(true);
    outline.filterKind(kind);
  });
}

for (const { label, command } of shortcuts) {
  const term = document.createElement("dt");
  const shortcut = document.createElement("kbd");
  const description = document.createElement("dd");
  shortcut.textContent = label;
  description.textContent = command;
  term.append(shortcut);
  $("shortcut-list").append(term, description);
}
$("home-shortcuts").onclick = showShortcuts;
const aboutDialog = $<HTMLDialogElement>("about");
const reportDialog = $<HTMLDialogElement>("report");
const reportError = $("report-error");
$("home-about").onclick = () => aboutDialog.showModal();
const updates = $<HTMLDialogElement>("updates");
const updateStatus = $("update-status");
const updateNotes = $("update-notes");
const updateDownload = $<HTMLButtonElement>("update-download");
let updateRequest = 0;
$("about-updates").onclick = async () => {
  const request = ++updateRequest;
  updateNotes.hidden = true;
  updateDownload.textContent = "Ver versões no GitHub";
  updateStatus.textContent = "Verificando atualizações…";
  $("update-error").hidden = true;
  updates.showModal();
  try {
    const result = await api.checkUpdates();
    if (request !== updateRequest) return;
    if (result.status === "available") {
      updateStatus.textContent = `Versão ${result.latest} disponível. Versão instalada ${result.current}.`;
      updateNotes.textContent = result.notes;
      updateNotes.hidden = !result.notes;
      updateDownload.textContent = "Baixar atualização";
    } else if (result.status === "current") {
      updateStatus.textContent = `O Portugol Studio RS ${result.current} está atualizado.`;
    } else {
      updateStatus.textContent = "Ainda não há uma versão estável publicada no GitHub.";
    }
  } catch (message) {
    if (request === updateRequest) {
      updateStatus.textContent = typeof message === "string" ? message : "Não foi possível verificar atualizações.";
    }
  }
};
updateDownload.onclick = async () => {
  updateDownload.disabled = true;
  $("update-error").hidden = true;
  try {
    await api.openProjectPage("releases");
  } catch {
    $("update-error").hidden = false;
  } finally {
    updateDownload.disabled = false;
  }
};
let alertRequest = 0;
$("home-alerts").onclick = async () => {
  const request = ++alertRequest;
  const dialog = $<HTMLDialogElement>("alerts");
  const status = $("alerts-status");
  const list = $("alerts-list");
  status.hidden = false;
  status.textContent = "Consultando alertas…";
  list.hidden = true;
  list.replaceChildren();
  dialog.showModal();
  try {
    const alerts = await api.projectAlerts();
    if (request !== alertRequest) return;
    if (!alerts.length) {
      status.textContent = "Não há alertas ou avisos.";
      return;
    }
    for (const alert of alerts) {
      const article = document.createElement("article");
      const title = document.createElement("h3");
      title.textContent = alert.title;
      const message = document.createElement("p");
      message.textContent = alert.message;
      article.append(title, message);
      list.append(article);
    }
    status.hidden = true;
    list.hidden = false;
  } catch (message) {
    if (request === alertRequest) {
      status.textContent = typeof message === "string" ? message : "Não foi possível consultar os alertas.";
    }
  }
};
$("home-report").onclick = $("about-report").onclick = () => {
  reportError.hidden = true;
  reportDialog.showModal();
};
$("home-contribute").onclick = () => {
  void api.openProjectPage("contribute").catch(() => setStatus("Não foi possível abrir o navegador.", "error"));
};
$("report-open").onclick = async () => {
  const button = $<HTMLButtonElement>("report-open");
  button.disabled = true;
  reportError.hidden = true;
  try {
    await api.openProjectPage("bug_report");
    reportDialog.close();
  } catch {
    reportError.textContent = "Não foi possível abrir o navegador. Use github.com/wendelmax/portugol-studio-rs/issues/new.";
    reportError.hidden = false;
  } finally {
    button.disabled = false;
  }
};
$("home-help").onclick = $("btn-help").onclick = () => activate(helpId);

window.addEventListener("keydown", (e) => {
  if (e.defaultPrevented || document.querySelector("dialog[open]")) return;
  if (e.key === "Escape" && !e.shiftKey) {
    const menu = optionMenus.find((options) => options.open);
    if (menu) {
      e.preventDefault();
      menu.open = false;
      menu.querySelector("summary")?.focus();
      return;
    }
  }
  const shortcut = shortcuts.find(({ matches }) => matches(e));
  if (!shortcut) return;
  e.preventDefault();
  void shortcut.action();
});

// Confirma antes de fechar a janela com alterações não salvas.
void runListeners.catch((error) => setStatus(String(error), "error"));
const appWindow = getCurrentWindow();
function windowAction(action: () => Promise<void>) {
  void action().catch((error) => setStatus(String(error), "error"));
}
$("window-min").onclick = () => windowAction(() => appWindow.minimize());
$("window-max").onclick = () => windowAction(() => appWindow.toggleMaximize());
$("window-close").onclick = () => windowAction(() => appWindow.close());
void appWindow.onCloseRequested(async (event) => {
  const current = activeTab();
  if (current) current.state = view.state;
  const dirty = tabs.filter(isDirty);
  if (dirty.length === 0) return;
  const names = dirty.map(tabName).join(", ");
  const quit = await ask(`Há alterações não salvas em: ${names}. Sair mesmo assim?`, {
    title: "Sair", kind: "warning", okLabel: "Sair sem salvar", cancelLabel: "Cancelar",
  });
  if (!quit) event.preventDefault();
});

const themeSelect = $<HTMLSelectElement>("theme");
const themeEditor = mountThemeEditor(themeSelect, applyTheme);
themeSelect.value = themeChoice();
themeSelect.onchange = () => applyTheme(themeSelect.value);
editorMenuAction("editor-themes", themeEditor.open);

for (const id of ["editor-font", "console-font"]) {
  const input = $<HTMLInputElement>(id);
  const key = `portugol.${id}`;
  const applyFontSize = () => {
    const size = Math.round(Math.max(10, Math.min(50, input.valueAsNumber || 12)));
    input.value = String(size);
    document.documentElement.style.setProperty(`--${id}`, `${size}px`);
    return input.value;
  };
  try {
    input.value = localStorage.getItem(key) ?? input.value;
  } catch {
    // O ajuste manual permanece disponível.
  }
  applyFontSize();
  input.onchange = () => {
    const value = applyFontSize();
    try {
      localStorage.setItem(key, value);
    } catch {
      // A preferência vale nesta sessão.
    }
  };
}

let help: ReturnType<typeof import("./help").mountHelp> | undefined;
let helpLoading: Promise<void> | undefined;

async function openHelp() {
  if (!help) {
    helpLoading ??= import("./help").then(({ mountHelp }) => {
      help = mountHelp((source, file) => newTab(source, null, file));
    }).catch((error) => {
      $("help-article").textContent = "Não foi possível carregar a ajuda.";
      setStatus(String(error), "error");
      helpLoading = undefined;
    });
    await helpLoading;
  }
  if (activeId === helpId) help?.focusSearch();
}
$("home-version").textContent = `v${version}`;
$("about-version").textContent = `Versão ${version}`;
$("about-license").textContent = license;
mountIcons();
renderRecent();
mountExamples((source, file, workPath) => newTab(source, null, file, workPath));
activate(0);
applyTheme(themeChoice());
showPanel("console");
setWorkspaceExplorerVisible(window.innerWidth >= 1440);
showUtilities(window.innerWidth >= 1440);
void api.rustToolchainAvailable().then((available) => {
  rustcAvailable = available;
  const rustOption = $<HTMLOptionElement>("learning-mode-rust-option");
  rustOption.disabled = !available;
  $("learning-mode-rust-help").textContent = available
    ? "Toolchain rustc disponível. Novo Rust cria um arquivo .rs; trocar de nível não altera o arquivo aberto."
    : "Instale a toolchain Rust (rustc) para habilitar este nível e executar arquivos .rs.";
  applyLearningMode(loadLearningMode(undefined, activeTab()?.workspacePreferenceKey), false);
  const tab = activeTab();
  if (!tab) return;
  syncLanguageControls(tab);
  if (tab.language === "rust") forceLinting(view);
  if (tab.language === "portugol-rs" && !available) {
    setStatus("Instale a toolchain Rust para executar arquivos .pors.", "error");
  }
}).catch(() => {
  rustcAvailable = false;
  $<HTMLOptionElement>("learning-mode-rust-option").disabled = true;
  $("learning-mode-rust-help").textContent = "Não foi possível verificar rustc. Instale ou configure a toolchain Rust para habilitar este nível.";
  applyLearningMode(loadLearningMode(undefined, activeTab()?.workspacePreferenceKey), false);
  const tab = activeTab();
  if (tab) {
    syncLanguageControls(tab);
    if (tab.language === "rust") forceLinting(view);
  }
});
void api.cargoToolchainAvailable().then((available) => {
  cargoAvailable = available;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
}).catch(() => {
  cargoAvailable = false;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
});
void api.rustfmtAvailable().then((available) => {
  rustfmtAvailable = available;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
}).catch(() => {
  rustfmtAvailable = false;
  const tab = activeTab();
  if (tab) syncLanguageControls(tab);
});
const tips = mountTips(themeEditor.open);
$("home-tips").onclick = tips.open;


async function openIncomingFiles(paths: string[], warnUnsupported = false) {
  let supported = false;
  for (const path of paths) {
    try {
      if (/\.pex$/i.test(path)) {
        supported = true;
        openExercise(await readExercise(path));
      } else if (/\.por$/i.test(path)) {
        supported = true;
        await openFile(path);
      } else if (/\.pors$/i.test(path)) {
        supported = true;
        await openFile(path);
      } else if (/\.rs$/i.test(path)) {
        supported = true;
        await openFile(path);
      }
    } catch (error) {
      setStatus(typeof error === "string" ? error : "Não foi possível abrir o arquivo.", "error");
    }
  }
  if (warnUnsupported && !supported && paths.length) setStatus("Escolha um arquivo .por, .pors, .rs ou .pex.", "error");
}

async function consumeSystemFiles() {
  try {
    await openIncomingFiles(await api.pendingFiles());
  } catch {
    setStatus("Não foi possível receber arquivos do sistema.", "error");
  }
}

void api.onSystemFiles(() => { void consumeSystemFiles(); }).then(consumeSystemFiles, consumeSystemFiles);
void getCurrentWindow().onDragDropEvent(({ payload }) => {
  if (payload.type === "drop") void openIncomingFiles(payload.paths, true);
}).catch(() => {});
