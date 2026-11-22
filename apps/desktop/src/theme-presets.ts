export const THEME_COLOR_FIELDS = [
  "--frame", "--bg", "--surface", "--border", "--text", "--muted", "--accent", "--accent-text",
  "--error", "--warning", "--ok", "--input-echo", "--tab-active", "--home-border", "--home-title-text",
  "--examples-surface", "--console-surface", "--editor-bg", "--editor-text", "--editor-caret",
  "--editor-error", "--editor-line", "--editor-selection", "--editor-gutter", "--syntax-keyword",
  "--syntax-type", "--syntax-function", "--syntax-logical", "--syntax-number", "--syntax-string",
  "--syntax-comment", "--syntax-operator",
] as const;

export type Color = typeof THEME_COLOR_FIELDS[number];
export type Base = "light" | "dark";
export type Colors = Partial<Record<Color, string>>;
export interface Theme {
  name: string;
  base: Base;
  colors: Colors;
}

type Palette = {
  bg: string;
  surface: string;
  border: string;
  text: string;
  muted: string;
  accent: string;
  accentText: string;
  error: string;
  warning: string;
  ok: string;
  frame: string;
  console: string;
  examples: string;
  editor: string;
  caret: string;
  editorError: string;
  line: string;
  selection: string;
  gutter: string;
  keyword: string;
  type: string;
  function: string;
  logical: string;
  number: string;
  string: string;
  comment: string;
  operator: string;
};

function theme(name: string, base: Base, p: Palette): Theme {
  const colors: Record<Color, string> = {
    "--frame": p.frame,
    "--bg": p.bg,
    "--surface": p.surface,
    "--border": p.border,
    "--text": p.text,
    "--muted": p.muted,
    "--accent": p.accent,
    "--accent-text": p.accentText,
    "--error": p.error,
    "--warning": p.warning,
    "--ok": p.ok,
    "--input-echo": p.accent,
    "--tab-active": p.selection,
    "--home-border": p.border,
    "--home-title-text": p.text,
    "--examples-surface": p.examples,
    "--console-surface": p.console,
    "--editor-bg": p.editor,
    "--editor-text": p.text,
    "--editor-caret": p.caret,
    "--editor-error": p.editorError,
    "--editor-line": p.line,
    "--editor-selection": p.selection,
    "--editor-gutter": p.gutter,
    "--syntax-keyword": p.keyword,
    "--syntax-type": p.type,
    "--syntax-function": p.function,
    "--syntax-logical": p.logical,
    "--syntax-number": p.number,
    "--syntax-string": p.string,
    "--syntax-comment": p.comment,
    "--syntax-operator": p.operator,
  };
  return { name, base, colors };
}

