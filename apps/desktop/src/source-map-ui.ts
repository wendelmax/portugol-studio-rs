import type { DiagnosticInfo, PortugolRsCompileDiagnostic, SourceMapInfo } from "./api";

export function sourceDiagnostic(diagnostic: PortugolRsCompileDiagnostic): DiagnosticInfo | null {
  if (!diagnostic.sourceStart || !diagnostic.sourceEnd) return null;
  const approximate = diagnostic.quality === "approximate";
  return {
    severity: diagnostic.severity === "warning" ? "warning" : "error",
    code: diagnostic.code ?? "rustc",
    message: approximate ? `${diagnostic.message} (localização aproximada)` : diagnostic.message,
    help: diagnostic.rendered,
    start: diagnostic.sourceStart,
    end: diagnostic.sourceEnd,
  };
}

export function mappedPreviewSegments(code: string, mappings: SourceMapInfo[]) {
  const candidates = mappings
    .filter((mapping) => mapping.sourceStart && mapping.sourceEnd)
    .map((mapping) => ({
      from: Math.min(code.length, mapping.generatedStart.offset),
      to: Math.min(code.length, mapping.generatedEnd.offset),
      sourceOffset: mapping.sourceStart!.offset,
      sourceLine: mapping.sourceStart!.line,
      quality: mapping.quality,
    }))
    .filter(({ from, to }) => to > from)
    .sort((a, b) =>
      (a.to - a.from) - (b.to - b.from)
      || Number(a.quality !== "exact") - Number(b.quality !== "exact")
      || a.from - b.from,
    );
  const selected: typeof candidates = [];
  for (const candidate of candidates) {
    if (selected.some((segment) => candidate.from < segment.to && candidate.to > segment.from)) continue;
    selected.push(candidate);
  }
  return selected.sort((a, b) => a.from - b.from);
}
