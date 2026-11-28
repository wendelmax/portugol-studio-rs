/**
 * Executar uma tela (ADR 0018, seção 5): a partir do `.design` ou do
 * code-behind, junta os dois arquivos (com o que ainda não foi salvo) e abre a
 * tela numa janela própria. A saída de `escreva` e os erros vão para o console.
 * Exportar (seção 6) usa os mesmos dois textos para gerar um projeto Cargo
 * com a tela em Slint.
 */
import { fileKind } from "../file-formats.ts";
import type { Tab } from "../state/tab.ts";
import { isScreenFile, screenCodeFor, screenDesignFor, screenFileSource, type ScreenFilesHost } from "./screen-files.ts";

export interface ScreenExecutionHost {
  files: ScreenFilesHost;
  /** Inicia a sessão e abre a janela; rejeita com a mensagem do primeiro problema. */
  start: (designSource: string, codeSource: string, workspaceId: string, designPath: string) => Promise<string>;
  /** Pede a pasta e grava o projeto Cargo; `null` se o aluno cancelou. */
  exportCargo?: (designSource: string, codeSource: string, workspaceId: string, designPath: string) => Promise<{ packageName: string; projectDir: string } | null>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  appendConsole: (text: string, kind?: string) => void;
  showConsole: () => void;
}

/** Os dois textos de uma tela, a partir do `.design` ou do code-behind aberto. */
async function screenSources(files: ScreenFilesHost, tab: Tab) {
  if (!tab.workspaceId || !tab.workspacePath) return null;
  const designPath = fileKind(tab.workspacePath) === "design" ? tab.workspacePath : screenDesignFor(files, tab);
  if (!designPath) return null;
  const design = await screenFileSource(files, tab.workspaceId, designPath);
  if (design === null) return null;
  const codePath = screenCodeFor(files, { ...tab, workspacePath: designPath });
  const code = codePath ? await screenFileSource(files, tab.workspaceId, codePath) : "";
  const name = designPath.split("/").pop() ?? designPath;
  return { workspaceId: tab.workspaceId, designPath, design, code: code ?? "", name };
}

function message(error: unknown): string {
  return String(error).replace(/^Error: /, "");
}

export function createScreenExecution(host: ScreenExecutionHost) {
  return {
    handles: (tab: Tab) => isScreenFile(host.files, tab),
    async run(tab: Tab): Promise<void> {
      const screen = await screenSources(host.files, tab);
      if (!screen) return;
      const { name } = screen;
      try {
        await host.start(screen.design, screen.code, screen.workspaceId, screen.designPath);
        host.setStatus(`Tela ${name} em execução numa janela própria`, "ok");
      } catch (error) {
        host.setStatus(`Não foi possível executar a tela: ${message(error)}`, "error");
        host.appendConsole(`Não foi possível executar a tela ${name}: ${message(error)}\n`, "error");
        host.showConsole();
      }
    },
    async exportCargo(tab: Tab): Promise<void> {
      const screen = await screenSources(host.files, tab);
      if (!screen || !host.exportCargo) return;
      try {
        const exported = await host.exportCargo(screen.design, screen.code, screen.workspaceId, screen.designPath);
        if (exported) host.setStatus(`Projeto Cargo "${exported.packageName}" com a tela em Slint exportado em ${exported.projectDir}`, "ok");
      } catch (error) {
        host.setStatus(`Não foi possível exportar a tela: ${message(error)}`, "error");
        host.appendConsole(`Não foi possível exportar a tela ${screen.name}:\n${message(error)}\n`, "error");
        host.showConsole();
      }
    },
  };
}
