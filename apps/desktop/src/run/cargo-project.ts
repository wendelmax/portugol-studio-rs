import type {
  CargoProjectCheckResult,
  CargoProjectTestResult,
  DiagnosticInfo,
} from "../api";
import type { Tab } from "../state/tab";

export type CargoProjectPanel = "messages" | "console" | "tests";
export type CargoProjectOperation = "check" | "test";

export interface CargoProjectControllerDependencies {
  tabs: () => Tab[];
  activeTab: () => Tab | undefined;
  isBusy: () => boolean;
  cargoAvailable: () => boolean | null;
  isDirty: (tab: Tab) => boolean;
  save: () => Promise<boolean>;
  currentDocument: () => object;
  isCurrent: (tab: Tab, document: object) => boolean;
  setPending: (pending: boolean) => void;
  setRunTabId: (tabId: number | null) => void;
  runId: () => number | null;
  setRunId: (id: number | null) => void;
  syncLanguageControls: (tab: Tab) => void;
  setRunning: (running: boolean) => void;
  hasCargoConsent: () => boolean;
  askCargoConsent: (operation: CargoProjectOperation) => Promise<boolean>;
  rememberCargoConsent: () => void;
  registerRun: () => Promise<number>;
  checkCargoProject: (id: number, workspaceId: string, workspacePath: string) => Promise<CargoProjectCheckResult>;
  testCargoProject: (id: number, workspaceId: string, workspacePath: string) => Promise<CargoProjectTestResult>;
  clearRuntimeError: () => void;
  clearConsole: () => void;
  showPanel: (panel: CargoProjectPanel) => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  setDiagnostics: (tab: Tab, diagnostics: DiagnosticInfo[]) => void;
  appendConsole: (text: string, kind?: "error") => void;
  renderExerciseTests: () => void;
}

interface PreparedCargoRun {
  tab: Tab;
  document: object;
  workspaceId: string;
  workspacePath: string;
}

