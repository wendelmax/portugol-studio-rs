/**
 * Monta o DOM do canvas a partir de {@link CanvasModel}. Só usa
 * `createElement` e `textContent` (nada de HTML bruto), como o renderizador da
 * prévia visual. Os controles são desenhados com elementos inertes (`div` e
 * `span`), não com `button` ou `input` reais: no designer, clicar seleciona o
 * controle em vez de acioná-lo.
 */
import type { CanvasBox, CanvasModel, TextRun } from "./canvas-model.ts";

const LABELS: Record<string, string> = {
  Temporizador: "Temporizador",
  DialogoArquivo: "Diálogo de arquivo",
};

/** Escreve o texto num único `span`, para que o layout flexível do controle não separe os pedaços. */
function runs(parent: HTMLElement, text: readonly TextRun[]): void {
  const wrapper = document.createElement("span");
  wrapper.className = "design-text";
  for (const run of text) {
    const span = document.createElement("span");
    span.textContent = run.text;
    if (run.kind === "access") span.className = "design-access-key";
    else if (run.kind === "binding") {
      span.className = "design-binding";
      span.title = "Vínculo: o valor aparece quando o programa roda";
    }
    wrapper.append(span);
  }
  parent.append(wrapper);
}

function content(element: HTMLElement, box: CanvasBox): void {
  switch (box.kind) {
    case "label":
    case "button":
      if (box.text) runs(element, box.text);
      break;
    case "checkbox":
    case "radio": {
      const mark = document.createElement("span");
      mark.className = "design-check-mark";
      mark.setAttribute("aria-hidden", "true");
      mark.textContent = box.kind === "checkbox" ? (box.checked ? "☑" : "☐") : box.checked ? "◉" : "○";
      element.append(mark);
      if (box.text) runs(element, box.text);
      break;
    }
    case "textbox":
      if (box.text) runs(element, box.text);
      else if (box.placeholder) {
        const placeholder = document.createElement("span");
        placeholder.className = "design-placeholder";
        placeholder.textContent = box.placeholder;
        element.append(placeholder);
      }
      break;
    case "list":
      for (const item of box.items) {
        const row = document.createElement("div");
        row.className = "design-list-item";
        row.textContent = item;
        element.append(row);
      }
      break;
    case "combo": {
      const value = document.createElement("span");
      value.textContent = box.items[0] ?? "";
      const arrow = document.createElement("span");
      arrow.className = "design-combo-arrow";
      arrow.setAttribute("aria-hidden", "true");
      arrow.textContent = "▾";
      element.append(value, arrow);
      break;
    }
    case "image":
      if (box.text) runs(element, box.text);
      break;
    case "panel":
      if (box.text) {
        const legend = document.createElement("span");
        legend.className = "design-panel-title";
        runs(legend, box.text);
        element.append(legend);
      }
      break;
  }
}

function describe(box: CanvasBox): string {
  return box.name ? `${box.name} (${box.tag})` : box.tag;
}

function build(box: CanvasBox): HTMLElement {
  const element = document.createElement("div");
  element.className = `design-el design-${box.kind}${box.disabled ? " is-disabled" : ""}`;
  for (const [property, value] of Object.entries(box.style)) element.style.setProperty(property, value);
  if (box.path !== null) {
    element.dataset.path = box.path;
    element.title = describe(box);
  }
  content(element, box);
  for (const child of box.children) element.append(build(child));
  return element;
}

/** Janela desenhada, com a barra de título, e a faixa de componentes abaixo dela. */
export function renderCanvas(model: CanvasModel): HTMLElement {
  const frame = document.createElement("div");
  frame.className = "design-frame";

  const titleBar = document.createElement("div");
  titleBar.className = "design-title-bar";
  titleBar.dataset.path = "";
  runs(titleBar, model.title.length ? model.title : [{ text: "Janela", kind: "plain" }]);

  const window = build(model.window);
  window.classList.add("design-window");
  frame.append(titleBar, window);

  if (model.components.length) {
    const tray = document.createElement("div");
    tray.className = "design-components";
    tray.setAttribute("aria-label", "Componentes sem aparência");
    for (const component of model.components) {
      const chip = document.createElement("div");
      chip.className = "design-el design-component";
      chip.dataset.path = component.path;
      chip.textContent = component.name ?? LABELS[component.tag] ?? component.tag;
      chip.title = `${LABELS[component.tag] ?? component.tag}: não aparece na tela`;
      tray.append(chip);
    }
    frame.append(tray);
  }
  return frame;
}

/** Destaca o elemento selecionado no canvas. */
export function markSelection(frame: HTMLElement, path: string | null): void {
  for (const selected of frame.querySelectorAll(".is-selected")) selected.classList.remove("is-selected");
  if (path === null) return;
  const target = path === "" ? frame.querySelector(".design-window") : frame.querySelector(`[data-path="${CSS.escape(path)}"]`);
  target?.classList.add("is-selected");
}
