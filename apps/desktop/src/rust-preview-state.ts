import type { RustExport, SourceMapInfo } from "./api";

export interface RustPreviewArtifact {
  source: string;
  code: string;
  sourceMap: SourceMapInfo[];
}

export interface RustPreviewState {
  artifact?: RustPreviewArtifact;
  pending: boolean;
  stale: boolean;
  error?: string;
}

export function beginRustPreview(
  state: RustPreviewState | undefined,
  source: string,
): RustPreviewState {
  return {
    artifact: state?.artifact,
    pending: true,
    stale: Boolean(state?.artifact && (state.stale || state.artifact.source !== source)),
    error: undefined,
  };
}

export function markRustPreviewStale(
  state: RustPreviewState,
  source: string,
): RustPreviewState {
  return {
    artifact: state.artifact,
    pending: false,
    stale: Boolean(state.artifact && (state.stale || state.artifact.source !== source)),
    error: undefined,
  };
}

export function acceptRustPreview(
  state: RustPreviewState,
  source: string,
  result: RustExport,
): RustPreviewState {
  if (result.code === null || result.diagnostic !== null) {
    const diagnostic = result.diagnostic;
    return {
      artifact: state.artifact,
      pending: false,
      stale: Boolean(state.artifact),
      error: diagnostic
        ? `Linha ${diagnostic.start.line}, coluna ${diagnostic.start.column}: ${diagnostic.message}`
        : "Não foi possível gerar Rust para este programa.",
    };
  }
  return {
    artifact: { source, code: result.code, sourceMap: result.sourceMap },
    pending: false,
    stale: false,
    error: undefined,
  };
}

export function failRustPreview(state: RustPreviewState, message: string): RustPreviewState {
  return {
    artifact: state.artifact,
    pending: false,
    stale: Boolean(state.artifact),
    error: message,
  };
}
