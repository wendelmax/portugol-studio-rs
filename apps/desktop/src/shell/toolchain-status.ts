import type { Tab } from "../state/tab";

export interface ToolchainStatusDependencies {
  checkRust: () => Promise<boolean>;
  checkCargo: () => Promise<boolean>;
  checkRustfmt: () => Promise<boolean>;
  setRustcAvailable: (available: boolean) => void;
  setCargoAvailable: (available: boolean) => void;
  setRustfmtAvailable: (available: boolean) => void;
  setRustOptionDisabled: (disabled: boolean) => void;
  setRustHelpText: (text: string) => void;
  restoreLearningMode: () => void;
  activeTab: () => Tab | undefined;
  syncLanguageControls: (tab: Tab) => void;
  forceRustLint: () => void;
}

/** Runs independent toolchain checks and updates capability-dependent shell state. */
export function createToolchainStatusController(dependencies: ToolchainStatusDependencies) {
  async function checkRust() {
    try {
      const available = await dependencies.checkRust();
      dependencies.setRustcAvailable(available);
      dependencies.setRustOptionDisabled(!available);
      dependencies.setRustHelpText(available
        ? "Toolchain rustc disponível. Novo Rust cria um arquivo .rs; trocar de nível não altera o arquivo aberto."
        : "Instale a toolchain Rust (rustc) para habilitar este nível e executar arquivos .rs.");
    } catch {
      dependencies.setRustcAvailable(false);
      dependencies.setRustOptionDisabled(true);
      dependencies.setRustHelpText("Não foi possível verificar rustc. Instale ou configure a toolchain Rust para habilitar este nível.");
    }
    dependencies.restoreLearningMode();
    const tab = dependencies.activeTab();
    if (!tab) return;
    dependencies.syncLanguageControls(tab);
    if (tab.language === "rust") dependencies.forceRustLint();
  }

  async function checkCargo() {
    let available = false;
    try {
      available = await dependencies.checkCargo();
    } catch {
      // A IDE continua disponível quando Cargo não pode ser detectado.
    }
    dependencies.setCargoAvailable(available);
    const tab = dependencies.activeTab();
    if (tab) dependencies.syncLanguageControls(tab);
  }

  async function checkRustfmt() {
    let available = false;
    try {
      available = await dependencies.checkRustfmt();
    } catch {
      // A IDE continua disponível quando rustfmt não pode ser detectado.
    }
    dependencies.setRustfmtAvailable(available);
    const tab = dependencies.activeTab();
    if (tab) dependencies.syncLanguageControls(tab);
  }

  async function initialize() {
    await Promise.all([checkRust(), checkCargo(), checkRustfmt()]);
  }

  return { initialize };
}
