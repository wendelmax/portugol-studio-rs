// Divisórias redimensionáveis (explorador, painel inferior e utilitários) e
// modo de editor expandido. Os tamanhos são frações do espaço disponível,
// salvas por workspace.

import { $ } from "../dom";
import {
  calculateSidePaneMinimums,
  clampPaneSize,
  DEFAULT_PANE_LAYOUT,
  loadPaneLayout,
  normalizePaneLayout,
  resetPaneLayout,
  resizePaneForKey,
  savePaneLayout,
  type PaneLayout,
} from "../pane-layout";

type Orientation = "horizontal" | "vertical";
type Bounds = { minimum: number; maximum: number; available: number };
type AppliedSize = Bounds & { size: number };

export interface PaneLayoutOptions {
  /** Chave das preferências do workspace da aba ativa. */
  preferenceKey: () => string | undefined;
  /** Expandir o editor e abrir utilitários só fazem sentido com uma aba aberta. */
  hasActiveTab: () => boolean;
}

function loadSavedPaneLayout(preferenceKey?: string): PaneLayout {
  try {
    return loadPaneLayout(localStorage, preferenceKey);
  } catch {
    return { ...DEFAULT_PANE_LAYOUT };
  }
}

function setSeparatorValue(separator: HTMLElement, value: number, bounds: { minimum: number; maximum: number }) {
  separator.setAttribute("aria-valuemin", String(Math.round(bounds.minimum)));
  separator.setAttribute("aria-valuemax", String(Math.round(bounds.maximum)));
  separator.setAttribute("aria-valuenow", String(Math.round(value)));
}

function installPointerResize(
  separator: HTMLElement,
  orientation: Orientation,
  readSize: () => number,
  applySize: (size: number) => AppliedSize,
  onCommit: (size: number, available: number) => void,
  deltaSign: 1 | -1,
) {
  separator.addEventListener("pointerdown", (start) => {
    if (start.button !== 0) return;
    separator.setPointerCapture(start.pointerId);
    const origin = orientation === "horizontal" ? start.clientY : start.clientX;
    const initial = readSize();
    const move = (event: PointerEvent) => {
      const coordinate = orientation === "horizontal" ? event.clientY : event.clientX;
      applySize(initial + (coordinate - origin) * deltaSign);
    };
    const finish = () => {
      const bounds = applySize(readSize());
      onCommit(bounds.size, bounds.available);
      separator.removeEventListener("pointermove", move);
      separator.removeEventListener("pointerup", finish);
      separator.removeEventListener("pointercancel", finish);
      separator.removeEventListener("lostpointercapture", finish);
    };
    separator.addEventListener("pointermove", move);
    separator.addEventListener("pointerup", finish, { once: true });
    separator.addEventListener("pointercancel", finish, { once: true });
    separator.addEventListener("lostpointercapture", finish, { once: true });
  });
}

function installKeyboardResize(
  separator: HTMLElement,
  orientation: Orientation,
  readSize: () => number,
  applySize: (size: number) => AppliedSize,
  onCommit: (size: number, available: number) => void,
  direction: 1 | -1,
) {
  separator.addEventListener("keydown", (event) => {
    const bounds = applySize(readSize());
    const next = resizePaneForKey(event.key, orientation, direction, bounds.size, bounds);
    if (next === null) return;
    event.preventDefault();
    const updated = applySize(next);
    onCommit(updated.size, updated.available);
  });
}

