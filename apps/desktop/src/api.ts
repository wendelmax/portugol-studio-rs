// Comandos e eventos do backend (apps/desktop/src-tauri/src/lib.rs).

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PorsUiBudget, PorsUiResult } from "./pors-wasm-host";
import type { UiEvent } from "./ui-preview-renderer";
import type { WorkspaceEntry } from "./workspace-explorer";

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
  | { status: "runtime_limit"; reason: "time" | "output" | "string_memory" }
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

export const formatPortugolRs = (source: string) =>
  invoke<{ from: number; to: number; insert: string }[] | null>("format_portugol_rs", { source });

export interface Exercise {
  id: number;
  statement: string;
  cases: number;
}

export interface ExerciseCheckResult {
  total: number;
  passed: number;
  failures: string[];
  hints: string[];
}

export const renameTargetPortugolRs = (source: string, offset: number) =>
  invoke<{ name: string; occurrences: number }>("rename_target_portugol_rs", { source, offset });

export const renameSymbolPortugolRs = (source: string, offset: number, name: string) =>
  invoke<{ from: number; to: number; insert: string }[]>("rename_symbol_portugol_rs", { source, offset, name });

export const resolveSymbolPortugolRs = (source: string, offset: number) =>
  invoke<SymbolLocation>("resolve_symbol_portugol_rs", { source, offset });

export const openProjectPage = (page: "bug_report" | "releases" | "contribute") =>
  invoke<void>("open_project_page", { page });

type UpdateInfo =
  | { status: "available"; current: string; latest: string; notes: string }
  | { status: "current"; current: string; latest: string }
  | { status: "unpublished"; current: string };

export const projectAlerts = () =>
  invoke<{ title: string; message: string }[]>("project_alerts");

export const checkUpdates = () => invoke<UpdateInfo>("check_updates");

export interface SourceMapInfo {
  generatedStart: Position;
  generatedEnd: Position;
  sourceStart: Position | null;
  sourceEnd: Position | null;
  quality: "exact" | "approximate" | "generated_only";
}

export interface RustExport {
  code: string | null;
  diagnostic: DiagnosticInfo | null;
  sourceMap: SourceMapInfo[];
}

export const inspectPortugolRs = (source: string) => invoke<Inspection>("inspect_portugol_rs", { source });
export const exportPortugolRs = (source: string) => invoke<RustExport>("export_portugol_rs", { source });
export interface PortugolRsCompileDiagnostic {
  severity: string;
  code: string | null;
  message: string;
  rendered: string | null;
  generatedStart: Position | null;
  generatedEnd: Position | null;
  sourceStart: Position | null;
  sourceEnd: Position | null;
  quality: SourceMapInfo["quality"];
}

export type PortugolRsRunResult =
  | {
      status: "executed";
      stdout: string;
      stderr: string;
      exitCode: number | null;
    }
  | {
      status: "compile_error";
      diagnostics: PortugolRsCompileDiagnostic[];
      stderr: string;
    }
  | { status: "compile_limit"; stderr: string; reason: "time" | "output" }
  | {
      status: "runtime_limit";
      stdout: string;
      stderr: string;
      reason: "time" | "output" | "string_memory";
    }
  | { status: "cancelled"; phase: "compilation" | "runtime"; stdout: string; stderr: string };
export const rustToolchainAvailable = () => invoke<boolean>("rust_toolchain_available");
export const cargoToolchainAvailable = () => invoke<boolean>("cargo_toolchain_available");
export interface CargoProjectRunResult {
  status: "executed" | "build_error" | "build_limit" | "run_error" | "runtime_limit" | "cancelled";
  stdout: string;
  stderr: string;
  exitCode: number | null;
  phase: "compilation" | "runtime" | null;
  reason: "time" | "output" | "memory" | "cancelled" | null;
}
export interface CargoRunTarget {
  packageName: string;
  targetName: string;
}
export interface CargoRunTargets {
  targets: CargoRunTarget[];
  defaultTarget: CargoRunTarget | null;
}
export const workspaceCargoProjectAvailable = (workspaceId: string, relativeSource: string) =>
  invoke<boolean>("workspace_cargo_project_available", { workspaceId, relativeSource });
