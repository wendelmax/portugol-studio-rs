// Comandos e eventos do backend (apps/desktop/src-tauri/src/lib.rs).

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Position {
  /** Offset em unidades UTF-16, o mesmo do CodeMirror. */
  offset: number;
  line: number;
  column: number;
}

export interface DiagnosticInfo {
  severity: "error" | "warning";
  code: string;
  message: string;
  help: string | null;
  start: Position;
  end: Position;
}

export interface TraceFrame {
  function: string;
  line: number;
  column: number;
}

export type RunOutcome =
  | { status: "finished" }
  | { status: "stopped" }
  | { status: "error"; message: string; start: Position; end: Position; trace: TraceFrame[] };

export interface DebugVar {
  name: string;
  ty: string;
  value: string;
  scope: "global" | "local";
}

export interface DebugFrame {
  function: string;
  position: Position;
}

export interface DebugInfo {
  position: Position;
  stack: DebugFrame[];
  variables: DebugVar[];
}

export type SymbolKind = "library" | "function" | "constant" | "variable" | "parameter" | "array" | "matrix";

export interface SymbolInfo {
  name: string;
  kind: SymbolKind;
  ty: string;
  position: Position;
  children: SymbolInfo[];
}

export interface SymbolLocation {
  start: Position;
  end: Position;
}

export interface Inspection {
  diagnostics: DiagnosticInfo[];
  symbols: SymbolInfo[];
}

export const includeLibrary = (source: string, name: string) =>
  invoke<{ from: number; to: number; insert: string } | null>("include_library", { source, name });

export const formatSource = (source: string) =>
  invoke<{ from: number; to: number; insert: string }[] | null>("format_source", { source });

export interface Exercise {
  id: number;
  statement: string;
  cases: number;
}

export const listExercises = () => invoke<Exercise[]>("exercise_catalog");
export const inspectExercise = (contents: string) => invoke<Exercise>("inspect_exercise", { contents });
export const checkExercise = (id: number, source: string, contents?: string) =>
  invoke<{ total: number; passed: number; failures: string[]; hints: string[] }>("check_exercise", { id, source, contents });

export const renameTarget = (source: string, offset: number) =>
  invoke<{ name: string; occurrences: number }>("rename_target", { source, offset });

export const renameSymbol = (source: string, offset: number, name: string) =>
  invoke<{ from: number; to: number; insert: string }[]>("rename_symbol", { source, offset, name });

export const resolveSymbol = (source: string, offset: number) =>
  invoke<SymbolLocation>("resolve_symbol", { source, offset });

export const openProjectPage = (page: "bug_report" | "releases" | "contribute") =>
  invoke<void>("open_project_page", { page });

type UpdateInfo =
  | { status: "available"; current: string; latest: string; notes: string }
  | { status: "current"; current: string; latest: string }
  | { status: "unpublished"; current: string };

export const projectAlerts = () =>
  invoke<{ title: string; message: string }[]>("project_alerts");

export const checkUpdates = () => invoke<UpdateInfo>("check_updates");

export const inspect = (source: string) => invoke<Inspection>("inspect", { source });

export interface RustExport {
  code: string | null;
  diagnostic: DiagnosticInfo | null;
}

export const exportRust = (source: string) => invoke<RustExport>("export_rust", { source });
export const inspectPortugolRs = (source: string) => invoke<Inspection>("inspect_portugol_rs", { source });
export const exportPortugolRs = (source: string) => invoke<RustExport>("export_portugol_rs", { source });

/** Id da execução; rejeita com a lista de erros se o programa não compila. */
export const run = (source: string, args: string[], path: string | null, breakpoints: number[] = [], pauseOnStart = false) =>
  invoke<number>("run", { source, args, path, breakpoints, pauseOnStart });

export const sendInput = (id: number, line: string) => invoke<void>("send_input", { id, line });

export const stop = (id: number) => invoke<void>("stop", { id });

export const debugPause = (id: number) => invoke<void>("debug_pause", { id });
export const debugContinue = (id: number) => invoke<void>("debug_continue", { id });
export const debugStepOver = (id: number) => invoke<void>("debug_step_over", { id });
export const debugStepInto = (id: number) => invoke<void>("debug_step_into", { id });
export const debugStepOut = (id: number) => invoke<void>("debug_step_out", { id });
export const debugSetBreakpoints = (id: number, source: string, lines: number[]) =>
  invoke<void>("debug_set_breakpoints", { id, source, lines });

export const pendingFiles = () => invoke<string[]>("pending_files");
export const onSystemFiles = (handler: () => void) => listen("project-files-opened", handler);

export const readFile = (path: string) => invoke<string>("read_file", { path });

export const writeFile = (path: string, text: string) => invoke<void>("write_file", { path, text });

export type CatalogEntry =
  | { kind: "folder"; name: string; entries: CatalogEntry[] }
  | { kind: "file"; name: string; file: string; description: string; preview: string | null };

export const exampleCatalog = () => invoke<CatalogEntry[]>("example_catalog");
export const openExample = (file: string) => invoke<{ source: string; workPath: string }>("open_example", { file });
export const examplePreview = (file: string) => invoke<ArrayBuffer>("example_preview", { file });

export interface RunEvents {
  output(id: number, text: string): void;
  clear(id: number): void;
  input(id: number, expected: string): void;
  paused(id: number, info: DebugInfo): void;
  finished(id: number, outcome: RunOutcome): void;
}

export async function listenRun(handlers: RunEvents): Promise<UnlistenFn> {
  const unlisten = await Promise.all([
    listen<{ id: number; text: string }>("run:output", (e) => handlers.output(e.payload.id, e.payload.text)),
    listen<{ id: number }>("run:clear", (e) => handlers.clear(e.payload.id)),
    listen<{ id: number; expected: string }>("run:input", (e) => handlers.input(e.payload.id, e.payload.expected)),
    listen<{ id: number; info: DebugInfo }>("run:paused", (e) => handlers.paused(e.payload.id, e.payload.info)),
    listen<{ id: number; outcome: RunOutcome }>("run:finished", (e) => handlers.finished(e.payload.id, e.payload.outcome)),
  ]);
  return () => unlisten.forEach((u) => u());
}
