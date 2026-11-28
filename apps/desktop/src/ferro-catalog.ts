// Adaptadores puros para integrar exemplos e exercícios Ferro (.fer)
// à tela inicial e ao avaliador da IDE desktop.

import type {
  CatalogEntry,
  ExerciseCheckResult,
  FerroExampleInfo,
  FerroExerciseEvaluation,
} from "./api";

export const FERRO_EXAMPLE_PREFIX = "ferro-example:";
export const FERRO_EXERCISE_OPTION_PREFIX = "ferro:";

/** Constrói a árvore de catálogo Ferro agrupando categorias aninhadas por ` / `. */
export function buildFerroExampleCatalog(
  examples: readonly FerroExampleInfo[],
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
    for (const segment of segments.length > 0 ? segments : ["Ferro"]) {
      target = findOrCreateFolder(target, segment).entries;
    }
    target.push({
      kind: "file",
      name: example.title,
      file: `${FERRO_EXAMPLE_PREFIX}${example.id}`,
      description: example.description,
      preview: null,
    });
  }

  return rootEntries;
}

/** Constrói a pasta de catálogo `Ferro (.fer)` agrupada por categoria. */
export function buildFerroExampleCatalogFolder(
  examples: readonly FerroExampleInfo[],
): CatalogEntry | null {
  const folders = buildFerroExampleCatalog(examples);
  if (folders.length === 0) return null;

  return {
    kind: "folder",
    name: "Ferro (.fer)",
    entries: folders,
  };
}

export function formatFerroExerciseOptionValue(id: string): string {
  return `${FERRO_EXERCISE_OPTION_PREFIX}${id}`;
}

export function parseFerroExerciseOptionValue(value: string): string | null {
  return value.startsWith(FERRO_EXERCISE_OPTION_PREFIX)
    ? value.slice(FERRO_EXERCISE_OPTION_PREFIX.length)
    : null;
}

/** Converte o resultado detalhado do avaliador `.fer` para o contrato do painel de exercícios. */
export function formatFerroExerciseOutcome(
  evaluation: FerroExerciseEvaluation,
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

/** Prefixo dos projetos desktop de exemplo na lista de exemplos. */
export const DESKTOP_EXAMPLE_PREFIX = "desktop:";

/** Pasta "Projetos desktop": cada exemplo cria um projeto com tela e código. */
export function buildDesktopExampleFolder(examples: readonly { id: string; name: string; description: string }[]): CatalogEntry | null {
  if (examples.length === 0) return null;
  return {
    kind: "folder",
    name: "Projetos desktop",
    entries: examples.map((example) => ({
      kind: "file",
      name: example.name,
      file: `${DESKTOP_EXAMPLE_PREFIX}${example.id}`,
      description: `${example.description} Cria um projeto desktop com a tela e o código.`,
      preview: null,
    })),
  };
}
