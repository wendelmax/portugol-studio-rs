/**
 * Executar uma tela (ADR 0018, seção 5): a partir do `.design` ou do
 * code-behind, junta os dois arquivos (com o que ainda não foi salvo) e abre a
 * tela numa janela própria. A saída de `escreva` e os erros vão para o console.
 */
import { fileKind } from "../file-formats.ts";
import type { Tab } from "../state/tab.ts";
import { isScreenFile, screenCodeFor, screenDesignFor, screenFileSource, type ScreenFilesHost } from "./screen-files.ts";

export interface ScreenExecutionHost {
  files: ScreenFilesHost;
  /** Inicia a sessão e abre a janela; rejeita com a mensagem do primeiro problema. */
  start: (designSource: string, codeSource: string, workspaceId: string, designPath: string) => Promise<string>;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  appendConsole: (text: string, kind?: string) => void;
  showConsole: () => void;
}

export function createScreenExecution(host: ScreenExecutionHost) {
  return {
    handles: (tab: Tab) => isScreenFile(host.files, tab),
    async run(tab: Tab): Promise<void> {
      if (!tab.workspaceId || !tab.workspacePath) return;
      const designPath = fileKind(tab.workspacePath) === "design" ? tab.workspacePath : screenDesignFor(host.files, tab);
      if (!designPath) return;
      const design = await screenFileSource(host.files, tab.workspaceId, designPath);
      if (design === null) return;
      const designTab = { ...tab, workspacePath: designPath };
      const codePath = screenCodeFor(host.files, designTab);
      const code = codePath ? await screenFileSource(host.files, tab.workspaceId, codePath) : "";
      const name = designPath.split("/").pop() ?? designPath;
      try {
        await host.start(design, code ?? "", tab.workspaceId, designPath);
        host.setStatus(`Tela ${name} em execução numa janela própria`, "ok");
      } catch (error) {
        const message = String(error).replace(/^Error: /, "");
        host.setStatus(`Não foi possível executar a tela: ${message}`, "error");
        host.appendConsole(`Não foi possível executar a tela ${name}: ${message}\n`, "error");
        host.showConsole();
      }
    },
  };
}
