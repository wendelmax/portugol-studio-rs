// Janela de uma tela desktop em execução (ADR 0018, seção 5). Pede o estado
// ao abrir, desenha a tela com controles de verdade e manda cada evento para a
// sessão; a resposta traz a janela atualizada pela função do aluno.
import { invoke } from "@tauri-apps/api/core";
import type { DesignNode } from "./api";
import { buildCanvasModel } from "./designer/canvas-model";
import { applySnapshot, type ScreenSnapshot } from "./designer/live-tree";
import { captureFocus, renderLive, restoreFocus, type LiveEvent } from "./designer/live-dom";
import "./screen-window.css";

type ScreenStep = { snapshot: ScreenSnapshot; handler: string | null; output: string; step: number };

const root = document.querySelector<HTMLElement>("#screen-root")!;
const errorBox = document.querySelector<HTMLElement>("#screen-error")!;
const label = new URLSearchParams(location.search).get("sessao") ?? "";
let tree: DesignNode | null = null;
let queue = Promise.resolve();

function showError(message: string | null) {
  errorBox.hidden = message === null;
  errorBox.textContent = message ?? "";
}

function render(snapshot: ScreenSnapshot) {
  if (!tree) return;
  const focus = captureFocus(root);
  const live = applySnapshot(tree, snapshot);
  // Os <DadosExemplo> são só do designer: na execução vale o estado de verdade.
  root.replaceChildren(renderLive(buildCanvasModel(live, {}, false), dispatch));
  restoreFocus(root, focus);
}

/** Eventos vão em fila, um de cada vez, na ordem em que aconteceram. */
function dispatch(event: LiveEvent) {
  queue = queue.then(async () => {
    try {
      const step = await invoke<ScreenStep>("screen_run_dispatch", { label, event });
      showError(null);
      render(step.snapshot);
    } catch (error) {
      showError(String(error));
    }
  });
}

async function start() {
  try {
    const state = await invoke<{ tree: DesignNode | null; step: ScreenStep }>("screen_run_state", { label });
    tree = state.tree;
    render(state.step.snapshot);
  } catch (error) {
    showError(String(error));
  }
}

void start();
