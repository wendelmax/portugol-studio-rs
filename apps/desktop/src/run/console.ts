// Saída do console da IDE.

import { $ } from "../dom";

const output = () => $("console-output");

/** Acrescenta texto, mantendo a rolagem no fim quando o usuário já estava lá. */
export function appendConsole(text: string, className?: string) {
  const box = $("console");
  const atBottom = box.scrollTop + box.clientHeight >= box.scrollHeight - 4;
  if (className) {
    const span = document.createElement("span");
    span.className = className;
    span.textContent = text;
    output().append(span);
  } else {
    output().append(text);
  }
  if (atBottom) box.scrollTop = box.scrollHeight;
}

/** Saída de um processo: stdout normal e stderr destacado como erro. */
export function appendProcessOutput(result: { stdout?: string | null; stderr?: string | null }) {
  if (result.stdout) appendConsole(result.stdout);
  if (result.stderr) appendConsole(result.stderr, "error");
}

export function clearConsole() {
  output().replaceChildren();
}

export function consoleText(): string {
  return output().textContent ?? "";
}
