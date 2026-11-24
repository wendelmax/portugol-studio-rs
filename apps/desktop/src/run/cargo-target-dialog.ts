import { $ } from "../dom";
import type { CargoRunTarget } from "../api";

/** Pergunta qual binário Cargo executar; `null` quando o usuário cancela. */
export function chooseCargoTarget(targets: CargoRunTarget[]): Promise<CargoRunTarget | null> {
  const dialog = $<HTMLDialogElement>("cargo-target-dialog");
  const select = $<HTMLSelectElement>("cargo-target-select");
  select.replaceChildren(...targets.map((target, index) => {
    const option = document.createElement("option");
    option.value = String(index);
    option.textContent = `${target.packageName} — ${target.targetName}`;
    return option;
  }));
  $<HTMLButtonElement>("cargo-target-cancel").onclick = () => dialog.close("cancel");
  dialog.returnValue = "";
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => {
      const index = Number(select.value);
      resolve(dialog.returnValue === "run" && Number.isInteger(index) ? targets[index] ?? null : null);
    }, { once: true });
    dialog.showModal();
    select.focus();
  });
}
