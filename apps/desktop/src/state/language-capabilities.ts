import type { Tab } from "./tab";
import { canDebugSource, canFormatSource, canRunSource } from "../runtime-capabilities.ts";
import { FERRO_STDLIB_MODULES } from "../ferro-stdlib-catalog.ts";

export const LANGUAGE_BUTTON_IDS = [
  "btn-run", "btn-debug", "btn-debug-pause", "btn-debug-continue", "btn-debug-step-over", "btn-debug-step-into", "btn-debug-step-out",
] as const;

export interface LanguageControlState {
  rustcAvailable: boolean | null;
  cargoAvailable: boolean | null;
  rustfmtAvailable: boolean | null;
  runId: number | null;
  paused: unknown | null;
  busy: boolean;
  defaultTitles: Partial<Record<(typeof LANGUAGE_BUTTON_IDS)[number], string>>;
  libraryNames: string[];
}

export function deriveLanguageControls(
  tab: Pick<Tab, "language" | "workspaceId" | "workspacePath" | "cargoProject" | "symbols">,
  state: LanguageControlState,
) {
  const ferro = tab.language === "ferro-rust";
  const rustFile = tab.language === "rust";
  const markupFile = tab.language === "markup";
  const buttons = Object.fromEntries(LANGUAGE_BUTTON_IDS.map((id) => {
    const isRun = id === "btn-run";
    const cargoDetectionPending = rustFile && Boolean(tab.workspaceId && tab.workspacePath) && tab.cargoProject === undefined;
    const runtimeUnavailable = isRun
      ? !canRunSource(tab.language, state.rustcAvailable) || cargoDetectionPending || (rustFile && tab.cargoProject === true && state.cargoAvailable === false)
      : !canDebugSource(tab.language);
    const stateDisabled = id === "btn-debug-pause"
      ? state.runId === null || state.paused !== null
      : id === "btn-run" || id === "btn-debug"
        ? state.busy && state.paused === null
        : state.paused === null;
    const unavailableReason = markupFile
      ? "Arquivos de marcação descrevem a tela; ela executa pelo projeto desktop."
      : cargoDetectionPending
      ? "Verificando se este arquivo pertence a um projeto Cargo."
      : rustFile && state.rustcAvailable !== true
        ? "Instale a toolchain Rust para executar arquivos .rs."
        : rustFile && tab.cargoProject === true && state.cargoAvailable === false
          ? "Instale Cargo para executar o projeto Rust."
          : rustFile && !isRun
            ? "Depuração Rust ainda não está disponível."
            : "";
    const title = unavailableReason
      || (ferro && isRun ? "Executar na VM do Ferro (Shift+F6)" : "")
      || (rustFile && isRun && tab.cargoProject === true ? "Compilar e executar o projeto Cargo (scripts e dependências executam localmente)" : "")
      || (rustFile && isRun ? "Compilar e executar Rust 2021 com rustc (execução sem sandbox)" : "")
      || state.defaultTitles[id]
      || "";
    return [id, { disabled: runtimeUnavailable || stateDisabled, title, description: unavailableReason }];
  }));
  const argumentsControl = {
    disabled: !rustFile,
    title: markupFile
      ? "Parâmetros de execução não se aplicam a arquivos de marcação."
      : ferro
      ? "Parâmetros ainda não são suportados em Ferro."
      : rustFile
        ? "Espaços separam itens; aspas duplas agrupam. O programa recebe argv, sem shell."
        : "",
  };
  const format = {
    disabled: !canFormatSource(tab.language, state.rustfmtAvailable),
    title: rustFile && state.rustfmtAvailable !== true
      ? "Instale o componente rustfmt com `rustup component add rustfmt`."
      : "Formatar código-fonte",
  };
// Os módulos vêm do catálogo da biblioteca padrão, a mesma lista do autocompletar.
const FERRO_LIBRARY_NAMES: ReadonlySet<string> = new Set(FERRO_STDLIB_MODULES);

  const libraries = Object.fromEntries(state.libraryNames.map((name) => {
    const ferroAvailable = ferro && FERRO_LIBRARY_NAMES.has(name);
    return [name, {
      disabled: rustFile || !ferroAvailable,
      title: rustFile
        ? "Bibliotecas Ferro não se aplicam a Rust."
        : markupFile
        ? "Bibliotecas Ferro não se aplicam a arquivos de marcação."
        : ferroAvailable
          ? `Inserir biblioteca Ferro ${name}.`
          : "Módulo indisponível em Ferro.",
    }];
  }));
  const symbolActions = {
    disabled: !ferro,
    description: rustFile
      ? "Navegação e renomeação de símbolos Rust ainda não estão implementadas."
      : markupFile
        ? "Navegação e renomeação de símbolos não se aplicam a arquivos de marcação."
        : "",
  };

  return { buttons, arguments: argumentsControl, format, libraries, symbolActions };
}