export function mountPaneLayout(options: PaneLayoutOptions) {
  const panel = document.querySelector<HTMLElement>(".panel")!;
  const bottomSplitter = $("splitter");
  const explorer = $("workspace-explorer");
  const explorerSplitter = $("workspace-explorer-splitter");
  const utilities = $("utilities");
  const utilitiesSplitter = $("utilities-splitter");

  let layout = loadSavedPaneLayout();

  function paneChromeWidth() {
    const rail = $("code").querySelector<HTMLElement>(".activity-rail")?.getBoundingClientRect().width || 56;
    const mappingGutter = $("rust-mapping-gutter").hidden ? 0 : ($("rust-mapping-gutter").getBoundingClientRect().width || 76);
    const uiPreview = $("pors-ui-preview").hidden ? 0 : Math.max(260, $("pors-ui-preview").getBoundingClientRect().width || 260);
    const editorMinimum = $("rust-preview").hidden ? 100 : 360;
    return rail + mappingGutter + uiPreview + editorMinimum;
  }

  function sideMinimums(available: number) {
    return calculateSidePaneMinimums(
      available,
      paneChromeWidth(),
      explorerSplitter.offsetWidth || 8,
      utilitiesSplitter.offsetWidth || 8,
      !explorer.hidden,
      !utilities.hidden,
    );
  }

  function explorerBounds(): Bounds {
    const available = Math.max(1, $("code").clientWidth || window.innerWidth);
    const minimum = sideMinimums(available).explorer;
    const utilitiesWidth = utilities.hidden ? 0 : utilities.getBoundingClientRect().width;
    const maximum = Math.max(minimum, Math.min(420, available * 0.42, available - paneChromeWidth()
      - (explorer.hidden ? 0 : explorerSplitter.offsetWidth)
      - (utilities.hidden ? 0 : utilitiesSplitter.offsetWidth) - utilitiesWidth));
    return { minimum, maximum, available };
  }

  function bottomBounds(): Bounds {
    const available = Math.max(280, panel.parentElement?.clientHeight || window.innerHeight - 100);
    const minimum = 90;
    const maximum = Math.max(minimum, Math.min(available - 90, available * 0.65));
    return { minimum, maximum, available };
  }

  function utilitiesBounds(): Bounds {
    const available = Math.max(1, $("code").clientWidth || window.innerWidth);
    const minimum = sideMinimums(available).utilities;
    const explorerWidth = explorer.hidden ? 0 : explorer.getBoundingClientRect().width;
    const maximum = Math.max(minimum, Math.min(available * 0.55, 720, available - paneChromeWidth()
      - (explorer.hidden ? 0 : explorerSplitter.offsetWidth)
      - (utilities.hidden ? 0 : utilitiesSplitter.offsetWidth) - explorerWidth));
    return { minimum, maximum, available };
  }

  function applyExplorer(width = layout.explorer * explorerBounds().available): AppliedSize {
    const bounds = explorerBounds();
    const size = clampPaneSize(width, bounds.minimum, bounds.maximum);
    explorer.style.minWidth = `${bounds.minimum}px`;
    explorer.style.flexBasis = `${size}px`;
    setSeparatorValue(explorerSplitter, size, bounds);
    return { size, ...bounds };
  }

  function applyBottom(height = layout.bottom * bottomBounds().available): AppliedSize {
    const bounds = bottomBounds();
    const size = clampPaneSize(height, bounds.minimum, bounds.maximum);
    panel.style.flex = `0 0 ${size}px`;
    setSeparatorValue(bottomSplitter, size, bounds);
    return { size, ...bounds };
  }

  function applyUtilities(width = layout.utilities * utilitiesBounds().available): AppliedSize {
    const bounds = utilitiesBounds();
    const size = clampPaneSize(width, bounds.minimum, bounds.maximum);
    utilities.style.minWidth = `${bounds.minimum}px`;
    utilities.style.flex = `0 0 ${size}px`;
    setSeparatorValue(utilitiesSplitter, size, bounds);
    return { size, ...bounds };
  }

  function persist() {
    try {
      savePaneLayout(localStorage, layout, options.preferenceKey());
    } catch {
      // A configuração permanece ativa enquanto a janela estiver aberta.
    }
  }

  function installPaneResize(
    separator: HTMLElement,
    orientation: Orientation,
    readSize: () => number,
    applySize: (size: number) => AppliedSize,
    key: keyof PaneLayout,
    direction: 1 | -1,
  ) {
    const commit = (size: number, available: number) => {
      layout = normalizePaneLayout({ ...layout, [key]: size / available });
      persist();
    };
    installPointerResize(separator, orientation, readSize, applySize, commit, direction);
    installKeyboardResize(separator, orientation, readSize, applySize, commit, direction);
  }

  /** Reaplica os tamanhos dos painéis visíveis. */
  function applyAll() {
    applyBottom();
    if (!explorer.hidden) applyExplorer();
    if (!utilities.hidden) applyUtilities();
  }

  function toggleExpanded() {
    if (!options.hasActiveTab()) return;
    const hidden = !panel.hidden;
    panel.hidden = hidden;
    bottomSplitter.hidden = hidden;
    $("code").classList.toggle("expanded", hidden);
    $("btn-unexpand").hidden = !hidden;
  }

  function showUtilities(visible: boolean) {
    if (visible && $("code").classList.contains("expanded")) toggleExpanded();
    utilities.hidden = utilitiesSplitter.hidden = !visible;
    $("btn-utilities").setAttribute("aria-expanded", String(visible));
    if (visible) {
      if (!explorer.hidden) applyExplorer();
      applyUtilities();
    } else if (!explorer.hidden) applyExplorer();
  }

  function setExplorerVisible(visible: boolean) {
    if (visible && $("code").classList.contains("expanded")) toggleExpanded();
    explorer.hidden = !visible;
    explorerSplitter.hidden = !visible;
    $("btn-explorer").setAttribute("aria-expanded", String(visible));
    if (visible) {
      applyExplorer();
      if (!utilities.hidden) applyUtilities();
    } else if (!utilities.hidden) applyUtilities();
  }

  installPaneResize(explorerSplitter, "vertical", () => explorer.getBoundingClientRect().width, applyExplorer, "explorer", 1);
  installPaneResize(bottomSplitter, "horizontal", () => panel.getBoundingClientRect().height, applyBottom, "bottom", -1);
  installPaneResize(utilitiesSplitter, "vertical", () => utilities.getBoundingClientRect().width, applyUtilities, "utilities", -1);
  applyBottom();
  window.addEventListener("resize", applyAll);

  return {
    /** Carrega os tamanhos salvos para o workspace; `applyAll` os aplica. */
    load(preferenceKey: string | undefined) {
      layout = loadSavedPaneLayout(preferenceKey);
    },
    /** Restaura os tamanhos padrão dos painéis e limpa a preferência salva. */
    reset() {
      try {
        layout = resetPaneLayout(localStorage, options.preferenceKey());
      } catch {
        layout = { ...DEFAULT_PANE_LAYOUT };
      }
      applyAll();
    },
    applyAll,
    applyUtilities,
    setExplorerVisible,
    showUtilities,
    toggleUtilities() {
      if (options.hasActiveTab()) showUtilities(utilities.hidden === true);
    },
    toggleExpanded,
    utilitiesVisible: () => !utilities.hidden,
  };
}
