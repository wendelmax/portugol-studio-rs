// Sistema proprietário de ícones vetoriais SVG (sem assets bitmap legados).

type SvgPrimitive =
  | { tag: "path"; attrs: Record<string, string> }
  | { tag: "circle"; attrs: Record<string, string> }
  | { tag: "rect"; attrs: Record<string, string> }
  | { tag: "line"; attrs: Record<string, string> }
  | { tag: "polyline"; attrs: Record<string, string> }
  | { tag: "polygon"; attrs: Record<string, string> };

const p = (d: string, extra: Record<string, string> = {}): SvgPrimitive => ({ tag: "path", attrs: { d, ...extra } });
const c = (cx: string, cy: string, r: string, extra: Record<string, string> = {}): SvgPrimitive => ({
  tag: "circle",
  attrs: { cx, cy, r, ...extra },
});
const rect = (
  x: string,
  y: string,
  width: string,
  height: string,
  rx = "2",
  extra: Record<string, string> = {},
): SvgPrimitive => ({ tag: "rect", attrs: { x, y, width, height, rx, ...extra } });
const line = (x1: string, y1: string, x2: string, y2: string): SvgPrimitive => ({
  tag: "line",
  attrs: { x1, y1, x2, y2 },
});

const SVG_ICONS: Readonly<Record<string, readonly SvgPrimitive[]>> = {
  "window-min": [line("5", "12", "19", "12")],
  "window-max": [rect("5", "5", "14", "14", "2")],
  "window-close": [line("6", "6", "18", "18"), line("18", "6", "6", "18")],
  "file-plus": [
    p("M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"),
    p("M14 2v6h6"),
    line("12", "12", "12", "18"),
    line("9", "15", "15", "15"),
  ],
  "file-code": [
    p("M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"),
    p("M14 2v6h6"),
    p("M10 13l-2 2 2 2"),
    p("M14 13l2 2-2 2"),
  ],
  "folder-open": [
    p("M3 7a2 2 0 0 1 2-2h4.6a2 2 0 0 1 1.6.8L12.6 8H19a2 2 0 0 1 2 2v1"),
    p("M3 11h17.5a1.5 1.5 0 0 1 1.45 1.88l-1.5 5.62A2 2 0 0 1 18.52 20H5a2 2 0 0 1-2-2z"),
  ],
  folder: [
    p("M3 7a2 2 0 0 1 2-2h4.6a2 2 0 0 1 1.6.8L12.6 8H19a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"),
  ],
  "book-open": [
    p("M2 5.5A2.5 2.5 0 0 1 4.5 3H10a3 3 0 0 1 2 1a3 3 0 0 1 2-1h5.5A2.5 2.5 0 0 1 22 5.5v13a1.5 1.5 0 0 1-1.5 1.5H14a2 2 0 0 0-2 1a2 2 0 0 0-2-1H3.5A1.5 1.5 0 0 1 2 18.5z"),
    line("12", "4", "12", "20"),
  ],
  sparkles: [
    p("M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9z"),
    p("M19 15l.9 2.1L22 18l-2.1.9L19 21l-.9-2.1L16 18l2.1-.9z"),
  ],
  "graduation-cap": [
    p("M22 10L12 5 2 10l10 5 10-5z"),
    p("M6 12v5c0 1.7 2.7 3 6 3s6-1.3 6-3v-5"),
    line("22", "10", "22", "15"),
  ],
  home: [
    p("M3 10.5L12 3l9 7.5"),
    p("M5 9.5V20a1 1 0 0 0 1 1h4.5v-6h3v6H18a1 1 0 0 0 1-1V9.5"),
  ],
  play: [
    p("M6 4.5L19 12 6 19.5z", { fill: "currentColor" }),
  ],
  "step-forward": [
    p("M5 6.5L13 12 5 17.5z"),
    line("17", "6", "17", "18"),
  ],
  square: [
    rect("6", "6", "12", "12", "2", { fill: "currentColor" }),
  ],
  save: [
    p("M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"),
    p("M17 21v-7H7v7"),
    p("M7 3v5h8"),
  ],
  "save-as": [
    p("M17 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h9l4 4v4"),
    p("M7 3v4h7"),
    p("M14.5 18.5l4.8-4.8a1.4 1.4 0 0 1 2 2l-4.8 4.8L13.5 21z"),
  ],
  bug: [
    p("M8 9a4 4 0 0 1 8 0v6a4 4 0 0 1-8 0z"),
    p("M9.5 5.5L8 4"),
    p("M14.5 5.5L16 4"),
    line("4", "11", "8", "11"),
    line("16", "11", "20", "11"),
    line("4", "15", "8", "15"),
    line("16", "15", "20", "15"),
    line("12", "9", "12", "19"),
  ],
  settings: [
    c("12", "12", "3"),
    p("M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3h.1a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8v.1a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"),
  ],
  library: [
    line("5", "4", "5", "20"),
    line("9", "4", "9", "20"),
    line("13", "5", "13", "20"),
    p("M17 5l3 15"),
  ],
  "alert-circle": [
    c("12", "12", "9"),
    line("12", "8", "12", "12.5"),
    c("12", "16", "0.7", { fill: "currentColor" }),
  ],
  "alert-triangle": [
    p("M10.3 3.9L1.8 18.5A2 2 0 0 0 3.5 21.5h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"),
    line("12", "9.5", "12", "14"),
    c("12", "17.5", "0.7", { fill: "currentColor" }),
  ],
  "minimize-2": [
    p("M4 14h6v6"),
    p("M20 10h-6V4"),
    line("14", "10", "21", "3"),
    line("3", "21", "10", "14"),
  ],
  "check-square": [
    rect("3", "3", "18", "18", "3"),
    p("M8.5 12.2l2.4 2.4 4.8-5.2"),
  ],
};

