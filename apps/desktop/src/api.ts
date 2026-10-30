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

export const check = (source: string) => invoke<DiagnosticInfo[]>("check", { source });

/** Id da execução; rejeita com a lista de erros se o programa não compila. */
export const run = (source: string, args: string[], path: string | null) =>
  invoke<number>("run", { source, args, path });

export const sendInput = (id: number, line: string) => invoke<void>("send_input", { id, line });

export const stop = (id: number) => invoke<void>("stop", { id });

export const readFile = (path: string) => invoke<string>("read_file", { path });

export const writeFile = (path: string, text: string) => invoke<void>("write_file", { path, text });

export interface RunEvents {
  output(id: number, text: string): void;
  clear(id: number): void;
  input(id: number, expected: string): void;
  finished(id: number, outcome: RunOutcome): void;
}

export async function listenRun(handlers: RunEvents): Promise<UnlistenFn> {
  const unlisten = await Promise.all([
    listen<{ id: number; text: string }>("run:output", (e) => handlers.output(e.payload.id, e.payload.text)),
    listen<{ id: number }>("run:clear", (e) => handlers.clear(e.payload.id)),
    listen<{ id: number; expected: string }>("run:input", (e) => handlers.input(e.payload.id, e.payload.expected)),
    listen<{ id: number; outcome: RunOutcome }>("run:finished", (e) => handlers.finished(e.payload.id, e.payload.outcome)),
  ]);
  return () => unlisten.forEach((u) => u());
}
