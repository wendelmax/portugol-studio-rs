// Explorador do workspace: árvore de arquivos, abrir e restaurar pasta, e os
// diálogos de criar, renomear, excluir e exportar arquivos.

import { ask } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { ALL_FILE_KINDS, describeExtensions, fileKind } from "../file-formats.ts";
import { $ } from "../dom";
import { porsExerciseDocument } from "../exercise-opening";
import type { ExerciseDocument } from "../exercises";
import { setStatus } from "../shell/status";
import type { Tab, WorkspaceFileRef } from "../state/tab";
import { applyWorkspaceRenameToTabs, deleteWorkspaceFileFlow, prepareWorkspaceRename } from "../workspace-file-actions";
import {
  collectWorkspaceFiles,
  flattenWorkspaceEntries,
  groupScreens,
  screensContaining,
  workspaceFileKind,
  workspaceFileMarker,
  workspaceParentFolders,
  workspaceRenameFileName,
  workspaceSourceFileName,
  workspaceSubfolderPath,
  type WorkspaceEntry,
} from "../workspace-explorer";
import { workspaceExportStatus } from "../workspace-export-state";

type WorkspaceFile = Extract<WorkspaceEntry, { kind: "file" }>;

/** O que o explorador precisa das abas e do resto da IDE. */
export interface WorkspaceExplorerHost {
  tabs(): Tab[];
  replaceTabs(tabs: Tab[]): void;
  activeTab(): Tab | undefined;
  activeId(): number;
  homeId: number;
  activate(id: number): void;
  newTab(): void;
  openWorkspaceSource(source: string, path: string, workspaceFile: WorkspaceFileRef): Tab;
  openExercise(document: ExerciseDocument, workspaceExercise: WorkspaceFileRef & { exercise: true }): void;
  isDirty(tab: Tab): boolean;
  save(): Promise<boolean>;
  renderTabs(): void;
  setExplorerVisible(visible: boolean): void;
  /** Detecção de Cargo.toml terminada para uma aba Rust (`checked` é falso se ela falhou). */
  cargoProjectDetected(tab: Tab, checked: boolean): void;
  recentFiles: {
    remember(path: string): void;
    rename(oldPath: string, newPath: string): void;
    forget(path: string): void;
  };
}