export const cargoProjectRunTargets = (id: number, workspaceId: string, relativeSource: string) =>
  invoke<CargoRunTargets>("cargo_project_run_targets", { id, workspaceId, relativeSource });
export const runCargoProject = (
  id: number,
  workspaceId: string,
  relativeSource: string,
  target: CargoRunTarget | null,
  args: string[],
) => invoke<CargoProjectRunResult>("run_cargo_project", { id, workspaceId, relativeSource, target, args });
export interface CargoProjectDiagnostic extends Omit<DiagnosticInfo, "severity"> {
  severity: string;
  rendered: string | null;
}
export interface CargoProjectCheckResult {
  status: "checked" | "check_error" | "check_limit" | "cancelled";
  diagnostics: CargoProjectDiagnostic[];
  stderr: string;
  reason: "time" | "output" | "memory" | "cancelled" | null;
}
export const checkCargoProject = (id: number, workspaceId: string, relativeSource: string) =>
  invoke<CargoProjectCheckResult>("check_cargo_project", { id, workspaceId, relativeSource });
export interface CargoProjectTestResult {
  status: "passed" | "failed" | "build_error" | "test_limit" | "cancelled";
  passed: number;
  failed: number;
  ignored: number;
  failures: string[];
  stdout: string;
  stderr: string;
  exitCode: number | null;
  reason: "time" | "output" | "memory" | "cancelled" | null;
}
export const testCargoProject = (id: number, workspaceId: string, relativeSource: string) =>
  invoke<CargoProjectTestResult>("test_cargo_project", { id, workspaceId, relativeSource });
export const rustfmtAvailable = () => invoke<boolean>("rustfmt_available");
export const formatRustSource = (source: string) => invoke<string>("format_rust_source", { source });
export const checkRustSource = (source: string) =>
  invoke<RustSourceCompileDiagnostic[]>("check_rust_source", { source });
export const registerPortugolRsRun = () => invoke<number>("register_portugol_rs_run");
export const runPortugolRs = (id: number, source: string) =>
  invoke<PortugolRsRunResult>("run_portugol_rs", { id, source });

export interface RustSourceCompileDiagnostic {
  severity: string;
  code: string | null;
  message: string;
  rendered: string | null;
  start: Position;
  end: Position;
}

export type RustSourceRunResult =
  | { status: "executed"; stdout: string; stderr: string; exitCode: number | null }
  | { status: "compile_error"; diagnostics: RustSourceCompileDiagnostic[]; stderr: string }
  | { status: "compile_limit"; stderr: string; reason: "time" | "output" }
  | { status: "runtime_limit"; stdout: string; stderr: string; reason: "time" | "output" }
  | { status: "cancelled"; phase: "compilation" | "runtime"; stdout: string; stderr: string };

export const registerRustSourceRun = () => invoke<number>("register_rust_source_run");
export const runRustSource = (id: number, source: string, args: string[] = []) =>
  invoke<RustSourceRunResult>("run_rust_source", { id, source, args });

/** Executa Portugol-RS na VM educacional; não depende de `rustc`. */
export const startPortugolRsVm = (
  source: string,
  breakpoints: number[] = [],
  pauseOnStart = false,
  path: string | null = null,
) => invoke<number>("debug_portugol_rs", { source, breakpoints, pauseOnStart, path });

export const sendInput = (id: number, line: string) => invoke<void>("send_input", { id, line });

export const stop = (id: number) => invoke<boolean>("stop", { id });

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

export interface WorkspaceListing {
  id: string;
  name: string;
  preferenceKey: string;
  entries: WorkspaceEntry[];
}

export interface WorkspaceFile {
  path: string;
  relativePath: string;
  source: string;
}

export interface WorkspaceRenameResult {
  listing: WorkspaceListing;
  file: { path: string; relativePath: string };
}

export interface WorkspaceDeleteResult {
  listing: WorkspaceListing;
  file: { path: string; relativePath: string };
}

