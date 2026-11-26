import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  checkPorsExercise,
  inspectPorsExercise,
  listPorsExercises,
  readFile,
  type Exercise,
  type ExerciseCheckResult,
  type PorsExerciseSummary,
} from "./api";
import { createPorsExerciseOpener, porsExerciseDocument } from "./exercise-opening";
import {
  createExerciseProgress,
  createValidatedExerciseGrader,
  exerciseOptionLabel,
  exerciseResultSummary,
  exerciseRevisionStatus,
  type ExerciseProgressOutcome,
} from "./exercise-progress";
import {
  formatPorsExerciseOptionValue,
  formatPorsExerciseOutcome,
  parsePorsExerciseOptionValue,
} from "./pors-catalog";

export interface ExerciseDocument {
  exercise: Exercise;
  name: string;
  contents?: string;
  pors: PorsExerciseSummary;
}

export interface ExerciseGrade {
  evaluation: ExerciseCheckResult;
  progress: ExerciseProgressOutcome;
  current: boolean;
}

export async function readExercise(path: string): Promise<ExerciseDocument> {
  if (/\.pex$/i.test(path)) {
    throw "Arquivos legados .pex não são suportados no Portugol Studio RS. Use o Portugol Studio original para abri-los ou importe um exercício .pors.json.";
  }
  const contents = await readFile(path);
  const fileName = path.split(/[\\/]/).pop() ?? "Exercício";
  const summary = await inspectPorsExercise(contents);
  return porsExerciseDocument(summary, summary.title || fileName, contents);
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
  let pending: Promise<PorsExerciseSummary[]> | null = null;
  let porsCatalog: PorsExerciseSummary[] = [];
  let catalogLoaded = false;
  let modalAttempt = 0;
  let porsAttempt = 0;
  const latestPorsByExercise = new Map<string, number>();
  const progress = createExerciseProgress();

  async function loadCatalog() {
    if (catalogLoaded) return porsCatalog;
    pending ??= listPorsExercises();
    try {
      porsCatalog = await pending;
      catalogLoaded = true;
      return porsCatalog;
    } finally {
      pending = null;
    }
  }

  const openPorsById = createPorsExerciseOpener(async () => {
    await loadCatalog();
    return porsCatalog;
  }, openSource);

  function showExercise() {
    const porsId = parsePorsExerciseOptionValue(list.value);
    const porsExercise = porsId !== null ? porsCatalog.find(({ id }) => id === porsId) : undefined;
    statement.textContent = porsExercise
      ? `${porsExercise.statement}\n\nCasos de teste (.pors): ${porsExercise.cases}`
      : "";
    solve.disabled = !porsExercise;
  }

  function renderCatalog() {
    const nodes: Array<HTMLOptionElement | HTMLOptGroupElement> = [];
    if (porsCatalog.length > 0) {
      const porsGroup = document.createElement("optgroup");
      porsGroup.label = "Portugol-RS (.pors)";
      for (const item of porsCatalog) {
        porsGroup.append(
          new Option(
            exerciseOptionLabel(item.title, progress.hasPassed(`pors:${item.id}`)),
            formatPorsExerciseOptionValue(item.id),
          ),
        );
      }
      nodes.push(porsGroup);
    }
    list.replaceChildren(...nodes);
    showExercise();
  }

  list.onchange = showExercise;
  solve.onclick = () => {
    const porsId = parsePorsExerciseOptionValue(list.value);
    const porsExercise = porsId !== null ? porsCatalog.find(({ id }) => id === porsId) : undefined;
    if (!porsExercise) return;
    chooser.close();
    openSource(porsExerciseDocument(porsExercise));
  };

  open.onclick = async () => {
    open.disabled = true;
    loadError.hidden = true;
    try {
      const path = await openDialog({
        filters: [
          { name: "Exercício Portugol-RS (.pors.json)", extensions: ["json"] },
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
    const pors = exerciseDocument.pors;
    const scope = `pors:${pors.id}:${exerciseDocument.contents ?? "builtin"}`;
    const request = ++porsAttempt;
    latestPorsByExercise.set(scope, request);
    try {
      const isRunCurrent = () => latestPorsByExercise.get(scope) === request && isCurrent();
      const grader = createValidatedExerciseGrader(
        async (gradedSource) =>
          formatPorsExerciseOutcome(
            await checkPorsExercise(pors.id, gradedSource, exerciseDocument.contents),
          ),
        progress,
        `pors:${pors.id}`,
        isRunCurrent,
      );
      const { evaluation, result } = await grader(source);
      return { evaluation, progress: result, current: isCurrent() };
    } finally {
      if (latestPorsByExercise.get(scope) === request) latestPorsByExercise.delete(scope);
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

  return { browse, grade, evaluate, openPorsById };
}
