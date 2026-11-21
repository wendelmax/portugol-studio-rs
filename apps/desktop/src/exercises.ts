import { checkExercise, listExercises, type Exercise } from "./api";

export function mountExercises(openSource: (exercise: Exercise) => void) {
  const chooser = document.querySelector<HTMLDialogElement>("#exercises")!;
  const list = document.querySelector<HTMLSelectElement>("#exercise-list")!;
  const statement = document.querySelector<HTMLElement>("#exercise-statement")!;
  const solve = document.querySelector<HTMLButtonElement>("#exercise-solve")!;
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
    openSource(exercise);
  };

  async function browse() {
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

  async function grade(exercise: Exercise, source: string) {
    const id = ++attempt;
    summary.textContent = `Corrigindo exercício ${exercise.id}…`;
    failures.replaceChildren();
    results.showModal();
    try {
      const result = await checkExercise(exercise.id, source);
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
