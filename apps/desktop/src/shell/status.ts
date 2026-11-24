import { $ } from "../dom";

export type StatusKind = "" | "ok" | "error";

/** Mensagem da barra de status (anunciada por `role="status"`). */
export function setStatus(text: string, kind: StatusKind = "") {
  const status = $("status");
  status.textContent = text;
  status.className = kind;
}
