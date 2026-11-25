import type { LearningMode } from "../learning-mode";
import type { Tab } from "../state/tab";
import type { ToolchainAvailability } from "./toolchain-state";
import { statusBarState, type StatusBarTab } from "./status-bar-state";

export interface StatusBarDependencies {
  activeTab: () => (Tab & StatusBarTab) | undefined;
  source: () => string;
  tabSize: () => number;
  learningMode: () => LearningMode;
  rustcAvailable: () => ToolchainAvailability;
  openMessages: () => void;
}

/** Apresenta o contexto do arquivo sem substituir as mensagens de ação da IDE. */
export function mountStatusBar(dependencies: StatusBarDependencies) {
  const context = document.querySelector<HTMLElement>("#statusbar-context")!;
  const language = document.querySelector<HTMLElement>("#statusbar-language")!;
  const mode = document.querySelector<HTMLElement>("#statusbar-mode")!;
  const engines = document.querySelector<HTMLElement>("#statusbar-engines")!;
  const format = document.querySelector<HTMLElement>("#statusbar-format")!;
  const diagnostics = document.querySelector<HTMLButtonElement>("#statusbar-diagnostics")!;

  diagnostics.addEventListener("click", dependencies.openMessages);

  function render() {
    const tab = dependencies.activeTab();
    if (!tab) {
      context.hidden = true;
      diagnostics.hidden = true;
      return;
    }

    const state = statusBarState(
      tab,
      dependencies.source(),
      dependencies.learningMode(),
      dependencies.rustcAvailable(),
      dependencies.tabSize(),
    );
    if (!state) return;
    context.hidden = false;
    language.textContent = state.language;
    mode.textContent = state.mode;
    engines.textContent = state.engines ?? "";
    engines.hidden = state.engines === null;
    format.textContent = `${state.indentation} · ${state.encoding}`;
    diagnostics.hidden = false;
    diagnostics.disabled = state.diagnostics.state === "pending";
    diagnostics.textContent = state.diagnostics.label;
    const errorLabel = `${state.diagnostics.errors} ${state.diagnostics.errors === 1 ? "erro" : "erros"}`;
    const warningLabel = `${state.diagnostics.warnings} ${state.diagnostics.warnings === 1 ? "aviso" : "avisos"}`;
    diagnostics.setAttribute("aria-label", state.diagnostics.state === "ready"
      ? state.diagnostics.label === "Sem erros"
        ? "Sem erros ou avisos no arquivo atual"
        : `${errorLabel} e ${warningLabel} no arquivo atual; abrir Mensagens`
      : state.diagnostics.label);
    diagnostics.title = state.diagnostics.state === "pending"
      ? "Aguardando a análise do arquivo atual"
      : "Abrir painel Mensagens";
  }

  return { render };
}
