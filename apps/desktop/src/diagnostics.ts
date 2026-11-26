import type { DiagnosticInfo } from "./api";
import { icon } from "./icons";

function kind(diagnostic: DiagnosticInfo) {
  return diagnostic.severity === "error" ? "Erro" : "Aviso";
}

function location(diagnostic: DiagnosticInfo) {
  return `Linha ${diagnostic.start.line}, coluna ${diagnostic.start.column}`;
}

function diagnosticTable(
  diagnostics: DiagnosticInfo[], jump: (offset: number) => void,
) {
  const table = document.querySelector<HTMLTableElement>("#diagnostic-table")!;
  document.querySelector<HTMLElement>("#no-diagnostics")!.hidden = diagnostics.length > 0;
  table.hidden = diagnostics.length === 0;
  table.tBodies[0].replaceChildren(...diagnostics.map((diagnostic) => {
    const row = document.createElement("tr");
    row.className = diagnostic.severity;
    const severity = row.insertCell();
    const badge = icon(diagnostic.severity === "error" ? "alert-circle" : "alert-triangle");
    badge.title = kind(diagnostic);
    severity.append(badge);

    const line = row.insertCell();
    line.className = "diagnostic-line";
    line.title = location(diagnostic);
    line.textContent = String(diagnostic.start.line);

    const message = row.insertCell();
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = diagnostic.message;
    button.setAttribute("aria-label", `${kind(diagnostic)}. ${location(diagnostic)}. ${diagnostic.message}`);
    button.onclick = (event) => {
      event.stopPropagation();
      jump(diagnostic.start.offset);
    };
    message.append(button);
    if (diagnostic.help) {
      const help = document.createElement("span");
      help.className = "diagnostic-help";
      help.textContent = diagnostic.help;
      message.append(help);
    }
    row.onclick = () => jump(diagnostic.start.offset);
    return row;
  }));
}

function errorStrip(diagnostics: DiagnosticInfo[], lines: number, jump: (offset: number) => void) {
  const strip = document.querySelector("#error-strip")!;
  const groups = new Map<number, DiagnosticInfo[]>();
  for (const diagnostic of diagnostics) {
    const line = diagnostic.start.line;
    const group = groups.get(line) ?? [];
    group.push(diagnostic);
    groups.set(line, group);
  }
  strip.replaceChildren(...[...groups].map(([line, group]) => {
    const diagnostic = group.find(({ severity }) => severity === "error") ?? group[0];
    const marker = document.createElement("button");
    marker.className = `strip-marker ${diagnostic.severity}`;
    marker.title = group.map((d) => `${kind(d)}. ${location(d)}. ${d.message}`).join("\n");
    marker.setAttribute("aria-label", marker.title);
    marker.style.top = `${Math.max(0, Math.min(100, (line - 1) / Math.max(1, lines - 1) * 100))}%`;
    marker.onclick = () => jump(diagnostic.start.offset);
    return marker;
  }));
}

export function renderDiagnostics(
  diagnostics: DiagnosticInfo[], lines: number, jump: (offset: number) => void,
) {
  const errors = diagnostics.filter(({ severity }) => severity === "error").length;
  document.querySelector("#message-count")!.textContent = errors ? String(errors) : "";
  diagnosticTable(diagnostics, jump);
  errorStrip(diagnostics, lines, jump);
}
