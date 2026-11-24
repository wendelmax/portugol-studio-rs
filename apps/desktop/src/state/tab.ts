// Uma aba aberta no editor.

import type { EditorState } from "@codemirror/state";
import type { CargoRunTarget, DiagnosticInfo, SymbolInfo } from "../api";
import type { SourceLanguage } from "../document-language";
import type { ExerciseDocument } from "../exercises";
import type { CargoTestRun, ExerciseTestRun } from "../exercise-test-panel";
import type { LearningPins } from "../learning-pins";
import type { RustPreviewState } from "../rust-preview-state";

export interface Tab {
  id: number;
  path: string | null;
  language: SourceLanguage;
  exampleName?: string;
  examplePath?: string;
  exercise?: ExerciseDocument;
  exerciseTestRun?: ExerciseTestRun;
  exerciseTestGeneration?: number;
  cargoTestRun?: CargoTestRun;
  workspaceId?: string;
  workspacePath?: string;
  workspacePreferenceKey?: string;
  cargoProject?: boolean;
  cargoTarget?: CargoRunTarget;
  workspaceExercisePath?: string;
  state: EditorState;
  rustPreview?: RustPreviewState;
  learningPins?: LearningPins;
  showRustDiff?: boolean;
  /** Texto da última gravação, para saber se há alterações. */
  saved: string;
  diagnostics: DiagnosticInfo[];
  runtimeCompileDiagnostics?: DiagnosticInfo[];
  symbols: SymbolInfo[];
}

/** Arquivo de um workspace aberto, associado a uma aba. */
export interface WorkspaceFileRef {
  id: string;
  path: string;
  preferenceKey: string;
}
