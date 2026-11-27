import type { SourceLanguage } from "../document-language";
import type { Tab } from "../state/tab";
import { acceptedExtensions, classifyFile, fileExtension, fileFormat, stripFileExtension, type FileKind } from "../file-formats.ts";

export interface FileDialogFilter {
  name: string;
  extensions: string[];
}

export const FILE_FILTERS: FileDialogFilter[] = [
  {
    name: `${fileFormat("source").label} e ${fileFormat("rust").label}`,
    extensions: [...acceptedExtensions("source"), ...acceptedExtensions("rust")],
  },
  { name: fileFormat("source").label, extensions: acceptedExtensions("source") },
  { name: fileFormat("rust").label, extensions: acceptedExtensions("rust") },
];

export interface FileIODependencies {
  tabs: () => Tab[];
  replaceTabs: (tabs: Tab[]) => void;
  activeTab: () => Tab | undefined;
  activate: (id: number) => void;
  newTab: (text: string, path: string) => void;
  isDirty: (tab: Tab) => boolean;
  readFile: (path: string) => Promise<string>;
  openDialog: (options: { multiple: true; filters: FileDialogFilter[] }) => Promise<string | string[] | null>;
  saveDialog: (options: { filters: FileDialogFilter[]; defaultPath: string }) => Promise<string | null>;
  writeFile: (path: string, text: string) => Promise<void>;
  writeWorkspaceFile: (workspaceId: string, workspacePath: string, text: string) => Promise<void>;
  currentText: () => string;
  detectSourceLanguage: (path: string) => SourceLanguage;
  ensureSourceExtension: (path: string, language: SourceLanguage) => string;
  rememberRecent: (path: string) => void;
  renderTabs: () => void;
  setStatus: (message: string, kind?: "error" | "ok") => void;
  afterSave: (tab: Tab, previousLanguage: SourceLanguage, languageChanged: boolean) => void;
}

/** Coordena diálogos e persistência sem acoplar o shell aos detalhes de E/S. */
export function createFileIO(dependencies: FileIODependencies) {
  async function openFile(path: string): Promise<void> {
    const existing = dependencies.tabs().find((tab) => tab.path === path);
    if (existing) {
      dependencies.activate(existing.id);
    } else {
      const text = await dependencies.readFile(path);
      const current = dependencies.activeTab();
      const tabs = dependencies.tabs();
      if (current && !current.path && !current.exampleName && !dependencies.isDirty(current) && tabs.length === 1) {
        dependencies.replaceTabs([]);
      }
      dependencies.newTab(text, path);
    }
    dependencies.rememberRecent(path);
  }

  async function openFiles(): Promise<void> {
    try {
      const selected = await dependencies.openDialog({ multiple: true, filters: FILE_FILTERS });
      if (!selected) return;
      for (const path of Array.isArray(selected) ? selected : [selected]) {
        await openFile(path);
      }
    } catch (error) {
      dependencies.setStatus(String(error), "error");
    }
  }

  async function save(as = false): Promise<boolean> {
    const tab = dependencies.activeTab();
    if (!tab) return false;
    let path = tab.path;
    let workspaceFile = Boolean(tab.workspaceId && tab.workspacePath);
    if (!path || as) {
      const kind: FileKind = tab.language === "rust" ? "rust" : tab.language === "markup"
        ? classifyFile(tab.path ?? "")?.kind === "manifest" ? "manifest" : "design"
        : "source";
      const extension = fileExtension(kind);
      const exerciseName = tab.exercise
        ? `${stripFileExtension(tab.exercise.name).replace(/\.json$/i, "")}.${extension}`
        : undefined;
      const filters = kind === "source"
        ? [{ name: fileFormat(kind).label, extensions: acceptedExtensions(kind) }, ...FILE_FILTERS]
        : [{ name: fileFormat(kind).label, extensions: acceptedExtensions(kind) }];
      path = await dependencies.saveDialog({
        filters,
        defaultPath: tab.path ?? exerciseName ?? tab.exampleName ?? `programa.${extension}`,
      });
      if (!path) return false;
      path = dependencies.ensureSourceExtension(path, tab.language);
      workspaceFile = false;
    }

    const text = dependencies.currentText();
    try {
      if (workspaceFile && tab.workspaceId && tab.workspacePath) {
        await dependencies.writeWorkspaceFile(tab.workspaceId, tab.workspacePath, text);
      } else {
        await dependencies.writeFile(path, text);
      }
    } catch (error) {
      dependencies.setStatus(String(error), "error");
      return false;
    }

    tab.path = path;
    if (!workspaceFile) {
      tab.workspaceId = undefined;
      tab.workspacePath = undefined;
      tab.workspaceExercisePath = undefined;
    }
    const previousLanguage = tab.language;
    tab.language = dependencies.detectSourceLanguage(path);
    const languageChanged = tab.language !== previousLanguage;
    dependencies.afterSave(tab, previousLanguage, languageChanged);
    tab.saved = text;
    dependencies.rememberRecent(path);
    dependencies.renderTabs();
    dependencies.setStatus(`Salvo em ${path}`, "ok");
    return true;
  }

  return { openFile, openFiles, save };
}
