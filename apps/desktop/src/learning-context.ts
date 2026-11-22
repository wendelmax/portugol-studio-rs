import type { SourceMapInfo } from "./api";

export interface LearningContext {
  conceptId: "mutability";
  provenance: "source-map";
  bindingMutability: "mutable" | "immutable";
  sourceStart: number;
  sourceEnd: number;
  generatedStart: number;
  generatedEnd: number;
  sourceSnippet: string;
  rustSnippet: string;
}

export function explainMappedMutability(context: LearningContext, language: "portugol" | "portugol-rs"): string {
  const rustBinding = context.bindingMutability === "mutable" ? "let mut" : "let";
  const rustEffect = context.bindingMutability === "mutable"
    ? "este vínculo pode ser alterado"
    : "este vínculo não pode ser reatribuído";
  return language === "portugol-rs"
    ? `No Portugol-RS, a declaração expressa a mutabilidade. O Rust gerado usa ${rustBinding}: ${rustEffect}.`
    : `No Portugol clássico, a reatribuição não exige uma anotação de mutabilidade. O Rust gerado usa ${rustBinding}: ${rustEffect}.`;
}

export function learningContextAtSelection(input: {
  source: string;
  selectionFrom: number;
  selectionTo: number;
  artifact: { source: string; code: string | null; sourceMap: SourceMapInfo[] } | null | undefined;
  stale: boolean;
}): LearningContext | null {
  const { source, selectionFrom, selectionTo, artifact, stale } = input;
  if (!artifact?.code || stale || artifact.source !== source) return null;
  if (selectionFrom < 0 || selectionTo < selectionFrom || selectionTo > source.length) return null;

  const candidates = artifact.sourceMap.flatMap((mapping) => {
    const sourceStart = mapping.sourceStart?.offset;
    const sourceEnd = mapping.sourceEnd?.offset;
    const generatedStart = mapping.generatedStart.offset;
    const generatedEnd = mapping.generatedEnd.offset;
    if (mapping.quality !== "exact" || sourceStart === undefined || sourceEnd === undefined) return [];
    const containsSelection = selectionFrom === selectionTo
      ? selectionFrom >= sourceStart && selectionFrom < sourceEnd
      : selectionFrom >= sourceStart && selectionTo <= sourceEnd;
    if (!containsSelection || sourceStart < 0 || sourceEnd > source.length
      || generatedStart < 0 || generatedEnd > artifact.code!.length || generatedEnd < generatedStart) return [];

    const rustSnippet = artifact.code!.slice(generatedStart, generatedEnd).trim();
    const binding = /^let\s+(mut\s+)?[A-Za-z_]\w*\b/.exec(rustSnippet);
    if (!binding) return [];
    return [{
      sourceStart,
      sourceEnd,
      generatedStart,
      generatedEnd,
      sourceSnippet: source.slice(sourceStart, sourceEnd),
      rustSnippet,
      bindingMutability: binding[1] ? "mutable" as const : "immutable" as const,
    }];
  });
  candidates.sort((a, b) => (a.sourceEnd - a.sourceStart) - (b.sourceEnd - b.sourceStart)
    || (a.generatedEnd - a.generatedStart) - (b.generatedEnd - b.generatedStart));
  const match = candidates[0];
  return match ? { conceptId: "mutability", provenance: "source-map", ...match } : null;
}
