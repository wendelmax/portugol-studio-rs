/**
 * Os dois arquivos de uma tela (ADR 0018, Emenda 2): o `.design` e o
 * code-behind. Cada um é analisado com o outro — o `.fer` enxerga os tipos da
 * tela e o `.design` confere se os eventos apontam para funções que existem.
 * O texto do irmão vem da aba aberta (com o que ainda não foi salvo) ou do
 * disco.
 */
import type { Inspection } from "../api.ts";
import { fileKind } from "../file-formats.ts";
import type { Tab } from "../state/tab.ts";
import { codeBehindCandidates, designPathForCode } from "./designer-shell.ts";

export interface ScreenFilesHost {
  tabs: () => Tab[];
  /** Texto atual de uma aba (o do editor, para a aba ativa). */
  tabSource: (tab: Tab) => string;
  hasFile: (workspaceId: string | undefined, path: string) => boolean;
  readFile: (workspaceId: string, path: string) => Promise<string>;
}

export interface ScreenInspectors {
  inspectPortugolRs: (source: string) => Promise<Inspection>;
  inspectCodeBehind: (source: string, designSource: string) => Promise<Inspection>;
  inspectMarkup: (source: string, path: string, checkCode?: boolean, codeSource?: string | null) => Promise<Inspection>;
}

/** Texto de um arquivo da tela: a aba aberta ganha do disco; `null` se ele não existe. */
export async function screenFileSource(host: ScreenFilesHost, workspaceId: string, path: string): Promise<string | null> {
  const open = host.tabs().find((tab) => tab.workspaceId === workspaceId && tab.workspacePath === path);
  if (open) return host.tabSource(open);
  if (!host.hasFile(workspaceId, path)) return null;
  return host.readFile(workspaceId, path).catch(() => null);
}

/** Caminho do `.design` de um code-behind aberto, se a tela existir. */
export function screenDesignFor(host: ScreenFilesHost, tab: Tab | undefined): string | null {
  if (!tab?.workspaceId || !tab.workspacePath) return null;
  const design = designPathForCode(tab.workspacePath);
  return design && host.hasFile(tab.workspaceId, design) ? design : null;
}

/** Code-behind existente de um `.design` (`.fer`, ou `.pors` antigo). */
export function screenCodeFor(host: ScreenFilesHost, tab: Tab): string | null {
  if (!tab.workspaceId || !tab.workspacePath) return null;
  return codeBehindCandidates(tab.workspacePath).find((path) => host.hasFile(tab.workspaceId, path)) ?? null;
}

/** Se a aba é um dos arquivos de uma tela (o `.design` ou o code-behind dele). */
export function isScreenFile(host: ScreenFilesHost, tab: Tab | undefined): boolean {
  if (!tab?.workspaceId || !tab.workspacePath) return false;
  return fileKind(tab.workspacePath) === "design" || screenDesignFor(host, tab) !== null;
}

/** Inspeção de código-fonte: o code-behind de uma tela usa os tipos do `.design`. */
export async function inspectSource(host: ScreenFilesHost, inspectors: ScreenInspectors, source: string, tab: Tab | undefined): Promise<Inspection> {
  const design = screenDesignFor(host, tab);
  if (tab?.workspaceId && design) {
    const designSource = await screenFileSource(host, tab.workspaceId, design);
    if (designSource !== null) return inspectors.inspectCodeBehind(source, designSource);
  }
  return inspectors.inspectPortugolRs(source);
}

/** Inspeção de marcação: um `.design` de workspace confere os eventos com o code-behind. */
export async function inspectMarkupFile(host: ScreenFilesHost, inspectors: ScreenInspectors, source: string, tab: Tab | undefined): Promise<Inspection> {
  const path = tab?.workspacePath ?? tab?.path ?? "tela.design";
  if (!tab?.workspaceId || !tab.workspacePath || fileKind(tab.workspacePath) !== "design") {
    return inspectors.inspectMarkup(source, path);
  }
  const code = screenCodeFor(host, tab);
  const codeSource = code ? await screenFileSource(host, tab.workspaceId, code) : null;
  return inspectors.inspectMarkup(source, path, true, codeSource);
}
