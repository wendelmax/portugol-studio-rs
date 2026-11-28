import { describeExtensions, fileFormat, isRetiredFile } from "./file-formats.ts";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  checkFerroExercise,
  inspectFerroExercise,
  listFerroExercises,
  readFile,
  type Exercise,
  type ExerciseCheckResult,
  type FerroExerciseSummary,
} from "./api";
import { createFerroExerciseOpener, ferroExerciseDocument } from "./exercise-opening";
import {
  createExerciseProgress,
  createValidatedExerciseGrader,
  exerciseOptionLabel,
  exerciseResultSummary,
  exerciseRevisionStatus,
  type ExerciseProgressOutcome,
} from "./exercise-progress";
import {
  formatFerroExerciseOptionValue,
  formatFerroExerciseOutcome,
  parseFerroExerciseOptionValue,
} from "./ferro-catalog";

export interface ExerciseDocument {
  exercise: Exercise;
  name: string;
  contents?: string;
  ferro: FerroExerciseSummary;
}

export interface ExerciseGrade {
  evaluation: ExerciseCheckResult;
  progress: ExerciseProgressOutcome;
  current: boolean;
}

export async function readExercise(path: string): Promise<ExerciseDocument> {
  if (isRetiredFile(path)) {
    throw `Arquivos legados .pex não são suportados no Ferro Studio. Importe uma lição ${describeExtensions(["lesson"])}.`;
  }
  const contents = await readFile(path);
  const fileName = path.split(/[\\/]/).pop() ?? "Exercício";
  const summary = await inspectFerroExercise(contents);
  return ferroExerciseDocument(summary, summary.title || fileName, contents);
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
  let pending: Promise<FerroExerciseSummary[]> | null = null;
  let ferroCatalog: FerroExerciseSummary[] = [];
  let catalogLoaded = false;
  let modalAttempt = 0;
  let ferroAttempt = 0;
  const latestFerroByExercise = new Map<string, number>();
  const progress = createExerciseProgress();

  async function loadCatalog() {
    if (catalogLoaded) return ferroCatalog;
    pending ??= listFerroExercises();
    try {
      ferroCatalog = await pending;
      catalogLoaded = true;
      return ferroCatalog;
    } finally {
      pending = null;
    }
  }

  const openFerroById = createFerroExerciseOpener(async () => {
    await loadCatalog();
    return ferroCatalog;
  }, openSource);

  function showExercise() {
    const ferroId = parseFerroExerciseOptionValue(list.value);
    const ferroExercise = ferroId !== null ? ferroCatalog.find(({ id }) => id === ferroId) : undefined;
    statement.textContent = ferroExercise
      ? `${ferroExercise.statement}\n\nCasos de teste: ${ferroExercise.cases}`
      : "";
    solve.disabled = !ferroExercise;
  }

  function renderCatalog() {
    const nodes: Array<HTMLOptionElement | HTMLOptGroupElement> = [];
    if (ferroCatalog.length > 0) {
      const ferroGroup = document.createElement("optgroup");
      ferroGroup.label = "Lições Ferro (.licao)";
      for (const item of ferroCatalog) {
        ferroGroup.append(
          new Option(
            exerciseOptionLabel(item.title, progress.hasPassed(`ferro:${item.id}`)),
            formatFerroExerciseOptionValue(item.id),
          ),
        );
      }
      nodes.push(ferroGroup);
    }
    list.replaceChildren(...nodes);
    showExercise();
  }

  list.onchange = showExercise;
  solve.onclick = () => {
    const ferroId = parseFerroExerciseOptionValue(list.value);
    const ferroExercise = ferroId !== null ? ferroCatalog.find(({ id }) => id === ferroId) : undefined;
    if (!ferroExercise) return;
    chooser.close();
    openSource(ferroExerciseDocument(ferroExercise));
  };

  open.onclick = async () => {
    open.disabled = true;
    loadError.hidden = true;
    try {
      const path = await openDialog({
        filters: [
          { name: `${fileFormat("lesson").label} (${describeExtensions(["lesson"])})`, extensions: ["licao"] },
        ],
      });
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
    if (catalogLoaded) return renderCatalog();
    statement.textContent = "Carregando exercícios…";
    try {
      await loadCatalog();
      renderCatalog();
    } catch {
      statement.textContent = "Não foi possível carregar os exercícios.";
    }
  }

  async function evaluate(
    exerciseDocument: ExerciseDocument,
    source: string,
    isCurrent: () => boolean = () => true,
  ): Promise<ExerciseGrade> {
    const ferro = exerciseDocument.ferro;
    const scope = `ferro:${ferro.id}:${exerciseDocument.contents ?? "builtin"}`;
    const request = ++ferroAttempt;
    latestFerroByExercise.set(scope, request);
    try {
      const isRunCurrent = () => latestFerroByExercise.get(scope) === request && isCurrent();
      const grader = createValidatedExerciseGrader(
        async (gradedSource) =>
          formatFerroExerciseOutcome(
            await checkFerroExercise(ferro.id, gradedSource, exerciseDocument.contents),
          ),
        progress,
        `ferro:${ferro.id}`,
        isRunCurrent,
      );
      const { evaluation, result } = await grader(source);
      return { evaluation, progress: result, current: isCurrent() };
    } finally {
      if (latestFerroByExercise.get(scope) === request) latestFerroByExercise.delete(scope);
    }
  }

  async function grade(exerciseDocument: ExerciseDocument, source: string, isCurrent: () => boolean = () => true) {
    const { name } = exerciseDocument;
    const id = ++modalAttempt;
    summary.textContent = `Corrigindo ${name}…`;
    revisionStatus.textContent = "Validando esta revisão do código…";
    failures.replaceChildren();
    hints.hidden = true;
    hintItems.replaceChildren();
    results.showModal();
    try {
      const { evaluation: result, progress: progressResult, current } = await evaluate(
        exerciseDocument,
        source,
        () => id === modalAttempt && isCurrent(),
      );
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
      if (id === modalAttempt && isCurrent()) {
        summary.textContent = typeof message === "string" ? message : "Não foi possível corrigir a solução.";
        revisionStatus.textContent = "A correção não terminou; o progresso não foi alterado.";
      }
    }
  }

  return { browse, grade, evaluate, openFerroById };
}
