// Pilha de chamadas e variáveis exibidas quando o depurador pausa.

import { $ } from "../dom.ts";
import type { DebugInfo, SourceMapInfo } from "../api";
import { sourceMapLinks, type SourceMapLink } from "../source-map-ui.ts";

type StackFrame = DebugInfo["stack"][number];

/** Descreve o nó HIR, a instrução bytecode e a linha Rust gerada na pausa. */
export function debugMappingText(info: DebugInfo, links: SourceMapLink[]): string | null {
  const mapping = info.mapping;
  if (!mapping) return null;
  const start = mapping.hirStart.offset;
  const end = mapping.hirEnd.offset;
  const link = links
    .filter((candidate) => candidate.sourceOffset <= start)
    .reduce<SourceMapLink | null>((best, candidate) =>
      best === null || candidate.sourceOffset > best.sourceOffset ? candidate : best, null);
  const line = link?.generatedLine ?? "—";
  return `HIR ${mapping.hirNode}@${start}..${end} · bytecode ${mapping.bytecodeOffset} · Rust linha ${line}`;
}

/** Mostra pilha e variáveis; clicar num quadro chama `onSelectFrame`. */
export function renderDebugInfo(
  info: DebugInfo,
  onSelectFrame: (frame: StackFrame) => void,
  sourceMap: SourceMapInfo[] = [],
) {
  const mappingText = debugMappingText(info, sourceMapLinks(sourceMap));
  const mappingElement = document.querySelector<HTMLElement>("#debug-mapping")!;
  if (mappingText === null) {
    mappingElement.hidden = true;
    mappingElement.replaceChildren();
  } else {
    mappingElement.hidden = false;
    mappingElement.textContent = mappingText;
  }
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
