// Adaptadores puros para integrar exemplos e exercícios Portugol-RS (.fer)
// à tela inicial e ao avaliador da IDE desktop.

import type {
  CatalogEntry,
  ExerciseCheckResult,
  PorsExampleInfo,
  PorsExerciseEvaluation,
} from "./api";

export const PORS_EXAMPLE_PREFIX = "pors-example:";
export const PORS_EXERCISE_OPTION_PREFIX = "pors:";

/** Constrói a árvore de catálogo Portugol-RS agrupando categorias aninhadas por ` / `. */
export function buildPorsExampleCatalog(
  examples: readonly PorsExampleInfo[],
): CatalogEntry[] {
  if (examples.length === 0) return [];

  const rootEntries: CatalogEntry[] = [];

  function findOrCreateFolder(entries: CatalogEntry[], name: string): Extract<CatalogEntry, { kind: "folder" }> {
    const existing = entries.find(
      (entry): entry is Extract<CatalogEntry, { kind: "folder" }> => entry.kind === "folder" && entry.name === name,
    );
    if (existing) return existing;
    const folder: Extract<CatalogEntry, { kind: "folder" }> = {
      kind: "folder",
      name,
      entries: [],
    };
    entries.push(folder);
    return folder;
  }

  for (const example of examples) {
    const segments = example.category
      .split(" / ")
      .map((segment) => segment.trim())
      .filter(Boolean);
    let target = rootEntries;
    for (const segment of segments.length > 0 ? segments : ["Portugol-RS"]) {
      target = findOrCreateFolder(target, segment).entries;
    }
    target.push({
      kind: "file",
      name: example.title,
      file: `${PORS_EXAMPLE_PREFIX}${example.id}`,
      description: example.description,
      preview: null,
    });
  }

  return rootEntries;
}

/** Constrói a pasta de catálogo `Portugol-RS (.fer)` agrupada por categoria. */
export function buildPorsExampleCatalogFolder(
  examples: readonly PorsExampleInfo[],
): CatalogEntry | null {
  const folders = buildPorsExampleCatalog(examples);
  if (folders.length === 0) return null;

  return {
    kind: "folder",
    name: "Portugol-RS (.fer)",
    entries: folders,
  };
}

export function formatPorsExerciseOptionValue(id: string): string {
  return `${PORS_EXERCISE_OPTION_PREFIX}${id}`;
}

export function parsePorsExerciseOptionValue(value: string): string | null {
  return value.startsWith(PORS_EXERCISE_OPTION_PREFIX)
    ? value.slice(PORS_EXERCISE_OPTION_PREFIX.length)
    : null;
}

/** Converte o resultado detalhado do avaliador `.fer` para o contrato do painel de exercícios. */
export function formatPorsExerciseOutcome(
  evaluation: PorsExerciseEvaluation,
): ExerciseCheckResult {
  const failures: string[] = [];

  if (evaluation.status === "compile_error") {
    if (evaluation.diagnostics.length > 0) {
      failures.push(...evaluation.diagnostics);
    } else {
      failures.push("O código .fer possui erros de compilação.");
    }
  } else {
    for (const item of evaluation.cases) {
      if (item.passed) continue;
      if (item.error) {
        failures.push(`Caso "${item.name}": ${item.error}`);
      } else {
        failures.push(
          `Caso "${item.name}": esperado "${item.expected_output}", obtido "${item.actual_output}".`,
        );
      }
    }
  }

  return {
    total: evaluation.total,
    passed: evaluation.passed,
    failures,
    hints: evaluation.hints,
  };
}
