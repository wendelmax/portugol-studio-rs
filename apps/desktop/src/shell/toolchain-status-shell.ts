import * as api from "../api";
import { $ } from "../dom";
import type { ToolchainStatusDependencies } from "./toolchain-status";
import { createToolchainStatusController } from "./toolchain-status";

export interface ToolchainStatusShellDependencies extends Pick<
  ToolchainStatusDependencies,
  "activeTab" | "syncLanguageControls" | "restoreLearningMode" | "forceRustLint"
> {
  state: Pick<ToolchainStatusDependencies, "setRustcAvailable" | "setCargoAvailable" | "setRustfmtAvailable">;
}

/** Liga as verificações de toolchain aos adaptadores de API e interface. */
export function mountToolchainStatusShell(dependencies: ToolchainStatusShellDependencies): void {
  const controller = createToolchainStatusController({
    checkRust: api.rustToolchainAvailable,
    checkCargo: api.cargoToolchainAvailable,
    checkRustfmt: api.rustfmtAvailable,
    setRustcAvailable: dependencies.state.setRustcAvailable,
    setCargoAvailable: dependencies.state.setCargoAvailable,
    setRustfmtAvailable: dependencies.state.setRustfmtAvailable,
    setRustOptionDisabled: (disabled) => {
      $<HTMLOptionElement>("learning-mode-rust-option").disabled = disabled;
      const titlebarOption = document.querySelector<HTMLOptionElement>("#titlebar-learning-mode-rust-option");
      if (titlebarOption) titlebarOption.disabled = disabled;
    },
    setRustHelpText: (text) => { $("learning-mode-rust-help").textContent = text; },
    restoreLearningMode: dependencies.restoreLearningMode,
    activeTab: dependencies.activeTab,
    syncLanguageControls: dependencies.syncLanguageControls,
    forceRustLint: dependencies.forceRustLint,
  });
  void controller.initialize();
}
