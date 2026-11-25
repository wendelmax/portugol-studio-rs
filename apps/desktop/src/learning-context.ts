import type { SourceMapInfo } from "./api";

export type MappedLearningConceptId =
  | "mutability"
  | "conditionals"
  | "loops"
  | "functions"
  | "option-result"
  | "collections"
  | "records-enums";

export interface LearningContext {
  conceptId: MappedLearningConceptId;
  provenance: "source-map";
  bindingMutability?: "mutable" | "immutable";
  sourceStart: number;
  sourceEnd: number;
  generatedStart: number;
  generatedEnd: number;
  sourceSnippet: string;
  rustSnippet: string;
}

export function explainMappedConcept(context: LearningContext, language: "portugol" | "portugol-rs"): string {
  if (context.conceptId === "conditionals") {
    return language === "portugol-rs"
      ? "Esta decisão Portugol-RS foi mapeada para uma condição if no Rust gerado."
      : "Esta decisão Portugol foi mapeada para uma condição if no Rust gerado.";
  }
  if (context.conceptId === "loops") {
    return "Esta repetição foi mapeada para a estrutura correspondente no Rust gerado.";
  }
  if (context.conceptId === "functions") {
    return "Esta função foi mapeada para uma declaração fn tipada no Rust gerado.";
  }
  if (context.conceptId === "option-result") {
    return "Esta expressão escolha/Opcao/Resultado foi mapeada para correspondência ou variante tipada (match/Option/Result) no Rust gerado.";
  }
  if (context.conceptId === "collections") {
    return "Esta coleção vetor<T> foi mapeada para um Vec<T> dinâmico no Rust gerado.";
  }
  if (context.conceptId === "records-enums") {
    return "Esta declaração de tipo composto foi mapeada para struct ou enum nominal no Rust gerado.";
  }
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
    const conceptId: LearningContext["conceptId"] | null = binding
      ? (/\bVec<|\bvec!\[|\bVec::new\(/.test(rustSnippet) ? "collections" : "mutability")
      : /^if\b/.test(rustSnippet)
        ? "conditionals"
        : /^while\b/.test(rustSnippet) || /^\{\n[\s\S]*?'pors_for_\d+: while\b/.test(rustSnippet)
          ? "loops"
          : /^match\b|\b(?:Option|Result)::(?:Some|None|Ok|Err)\b/.test(rustSnippet)
            ? "option-result"
            : /^(?:#\[[^\]]+\]\s*)*(?:pub\s+)?(?:struct|enum)\s+[A-Za-z_]\w*\b/.test(rustSnippet)
              ? "records-enums"
              : /^(?:pub\s+)?fn\s+[A-Za-z_]\w*\s*\(/.test(rustSnippet)
                ? "functions"
                : null;
    if (!conceptId) return [];
    return [{
      conceptId,
      sourceStart,
      sourceEnd,
      generatedStart,
      generatedEnd,
      sourceSnippet: source.slice(sourceStart, sourceEnd),
      rustSnippet,
      ...(conceptId === "mutability" && binding ? { bindingMutability: binding[1] ? "mutable" as const : "immutable" as const } : {}),
    }];
  });
  candidates.sort((a, b) => (a.sourceEnd - a.sourceStart) - (b.sourceEnd - b.sourceStart)
    || (a.generatedEnd - a.generatedStart) - (b.generatedEnd - b.generatedStart));
  const match = candidates[0];
  return match ? { provenance: "source-map", ...match } : null;
}
