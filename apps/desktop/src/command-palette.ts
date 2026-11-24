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

/** Comandos que só fazem sentido com uma aba de código aberta. */
export const TAB_COMMANDS: ReadonlySet<string> = new Set([
  "Executar", "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução",
  "Entrar na função", "Sair da função", "Renomear símbolo", "Formatar código-fonte", "Analisar projeto Cargo", "Testar projeto Cargo",
  "Executar como Rust",
  "Salvar", "Salvar como", "Parâmetros de execução", "Fechar aba", "Fechar todas as abas", "Localizar e substituir",
  "Buscar na estrutura", "Filtrar variável", "Filtrar vetor", "Filtrar matriz", "Filtrar função",
  "Filtrar constante", "Mostrar todos os tipos", "Filtrar tipo inteiro", "Filtrar tipo caracter",
  "Filtrar tipo real", "Filtrar tipo cadeia", "Filtrar tipo logico", "Filtrar tipo vazio",
]);

/** Comandos sem suporte em arquivos Rust. */
export const RUST_UNSUPPORTED_COMMANDS: ReadonlySet<string> = new Set([
  "Executar", "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução",
  "Entrar na função", "Sair da função", "Renomear símbolo", "Formatar código-fonte",
  "Buscar na estrutura", "Filtrar variável", "Filtrar vetor", "Filtrar matriz", "Filtrar função",
  "Filtrar constante", "Mostrar todos os tipos", "Filtrar tipo inteiro", "Filtrar tipo caracter",
  "Filtrar tipo real", "Filtrar tipo cadeia", "Filtrar tipo logico", "Filtrar tipo vazio",
]);

export const DEBUG_COMMANDS: ReadonlySet<string> = new Set([
  "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução", "Entrar na função", "Sair da função",
]);

/** Estado da IDE que decide quais comandos aparecem na paleta. */
export interface PaletteContext {
  /** Linguagem da aba ativa; `null` sem aba de código. */
  language: "portugol" | "portugol-rs" | "rust" | null;
  cargoProject: boolean;
  cargoAvailable: boolean;
  /** A aba ativa pode executar o Rust gerado com `rustc`. */
  canRunAsRust: boolean;
  runActive: boolean;
  paused: boolean;
  porsRunPending: boolean;
}

export function isPaletteCommandAvailable(command: string, context: PaletteContext): boolean {
  if (command === "Paleta de comandos") return false;
  if (command === "Sobre (fora do editor); ir à declaração (no editor)") return false;
  const hasTab = context.language !== null;
  if (TAB_COMMANDS.has(command) && !hasTab) return false;
  const idle = !context.runActive && !context.porsRunPending;
  if (command === "Executar como Rust") return hasTab && context.canRunAsRust && idle;
  if (command === "Analisar projeto Cargo" || command === "Testar projeto Cargo") {
    return context.language === "rust" && context.cargoProject && context.cargoAvailable && idle;
  }
  if (command === "Parâmetros de execução" && context.language === "portugol-rs") return false;
  if (context.language === "rust" && RUST_UNSUPPORTED_COMMANDS.has(command)) return false;
  if (DEBUG_COMMANDS.has(command) && !isDebugCommandAvailable(command, context.language, context.runActive, context.paused)) {
    return false;
  }
  return true;
}
