import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { workspaceExportStatus } from "../src/workspace-export-state.ts";

const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
const api = readFileSync(new URL("../src/api.ts", import.meta.url), "utf8");
const main = readFileSync(new URL("../src/main.ts", import.meta.url), "utf8");
const workspaceRust = readFileSync(new URL("../src-tauri/src/workspace.rs", import.meta.url), "utf8");
const libRust = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

test("workspace export exposes a selectable local-copy dialog", () => {
  assert.match(html, /id="workspace-export"[^>]*disabled/);
  assert.match(html, /id="workspace-export-dialog"[^>]*aria-labelledby="workspace-export-title"/);
  assert.match(html, /id="workspace-export-files"[^>]*role="group"/);
  assert.match(html, /id="workspace-export-empty"[^>]*role="status"/);
  assert.match(html, /id="workspace-export-status"[^>]*role="status"/);
  assert.match(html, /id="workspace-export-error"[^>]*role="alert"/);
  assert.match(html, /id="status" role="status" aria-live="polite"/);
  assert.match(html, /cópia local/i);
  assert.doesNotMatch(html, /publicar|upload|nuvem/i);
});

test("export sends only checked relative paths through the typed Tauri command", () => {
  assert.match(api, /exportWorkspaceFiles[\s\S]*invoke<WorkspaceExportResult\s*\|\s*null>\("workspace_export"/);
  assert.match(main, /collectWorkspaceFiles\(workspaceState\?\.entries/);
  assert.match(main, /#workspace-export-files input:checked/);
  assert.match(main, /exportWorkspaceFiles\([^,]+,\s*relativePaths\)/);
  assert.match(main, /workspace-export-select-all/);
  assert.match(main, /workspace-export-select-none/);
  assert.match(main, /input.disabled = workspaceExportPending/);
  assert.match(main, /if \(cancelled\)[\s\S]*workspaceExportStatus\(selectedCount, "cancelled"\)/);
});

test("Rust export command is registered and uses the native folder picker", () => {
  assert.match(workspaceRust, /pub async fn workspace_export[\s\S]*relative_paths: Vec<String>[\s\S]*blocking_pick_folder/);
  assert.match(workspaceRust, /fn copy_selected_files_from_dir_with_hook[\s\S]*open_selected_file/);
  assert.match(workspaceRust, /options\.write\(true\)\.create_new\(true\)/);
  assert.match(workspaceRust, /fn same_directory_identity[\s\S]*same_file_identity/);
  assert.match(workspaceRust, /fn rollback_export[\s\S]*created_files[\s\S]*created_directories/);
  assert.match(workspaceRust, /fn directory\(&self, id: &str\)[\s\S]*try_clone/);
  assert.match(libRust, /workspace::workspace_export/);
});

test("cancelled export keeps its cancellation message after controls refresh", () => {
  assert.equal(workspaceExportStatus(1, "cancelled"), "Exportação cancelada. Nenhum arquivo foi copiado.");
  assert.equal(workspaceExportStatus(1, "selection"), "1 arquivo selecionado.");
});
