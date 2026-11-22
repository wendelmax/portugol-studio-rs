import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { checkExercise, inspectExercise, listExercises, readFile, type Exercise, type ExerciseCheckResult } from "./api";
import {
  createExerciseProgress,
  createValidatedExerciseGrader,
  exerciseIdentity,
  exerciseOptionLabel,
  exerciseResultSummary,
  exerciseRevisionStatus,
  type ExerciseProgressOutcome,
} from "./exercise-progress";

export interface ExerciseDocument {
  exercise: Exercise;
  name: string;
  contents?: string;
}

export interface ExerciseGrade {
  evaluation: ExerciseCheckResult;
  progress: ExerciseProgressOutcome;
  current: boolean;
}

export async function readExercise(path: string): Promise<ExerciseDocument> {
  const contents = await readFile(path);
  const exercise = await inspectExercise(contents);
  return { exercise, name: path.split(/[\\/]/).pop() ?? "Exercício", contents };
}

export function mountExercises(openSource: (document: ExerciseDocument) => void) {
  const chooser = document.querySelector<HTMLDialogElement>("#exercises")!;
  const list = document.querySelector<HTMLSelectElement>("#exercise-list")!;
  const statement = document.querySelector<HTMLElement>("#exercise-statement")!;
  const solve = document.querySelector<HTMLButtonElement>("#exercise-solve")!;
  const open = document.querySelector<HTMLButtonElement>("#exercise-import")!;
  const loadError = document.querySelector<HTMLElement>("#exercise-error")!;
  const results = document.querySelector<HTMLDialogElement>("#exercise-results")!;
  const summary = document.querySelector<HTMLElement>("#exercise-summary")!;
  const revisionStatus = document.querySelector<HTMLElement>("#exercise-revision")!;
  const failures = document.querySelector<HTMLUListElement>("#exercise-failures")!;
  const hints = document.querySelector<HTMLElement>("#exercise-hints")!;
  const hintItems = hints.querySelector("ul")!;
  let pending: Promise<Exercise[]> | null = null;
  let catalog: Exercise[] = [];
  let attempt = 0;
  const progress = createExerciseProgress();

  function showExercise() {
    const exercise = catalog.find(({ id }) => id === Number(list.value));
    statement.textContent = exercise ? `${exercise.statement}\n\nCasos de teste: ${exercise.cases}` : "";
    solve.disabled = !exercise;
  }

  function renderCatalog() {
    list.replaceChildren(...catalog.map(({ id }) => new Option(
      exerciseOptionLabel(`Exercício ${id}`, progress.hasPassed(`builtin:${id}`)),
      String(id),
    )));
    showExercise();
  }

  list.onchange = showExercise;
  solve.onclick = () => {
    const exercise = catalog.find(({ id }) => id === Number(list.value));
    if (!exercise) return;
    chooser.close();
    openSource({ exercise, name: `Exercício ${exercise.id}` });
  };

  open.onclick = async () => {
    open.disabled = true;
    loadError.hidden = true;
    try {
      const path = await openDialog({ filters: [{ name: "Exercício do Portugol", extensions: ["pex"] }] });
      if (typeof path !== "string" || !chooser.open) return;
      const document = await readExercise(path);
      if (!chooser.open) return;
      chooser.close();
      openSource(document);
    } catch (message) {
      loadError.textContent = typeof message === "string" ? message : "Não foi possível abrir o exercício.";
      loadError.hidden = false;
    } finally {
      open.disabled = false;
    }
  };

  async function browse() {
    loadError.hidden = true;
    chooser.showModal();
    if (catalog.length) {
      renderCatalog();
      return;
    }
    statement.textContent = "Carregando exercícios…";
    try {
      catalog = await (pending ??= listExercises());
      renderCatalog();
    } catch {
      pending = null;
      statement.textContent = "Não foi possível carregar os exercícios.";
    }
  }

  async function runEvaluation(
    { exercise, contents }: ExerciseDocument,
    source: string,
    id: number,
    isCurrent: () => boolean,
  ): Promise<ExerciseGrade> {
    const identity = await exerciseIdentity(exercise.id, contents);
    const isRunCurrent = () => id === attempt && isCurrent();
    const grader = createValidatedExerciseGrader(
      (gradedSource) => checkExercise(exercise.id, gradedSource, contents),
      progress,
      identity,
      isRunCurrent,
    );
    const { evaluation, result } = await grader(source);
    return { evaluation, progress: result, current: isRunCurrent() };
  }

  async function evaluate(
    exerciseDocument: ExerciseDocument,
    source: string,
    isCurrent: () => boolean = () => true,
  ): Promise<ExerciseGrade> {
    const id = ++attempt;
    return runEvaluation(exerciseDocument, source, id, isCurrent);
  }

  async function grade(exerciseDocument: ExerciseDocument, source: string) {
    const { name } = exerciseDocument;
    const id = ++attempt;
    summary.textContent = `Corrigindo ${name}…`;
    revisionStatus.textContent = "Validando esta revisão do código…";
    failures.replaceChildren();
    hints.hidden = true;
    hintItems.replaceChildren();
    results.showModal();
    try {
      const { evaluation: result, progress: progressResult, current } = await runEvaluation(exerciseDocument, source, id, () => true);
      if (!current) return;
      summary.textContent = exerciseResultSummary(result, progressResult);
      revisionStatus.textContent = exerciseRevisionStatus(progressResult);
      for (const failure of result.failures) {
        const item = document.createElement("li");
        item.textContent = failure;
        failures.append(item);
      }
      for (const hint of result.hints) {
        const item = document.createElement("li");
        item.textContent = hint;
        hintItems.append(item);
      }
      hints.hidden = !result.hints.length;
      return { evaluation: result, progress: progressResult, current };
    } catch (message) {
      if (id === attempt) {
        summary.textContent = typeof message === "string" ? message : "Não foi possível corrigir a solução.";
        revisionStatus.textContent = "A correção não terminou; o progresso não foi alterado.";
      }
    }
  }

  return { browse, grade, evaluate };
}
