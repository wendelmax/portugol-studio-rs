// Laboratório de ownership: cenários guiados de `String` (move, `&T`,
// `&mut T` e escopo) com verificação opcional real pelo compilador `rustc`.

import type { RustSourceCompileDiagnostic } from "../api";
import { $ } from "../dom";
import {
  getOwnershipScenario,
  getOwnershipSnapshot,
  ownershipScenarioIds,
  type OwnershipBindingState,
  type OwnershipScenarioId,
} from "../ownership-lab";
import {
  formatOwnershipRustcCheck,
  formatOwnershipRustcUnavailable,
  type OwnershipRustcReport,
} from "./ownership-rustc";

export interface OwnershipLabHost {
  isRustcAvailable(): boolean;
  checkRustSource(source: string): Promise<RustSourceCompileDiagnostic[]>;
  openRustScenario(source: string, fileName: string): void;
}

const ownershipStateLabels: Record<OwnershipBindingState, string> = {
  owner: "Dono",
  "shared-reference": "Empréstimo compartilhado (&T)",
  "mutable-reference": "Empréstimo mutável exclusivo (&mut T)",
  moved: "Movido; valor indisponível",
  expired: "Empréstimo encerrado",
  dropped: "Liberado ao sair do escopo",
};

export function mountOwnershipLab(host?: OwnershipLabHost) {
  let ownershipScenarioId: OwnershipScenarioId = "move";
  let ownershipStepIndex = 0;
  let rustcCheckBusy = false;
  let rustcReportScenarioId: OwnershipScenarioId | null = null;
  let rustcReport: OwnershipRustcReport | null = null;
  let checkSeq = 0;

  function renderRustcPanel() {
    const panel = document.getElementById("ownership-rustc-panel");
    const provenance = document.getElementById("ownership-rustc-provenance");
    const summary = document.getElementById("ownership-rustc-summary");
    const list = document.getElementById("ownership-rustc-diagnostics");
    const checkButton = document.getElementById("ownership-check-rustc") as HTMLButtonElement | null;

    if (checkButton) {
      checkButton.disabled = rustcCheckBusy;
      checkButton.textContent = rustcCheckBusy ? "Verificando com rustc…" : "Verificar com rustc";
    }
    if (!panel || !provenance || !summary || !list) return;

    if (!rustcReport || rustcReportScenarioId !== ownershipScenarioId) {
      panel.hidden = true;
      list.replaceChildren();
      return;
    }

    panel.hidden = false;
    panel.dataset.status = rustcReport.status;
    provenance.textContent = rustcReport.provenanceLabel;
    summary.textContent = rustcReport.summary;
    list.replaceChildren(...rustcReport.items.map((item) => {
      const li = document.createElement("li");
      li.className = "ownership-rustc-item";
      li.dataset.severity = item.severity;
      const meta = document.createElement("span");
      meta.className = "ownership-rustc-meta";
      const parts = [
        item.severity === "error" ? "erro" : "aviso",
        item.code ? `[${item.code}]` : null,
        item.location,
      ].filter((part): part is string => Boolean(part));
      meta.textContent = parts.join(" · ");
      const message = document.createElement("code");
      message.textContent = item.message;
      li.append(meta, message);
      return li;
    }));
  }

  function renderOwnershipLab() {
    const scenario = getOwnershipScenario(ownershipScenarioId);
    const snapshot = getOwnershipSnapshot(ownershipScenarioId, ownershipStepIndex);
    if (!scenario || !snapshot) return;

    const picker = $<HTMLSelectElement>("ownership-scenario");
    picker.replaceChildren(...ownershipScenarioIds().map((id) => {
      const option = document.createElement("option");
      const item = getOwnershipScenario(id);
      option.value = id;
      option.textContent = item?.title ?? id;
      return option;
    }));
    picker.value = scenario.id;

    const code = $<HTMLElement>("ownership-source");
    code.replaceChildren(...scenario.source.split("\n").map((text, index) => {
      const line = document.createElement("span");
      line.className = "ownership-code-line";
      line.dataset.line = String(index + 1);
      if (index + 1 === snapshot.currentLine) {
        line.dataset.current = "true";
        line.setAttribute("aria-current", "location");
      }
      line.textContent = text || " ";
      return line;
    }));

    const bindings = $<HTMLUListElement>("ownership-bindings");
    bindings.replaceChildren(...snapshot.bindings.map((binding) => {
      const item = document.createElement("li");
      item.className = "ownership-binding";
      item.dataset.state = binding.state;

      const heading = document.createElement("div");
      heading.className = "ownership-binding-heading";
      const name = document.createElement("code");
      name.textContent = binding.name;
      const type = document.createElement("span");
      type.textContent = binding.type;
      const state = document.createElement("span");
      state.className = "ownership-state";
      state.textContent = ownershipStateLabels[binding.state];
      heading.append(name, type, state);

      const description = document.createElement("p");
      description.textContent = binding.description;
      item.append(heading, description);
      if (binding.target) {
        const relation = document.createElement("p");
        relation.className = "ownership-relation";
        relation.textContent = binding.state === "owner"
          ? `Controla: ${binding.target}`
          : `Empresta de: ${binding.target}`;
        item.append(relation);
      }
      if (binding.value !== undefined) {
        const value = document.createElement("p");
        value.className = "ownership-value";
        value.textContent = `Valor: ${binding.value}`;
        item.append(value);
      }
      return item;
    }));

    const heap = $<HTMLUListElement>("ownership-heap");
    heap.replaceChildren(...snapshot.heap.map((block) => {
      const item = document.createElement("li");
      item.className = "ownership-heap-block";
      item.dataset.state = block.state;
      const title = document.createElement("strong");
      title.textContent = block.id;
      const value = document.createElement("p");
      value.textContent = block.state === "allocated" ? `Conteúdo UTF-8: ${block.value}` : `Conteúdo liberado: ${block.value}`;
      const allocation = document.createElement("p");
      allocation.textContent = block.state === "allocated"
        ? `Alocação ativa; owner: ${block.owner ?? "não identificado"}.`
        : "Alocação liberada quando o owner saiu do escopo.";
      item.append(title, value, allocation);
      return item;
    }));

    $("ownership-step-title").textContent = snapshot.title;
    $("ownership-explanation").textContent = snapshot.explanation;
    $("ownership-step-status").textContent = `Passo ${ownershipStepIndex + 1} de ${scenario.snapshots.length}: ${snapshot.title}`;
    $("ownership-provenance").textContent = snapshot.provenance === "verified-simulation"
      ? "Simulação didática — não é execução nem trace de memória Rust."
      : "Proveniência indisponível.";
    const diagnostic = $<HTMLElement>("ownership-diagnostic");
    diagnostic.hidden = !snapshot.diagnostic;
    diagnostic.textContent = snapshot.diagnostic ?? "";

    $<HTMLButtonElement>("ownership-previous").disabled = ownershipStepIndex === 0;
    $<HTMLButtonElement>("ownership-next").disabled = ownershipStepIndex >= scenario.snapshots.length - 1;
    $<HTMLButtonElement>("ownership-restart").disabled = ownershipStepIndex === 0;
    renderRustcPanel();
  }

  $<HTMLSelectElement>("ownership-scenario").onchange = (event) => {
    const selected = (event.currentTarget as HTMLSelectElement).value;
    if (ownershipScenarioIds().includes(selected as OwnershipScenarioId)) ownershipScenarioId = selected as OwnershipScenarioId;
    ownershipStepIndex = 0;
    rustcReport = null;
    rustcReportScenarioId = null;
    checkSeq += 1;
    rustcCheckBusy = false;
    renderOwnershipLab();
  };
  $<HTMLButtonElement>("ownership-previous").onclick = () => {
    ownershipStepIndex = Math.max(0, ownershipStepIndex - 1);
    renderOwnershipLab();
  };
  $<HTMLButtonElement>("ownership-next").onclick = () => {
    const scenario = getOwnershipScenario(ownershipScenarioId);
    ownershipStepIndex = Math.min(scenario ? scenario.snapshots.length - 1 : 0, ownershipStepIndex + 1);
    renderOwnershipLab();
  };
  $<HTMLButtonElement>("ownership-restart").onclick = () => {
    ownershipStepIndex = 0;
    renderOwnershipLab();
  };

  const checkButton = document.getElementById("ownership-check-rustc") as HTMLButtonElement | null;
  if (checkButton) {
    checkButton.onclick = async () => {
      const scenario = getOwnershipScenario(ownershipScenarioId);
      if (!scenario || rustcCheckBusy) return;
      if (!host || !host.isRustcAvailable()) {
        rustcReportScenarioId = scenario.id;
        rustcReport = formatOwnershipRustcUnavailable();
        renderRustcPanel();
        return;
      }
      const currentSeq = ++checkSeq;
      rustcCheckBusy = true;
      renderRustcPanel();
      try {
        const diagnostics = await host.checkRustSource(scenario.source);
        if (currentSeq !== checkSeq) return;
        rustcReportScenarioId = scenario.id;
        rustcReport = formatOwnershipRustcCheck(scenario, diagnostics);
      } catch {
        if (currentSeq !== checkSeq) return;
        rustcReportScenarioId = scenario.id;
        rustcReport = formatOwnershipRustcUnavailable();
      } finally {
        if (currentSeq === checkSeq) {
          rustcCheckBusy = false;
          renderRustcPanel();
        }
      }
    };
  }

  const openRsButton = document.getElementById("ownership-open-rs") as HTMLButtonElement | null;
  if (openRsButton) {
    openRsButton.onclick = () => {
      const scenario = getOwnershipScenario(ownershipScenarioId);
      if (!scenario || !host) return;
      host.openRustScenario(scenario.source, scenario.fixture);
    };
  }

  return { render: renderOwnershipLab };
}
