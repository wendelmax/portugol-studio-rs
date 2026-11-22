import type { RustExport, SourceMapInfo } from "./api";

export interface RustPreviewArtifact {
  source: string;
  code: string;
  sourceMap: SourceMapInfo[];
}

export interface RustPreviewState {
  artifact?: RustPreviewArtifact;
  previousArtifact?: RustPreviewArtifact;
  pending: boolean;
  stale: boolean;
  error?: string;
}

export function generatedRustComparison(state: RustPreviewState | undefined): {
  previous: RustPreviewArtifact;
  current: RustPreviewArtifact;
} | null {
  const previous = state?.previousArtifact;
  const current = state?.artifact;
  if (!previous || !current || state.pending || state.stale || previous.code === current.code) return null;
  return { previous, current };
}

export function beginRustPreview(
  state: RustPreviewState | undefined,
  source: string,
): RustPreviewState {
  return {
    artifact: state?.artifact,
    ...(state?.previousArtifact ? { previousArtifact: state.previousArtifact } : {}),
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
    ...(state.previousArtifact ? { previousArtifact: state.previousArtifact } : {}),
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
      ...(state.previousArtifact ? { previousArtifact: state.previousArtifact } : {}),
      pending: false,
      stale: Boolean(state.artifact),
      error: diagnostic
        ? `Linha ${diagnostic.start.line}, coluna ${diagnostic.start.column}: ${diagnostic.message}`
        : "Não foi possível gerar Rust para este programa.",
    };
  }
  return {
    artifact: { source, code: result.code, sourceMap: result.sourceMap },
    ...(state.artifact && state.artifact.code !== result.code ? { previousArtifact: state.artifact } : {}),
    pending: false,
    stale: false,
    error: undefined,
  };
}

export function failRustPreview(state: RustPreviewState, message: string): RustPreviewState {
  return {
    artifact: state.artifact,
    ...(state.previousArtifact ? { previousArtifact: state.previousArtifact } : {}),
    pending: false,
    stale: Boolean(state.artifact),
    error: message,
  };
}
