import type { SourceLanguage } from "./document-language";
import { canDebugSource } from "./runtime-capabilities.ts";

export interface CommandSearchItem {
  command: string;
  label: string;
}

export function isDebugCommandAvailable(
  command: string,
  language: SourceLanguage | null,
  runActive: boolean,
  paused: boolean,
): boolean {
  if (language === null || !canDebugSource(language)) return false;
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
  "Executar como Rust", "Alternar prévia Rust", "Alternar prévia visual",
  "Abrir painel Aprender", "Abrir Console", "Abrir Mensagens", "Abrir Testes", "Abrir Depuração",
  "Limpar console", "Copiar saída do console",
  "Salvar", "Salvar como", "Parâmetros de execução", "Fechar aba", "Fechar todas as abas", "Localizar e substituir",
  "Buscar na estrutura", "Filtrar variável", "Filtrar vetor", "Filtrar matriz", "Filtrar função",
  "Filtrar constante", "Mostrar todos os tipos", "Filtrar tipo inteiro", "Filtrar tipo caracter",
  "Filtrar tipo real", "Filtrar tipo cadeia", "Filtrar tipo logico", "Filtrar tipo vazio",
]);

/** Comandos sem suporte em arquivos Rust. */
export const RUST_UNSUPPORTED_COMMANDS: ReadonlySet<string> = new Set([
  "Executar", "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução",
  "Entrar na função", "Sair da função", "Renomear símbolo", "Formatar código-fonte",
  "Alternar prévia Rust", "Alternar prévia visual",
  "Buscar na estrutura", "Filtrar variável", "Filtrar vetor", "Filtrar matriz", "Filtrar função",
  "Filtrar constante", "Mostrar todos os tipos", "Filtrar tipo inteiro", "Filtrar tipo caracter",
  "Filtrar tipo real", "Filtrar tipo cadeia", "Filtrar tipo logico", "Filtrar tipo vazio",
]);

/** Comandos de programa que não se aplicam à marcação (.design e .ferproj). */
export const MARKUP_UNSUPPORTED_COMMANDS: ReadonlySet<string> = new Set([
  "Executar", "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução",
  "Entrar na função", "Sair da função", "Renomear símbolo", "Parâmetros de execução",
  "Alternar prévia Rust", "Alternar prévia visual",
  "Filtrar variável", "Filtrar vetor", "Filtrar matriz", "Filtrar função",
  "Filtrar constante", "Mostrar todos os tipos", "Filtrar tipo inteiro", "Filtrar tipo caracter",
  "Filtrar tipo real", "Filtrar tipo cadeia", "Filtrar tipo logico", "Filtrar tipo vazio",
]);

export const DEBUG_COMMANDS: ReadonlySet<string> = new Set([
  "Depurar", "Interromper", "Pausar", "Continuar", "Próxima instrução", "Entrar na função", "Sair da função",
]);

/** Estado da IDE que decide quais comandos aparecem na paleta. */
export interface PaletteContext {
  /** Linguagem da aba ativa; `null` sem aba de código. */
  language: SourceLanguage | null;
  cargoProject: boolean;
  cargoAvailable: boolean;
  /** A aba ativa pode executar o Rust gerado com `rustc`. */
  canRunAsRust: boolean;
  runActive: boolean;
  paused: boolean;
  ferroRunPending: boolean;
}

export type PaletteLanguage = Exclude<SourceLanguage, null>;

export interface PaletteContextDependencies {
  activeTab?: { language: PaletteLanguage; cargoProject?: boolean };
  cargoAvailable: boolean | null;
  rustcAvailable: boolean | null;
  runActive: boolean;
  paused: boolean;
  ferroRunPending: boolean;
  canRunAsRust: (language: PaletteLanguage, rustcAvailable: boolean | null) => boolean;
}

/** Deriva o estado da paleta a partir do snapshot atual da IDE. */
export function createPaletteContext(dependencies: PaletteContextDependencies): PaletteContext {
  const tab = dependencies.activeTab;
  return {
    language: tab?.language ?? null,
    cargoProject: tab?.cargoProject === true,
    cargoAvailable: dependencies.cargoAvailable === true,
    canRunAsRust: tab !== undefined && dependencies.canRunAsRust(tab.language, dependencies.rustcAvailable),
    runActive: dependencies.runActive,
    paused: dependencies.paused,
    ferroRunPending: dependencies.ferroRunPending,
  };
}

export function isPaletteCommandAvailable(command: string, context: PaletteContext): boolean {
  if (command === "Paleta de comandos") return false;
  if (command === "Sobre (fora do editor); ir à declaração (no editor)") return false;
  const hasTab = context.language !== null;
  if (TAB_COMMANDS.has(command) && !hasTab) return false;
  const idle = !context.runActive && !context.ferroRunPending;
  if (command === "Executar como Rust") return hasTab && context.canRunAsRust && idle;
  if (command === "Analisar projeto Cargo" || command === "Testar projeto Cargo") {
    return context.language === "rust" && context.cargoProject && context.cargoAvailable && idle;
  }
  if (command === "Parâmetros de execução" && context.language === "ferro-rust") return false;
  if (context.language === "rust" && RUST_UNSUPPORTED_COMMANDS.has(command)) return false;
  if (context.language === "markup" && MARKUP_UNSUPPORTED_COMMANDS.has(command)) return false;
  if (DEBUG_COMMANDS.has(command) && !isDebugCommandAvailable(command, context.language, context.runActive, context.paused)) {
    return false;
  }
  return true;
}