/** Orquestra cargo check e cargo test; a shell fornece UI e serviços Tauri por callbacks. */
export function createCargoProjectController(dependencies: CargoProjectControllerDependencies) {
  async function prepare(operation: CargoProjectOperation): Promise<PreparedCargoRun | null> {
    if (dependencies.isBusy()) return null;
    const tab = dependencies.activeTab();
    if (!tab || tab.language !== "rust") return null;
    if (tab.workspaceId && tab.workspacePath && tab.cargoProject === undefined) {
      dependencies.setStatus("Verificando se este arquivo pertence a um projeto Cargo.");
      return null;
    }
    if (tab.cargoProject !== true || !tab.workspaceId || !tab.workspacePath) {
      dependencies.setStatus("Abra um arquivo Rust pelo explorador dentro de um projeto Cargo.", "error");
      return null;
    }
    if (dependencies.cargoAvailable() !== true) {
      dependencies.setStatus(operation === "check"
        ? "Instale Cargo para analisar o projeto Rust."
        : "Instale Cargo para executar os testes do projeto Rust.", "error");
      return null;
    }
    const dirtyFiles = dependencies.tabs().filter((candidate) =>
      candidate.workspaceId === tab.workspaceId
      && candidate.language === "rust"
      && candidate !== tab
      && dependencies.isDirty(candidate),
    );
    if (dirtyFiles.length > 0) {
      dependencies.setStatus(operation === "check"
        ? "Salve os outros arquivos Rust alterados deste workspace antes da análise Cargo."
        : "Salve os outros arquivos Rust alterados deste workspace antes de executar `cargo test`.", "error");
      return null;
    }
    if (dependencies.isDirty(tab) && !(await dependencies.save())) {
      dependencies.setStatus(operation === "check"
        ? "Salve o arquivo Rust antes de analisar o projeto Cargo."
        : "Salve o arquivo Rust antes de executar `cargo test`.", "error");
      return null;
    }
    if (dependencies.activeTab() !== tab) return null;
    return {
      tab,
      document: dependencies.currentDocument(),
      workspaceId: tab.workspaceId,
      workspacePath: tab.workspacePath,
    };
  }

  function begin(tab: Tab) {
    dependencies.setPending(true);
    dependencies.setRunTabId(tab.id);
    dependencies.syncLanguageControls(tab);
  }

  function finish(registeredId: number | null) {
    if (registeredId !== null && dependencies.runId() === registeredId) dependencies.setRunId(null);
    dependencies.setPending(false);
    dependencies.setRunTabId(null);
    dependencies.setRunning(false);
    const current = dependencies.activeTab();
    if (current) dependencies.syncLanguageControls(current);
  }

  async function confirmConsent(operation: CargoProjectOperation, tab: Tab): Promise<boolean> {
    if (dependencies.hasCargoConsent()) return true;
    const accepted = await dependencies.askCargoConsent(operation);
    if (!accepted) {
      if (dependencies.activeTab() === tab) {
        dependencies.setStatus(operation === "check" ? "Análise Cargo cancelada." : "Execução de `cargo test` cancelada.");
      }
      return false;
    }
    dependencies.rememberCargoConsent();
    return true;
  }

  async function check(): Promise<void> {
    const prepared = await prepare("check");
    if (!prepared) return;
    const { tab, document, workspaceId, workspacePath } = prepared;
    let registeredId: number | null = null;
    begin(tab);
    try {
      if (!(await confirmConsent("check", tab))) return;
      if (!dependencies.isCurrent(tab, document)) return;
      registeredId = await dependencies.registerRun();
      dependencies.setRunId(registeredId);
      dependencies.setRunning(true);
      dependencies.clearRuntimeError();
      dependencies.clearConsole();
      dependencies.showPanel("messages");
      dependencies.setStatus("Analisando projeto com cargo check…");
      const result = await dependencies.checkCargoProject(registeredId, workspaceId, workspacePath);
      if (!dependencies.isCurrent(tab, document)) return;
      const diagnostics: DiagnosticInfo[] = result.diagnostics.map((item) => ({
        severity: item.severity === "warning" ? "warning" : "error",
        code: item.code ?? "cargo",
        message: item.message,
        help: item.rendered,
        start: item.start,
        end: item.end,
      }));
      dependencies.setDiagnostics(tab, diagnostics);
      if (result.stderr) dependencies.appendConsole(result.stderr, result.status === "check_error" ? "error" : undefined);
      if (result.status === "cancelled") {
        dependencies.setStatus("Análise Cargo cancelada.");
      } else if (result.status === "check_limit") {
        const reason = result.reason === "time" ? "20 segundos" : "limite de saída de 1 MiB";
        dependencies.setStatus(`Análise Cargo interrompida: ${reason}.`, "error");
      } else if (result.status === "check_error") {
        dependencies.showPanel(diagnostics.length ? "messages" : "console");
        dependencies.setStatus(diagnostics.length
          ? "Cargo encontrou diagnósticos no arquivo aberto; confira o Console para detalhes do projeto."
          : "Cargo check terminou com erros em outros arquivos ou no manifesto.", "error");
      } else {
        dependencies.showPanel(diagnostics.length ? "messages" : result.stderr ? "console" : "messages");
        const warnings = diagnostics.filter((item) => item.severity === "warning").length;
        dependencies.setStatus(diagnostics.length
          ? `Análise concluída: ${warnings} aviso(s) no arquivo aberto.`
          : "Análise Cargo concluída sem diagnósticos no arquivo aberto.", "ok");
      }
    } catch (error) {
      if (dependencies.isCurrent(tab, document)) {
        dependencies.appendConsole(`${String(error)}\n`, "error");
        dependencies.showPanel("console");
        dependencies.setStatus("Não foi possível analisar o projeto Cargo.", "error");
      }
    } finally {
      finish(registeredId);
    }
  }

  async function runTests(): Promise<void> {
    const prepared = await prepare("test");
    if (!prepared) return;
    const { tab, document, workspaceId, workspacePath } = prepared;
    let registeredId: number | null = null;
    begin(tab);
    try {
      if (!(await confirmConsent("test", tab))) return;
      if (!dependencies.isCurrent(tab, document)) return;
      registeredId = await dependencies.registerRun();
      dependencies.setRunId(registeredId);
      dependencies.setRunning(true);
      tab.cargoTestRun = { source: document, status: "running" };
      dependencies.showPanel("tests");
      dependencies.renderExerciseTests();
      dependencies.clearConsole();
      dependencies.setStatus("Executando testes do projeto com cargo test…");
      const result = await dependencies.testCargoProject(registeredId, workspaceId, workspacePath);
      if (!dependencies.isCurrent(tab, document)) return;
      if (result.stdout) dependencies.appendConsole(result.stdout);
      if (result.stderr) dependencies.appendConsole(result.stderr, result.status === "build_error" ? "error" : undefined);
      tab.cargoTestRun = {
        source: document,
        status: "completed",
        result: {
          status: result.status,
          passed: result.passed,
          failed: result.failed,
          ignored: result.ignored,
          failures: result.failures,
          reason: result.reason,
        },
      };
      dependencies.renderExerciseTests();
      if (result.status === "passed") {
        dependencies.setStatus(`Cargo test aprovado (${result.passed} teste(s)).`, "ok");
      } else if (result.status === "cancelled") {
        dependencies.setStatus("Execução de `cargo test` cancelada.");
      } else {
        dependencies.setStatus(result.status === "build_error"
          ? "Falha de compilação ao executar `cargo test`."
          : `Cargo test falhou (${result.failed} falha(s)).`, "error");
      }
    } catch (error) {
      if (dependencies.isCurrent(tab, document)) {
        const message = typeof error === "string" ? error : "Não foi possível executar `cargo test`.";
        tab.cargoTestRun = { source: document, status: "error", message };
        dependencies.renderExerciseTests();
        dependencies.appendConsole(`${message}\n`, "error");
        dependencies.setStatus("Não foi possível executar `cargo test`.", "error");
      }
    } finally {
      finish(registeredId);
      dependencies.renderExerciseTests();
    }
  }

  return { check, runTests };
}
