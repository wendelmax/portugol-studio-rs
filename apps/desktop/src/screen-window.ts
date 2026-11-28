// Janela de uma tela desktop em execução (ADR 0018, seção 5). Pede o estado
// ao abrir, desenha a tela com controles de verdade e manda cada evento para a
// sessão; a resposta traz a janela atualizada pela função do aluno. Quando uma
// tela aberta por cima com `Telas.abrir` fecha, o estado novo chega pelo evento
// `screen-run:step` (ADR 0018, Emenda 5).
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { DesignNode } from "./api";
import { buildCanvasModel } from "./designer/canvas-model";
import { applySnapshot, type ScreenSnapshot } from "./designer/live-tree";
import { captureFocus, renderLive, restoreFocus, type LiveEvent } from "./designer/live-dom";
import { createTimers, timerSpecs } from "./designer/timers";
import "./screen-window.css";

type ScreenStep = { snapshot: ScreenSnapshot; handler: string | null; output: string; step: number };

const root = document.querySelector<HTMLElement>("#screen-root")!;
const errorBox = document.querySelector<HTMLElement>("#screen-error")!;
const label = new URLSearchParams(location.search).get("sessao") ?? "";
let tree: DesignNode | null = null;
let queue = Promise.resolve();
const timers = createTimers(
  { setInterval: (callback, ms) => window.setInterval(callback, ms), clearInterval: (handle) => window.clearInterval(handle as number) },
  (name) => dispatch({ control: name, event: "AoDisparar" }),
);
addEventListener("pagehide", () => timers.stopAll());

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
  timers.sync(timerSpecs(tree, snapshot));
}

/** Eventos vão em fila, um de cada vez, na ordem em que aconteceram. */
function dispatch(event: LiveEvent): Promise<void> {
  queue = queue.then(async () => {
    try {
      const step = await invoke<ScreenStep>("screen_run_dispatch", { label, event });
      showError(null);
      render(step.snapshot);
    } catch (error) {
      showError(String(error));
    }
  });
  return queue;
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

const current = getCurrentWebviewWindow();
void current.listen<ScreenStep>("screen-run:step", (event) => {
  queue = queue.then(() => {
    showError(null);
    render(event.payload.snapshot);
  });
});
void current.listen<string>("screen-run:error", (event) => showError(event.payload));
void start();
