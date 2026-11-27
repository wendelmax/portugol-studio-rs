export const THEME_COLOR_FIELDS = [
  "--frame", "--bg", "--surface", "--border", "--text", "--muted", "--accent", "--accent-text",
  "--error", "--error-text", "--warning", "--ok", "--input-echo", "--tab-active", "--home-border", "--home-title-text",
  "--examples-surface", "--console-surface", "--editor-bg", "--editor-text", "--editor-caret",
  "--editor-error", "--editor-line", "--editor-selection", "--editor-gutter", "--syntax-keyword",
  "--syntax-type", "--syntax-function", "--syntax-logical", "--syntax-number", "--syntax-string",
  "--syntax-interpolation", "--syntax-comment", "--syntax-operator",
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
  errorText: string;
  homeTitleText: string;
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
  interpolation: string;
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
    "--error-text": p.errorText,
    "--warning": p.warning,
    "--ok": p.ok,
    "--input-echo": p.accent,
    "--tab-active": p.selection,
    "--home-border": p.border,
    "--home-title-text": p.homeTitleText,
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
    "--syntax-interpolation": p.interpolation,
    "--syntax-comment": p.comment,
    "--syntax-operator": p.operator,
  };
  return { name, base, colors };
}

// Paletas padronizadas do Portugol Studio RS: IntelliJ Rust (RustRover) e os temas claros e
// escuros mais usados do VS Code, calibrados para contraste WCAG AA (>= 4,5:1).
export const BUILTIN_THEMES: Readonly<Record<string, Theme>> = {
  "preset:intellij-rust-dark": theme("IntelliJ Rust Dark", "dark", {
    bg: "#1e1f22", surface: "#2b2d30", border: "#393b40", text: "#dfe1e5", muted: "#a3a7b0",
    accent: "#6b9eff", accentText: "#0f172a", error: "#f77681", errorText: "#1e1f22", warning: "#e5c07b", ok: "#73bd79",
    frame: "#18191c", homeTitleText: "#dfe1e5", console: "#25272a", examples: "#25272a", editor: "#1e1f22", caret: "#ced0d6",
    editorError: "#472529", line: "#26282e", selection: "#21385c", gutter: "#8c919c",
    keyword: "#e09f7d", type: "#d896d0", function: "#7cbcff", logical: "#e09f7d",
    number: "#56c8d3", string: "#82c78a", interpolation: "#d896d0", comment: "#b0b5bf", operator: "#dfe1e5",
  }),
  "preset:intellij-rust-light": theme("IntelliJ Rust Light", "light", {
    bg: "#ffffff", surface: "#f7f8fa", border: "#d3d5db", text: "#080808", muted: "#5a5d66",
    accent: "#285fd0", accentText: "#ffffff", error: "#c92a2a", errorText: "#ffffff", warning: "#8f5600", ok: "#067d17",
    frame: "#2b2d30", homeTitleText: "#ffffff", console: "#f2f4f7", examples: "#f7f8fa", editor: "#ffffff", caret: "#080808",
    editorError: "#ffe6e6", line: "#f2f5fc", selection: "#d7e8ff", gutter: "#5e626c",
    keyword: "#0033b3", type: "#7c1089", function: "#005a70", logical: "#0033b3",
    number: "#1746d4", string: "#066c14", interpolation: "#7c1089", comment: "#565a63", operator: "#080808",
  }),
  "preset:vscode-dark-modern": theme("VS Code Dark Modern", "dark", {
    bg: "#1f1f1f", surface: "#181818", border: "#3c3c3c", text: "#d4d4d4", muted: "#9d9d9d",
    accent: "#4daafc", accentText: "#0b1929", error: "#f48771", errorText: "#1f1f1f", warning: "#e2b93d", ok: "#89d185",
    frame: "#181818", homeTitleText: "#e0e0e0", console: "#181818", examples: "#181818", editor: "#1f1f1f", caret: "#aeafad",
    editorError: "#4b1f22", line: "#282828", selection: "#1d3b5a", gutter: "#969696",
    keyword: "#75b7f0", type: "#6ad9c3", function: "#dcdcaa", logical: "#75b7f0",
    number: "#b5cea8", string: "#e0a58d", interpolation: "#6ad9c3", comment: "#9cc986", operator: "#d4d4d4",
  }),
  "preset:vscode-light-modern": theme("VS Code Light Modern", "light", {
    bg: "#ffffff", surface: "#f8f8f8", border: "#d4d4d4", text: "#1f1f1f", muted: "#595959",
    accent: "#005fb8", accentText: "#ffffff", error: "#c72e0f", errorText: "#ffffff", warning: "#8a5a00", ok: "#167a32",
    frame: "#2c2c2c", homeTitleText: "#ffffff", console: "#f3f3f3", examples: "#f8f8f8", editor: "#ffffff", caret: "#000000",
    editorError: "#fde8e8", line: "#f3f6fa", selection: "#d6ebff", gutter: "#5e5e5e",
    keyword: "#0000df", type: "#126378", function: "#6b521f", logical: "#0000df",
    number: "#066e47", string: "#a31515", interpolation: "#126378", comment: "#006c00", operator: "#1f1f1f",
  }),
  "preset:github-light": theme("GitHub Light", "light", {
    bg: "#ffffff", surface: "#f6f8fa", border: "#d0d7de", text: "#1f2328", muted: "#656d76",
    accent: "#0969da", accentText: "#ffffff", error: "#cf222e", errorText: "#ffffff", warning: "#9a6700", ok: "#1a7f37",
    frame: "#24292f", homeTitleText: "#ffffff", console: "#f6f8fa", examples: "#f6f8fa", editor: "#ffffff", caret: "#0969da",
    editorError: "#ffebe9", line: "#f6f8fa", selection: "#ddf4ff", gutter: "#656d76",
    keyword: "#cf222e", type: "#0550ae", function: "#6f42c1", logical: "#953800",
    number: "#0550ae", string: "#0a3069", interpolation: "#0550ae", comment: "#57606a", operator: "#1f2328",
  }),
  "preset:github-dark": theme("GitHub Dark", "dark", {
    bg: "#0d1117", surface: "#161b22", border: "#30363d", text: "#e6edf3", muted: "#8b949e",
    accent: "#2f81f7", accentText: "#000000", error: "#f85149", errorText: "#000000", warning: "#d29922", ok: "#3fb950",
    frame: "#010409", homeTitleText: "#e6edf3", console: "#161b22", examples: "#161b22", editor: "#0d1117", caret: "#2f81f7",
    editorError: "#490202", line: "#161b22", selection: "#1b3a57", gutter: "#8b949e",
    keyword: "#ff7b72", type: "#79c0ff", function: "#d2a8ff", logical: "#ffab70",
    number: "#79c0ff", string: "#a5d6ff", interpolation: "#79c0ff", comment: "#b1bac4", operator: "#e6edf3",
  }),
  "preset:one-dark-pro": theme("One Dark Pro", "dark", {
    bg: "#282c34", surface: "#21252b", border: "#3e4451", text: "#abb2bf", muted: "#9da4b0",
    accent: "#61afef", accentText: "#282c34", error: "#ff8a82", errorText: "#000000", warning: "#e5c07b", ok: "#98c379",
    frame: "#21252b", homeTitleText: "#abb2bf", console: "#21252b", examples: "#21252b", editor: "#282c34", caret: "#528bff",
    editorError: "#3b2529", line: "#2c313a", selection: "#3e4451", gutter: "#a6adbb",
    keyword: "#e0a5f5", type: "#e5c07b", function: "#8cc8ff", logical: "#74d0dd",
    number: "#e6b886", string: "#98c379", interpolation: "#e5c07b", comment: "#c1c7d0", operator: "#74d0dd",
  }),
  "preset:dracula": theme("Dracula", "dark", {
    bg: "#282a36", surface: "#21222c", border: "#44475a", text: "#f8f8f2", muted: "#a5aec9",
    accent: "#bd93f9", accentText: "#282a36", error: "#ff5555", errorText: "#000000", warning: "#f1fa8c", ok: "#50fa7b",
    frame: "#21222c", homeTitleText: "#f8f8f2", console: "#21222c", examples: "#21222c", editor: "#282a36", caret: "#f8f8f0",
    editorError: "#4a292f", line: "#303241", selection: "#44475a", gutter: "#a6b0d1",
    keyword: "#ff9bd5", type: "#8be9fd", function: "#50fa7b", logical: "#ffb86c",
    number: "#d6b3ff", string: "#f1fa8c", interpolation: "#8be9fd", comment: "#c2c9e3", operator: "#ff9bd5",
  }),
  "preset:catppuccin-mocha": theme("Catppuccin Mocha", "dark", {
    bg: "#1e1e2e", surface: "#181825", border: "#45475a", text: "#cdd6f4", muted: "#a6adc8",
    accent: "#cba6f7", accentText: "#1e1e2e", error: "#f38ba8", errorText: "#000000", warning: "#f9e2af", ok: "#a6e3a1",
    frame: "#181825", homeTitleText: "#cdd6f4", console: "#181825", examples: "#181825", editor: "#1e1e2e", caret: "#f5e0dc",
    editorError: "#442d3a", line: "#242438", selection: "#45475a", gutter: "#a6adc8",
    keyword: "#d6b7ff", type: "#89dceb", function: "#9dc1ff", logical: "#fab387",
    number: "#fab387", string: "#a6e3a1", interpolation: "#89dceb", comment: "#c2c9e0", operator: "#94e2d5",
  }),
  "preset:night-owl": theme("Night Owl", "dark", {
    bg: "#011627", surface: "#0b253a", border: "#1d3b53", text: "#d6deeb", muted: "#8ea6b0",
    accent: "#82aaff", accentText: "#011627", error: "#ff756e", errorText: "#000000", warning: "#ffeb95", ok: "#addb67",
    frame: "#01111d", homeTitleText: "#d6deeb", console: "#0b253a", examples: "#0b253a", editor: "#011627", caret: "#80a4c2",
    editorError: "#442f3c", line: "#0b253a", selection: "#1d3b53", gutter: "#91a5b8",
    keyword: "#c792ea", type: "#ffcb8b", function: "#82aaff", logical: "#ff9ba6",
    number: "#f78c6c", string: "#ecc48d", interpolation: "#ffcb8b", comment: "#8ea6b0", operator: "#7fdbca",
  }),
  "preset:high-contrast": theme("Alto contraste", "dark", {
    bg: "#000000", surface: "#000000", border: "#ffffff", text: "#ffffff", muted: "#ffffff",
    accent: "#ffff00", accentText: "#000000", error: "#ff4d4f", errorText: "#000000", warning: "#ffff00", ok: "#00ff66",
    frame: "#000000", homeTitleText: "#ffffff", console: "#000000", examples: "#000000", editor: "#000000", caret: "#ffff00",
    editorError: "#5c0000", line: "#1a1a1a", selection: "#003399", gutter: "#ffffff",
    keyword: "#ff928c", type: "#7bdcff", function: "#ffffff", logical: "#ffff00",
    number: "#ffffff", string: "#78ffb3", interpolation: "#7bdcff", comment: "#ffffff", operator: "#f2f2f2",
  }),
};
