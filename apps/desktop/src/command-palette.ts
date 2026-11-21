export interface CommandSearchItem {
  command: string;
  label: string;
}

export function isDebugCommandAvailable(
  command: string,
  language: string | null,
  runActive: boolean,
  paused: boolean,
): boolean {
  if (language !== "portugol") return false;
  if (command === "Depurar") return !runActive || paused;
  if (command === "Interromper") return runActive;
  if (command === "Pausar") return runActive && !paused;
  if (["Continuar", "Próxima instrução", "Entrar na função", "Sair da função"].includes(command)) {
    return runActive && paused;
  }
  return true;
}

function normalize(value: string): string {
  return value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase("pt-BR");
}

export function filterCommands<T extends CommandSearchItem>(items: readonly T[], query: string): T[] {
  const terms = normalize(query).trim().split(/\s+/).filter(Boolean);
  if (terms.length === 0) return [...items];
  return items.filter(({ command, label }) => {
    const text = normalize(`${command} ${label}`);
    return terms.every((term) => text.includes(term));
  });
}

export function nextCommandIndex(count: number, current: number, direction: "ArrowUp" | "ArrowDown"): number {
  if (count <= 0) return -1;
  if (current < 0) return direction === "ArrowDown" ? 0 : count - 1;
  const delta = direction === "ArrowDown" ? 1 : -1;
  return (current + delta + count) % count;
}
