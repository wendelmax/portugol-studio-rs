import type {
  CargoProjectRunResult,
  CargoRunTarget,
  CargoRunTargets,
  DiagnosticInfo,
  RustSourceRunResult,
} from "../api";
import type { Tab } from "../state/tab";

export type RustExecutionConsent = "file" | "cargo";

export interface RustExecutionDependencies {
  activeTab: () => Tab | undefined;
  tabs: () => Tab[];
  isBusy: () => boolean;
  rustcAvailable: () => boolean | null;
  cargoAvailable: () => boolean | null;
  isDirty: (tab: Tab) => boolean;
  save: () => Promise<boolean>;
  argumentsText: () => string;
  parseArguments: (text: string) => string[];
  currentDocument: () => { toString(): string };
  isCurrent: (tab: Tab, document: object) => boolean;
  runId: () => number | null;
  setRunId: (id: number | null) => void;
  setPending: (pending: boolean) => void;
  setRunTabId: (tabId: number | null) => void;
  syncLanguageControls: (tab: Tab) => void;
  setRunning: (running: boolean) => void;
  hasConsent: (consent: RustExecutionConsent) => boolean;
  askConsent: (consent: RustExecutionConsent) => Promise<boolean>;
  rememberConsent: (consent: RustExecutionConsent) => void;
  chooseCargoTarget: (targets: CargoRunTarget[]) => Promise<CargoRunTarget | null>;
  registerRun: () => Promise<number>;
  cargoRunTargets: (id: number, workspaceId: string, workspacePath: string) => Promise<CargoRunTargets>;
  runCargoProject: (
    id: number,
    workspaceId: string,
    workspacePath: string,
    target: CargoRunTarget | null,
    args: string[],
  ) => Promise<CargoProjectRunResult>;
  runRustSource: (id: number, source: string, args: string[]) => Promise<RustSourceRunResult>;
  clearRuntimeError: () => void;
  clearDebugLine: () => void;
  clearPaused: () => void;
  showConsole: () => void;
  clearConsole: () => void;
  appendProcessOutput: (result: CargoProjectRunResult | RustSourceRunResult) => void;
  appendConsole: (text: string, kind?: "error") => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  renderMessages: () => void;
  setDiagnostics: (tab: Tab, diagnostics: DiagnosticInfo[]) => void;
  showMessages: () => void;
}