export interface WorkspaceExportResult {
  projectName: string;
  fileCount: number;
}

export const openWorkspace = () => invoke<WorkspaceListing | null>("workspace_open");
export const restoreWorkspace = () => invoke<WorkspaceListing | null>("workspace_restore");
export const readWorkspaceFile = (workspaceId: string, relativePath: string) =>
  invoke<WorkspaceFile>("workspace_read_file", { workspaceId, relativePath });
export const writeWorkspaceFile = (workspaceId: string, relativePath: string, source: string) =>
  invoke<void>("workspace_write_file", { workspaceId, relativePath, source });
export const createWorkspaceFile = (workspaceId: string, fileName: string) =>
  invoke<WorkspaceListing>("workspace_create_file", { workspaceId, fileName });
export const renameWorkspaceFile = (workspaceId: string, relativePath: string, newName: string) =>
  invoke<WorkspaceRenameResult>("workspace_rename_file", { workspaceId, relativePath, newName });
export const deleteWorkspaceFile = (workspaceId: string, relativePath: string) =>
  invoke<WorkspaceDeleteResult>("workspace_delete_file", { workspaceId, relativePath });
export const exportWorkspaceFiles = (workspaceId: string, relativePaths: string[]) =>
  invoke<WorkspaceExportResult | null>("workspace_export", { workspaceId, relativePaths });

export type CatalogEntry =
  | { kind: "folder"; name: string; entries: CatalogEntry[] }
  | { kind: "file"; name: string; file: string; description: string; preview: string | null };

export async function porsUiStart(source: string, budget?: PorsUiBudget): Promise<PorsUiResult> {
  const raw = await invoke<string>("pors_ui_start", {
    source,
    maxSteps: budget?.maxSteps ?? null,
    maxNodes: budget?.maxNodes ?? null,
    maxDepth: budget?.maxDepth ?? null,
  });
  return JSON.parse(raw) as PorsUiResult;
}

export async function porsUiDispatch(sessionId: number, event: UiEvent): Promise<PorsUiResult> {
  const raw = await invoke<string>("pors_ui_dispatch", {
    sessionId,
    kind: event.kind,
    targetId: event.id,
    value: event.kind === "entrada_alterada" ? event.value : "",
  });
  return JSON.parse(raw) as PorsUiResult;
}

export const porsUiCancel = (sessionId: number) => invoke<boolean>("pors_ui_cancel", { sessionId });
export const porsUiClose = (sessionId: number) => invoke<boolean>("pors_ui_close", { sessionId });

export interface PorsExampleInfo {
  id: string;
  category: string;
  title: string;
  file_name: string;
  description: string;
  is_visual: boolean;
  license: string;
  source: string;
}

export interface PorsExerciseSummary {
  schema_version: number;
  id: string;
  title: string;
  statement: string;
  starter_code: string;
  hints: string[];
  cases: number;
}

export type PorsExerciseStatus =
  | "passed"
  | "failed"
  | "compile_error"
  | "runtime_error"
  | "limit_exceeded";

export interface PorsExerciseCaseOutcome {
  name: string;
  passed: boolean;
  expected_output: string;
  actual_output: string;
  error: string | null;
}

export interface PorsExerciseEvaluation {
  schema_version: number;
  exercise_id: string;
  status: PorsExerciseStatus;
  total: number;
  passed: number;
  cases: PorsExerciseCaseOutcome[];
  diagnostics: string[];
  hints: string[];
}

export const listPorsExamples = () => invoke<PorsExampleInfo[]>("pors_example_catalog");
export const listPorsExercises = () => invoke<PorsExerciseSummary[]>("pors_exercise_catalog");
export const checkPorsExercise = (id: string | null, source: string, exerciseJson?: string) =>
  invoke<PorsExerciseEvaluation>("check_pors_exercise", {
    id,
    source,
    exerciseJson: exerciseJson ?? null,
  });
export const inspectPorsExercise = (rawJson: string) =>
  invoke<PorsExerciseSummary>("inspect_pors_exercise", { rawJson });

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