const ICON_ALIASES: Readonly<Record<string, string>> = {
  "pequeno/window_min": "window-min",
  "pequeno/window_max": "window-max",
  "pequeno/window_close": "window-close",
  "grande/newfile64": "file-plus",
  "grande/newfile": "file-plus",
  "grande/file": "file-code",
  "grande/openfile64": "folder-open",
  "grande/openfile": "folder-open",
  "grande/open": "folder-open",
  "pequeno/folder_closed": "folder",
  "grande/helplow": "book-open",
  "pequeno/light_pix": "sparkles",
  "grande/resultset_next": "play",
  "grande/step": "step-forward",
  "grande/stop": "square",
  "grande/save": "save",
  "grande/save_as": "save-as",
  "pequeno/bug": "bug",
  "pequeno/gear_in": "settings",
  "pequeno/biblioteca": "library",
  "pequeno/exclamation": "alert-circle",
  "pequeno/warning": "alert-triangle",
  "pequeno/expandir_componente": "minimize-2",
};

export const ICON_NAMES = Object.keys(SVG_ICONS);

export function resolveIconName(name: string): string | null {
  const canonical = ICON_ALIASES[name] ?? name;
  return canonical in SVG_ICONS ? canonical : null;
}

const SVG_NS = "http://www.w3.org/2000/svg";

function buildSvg(canonical: string): SVGSVGElement {
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", "1.8");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("focusable", "false");
  for (const primitive of SVG_ICONS[canonical] ?? []) {
    const node = document.createElementNS(SVG_NS, primitive.tag);
    for (const [key, value] of Object.entries(primitive.attrs)) {
      node.setAttribute(key, value);
    }
    svg.append(node);
  }
  return svg;
}

function setIcon(element: HTMLElement, file: string) {
  const canonical = resolveIconName(file) ?? "file-code";
  element.classList.add("ui-icon");
  element.dataset.iconName = canonical;
  element.setAttribute("aria-hidden", "true");
  element.replaceChildren(buildSvg(canonical));
}

export function icon(file: string) {
  const element = document.createElement("span");
  setIcon(element, file);
  return element;
}

export function mountIcons() {
  for (const element of document.querySelectorAll<HTMLElement>("[data-icon]")) {
    const file = element.dataset.icon;
    if (file) setIcon(element, file);
  }
}
