// Adaptadores puros para integrar exemplos e exercícios Portugol-RS (.pors)
// à tela inicial e ao avaliador da IDE desktop, preservando .por/.pex.

import type {
  CatalogEntry,
  ExerciseCheckResult,
  PorsExampleInfo,
  PorsExerciseEvaluation,
} from "./api";

export const PORS_EXAMPLE_PREFIX = "pors-example:";
export const PORS_EXERCISE_OPTION_PREFIX = "pors:";

/** Constrói a pasta de catálogo `Portugol-RS (.pors)` agrupada por categoria. */
export function buildPorsExampleCatalogFolder(
  examples: readonly PorsExampleInfo[],
): CatalogEntry | null {
  if (examples.length === 0) return null;

  const byCategory = new Map<string, CatalogEntry[]>();
  for (const example of examples) {
    const entries = byCategory.get(example.category) ?? [];
    entries.push({
      kind: "file",
      name: example.title,
      file: `${PORS_EXAMPLE_PREFIX}${example.id}`,
      description: example.description,
      preview: null,
    });
    byCategory.set(example.category, entries);
  }

  const folders: CatalogEntry[] = [];
  for (const [category, entries] of byCategory) {
    folders.push({
      kind: "folder",
      name: category,
      entries,
    });
  }

  return {
    kind: "folder",
    name: "Portugol-RS (.pors)",
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

/** Converte o resultado detalhado do avaliador `.pors` para o contrato do painel de exercícios. */
export function formatPorsExerciseOutcome(
  evaluation: PorsExerciseEvaluation,
): ExerciseCheckResult {
  const failures: string[] = [];

  if (evaluation.status === "compile_error") {
    if (evaluation.diagnostics.length > 0) {
      failures.push(...evaluation.diagnostics);
    } else {
      failures.push("O código .pors possui erros de compilação.");
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
