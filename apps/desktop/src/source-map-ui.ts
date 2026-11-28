import type { DiagnosticInfo, FerroRustCompileDiagnostic, SourceMapInfo } from "./api";

export interface SourceMapLink {
  sourceLine: number;
  generatedLine: number;
  sourceOffset: number;
  generatedOffset: number;
  quality: "exact" | "approximate";
}

export function sourceMapLinks(mappings: SourceMapInfo[]): SourceMapLink[] {
  const byLines = new Map<string, SourceMapLink>();
  const candidates = mappings
    .filter((mapping) => mapping.sourceStart && mapping.sourceEnd && mapping.quality !== "generated_only")
    .map((mapping) => ({ mapping, from: mapping.generatedStart.offset, to: mapping.generatedEnd.offset }))
    .filter(({ mapping, from, to }) =>
      mapping.sourceStart!.line > 0 && mapping.generatedStart.line > 0
      && mapping.sourceStart!.offset >= 0 && from >= 0 && to > from,
    )
    .sort((left, right) => (left.to - left.from) - (right.to - right.from)
      || Number(left.mapping.quality !== "exact") - Number(right.mapping.quality !== "exact")
      || left.from - right.from);
  const selected: typeof candidates = [];
  for (const candidate of candidates) {
    if (selected.some((segment) => candidate.from < segment.to && candidate.to > segment.from)) continue;
    selected.push(candidate);
  }
  for (const { mapping } of selected) {
    if (mapping.quality === "generated_only") continue;
    const source = mapping.sourceStart;
    const generated = mapping.generatedStart;
    if (!source) continue;
    const link: SourceMapLink = {
      sourceLine: source.line,
      generatedLine: generated.line,
      sourceOffset: source.offset,
      generatedOffset: generated.offset,
      quality: mapping.quality,
    };
    const key = `${link.sourceLine}:${link.generatedLine}`;
    const current = byLines.get(key);
    if (!current || (current.quality === "approximate" && link.quality === "exact")) {
      byLines.set(key, link);
    }
  }
  return [...byLines.values()].sort((left, right) =>
    left.sourceLine - right.sourceLine || left.generatedLine - right.generatedLine,
  );
}

export function sourceDiagnostic(diagnostic: FerroRustCompileDiagnostic): DiagnosticInfo | null {
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
    .filter((mapping) => mapping.sourceStart && mapping.sourceEnd && mapping.quality !== "generated_only")
    .map((mapping) => ({
      from: Math.min(code.length, mapping.generatedStart.offset),
      to: Math.min(code.length, mapping.generatedEnd.offset),
      sourceOffset: mapping.sourceStart!.offset,
      sourceEnd: mapping.sourceEnd!.offset,
      sourceLine: mapping.sourceStart!.line,
      generatedLine: mapping.generatedStart.line,
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

export function activeGeneratedMappings<T extends { sourceOffset: number; sourceEnd?: number }>(
  mappings: readonly T[],
  sourceOffset: number,
): (T & { active: boolean })[] {
  return mappings.map((mapping) => ({
    ...mapping,
    active: mapping.sourceEnd !== undefined
      && mapping.sourceOffset <= sourceOffset && sourceOffset < mapping.sourceEnd,
  }));
}
