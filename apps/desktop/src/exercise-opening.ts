import type { PorsExerciseSummary } from "./api";
import type { ExerciseDocument } from "./exercises";

export type ExerciseDocumentOpener = (document: ExerciseDocument) => void;

export function porsExerciseDocument(summary: PorsExerciseSummary, name = summary.title, contents?: string): ExerciseDocument {
  return {
    exercise: { id: 0, statement: summary.statement, cases: summary.cases },
    name,
    contents,
    pors: summary,
  };
}

export function createPorsExerciseOpener(
  load: () => Promise<readonly PorsExerciseSummary[]>,
  open: ExerciseDocumentOpener,
) {
  let catalog: readonly PorsExerciseSummary[] | undefined;
  let pending: Promise<readonly PorsExerciseSummary[]> | null = null;

  return async (id: string): Promise<PorsExerciseSummary> => {
    if (!id.trim()) throw new Error("Identificador de exercício Ferro inválido.");
    if (!catalog) {
      pending ??= load()
        .then((loaded) => {
          catalog = loaded;
          return loaded;
        })
        .finally(() => { pending = null; });
      await pending;
    }

    const exercise = catalog?.find((item) => item.id === id);
    if (!exercise) throw new Error(`Exercício Ferro ${id} não encontrado no catálogo.`);
    open(porsExerciseDocument(exercise));
    return exercise;
  };
}