/** Executa arquivos Rust avulsos e binários Cargo sem acoplar a lógica à shell. */
export function createRustExecutionController(dependencies: RustExecutionDependencies) {
  async function run(pauseOnStart = false): Promise<void> {
    if (dependencies.isBusy()) return;
    const tab = dependencies.activeTab();
    if (!tab) return;
    if (tab.language !== "rust") return;
    if (tab.workspaceId && tab.workspacePath && tab.cargoProject === undefined) {
      dependencies.setStatus("Verificando se este arquivo pertence a um projeto Cargo.");
      return;
    }
    if (pauseOnStart) {
      dependencies.setStatus("Depuração de arquivos Rust ainda não está disponível.");
      return;
    }
    if (dependencies.cargoAvailable() !== true && tab.cargoProject === true) {
      dependencies.setStatus("Instale Cargo para executar o projeto Rust.", "error");
      return;
    }
    if (dependencies.rustcAvailable() !== true) {
      dependencies.setStatus("Instale a toolchain Rust para executar arquivos .rs.", "error");
      return;
    }
    let rustArgs: string[];
    try {
      rustArgs = dependencies.parseArguments(dependencies.argumentsText());
    } catch (error) {
      dependencies.setStatus(error instanceof Error ? error.message : "Argumentos inválidos.", "error");
      return;
    }
    if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
      const dirtyFiles = dependencies.tabs().filter((candidate) =>
        candidate.workspaceId === tab.workspaceId
        && candidate.language === "rust"
        && candidate !== tab
        && dependencies.isDirty(candidate),
      );
      if (dirtyFiles.length > 0) {
        dependencies.setStatus("Salve os outros arquivos alterados deste projeto antes de executar Cargo.", "error");
        return;
      }
      if (dependencies.isDirty(tab) && !(await dependencies.save())) {
        dependencies.setStatus("Salve o arquivo Rust antes de executar o projeto Cargo.", "error");
        return;
      }
    }
    if (dependencies.activeTab() !== tab) return;
    const document = dependencies.currentDocument();
    const source = document.toString();
    dependencies.setPending(true);
    dependencies.setRunTabId(tab.id);
    dependencies.syncLanguageControls(tab);
    let cargoTarget: CargoRunTarget | null = null;
    try {
      if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
        dependencies.setStatus("Identificando os binários do projeto Cargo…");
        const discoveryId = await dependencies.registerRun();
        dependencies.setRunId(discoveryId);
        dependencies.setRunning(true);
        let targetInfo: CargoRunTargets;
        try {
          targetInfo = await dependencies.cargoRunTargets(discoveryId, tab.workspaceId, tab.workspacePath);
        } finally {
          if (dependencies.runId() === discoveryId) dependencies.setRunId(null);
          dependencies.setRunning(false);
        }
        if (!dependencies.isCurrent(tab, document)) return;
        if (targetInfo.targets.length === 0) {
          dependencies.setStatus("Este projeto Cargo não contém um binário para executar.", "error");
          return;
        }
        cargoTarget = targetInfo.targets.find((target) =>
          target.packageName === tab.cargoTarget?.packageName && target.targetName === tab.cargoTarget?.targetName,
        ) ?? targetInfo.defaultTarget ?? (targetInfo.targets.length === 1 ? targetInfo.targets[0] : null);
        if (!cargoTarget) {
          cargoTarget = await dependencies.chooseCargoTarget(targetInfo.targets);
          if (!cargoTarget) {
            if (dependencies.activeTab() === tab) dependencies.setStatus("Execução Cargo cancelada.");
            return;
          }
        }
        tab.cargoTarget = cargoTarget;
      }
      const consent: RustExecutionConsent = tab.cargoProject === true ? "cargo" : "file";
      if (!dependencies.hasConsent(consent)) {
        if (!(await dependencies.askConsent(consent))) {
          if (dependencies.activeTab() === tab) dependencies.setStatus("Execução Rust cancelada.");
          return;
        }
        dependencies.rememberConsent(consent);
      }
      if (!dependencies.isCurrent(tab, document)) return;
      const id = await dependencies.registerRun();
      dependencies.setRunId(id);
      dependencies.clearRuntimeError();
      dependencies.clearDebugLine();
      dependencies.clearPaused();
      dependencies.showConsole();
      dependencies.clearConsole();
      dependencies.setRunning(true);
      if (tab.cargoProject === true && tab.workspaceId && tab.workspacePath) {
        dependencies.setStatus("Compilando o projeto Cargo…");
        const result = await dependencies.runCargoProject(id, tab.workspaceId, tab.workspacePath, cargoTarget, rustArgs);
        if (!dependencies.isCurrent(tab, document)) return;
        dependencies.appendProcessOutput(result);
        if (result.status === "executed") {
          dependencies.setStatus(result.exitCode === 0 ? "Projeto Cargo finalizado." : `Programa Rust terminou com código ${result.exitCode ?? "desconhecido"}.`, result.exitCode === 0 ? "ok" : "error");
        } else if (result.status === "cancelled") {
          dependencies.setStatus(`Execução Cargo cancelada durante a ${result.phase === "compilation" ? "compilação" : "execução"}.`);
        } else if (result.status === "build_limit" || result.status === "runtime_limit") {
          const reason = result.reason === "time" ? (result.phase === "compilation" ? "20 segundos de compilação" : "3 segundos de execução") : "limite de saída de 1 MiB";
          dependencies.setStatus(`Projeto Cargo interrompido: ${reason}.`, "error");
        } else {
          dependencies.setStatus(result.phase === "compilation" ? "Cargo não conseguiu compilar o projeto." : "O programa Cargo terminou com erro.", "error");
        }
        return;
      }
      dependencies.setStatus("Compilando e executando Rust 2021 com rustc…");
      const result = await dependencies.runRustSource(id, source, rustArgs);
      if (!dependencies.isCurrent(tab, document)) return;
      if (result.status === "cancelled") {
        dependencies.appendProcessOutput(result);
        const phase = result.phase === "compilation" ? "compilação" : "execução";
        dependencies.setStatus(`Execução Rust cancelada durante a ${phase}.`);
      } else if (result.status === "executed") {
        dependencies.appendProcessOutput(result);
        dependencies.setStatus(
          result.exitCode === 0
            ? "Programa Rust finalizado com rustc."
            : `Programa Rust terminou com código ${result.exitCode ?? "desconhecido"}.`,
          result.exitCode === 0 ? "ok" : "error",
        );
      } else if (result.status === "runtime_limit") {
        dependencies.appendProcessOutput(result);
        const reason = result.reason === "time" ? "tempo máximo de 3 segundos" : "limite de saída de 1 MiB";
        dependencies.setStatus(`Execução Rust interrompida: ${reason}.`, "error");
      } else if (result.status === "compile_limit") {
        if (result.stderr) dependencies.appendConsole(result.stderr, "error");
        const reason = result.reason === "time" ? "tempo máximo de compilação de 20 segundos" : "limite de saída de compilação de 1 MiB";
        dependencies.setStatus(`Compilação Rust interrompida: ${reason}.`, "error");
      } else {
        const diagnostics: DiagnosticInfo[] = result.diagnostics.map((item) => ({
          severity: item.severity === "warning" ? "warning" : "error",
          code: item.code ?? "rustc",
          message: item.message,
          help: item.rendered,
          start: item.start,
          end: item.end,
        }));
        dependencies.setDiagnostics(tab, diagnostics);
        if (result.stderr) dependencies.appendConsole(result.stderr, "error");
        if (diagnostics.length) {
          dependencies.renderMessages();
          dependencies.showMessages();
          dependencies.setStatus("rustc encontrou erros no arquivo Rust.", "error");
        } else {
          dependencies.setStatus("rustc encontrou erros; veja o detalhe no console.", "error");
        }
      }
    } catch (error) {
      if (dependencies.isCurrent(tab, document)) {
        const message = String(error);
        if (message.includes("Descoberta de alvos Cargo cancelada")) {
          dependencies.setStatus("Execução Cargo cancelada.");
        } else {
          dependencies.appendConsole(`${message}\n`, "error");
          dependencies.setStatus("Não foi possível compilar ou executar o arquivo Rust.", "error");
        }
      }
    } finally {
      dependencies.setRunId(null);
      dependencies.setPending(false);
      dependencies.setRunTabId(null);
      dependencies.setRunning(false);
      const current = dependencies.activeTab();
      if (current) dependencies.syncLanguageControls(current);
    }
  }

  return { run };
}