// Cores próprias do Portugol Studio, inspiradas em paletas conhecidas do VS Code.
export const BUILTIN_THEMES: Readonly<Record<string, Theme>> = {
  "preset:github-light": theme("GitHub Light", "light", {
    bg: "#ffffff", surface: "#f6f8fa", border: "#d0d7de", text: "#1f2328", muted: "#656d76",
    accent: "#0969da", accentText: "#ffffff", error: "#cf222e", warning: "#9a6700", ok: "#1a7f37",
    frame: "#24292f", console: "#f6f8fa", examples: "#f6f8fa", editor: "#ffffff", caret: "#0969da",
    editorError: "#ffebe9", line: "#f6f8fa", selection: "#ddf4ff", gutter: "#656d76",
    keyword: "#cf222e", type: "#0550ae", function: "#8250df", logical: "#953800",
    number: "#0550ae", string: "#0a3069", comment: "#6e7781", operator: "#1f2328",
  }),
  "preset:github-dark": theme("GitHub Dark", "dark", {
    bg: "#0d1117", surface: "#161b22", border: "#30363d", text: "#e6edf3", muted: "#8b949e",
    accent: "#2f81f7", accentText: "#ffffff", error: "#f85149", warning: "#d29922", ok: "#3fb950",
    frame: "#010409", console: "#161b22", examples: "#161b22", editor: "#0d1117", caret: "#2f81f7",
    editorError: "#490202", line: "#161b22", selection: "#264f78", gutter: "#6e7681",
    keyword: "#ff7b72", type: "#79c0ff", function: "#d2a8ff", logical: "#ffab70",
    number: "#79c0ff", string: "#a5d6ff", comment: "#8b949e", operator: "#e6edf3",
  }),
  "preset:one-dark-pro": theme("One Dark Pro", "dark", {
    bg: "#282c34", surface: "#21252b", border: "#3e4451", text: "#abb2bf", muted: "#9da4b0",
    accent: "#61afef", accentText: "#282c34", error: "#e06c75", warning: "#e5c07b", ok: "#98c379",
    frame: "#21252b", console: "#21252b", examples: "#21252b", editor: "#282c34", caret: "#528bff",
    editorError: "#3b2529", line: "#2c313a", selection: "#3e4451", gutter: "#636d83",
    keyword: "#c678dd", type: "#e5c07b", function: "#61afef", logical: "#56b6c2",
    number: "#d19a66", string: "#98c379", comment: "#5c6370", operator: "#56b6c2",
  }),
  "preset:dracula": theme("Dracula", "dark", {
    bg: "#282a36", surface: "#21222c", border: "#44475a", text: "#f8f8f2", muted: "#a5aec9",
    accent: "#bd93f9", accentText: "#282a36", error: "#ff5555", warning: "#f1fa8c", ok: "#50fa7b",
    frame: "#21222c", console: "#21222c", examples: "#21222c", editor: "#282a36", caret: "#f8f8f0",
    editorError: "#4a292f", line: "#303241", selection: "#44475a", gutter: "#6272a4",
    keyword: "#ff79c6", type: "#8be9fd", function: "#50fa7b", logical: "#ffb86c",
    number: "#bd93f9", string: "#f1fa8c", comment: "#6272a4", operator: "#ff79c6",
  }),
  "preset:catppuccin-mocha": theme("Catppuccin Mocha", "dark", {
    bg: "#1e1e2e", surface: "#181825", border: "#45475a", text: "#cdd6f4", muted: "#a6adc8",
    accent: "#cba6f7", accentText: "#1e1e2e", error: "#f38ba8", warning: "#f9e2af", ok: "#a6e3a1",
    frame: "#181825", console: "#181825", examples: "#181825", editor: "#1e1e2e", caret: "#f5e0dc",
    editorError: "#442d3a", line: "#242438", selection: "#45475a", gutter: "#6c7086",
    keyword: "#cba6f7", type: "#89dceb", function: "#89b4fa", logical: "#fab387",
    number: "#fab387", string: "#a6e3a1", comment: "#a6adc8", operator: "#94e2d5",
  }),
  "preset:night-owl": theme("Night Owl", "dark", {
    bg: "#011627", surface: "#0b253a", border: "#1d3b53", text: "#d6deeb", muted: "#8ea6b0",
    accent: "#82aaff", accentText: "#011627", error: "#ef5350", warning: "#ffeb95", ok: "#addb67",
    frame: "#01111d", console: "#0b253a", examples: "#0b253a", editor: "#011627", caret: "#80a4c2",
    editorError: "#442f3c", line: "#0b253a", selection: "#1d3b53", gutter: "#5f7e97",
    keyword: "#c792ea", type: "#ffcb8b", function: "#82aaff", logical: "#ff5874",
    number: "#f78c6c", string: "#ecc48d", comment: "#637777", operator: "#7fdbca",
  }),
  "preset:portugol-studio-next": theme("Portugol Studio Next", "dark", {
    bg: "#0c1118", surface: "#111923", border: "#263241", text: "#e6edf3", muted: "#9aa9b8",
    accent: "#34d399", accentText: "#07140f", error: "#f87171", warning: "#fbbf24", ok: "#34d399",
    frame: "#090e14", console: "#111923", examples: "#111923", editor: "#0d141d", caret: "#34d399",
    editorError: "#3b2025", line: "#111923", selection: "#18352f", gutter: "#718096",
    keyword: "#c792ea", type: "#82aaff", function: "#7dd3fc", logical: "#34d399",
    number: "#fbbf24", string: "#a5d6a7", comment: "#718096", operator: "#89ddff",
  }),
};
