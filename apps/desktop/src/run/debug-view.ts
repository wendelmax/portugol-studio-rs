// Pilha de chamadas e variáveis exibidas quando o depurador pausa.

import { $ } from "../dom";
import type { DebugInfo } from "../api";

type StackFrame = DebugInfo["stack"][number];

/** Mostra pilha e variáveis; clicar num quadro chama `onSelectFrame`. */
export function renderDebugInfo(info: DebugInfo, onSelectFrame: (frame: StackFrame) => void) {
  $("debug-stack").replaceChildren(...info.stack.map((frame, index) => {
    const li = document.createElement("li");
    if (index === 0) li.className = "current";
    const button = document.createElement("button");
    button.type = "button";
    button.className = "debug-frame-btn";
    button.textContent = `${frame.function} — linha ${frame.position.line}:${frame.position.column}`;
    button.onclick = () => onSelectFrame(frame);
    li.append(button);
    return li;
  }));
  const body = document.querySelector<HTMLTableSectionElement>("#debug-vars tbody")!;
  body.replaceChildren(...info.variables.map((variable) => {
    const row = document.createElement("tr");
    for (const text of [variable.name, variable.ty, variable.value, variable.scope]) {
      const cell = document.createElement("td");
      cell.textContent = text;
      row.append(cell);
    }
    return row;
  }));
}