export function mountWorkspaceExplorer(host: WorkspaceExplorerHost) {
  let workspaceState: api.WorkspaceListing | null = null;
  let workspaceLoading = false;
  let workspaceRequest = 0;
  let selectedWorkspacePath: string | null = null;
  let selectedWorkspaceFolder: string | null = null;
  const expandedWorkspaceFolders = new Set<string>();
  let workspaceError: string | null = null;
  let workspaceFileCreationPending = false;
  let workspaceExportPending = false;
  let workspaceRenameTarget: WorkspaceFile | null = null;
  let workspaceRenamePending = false;
  let workspaceDeleteTarget: WorkspaceFile | null = null;
  let workspaceDeletePending = false;

  const saveTab = async (tab: Tab) => {
    host.activate(tab.id);
    try {
      return await host.save();
    } catch {
      return false;
    }
  };

  const isOpenIn = (tab: Tab, workspaceId: string | undefined, path: string | undefined) =>
    tab.workspaceId === workspaceId && (tab.workspacePath === path || tab.workspaceExercisePath === path);

  function renderWorkspaceExplorer() {
    $("workspace-explorer").setAttribute("aria-busy", String(workspaceLoading));
    const tree = $<HTMLUListElement>("workspace-tree");
    const message = $("workspace-tree-state");
    const name = $("workspace-name");
    const openButtons = [$<HTMLButtonElement>("home-open-folder"), $<HTMLButtonElement>("workspace-open-folder")];
    openButtons.forEach((button) => { button.disabled = workspaceLoading; });
    $<HTMLButtonElement>("workspace-refresh").disabled = !workspaceState || workspaceLoading;
    $<HTMLButtonElement>("workspace-export").disabled = !workspaceState || workspaceLoading || workspaceExportPending;
    $<HTMLButtonElement>("workspace-new-file").disabled = !workspaceState || workspaceLoading;
    name.hidden = !workspaceState;
    name.textContent = workspaceState?.name ?? "";
    const titlebarWorkspaceName = document.querySelector<HTMLElement>("#titlebar-workspace-name");
    if (titlebarWorkspaceName) titlebarWorkspaceName.textContent = workspaceState?.name ?? "Sem projeto";
    const titlebarWorkspaceBtn = document.querySelector<HTMLButtonElement>("#titlebar-workspace");
    if (titlebarWorkspaceBtn) {
      titlebarWorkspaceBtn.title = workspaceState
        ? `Projeto aberto: ${workspaceState.name} (clique para alternar o Explorador)`
        : "Abrir pasta de projeto ou alternar o Explorador (Ctrl+Shift+E)";
    }

    const explorerEntries = groupScreens(workspaceState?.entries ?? []);
    if (selectedWorkspacePath) {
      for (const screen of screensContaining(explorerEntries, selectedWorkspacePath)) expandedWorkspaceFolders.add(screen);
    }
    const rows = flattenWorkspaceEntries(explorerEntries, expandedWorkspaceFolders);
    tree.setAttribute("aria-label", workspaceState ? `Arquivos em ${workspaceState.name}` : "Arquivos do projeto");
    const items = rows.map(({ entry, depth, position, setSize }) => {
      const item = document.createElement("li");
      item.setAttribute("role", "none");
      const button = document.createElement("button");
      const isScreen = entry.kind === "screen";
      const isFolder = entry.kind !== "file";
      const isExpanded = isFolder && expandedWorkspaceFolders.has(entry.path);
      const marker = document.createElement("span");
      marker.className = "workspace-tree-marker";
      marker.setAttribute("aria-hidden", "true");
      marker.textContent = isFolder ? (isExpanded ? "▾" : "▸") : workspaceFileMarker(entry.path);
      const label = document.createElement("span");
      label.className = "workspace-tree-label";
      label.textContent = entry.name;
      button.type = "button";
      button.className = `workspace-tree-item${isFolder ? " is-folder" : " is-file"}${isScreen ? " is-screen" : ""}`;
      button.dataset.path = entry.path;
      button.dataset.depth = String(depth);
      button.setAttribute("role", "treeitem");
      button.setAttribute("aria-level", String(depth + 1));
      button.setAttribute("aria-posinset", String(position));
      button.setAttribute("aria-setsize", String(setSize));
      button.setAttribute("aria-selected", String(
        isScreen ? selectedWorkspacePath === entry.design.path : !isFolder && selectedWorkspacePath === entry.path,
      ));
      button.style.paddingInlineStart = "8px";
      if (entry.kind === "screen") {
        // Nó de tela: abre o desenho e mostra o .design e o .pors dentro dele.
        const screen = entry;
        button.setAttribute("aria-expanded", String(isExpanded));
        button.setAttribute("aria-label", `${screen.name}, tela${screen.code ? " com código" : " sem código"}${isExpanded ? ", expandida" : ", recolhida"}`);
        button.title = screen.code
          ? `Tela ${screen.name}: ${screen.design.name} (desenho) e ${screen.code.name} (código)`
          : `Tela ${screen.name}: ${screen.design.name} (desenho); o código é criado no primeiro evento`;
        button.onclick = () => {
          selectedWorkspacePath = screen.design.path;
          const parents = workspaceParentFolders(screen.design.path);
          selectedWorkspaceFolder = parents.length ? parents[parents.length - 1] : null;
          expandedWorkspaceFolders.add(screen.path);
          renderWorkspaceExplorer();
          void openWorkspaceFile(screen.design).catch((error) => {
            workspaceError = String(error);
            renderWorkspaceExplorer();
            setStatus(String(error), "error");
          });
        };
      } else if (isFolder) {
        button.setAttribute("aria-expanded", String(isExpanded));
        button.setAttribute("aria-label", `${entry.name}, pasta${isExpanded ? ", expandida" : ", recolhida"}`);
        button.onclick = () => {
          selectedWorkspaceFolder = entry.path;
          if (expandedWorkspaceFolders.has(entry.path)) expandedWorkspaceFolders.delete(entry.path);
          else expandedWorkspaceFolders.add(entry.path);
          renderWorkspaceExplorer();
          tree.querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(entry.path)}"]`)?.focus();
        };
      } else {
        button.setAttribute("aria-label", `${entry.name}, arquivo`);
        button.onclick = () => {
          selectedWorkspacePath = entry.path;
          const parents = workspaceParentFolders(entry.path);
          selectedWorkspaceFolder = parents.length ? parents[parents.length - 1] : null;
          renderWorkspaceExplorer();
          void openWorkspaceFile(entry).catch((error) => {
            workspaceError = String(error);
            renderWorkspaceExplorer();
            setStatus(String(error), "error");
          });
        };
      }
      button.append(marker, label);
      if (isScreen) {
        const badge = document.createElement("span");
        badge.className = "workspace-tree-badge";
        badge.textContent = "tela";
        badge.setAttribute("aria-hidden", "true");
        button.append(badge);
      }
      const row = document.createElement("div");
      row.className = "workspace-tree-row";
      row.style.paddingInlineStart = `${depth * 16}px`;
      row.append(button);
      if (entry.kind === "file") {
        const actions = document.createElement("div");
        actions.className = "workspace-file-actions";
        const renameButton = document.createElement("button");
        renameButton.type = "button";
        renameButton.textContent = "✎";
        renameButton.title = `Renomear ${entry.name}`;
        renameButton.setAttribute("aria-label", `Renomear ${entry.name}`);
        renameButton.dataset.action = "workspace-rename-file";
        renameButton.onclick = (event) => {
          event.stopPropagation();
          openWorkspaceRenameDialog(entry);
        };
        actions.append(renameButton);
        const deleteButton = document.createElement("button");
        deleteButton.type = "button";
        deleteButton.textContent = "🗑";
        deleteButton.title = `Excluir ${entry.name}`;
        deleteButton.setAttribute("aria-label", `Excluir ${entry.name}`);
        deleteButton.dataset.action = "workspace-delete-file";
        deleteButton.onclick = (event) => {
          event.stopPropagation();
          openWorkspaceDeleteDialog(entry);
        };
        actions.append(deleteButton);
        actions.dataset.testid = "file-actions";
        row.append(actions);
      }
      item.append(row);
      return item;
    });
    tree.replaceChildren(...items);
    tree.hidden = rows.length === 0;

    if (workspaceLoading) {
      message.textContent = "Abrindo pasta…";
      message.hidden = false;
    } else if (workspaceError) {
      message.textContent = workspaceError;
      message.hidden = false;
    } else if (!workspaceState) {
      message.textContent = "Abra uma pasta para navegar pelos arquivos do projeto.";
      message.hidden = false;
    } else if (rows.length === 0) {
      message.textContent = `Esta pasta não contém arquivos ${describeExtensions(ALL_FILE_KINDS, "ou")}.`;
      message.hidden = false;
    } else {
      message.textContent = "";
      message.hidden = true;
    }
  }

  async function openWorkspaceFolder() {
    if (workspaceLoading) return;
    const request = ++workspaceRequest;
    workspaceLoading = true;
    workspaceError = null;
    renderWorkspaceExplorer();
    try {
      const selected = await api.openWorkspace();
      if (request !== workspaceRequest || !selected) return;
      workspaceState = selected;
      selectedWorkspacePath = null;
      selectedWorkspaceFolder = null;
      expandedWorkspaceFolders.clear();
      workspaceError = null;
      host.setExplorerVisible(true);
      if (!host.activeTab()) {
        const tabs = host.tabs();
        const previousTab = tabs[tabs.length - 1];
        if (previousTab) host.activate(previousTab.id);
        else host.newTab();
      }
      renderWorkspaceExplorer();
      setStatus(`Pasta aberta: ${selected.name}`, "ok");
    } catch (error) {
      if (request === workspaceRequest) {
        workspaceError = String(error);
        renderWorkspaceExplorer();
        setStatus(String(error), "error");
      }
    } finally {
      if (request === workspaceRequest) {
        workspaceLoading = false;
        renderWorkspaceExplorer();
      }
    }
  }

  /** Diálogo "Novo projeto desktop": nome do projeto, depois o local no seletor do sistema. */
  function openDesktopProjectDialog() {
    const dialog = $<HTMLDialogElement>("desktop-project-dialog");
    $<HTMLFormElement>("desktop-project-form").reset();
    showDesktopProjectError(null);
    dialog.showModal();
    $<HTMLInputElement>("desktop-project-name").focus();
  }

  function showDesktopProjectError(message: string | null) {
    const error = $("desktop-project-error");
    error.textContent = message ?? "";
    error.hidden = !message;
  }

  async function submitDesktopProjectDialog(event: SubmitEvent) {
    event.preventDefault();
    if (workspaceLoading) return;
    const name = $<HTMLInputElement>("desktop-project-name").value.trim();
    if (!name) {
      showDesktopProjectError("Informe o nome do projeto.");
      return;
    }
    const submit = $<HTMLButtonElement>("desktop-project-submit");
    const request = ++workspaceRequest;
    submit.disabled = true;
    showDesktopProjectError(null);
    try {
      const created = await api.createDesktopProject(name);
      if (request !== workspaceRequest || !created) return;
      $<HTMLDialogElement>("desktop-project-dialog").close();
      workspaceState = created.listing;
      workspaceError = null;
      selectedWorkspacePath = created.designPath;
      selectedWorkspaceFolder = null;
      expandedWorkspaceFolders.clear();
      host.setExplorerVisible(true);
      renderWorkspaceExplorer();
      await openWorkspaceFile({
        kind: "file",
        name: created.designPath.split("/").pop() ?? created.designPath,
        path: created.designPath,
      });
      setStatus(`Projeto criado: ${created.listing.name}`, "ok");
    } catch (message) {
      if (request === workspaceRequest) {
        showDesktopProjectError(typeof message === "string" ? message : String(message));
      }
    } finally {
      submit.disabled = false;
    }
  }

  async function refreshWorkspace() {
    const selected = workspaceState;
    if (!selected || workspaceLoading) return;
    const request = ++workspaceRequest;
    workspaceLoading = true;
    workspaceError = null;
    renderWorkspaceExplorer();
    try {
      const refreshed = await api.refreshWorkspace(selected.id);
      if (request !== workspaceRequest || workspaceState?.id !== selected.id) return;
      workspaceState = refreshed;
      workspaceError = null;
      renderWorkspaceExplorer();
      setStatus(`Projeto atualizado: ${refreshed.name}`, "ok");
    } catch (error) {
      if (request === workspaceRequest) {
        workspaceError = String(error);
        renderWorkspaceExplorer();
        setStatus(String(error), "error");
      }
    } finally {
      if (request === workspaceRequest) {
        workspaceLoading = false;
        renderWorkspaceExplorer();
      }
    }
  }

  async function restoreWorkspaceOnStartup() {
    const request = ++workspaceRequest;
    try {
      const restored = await api.restoreWorkspace();
      if (request !== workspaceRequest || !restored) return;
      workspaceState = restored;
      selectedWorkspacePath = null;
      expandedWorkspaceFolders.clear();
      host.setExplorerVisible(true);
      renderWorkspaceExplorer();
      setStatus(`Última pasta reaberta: ${restored.name}`, "ok");
    } catch (error) {
      if (request === workspaceRequest) setStatus(String(error), "error");
    }
  }

  async function openWorkspaceFile(entry: WorkspaceFile) {
    const selected = workspaceState;
    if (!selected) return;
    const kind = workspaceFileKind(entry.path);
    if (!kind) return;
    if (kind === "exercise") {
      const existingExercise = host.tabs().find((tab) =>
        tab.workspaceId === selected.id && tab.workspaceExercisePath === entry.path,
      );
      if (existingExercise) {
        host.activate(existingExercise.id);
        return;
      }
      const loaded = await api.readWorkspaceFile(selected.id, entry.path);
      if (workspaceState?.id !== selected.id) return;
      const summary = await api.inspectPorsExercise(loaded.source);
      if (workspaceState?.id !== selected.id) return;
      const exerciseDoc = porsExerciseDocument(
        summary,
        summary.title || loaded.path.split("/").pop() || entry.name,
        loaded.source,
      );
      host.openExercise(exerciseDoc, {
        id: selected.id,
        path: loaded.relativePath,
        preferenceKey: selected.preferenceKey,
        exercise: true,
      });
      setStatus(`Exercício aberto: ${entry.path}`, "ok");
      return;
    }

    const existing = host.tabs().find((tab) => tab.workspaceId === selected.id && tab.workspacePath === entry.path);
    if (existing) {
      host.activate(existing.id);
      return;
    }
    const loaded = await api.readWorkspaceFile(selected.id, entry.path);
    if (workspaceState?.id !== selected.id) return;
    const current = host.activeTab();
    if (current && !current.path && !current.exampleName && !host.isDirty(current) && host.tabs().length === 1) {
      host.replaceTabs([]);
    }
    const tab = host.openWorkspaceSource(loaded.source, loaded.path, {
      id: selected.id,
      path: loaded.relativePath,
      preferenceKey: selected.preferenceKey,
    });
    tab.workspacePath = loaded.relativePath;
    if (tab.language === "rust") {
      void api.workspaceCargoProjectAvailable(selected.id, loaded.relativePath).then((available) => {
        tab.cargoProject = available;
        host.cargoProjectDetected(tab, true);
      }).catch((error: unknown) => {
        tab.cargoProject = false;
        console.error("Não foi possível verificar Cargo.toml:", error);
        host.cargoProjectDetected(tab, false);
      });
    }
    host.recentFiles.remember(loaded.path);
    setStatus(`Aberto: ${entry.path}`, "ok");
  }

  function showWorkspaceCreateError(message: string | null) {
    const error = $("workspace-create-error");
    error.textContent = message ?? "";
    error.hidden = !message;
    $("workspace-create-name").setAttribute("aria-invalid", String(Boolean(message)));
  }

  function showWorkspaceRenameError(message: string | null) {
    const error = $("workspace-rename-error");
    error.textContent = message ?? "";
    error.hidden = !message;
    $("workspace-rename-name").setAttribute("aria-invalid", String(Boolean(message)));
  }

  function openWorkspaceRenameDialog(entry: WorkspaceFile) {
    if (!workspaceState || workspaceRenamePending) return;
    workspaceRenameTarget = entry;
    const input = $<HTMLInputElement>("workspace-rename-name");
    showWorkspaceRenameError(null);
    $<HTMLFormElement>("workspace-rename-form").reset();
    input.value = entry.name;
    $<HTMLDialogElement>("workspace-rename-dialog").showModal();
    const dot = entry.name.lastIndexOf(".");
    input.setSelectionRange(0, dot > 0 ? dot : entry.name.length);
    input.focus();
  }

  async function submitWorkspaceRenameDialog(event: SubmitEvent) {
    event.preventDefault();
    const workspace = workspaceState;
    const original = workspaceRenameTarget;
    if (!workspace || !original) return;

    const input = $<HTMLInputElement>("workspace-rename-name");
    let newName: string;
    try {
      newName = workspaceRenameFileName(input.value, original.path);
    } catch (error) {
      showWorkspaceRenameError(String(error).replace(/^Error: /, ""));
      input.focus();
      return;
    }
    if (newName === original.name) {
      $<HTMLDialogElement>("workspace-rename-dialog").close();
      workspaceRenameTarget = null;
      return;
    }

    const submit = $<HTMLButtonElement>("workspace-rename-submit");
    const cancel = $<HTMLButtonElement>("workspace-rename-cancel");
    workspaceRenamePending = true;
    submit.disabled = true;
    cancel.disabled = true;
    showWorkspaceRenameError(null);
    try {
      const openTabs = host.tabs().filter((tab) => isOpenIn(tab, workspace.id, original.path));
      const previousActive = host.activeId();
      const preparation = await prepareWorkspaceRename({
        tabs: openTabs,
        sourcePath: original.path,
        isDirty: host.isDirty,
        confirmSave: () => ask(`"${original.name}" tem alterações não salvas. Salvar antes de renomear?`, {
          title: "Renomear arquivo",
          kind: "warning",
          okLabel: "Salvar e renomear",
          cancelLabel: "Cancelar",
        }),
        saveTab,
      });
      host.activate(previousActive);
      if (preparation === "cancelled") return;
      if (preparation === "save-failed") {
        showWorkspaceRenameError("Não foi possível salvar. O arquivo não foi renomeado.");
        return;
      }

      const renamed = await api.renameWorkspaceFile(workspace.id, original.path, newName);
      if (workspaceState?.id !== workspace.id) return;
      for (const oldPath of applyWorkspaceRenameToTabs(openTabs, original.path, renamed.file)) {
        host.recentFiles.rename(oldPath, renamed.file.path);
      }
      workspaceState = renamed.listing;
      workspaceError = null;
      selectedWorkspacePath = renamed.file.relativePath;
      renderWorkspaceExplorer();
      host.renderTabs();
      $<HTMLDialogElement>("workspace-rename-dialog").close();
      $<HTMLFormElement>("workspace-rename-form").reset();
      workspaceRenameTarget = null;
      if (fileKind(renamed.file.relativePath) !== "lesson") {
        host.recentFiles.remember(renamed.file.path);
      }
      setStatus(`Renomeado: ${newName}`, "ok");
      queueMicrotask(() => $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(renamed.file.relativePath)}"]`)?.focus());
    } catch (error) {
      const message = String(error).replace(/^Error: /, "");
      showWorkspaceRenameError(message);
      input.focus();
    } finally {
      workspaceRenamePending = false;
      submit.disabled = false;
      cancel.disabled = false;
    }
  }

  function showWorkspaceDeleteError(message: string | null) {
    const error = $("workspace-delete-error");
    error.textContent = message ?? "";
    error.hidden = !message;
  }

  function hasDirtyTabsFor(entry: WorkspaceFile | null, workspaceId: string | undefined) {
    return host.tabs().some((tab) => isOpenIn(tab, workspaceId, entry?.path) && host.isDirty(tab));
  }

  function updateWorkspaceDeleteDialog(entry: WorkspaceFile, workspaceId: string) {
    const dirty = hasDirtyTabsFor(entry, workspaceId);
    $("workspace-delete-title").textContent = `Excluir ${entry.name}?`;
    const warning = $("workspace-delete-warning");
    warning.textContent = dirty
      ? "Há alterações não salvas em abas abertas. Salvar mantém o arquivo; excluir com descarte fecha essas abas."
      : "";
    warning.hidden = !dirty;
    $<HTMLButtonElement>("workspace-delete-save").hidden = !dirty;
    $<HTMLButtonElement>("workspace-delete-submit").textContent = dirty
      ? "Excluir e descartar alterações"
      : "Excluir arquivo";
  }

  function openWorkspaceDeleteDialog(entry: WorkspaceFile) {
    if (!workspaceState || workspaceDeletePending) return;
    workspaceDeleteTarget = entry;
    showWorkspaceDeleteError(null);
    updateWorkspaceDeleteDialog(entry, workspaceState.id);
    $<HTMLDialogElement>("workspace-delete-dialog").showModal();
    $("workspace-delete-cancel").focus();
  }

  async function chooseWorkspaceDeleteAction(choice: "save" | "delete" | "discard") {
    const workspace = workspaceState;
    const target = workspaceDeleteTarget;
    if (!workspace || !target || workspaceDeletePending) return;

    const buttons = [
      $<HTMLButtonElement>("workspace-delete-save"),
      $<HTMLButtonElement>("workspace-delete-submit"),
      $<HTMLButtonElement>("workspace-delete-cancel"),
    ];
    workspaceDeletePending = true;
    buttons.forEach((button) => { button.disabled = true; });
    showWorkspaceDeleteError(null);
    try {
      const previousActive = host.activeId();
      const result = await deleteWorkspaceFileFlow({
        tabs: host.tabs(),
        workspaceId: workspace.id,
        filePath: target.path,
        choice,
        isDirty: host.isDirty,
        saveTab,
        deleteFile: () => api.deleteWorkspaceFile(workspace.id, target.path),
      });
      if (choice === "save") host.activate(previousActive);
      if (result.status === "cancelled") return;
      if (result.status === "save-failed") {
        showWorkspaceDeleteError("Não foi possível salvar todas as alterações. O arquivo foi mantido.");
        updateWorkspaceDeleteDialog(target, workspace.id);
        return;
      }
      if (result.status === "needs-decision") {
        showWorkspaceDeleteError("Há alterações não salvas. Escolha salvar ou descartar antes de excluir.");
        updateWorkspaceDeleteDialog(target, workspace.id);
        return;
      }
      if (result.status === "saved") {
        $<HTMLDialogElement>("workspace-delete-dialog").close();
        workspaceDeleteTarget = null;
        setStatus(`Alterações salvas; arquivo mantido: ${target.name}`, "ok");
        return;
      }

      if (workspaceState?.id !== workspace.id) return;
      const deleted = result.deletion;
      workspaceState = deleted.listing;
      workspaceError = null;
      if (selectedWorkspacePath === target.path) selectedWorkspacePath = null;
      host.recentFiles.forget(deleted.file.path);
      host.replaceTabs(result.remainingTabs);
      const activeWasDeleted = result.deletedTabs.some((tab) => tab.id === host.activeId());
      renderWorkspaceExplorer();
      if (activeWasDeleted) {
        const tabs = host.tabs();
        host.activate(tabs[tabs.length - 1]?.id ?? host.homeId);
      } else {
        host.renderTabs();
      }
      $<HTMLDialogElement>("workspace-delete-dialog").close();
      workspaceDeleteTarget = null;
      setStatus(`Arquivo excluído: ${target.name}`, "ok");
      queueMicrotask(() => {
        const first = $("workspace-tree").querySelector<HTMLButtonElement>("[role=treeitem]");
        if (first) first.focus();
        else $("workspace-new-file").focus();
      });
    } catch (error) {
      const message = String(error).replace(/^Error: /, "");
      showWorkspaceDeleteError(message);
      setStatus(message, "error");
    } finally {
      workspaceDeletePending = false;
      buttons.forEach((button) => { button.disabled = false; });
    }
  }

  function openWorkspaceCreateDialog() {
    if (!workspaceState) return;
    const dialog = $<HTMLDialogElement>("workspace-create-dialog");
    showWorkspaceCreateError(null);
    $<HTMLFormElement>("workspace-create-form").reset();
    const folderInput = $<HTMLInputElement>("workspace-create-folder");
    folderInput.value = selectedWorkspaceFolder ?? "";
    dialog.showModal();
    $("workspace-create-name").focus();
  }

  function updateWorkspaceExportSelection() {
    const selectedCount = document.querySelectorAll<HTMLInputElement>("#workspace-export-files input:checked").length;
    document.querySelectorAll<HTMLInputElement>("#workspace-export-files input").forEach((input) => {
      input.disabled = workspaceExportPending;
    });
    const submit = $<HTMLButtonElement>("workspace-export-submit");
    submit.disabled = workspaceExportPending || selectedCount === 0;
    $<HTMLButtonElement>("workspace-export-select-all").disabled = workspaceExportPending;
    $<HTMLButtonElement>("workspace-export-select-none").disabled = workspaceExportPending;
    $<HTMLButtonElement>("workspace-export-cancel").disabled = workspaceExportPending;
    if (!workspaceExportPending) $("workspace-export-status").textContent = workspaceExportStatus(selectedCount);
  }

  function renderWorkspaceExportSelection() {
    const files = collectWorkspaceFiles(workspaceState?.entries ?? []);
    const list = $("workspace-export-files");
    list.replaceChildren();
    for (const [index, entry] of files.entries()) {
      const label = document.createElement("label");
      label.className = "workspace-export-file";
      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.value = entry.path;
      checkbox.checked = true;
      checkbox.id = `workspace-export-file-${index}`;
      checkbox.addEventListener("change", updateWorkspaceExportSelection);
      const name = document.createElement("span");
      name.textContent = entry.path;
      label.htmlFor = checkbox.id;
      label.append(checkbox, name);
      list.append(label);
    }
    $("workspace-export-empty").hidden = files.length > 0;
    $<HTMLButtonElement>("workspace-export-select-all").hidden = files.length === 0;
    $<HTMLButtonElement>("workspace-export-select-none").hidden = files.length === 0;
    updateWorkspaceExportSelection();
  }

  function openWorkspaceExportDialog() {
    if (!workspaceState || workspaceExportPending) return;
    $("workspace-export-error").hidden = true;
    $("workspace-export-error").textContent = "";
    renderWorkspaceExportSelection();
    const dialog = $<HTMLDialogElement>("workspace-export-dialog");
    dialog.showModal();
    document.querySelector<HTMLInputElement>("#workspace-export-files input")?.focus();
  }

  async function submitWorkspaceExport() {
    const workspace = workspaceState;
    if (!workspace || workspaceExportPending) return;
    const relativePaths = [...document.querySelectorAll<HTMLInputElement>("#workspace-export-files input:checked")]
      .map((input) => input.value);
    const error = $("workspace-export-error");
    error.hidden = true;
    error.textContent = "";
    if (relativePaths.length === 0) {
      error.textContent = "Selecione ao menos um arquivo para exportar.";
      error.hidden = false;
      return;
    }

    workspaceExportPending = true;
    $("workspace-export-dialog").setAttribute("aria-busy", "true");
    $("workspace-export-status").textContent = "Escolha uma pasta de destino no seletor do sistema…";
    updateWorkspaceExportSelection();
    renderWorkspaceExplorer();
    let cancelled = false;
    try {
      const result = await api.exportWorkspaceFiles(workspace.id, relativePaths);
      if (!result) {
        cancelled = true;
        return;
      }
      $<HTMLDialogElement>("workspace-export-dialog").close();
      setStatus(`${result.fileCount} ${result.fileCount === 1 ? "arquivo exportado" : "arquivos exportados"} para a pasta “${result.projectName}”.`, "ok");
    } catch (message) {
      error.textContent = typeof message === "string" ? message : String(message);
      error.hidden = false;
    } finally {
      workspaceExportPending = false;
      $("workspace-export-dialog").removeAttribute("aria-busy");
      updateWorkspaceExportSelection();
      if (cancelled) {
        const selectedCount = document.querySelectorAll<HTMLInputElement>("#workspace-export-files input:checked").length;
        $("workspace-export-status").textContent = workspaceExportStatus(selectedCount, "cancelled");
      }
      renderWorkspaceExplorer();
    }
  }

  async function submitWorkspaceCreateDialog(event: SubmitEvent) {
    event.preventDefault();
    const selected = workspaceState;
    if (!selected) return;

    const name = $<HTMLInputElement>("workspace-create-name");
    const folderInput = $<HTMLInputElement>("workspace-create-folder");
    const language = $<HTMLSelectElement>("workspace-create-language").value as "source" | "rust";
    let fileName: string;
    let folder: string;
    try {
      fileName = workspaceSourceFileName(name.value, language);
      folder = workspaceSubfolderPath(folderInput.value);
    } catch (error) {
      showWorkspaceCreateError(String(error).replace(/^Error: /, ""));
      if (folderInput.value.trim() && /[\\/]|\.\./.test(folderInput.value)) folderInput.focus();
      else name.focus();
      return;
    }

    const submit = $<HTMLButtonElement>("workspace-create-submit");
    const cancel = $<HTMLButtonElement>("workspace-create-cancel");
    workspaceFileCreationPending = true;
    submit.disabled = true;
    cancel.disabled = true;
    showWorkspaceCreateError(null);
    try {
      const refreshed = await api.createWorkspaceFile(selected.id, fileName, folder || undefined);
      if (workspaceState?.id !== selected.id) return;
      const createdPath = folder ? `${folder}/${fileName}` : fileName;
      workspaceState = refreshed;
      workspaceError = null;
      selectedWorkspacePath = createdPath;
      selectedWorkspaceFolder = folder || null;
      for (const parent of workspaceParentFolders(createdPath)) {
        expandedWorkspaceFolders.add(parent);
      }
      host.setExplorerVisible(true);
      renderWorkspaceExplorer();
      $<HTMLDialogElement>("workspace-create-dialog").close();
      $<HTMLFormElement>("workspace-create-form").reset();
      const created = collectWorkspaceFiles(refreshed.entries).find((entry) => entry.path === createdPath);
      if (created) await openWorkspaceFile(created);
      else setStatus(`Arquivo criado: ${createdPath}`, "ok");
    } catch (error) {
      const message = String(error).replace(/^Error: /, "");
      if ($<HTMLDialogElement>("workspace-create-dialog").open) {
        showWorkspaceCreateError(message);
        name.focus();
      } else {
        workspaceError = message;
        renderWorkspaceExplorer();
        setStatus(message, "error");
      }
    } finally {
      workspaceFileCreationPending = false;
      submit.disabled = false;
      cancel.disabled = false;
    }
  }

  /** Abre a pasta quando não há workspace; senão mostra ou oculta o explorador. */
  function toggleExplorer() {
    if (!workspaceState) {
      void openWorkspaceFolder();
      return;
    }
    host.setExplorerVisible(Boolean($("workspace-explorer").hidden));
  }

  $("home-open-folder").onclick = () => void openWorkspaceFolder();
  $("home-new-project").onclick = openDesktopProjectDialog;
  $("desktop-project-form").addEventListener("submit", (event) => void submitDesktopProjectDialog(event as SubmitEvent));
  $<HTMLButtonElement>("desktop-project-cancel").onclick = () => $<HTMLDialogElement>("desktop-project-dialog").close();
  $("workspace-open-folder").onclick = () => void openWorkspaceFolder();
  $("workspace-refresh").onclick = () => void refreshWorkspace();
  $("workspace-export").onclick = openWorkspaceExportDialog;
  $("workspace-export-select-all").onclick = () => {
    document.querySelectorAll<HTMLInputElement>("#workspace-export-files input").forEach((input) => { input.checked = true; });
    updateWorkspaceExportSelection();
  };
  $("workspace-export-select-none").onclick = () => {
    document.querySelectorAll<HTMLInputElement>("#workspace-export-files input").forEach((input) => { input.checked = false; });
    updateWorkspaceExportSelection();
  };
  $("workspace-export-submit").onclick = () => void submitWorkspaceExport();
  $<HTMLButtonElement>("workspace-export-cancel").onclick = () => $<HTMLDialogElement>("workspace-export-dialog").close();
  $<HTMLDialogElement>("workspace-export-dialog").addEventListener("cancel", (event) => {
    if (workspaceExportPending) event.preventDefault();
  });
  $("workspace-new-file").onclick = openWorkspaceCreateDialog;
  $("workspace-create-form").addEventListener("submit", (event) => void submitWorkspaceCreateDialog(event as SubmitEvent));
  $("workspace-create-dialog").addEventListener("cancel", (event) => {
    if (workspaceFileCreationPending) event.preventDefault();
  });
  $("workspace-create-cancel").onclick = () => {
    $<HTMLDialogElement>("workspace-create-dialog").close();
    $<HTMLFormElement>("workspace-create-form").reset();
    showWorkspaceCreateError(null);
  };
  $("workspace-rename-form").addEventListener("submit", (event) => void submitWorkspaceRenameDialog(event as SubmitEvent));
  $("workspace-rename-dialog").addEventListener("cancel", (event) => {
    if (workspaceRenamePending) event.preventDefault();
  });
  $("workspace-rename-cancel").onclick = () => {
    $<HTMLDialogElement>("workspace-rename-dialog").close();
    $<HTMLFormElement>("workspace-rename-form").reset();
    workspaceRenameTarget = null;
    showWorkspaceRenameError(null);
  };
  $<HTMLButtonElement>("workspace-delete-save").onclick = () => void chooseWorkspaceDeleteAction("save");
  $<HTMLButtonElement>("workspace-delete-submit").onclick = () => {
    const choice = hasDirtyTabsFor(workspaceDeleteTarget, workspaceState?.id) ? "discard" : "delete";
    void chooseWorkspaceDeleteAction(choice);
  };
  $("workspace-delete-dialog").addEventListener("cancel", (event) => {
    if (workspaceDeletePending) event.preventDefault();
    else workspaceDeleteTarget = null;
  });
  $<HTMLButtonElement>("workspace-delete-cancel").onclick = () => {
    if (workspaceDeletePending) return;
    $<HTMLDialogElement>("workspace-delete-dialog").close();
    workspaceDeleteTarget = null;
    showWorkspaceDeleteError(null);
  };
  $("btn-explorer").onclick = toggleExplorer;
  const titlebarWorkspace = document.querySelector<HTMLButtonElement>("#titlebar-workspace");
  if (titlebarWorkspace) titlebarWorkspace.onclick = toggleExplorer;

  $("workspace-tree").addEventListener("keydown", (event) => {
    if (!(event instanceof KeyboardEvent)) return;
    const current = event.target instanceof HTMLButtonElement ? event.target : null;
    if (!current?.matches("[role=treeitem]")) return;
    const items = [...$("workspace-tree").querySelectorAll<HTMLButtonElement>("[role=treeitem]")];
    const index = items.indexOf(current);
    const focus = (item?: HTMLButtonElement) => {
      if (item) {
        event.preventDefault();
        item.focus();
      }
    };
    if (event.key === "ArrowDown") focus(items[index + 1]);
    else if (event.key === "ArrowUp") focus(items[index - 1]);
    else if (event.key === "Home") focus(items[0]);
    else if (event.key === "End") focus(items[items.length - 1]);
    else if (event.key === "ArrowRight" && current.classList.contains("is-folder")) {
      const path = current.dataset.path!;
      if (!expandedWorkspaceFolders.has(path)) {
        event.preventDefault();
        expandedWorkspaceFolders.add(path);
        renderWorkspaceExplorer();
        $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(path)}"]`)?.focus();
      } else if (items[index + 1]?.dataset.depth === String(Number(current.dataset.depth) + 1)) {
        focus(items[index + 1]);
      }
    } else if (event.key === "ArrowLeft") {
      const path = current.dataset.path!;
      if (current.classList.contains("is-folder") && expandedWorkspaceFolders.has(path)) {
        event.preventDefault();
        expandedWorkspaceFolders.delete(path);
        renderWorkspaceExplorer();
        $("workspace-tree").querySelector<HTMLButtonElement>(`[data-path="${CSS.escape(path)}"]`)?.focus();
      } else {
        const depth = Number(current.dataset.depth);
        focus(items.slice(0, index).reverse().find((item) => Number(item.dataset.depth) === depth - 1));
      }
    }
  });

  return {
    render: renderWorkspaceExplorer,
    restore: restoreWorkspaceOnStartup,
    openFolder: openWorkspaceFolder,
    toggleExplorer,
    /** Marca o arquivo como selecionado na árvore, se pertencer ao workspace aberto. */
    select(workspaceId: string | undefined, path: string | undefined) {
      if (workspaceId === workspaceState?.id && path) {
        selectedWorkspacePath = path;
        renderWorkspaceExplorer();
      }
    },
  };
}
