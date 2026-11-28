/**
 * A tela em execução: o mesmo modelo de caixas do canvas (mesmo layout), mas
 * com controles de verdade — botões, campos, caixas de seleção e listas — que
 * avisam os eventos para a VM. Só `createElement` e `textContent`.
 */
import type { CanvasBox, CanvasModel, TextRun } from "./canvas-model.ts";

/** Evento de um controle, no formato da sessão de execução. */
export type LiveEvent = { control: string; event: string; value?: string | boolean | number };

function plain(text: readonly TextRun[] | null): string {
  return (text ?? []).map((run) => run.text).join("");
}

function runs(parent: HTMLElement, text: readonly TextRun[] | null): void {
  for (const run of text ?? []) {
    const span = document.createElement("span");
    span.textContent = run.text;
    if (run.kind === "access") span.className = "design-access-key";
    parent.append(span);
  }
}

function control(box: CanvasBox, emit: (event: LiveEvent) => void): HTMLElement {
  const name = box.name;
  const send = (event: string, value?: LiveEvent["value"]) => {
    if (name) emit({ control: name, event, value });
  };
  switch (box.kind) {
    case "button": {
      const button = document.createElement("button");
      button.type = "button";
      runs(button, box.text);
      button.disabled = box.disabled;
      button.addEventListener("click", () => send("AoClicar"));
      return button;
    }
    case "textbox": {
      const input = document.createElement("input");
      input.type = "text";
      input.value = plain(box.text);
      input.placeholder = box.placeholder ?? "";
      input.disabled = box.disabled;
      input.addEventListener("input", () => send("AoAlterar", input.value));
      return input;
    }
    case "checkbox":
    case "radio": {
      const label = document.createElement("label");
      const input = document.createElement("input");
      input.type = box.kind;
      input.checked = box.checked;
      input.disabled = box.disabled;
      input.addEventListener("change", () => send("AoAlterar", input.checked));
      label.append(input);
      runs(label, box.text);
      return label;
    }
    case "combo":
    case "list": {
      const select = document.createElement("select");
      if (box.kind === "list") select.size = Math.max(2, box.items.length);
      for (const item of box.items) {
        const option = document.createElement("option");
        option.textContent = item;
        select.append(option);
      }
      select.selectedIndex = box.selected;
      select.disabled = box.disabled;
      select.addEventListener("change", () => send("AoSelecionar", select.selectedIndex));
      return select;
    }
    case "label":
    case "image": {
      const span = document.createElement("span");
      runs(span, box.text);
      return span;
    }
    default:
      return document.createElement("span");
  }
}

const CONTROLS = new Set(["button", "textbox", "checkbox", "radio", "combo", "list", "label", "image"]);

function build(box: CanvasBox, emit: (event: LiveEvent) => void): HTMLElement {
  const element = document.createElement("div");
  element.className = `live-el live-${box.kind}`;
  for (const [property, value] of Object.entries(box.style)) {
    if (property === "opacity") continue;
    element.style.setProperty(property, value);
  }
  if (box.hidden) element.style.display = "none";
  if (box.name) element.dataset.name = box.name;
  if (CONTROLS.has(box.kind)) element.append(control(box, emit));
  if (box.kind === "panel" && box.text) {
    const legend = document.createElement("span");
    legend.className = "live-panel-title";
    runs(legend, box.text);
    element.append(legend);
  }
  for (const child of box.children) element.append(build(child, emit));
  return element;
}

/** A janela em execução (sem a barra de título: ela é a do sistema). */
export function renderLive(model: CanvasModel, emit: (event: LiveEvent) => void): HTMLElement {
  const window = build(model.window, emit);
  window.classList.add("live-window");
  return window;
}

/** Foco e cursor do controle focado, para não perder a digitação ao redesenhar. */
export function captureFocus(root: ParentNode): { name: string; start: number | null; end: number | null } | null {
  const active = document.activeElement as HTMLElement | null;
  const holder = active?.closest<HTMLElement>("[data-name]");
  if (!active || !holder || !root.contains(active)) return null;
  const input = active instanceof HTMLInputElement ? active : null;
  return { name: holder.dataset.name ?? "", start: input?.selectionStart ?? null, end: input?.selectionEnd ?? null };
}

export function restoreFocus(root: ParentNode, focus: ReturnType<typeof captureFocus>): void {
  if (!focus) return;
  const holder = root.querySelector<HTMLElement>(`[data-name="${CSS.escape(focus.name)}"]`);
  const target = holder?.querySelector<HTMLElement>("input, button, select");
  if (!target) return;
  target.focus();
  if (target instanceof HTMLInputElement && target.type === "text" && focus.start !== null) {
    target.setSelectionRange(focus.start, focus.end ?? focus.start);
  }
}
