import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { checkExercise, inspectExercise, listExercises, readFile, type Exercise } from "./api";
import {
  createExerciseProgress,
  createValidatedExerciseGrader,
  exerciseIdentity,
  exerciseOptionLabel,
  exerciseResultSummary,
  exerciseRevisionStatus,
} from "./exercise-progress";

export interface ExerciseDocument {
  exercise: Exercise;
  name: string;
  contents?: string;
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

  async function grade({ exercise, name, contents }: ExerciseDocument, source: string) {
    const id = ++attempt;
    summary.textContent = `Corrigindo ${name}…`;
    revisionStatus.textContent = "Validando esta revisão do código…";
    failures.replaceChildren();
    hints.hidden = true;
    hintItems.replaceChildren();
    results.showModal();
    try {
      const identity = await exerciseIdentity(exercise.id, contents);
      const grade = createValidatedExerciseGrader(
        (gradedSource) => checkExercise(exercise.id, gradedSource, contents),
        progress,
        identity,
        () => id === attempt,
      );
      const { evaluation: result, result: progressResult } = await grade(source);
      if (id !== attempt) return;
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
    } catch (message) {
      if (id === attempt) {
        summary.textContent = typeof message === "string" ? message : "Não foi possível corrigir a solução.";
        revisionStatus.textContent = "A correção não terminou; o progresso não foi alterado.";
      }
    }
  }

  return { browse, grade };
}
