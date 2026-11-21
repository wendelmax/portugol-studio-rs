import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { checkExercise, inspectExercise, listExercises, readFile, type Exercise } from "./api";

export interface ExerciseDocument {
  exercise: Exercise;
  name: string;
  contents?: string;
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
  const failures = document.querySelector<HTMLUListElement>("#exercise-failures")!;
  let pending: Promise<Exercise[]> | null = null;
  let catalog: Exercise[] = [];
  let attempt = 0;

  function showExercise() {
    const exercise = catalog.find(({ id }) => id === Number(list.value));
    statement.textContent = exercise ? `${exercise.statement}\n\nCasos de teste: ${exercise.cases}` : "";
    solve.disabled = !exercise;
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
      const contents = await readFile(path);
      const exercise = await inspectExercise(contents);
      if (!chooser.open) return;
      chooser.close();
      openSource({ exercise, name: path.split(/[\\/]/).pop() ?? "Exercício", contents });
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
    if (catalog.length) return;
    statement.textContent = "Carregando exercícios…";
    try {
      catalog = await (pending ??= listExercises());
      list.replaceChildren(...catalog.map(({ id }) => new Option(`Exercício ${id}`, String(id))));
      showExercise();
    } catch {
      pending = null;
      statement.textContent = "Não foi possível carregar os exercícios.";
    }
  }

  async function grade({ exercise, name, contents }: ExerciseDocument, source: string) {
    const id = ++attempt;
    summary.textContent = `Corrigindo ${name}…`;
    failures.replaceChildren();
    results.showModal();
    try {
      const result = await checkExercise(exercise.id, source, contents);
      if (id !== attempt) return;
      summary.textContent = `${result.passed} de ${result.total} casos aprovados.`;
      for (const failure of result.failures) {
        const item = document.createElement("li");
        item.textContent = failure;
        failures.append(item);
      }
    } catch (message) {
      if (id === attempt) {
        summary.textContent = typeof message === "string" ? message : "Não foi possível corrigir a solução.";
      }
    }
  }

  return { browse, grade };
}
