import { BUILTIN_THEMES, THEME_COLOR_FIELDS, type Base, type Colors, type Theme } from "./theme-presets";

const fields = {
  "--frame": "Moldura e tela inicial",
  "--bg": "Fundo da janela",
  "--surface": "Painéis",
  "--border": "Bordas",
  "--text": "Texto",
  "--muted": "Texto secundário",
  "--accent": "Destaque",
  "--accent-text": "Texto do destaque",
  "--error": "Erros",
  "--error-text": "Texto sobre erros",
  "--warning": "Avisos",
  "--ok": "Sucesso",
  "--input-echo": "Entrada digitada",
  "--tab-active": "Aba selecionada",
  "--home-border": "Borda da tela inicial",
  "--home-title-text": "Título da tela inicial",
  "--examples-surface": "Painel de exemplos",
  "--console-surface": "Console",
  "--editor-bg": "Fundo do editor",
  "--editor-text": "Identificadores",
  "--editor-caret": "Cursor",
  "--editor-error": "Linha com erro",
  "--editor-line": "Linha atual",
  "--editor-selection": "Seleção",
  "--editor-gutter": "Números de linha",
  "--syntax-keyword": "Palavras reservadas",
  "--syntax-type": "Tipos",
  "--syntax-function": "Funções",
  "--syntax-logical": "Valores lógicos",
  "--syntax-number": "Números",
  "--syntax-string": "Cadeias e caracteres",
  "--syntax-interpolation": "Interpolação em cadeias",
  "--syntax-comment": "Comentários",
  "--syntax-operator": "Operadores",
} as const;

const colorKeys = [...THEME_COLOR_FIELDS];
const storeKey = "ferro.customThemes";
const validColor = (value: unknown): value is string => typeof value === "string" && /^#[0-9a-f]{6}$/i.test(value);
const key = (name: string) => `custom:${name}`;

function readThemes(): Map<string, Theme> {
  try {
    const saved: unknown = JSON.parse(localStorage.getItem(storeKey) ?? "[]");
    if (!Array.isArray(saved)) return new Map();
    return new Map(saved.flatMap((item) => {
      if (!item || typeof item.name !== "string" || !item.name.trim() || !["light", "dark"].includes(item.base)) return [];
      const colors: Colors = {};
      for (const field of colorKeys) {
        const value: unknown = item.colors?.[field];
        if (validColor(value)) colors[field] = value;
      }
      return [[key(item.name), { name: item.name, base: item.base, colors }]];
    }));
  } catch {
    return new Map();
  }
}

const themes = readThemes();
export const customTheme = (choice: string) => themes.get(choice);
export function themeProfile(choice: string): Theme | undefined {
  return BUILTIN_THEMES[choice] ?? customTheme(choice);
}

export function applyThemeColors(element: HTMLElement, colors: Colors = {}) {
  for (const field of colorKeys) {
    const value = colors[field];
    if (value) element.style.setProperty(field, value);
    else element.style.removeProperty(field);
  }
}

export function mountThemeEditor(select: HTMLSelectElement, apply: (choice: string) => void) {
  const dialog = document.querySelector<HTMLDialogElement>("#theme-editor")!;
  const form = document.querySelector<HTMLFormElement>("#theme-form")!;
  const list = document.querySelector<HTMLSelectElement>("#theme-profiles")!;
  const name = document.querySelector<HTMLInputElement>("#theme-name")!;
  const base = document.querySelector<HTMLSelectElement>("#theme-base")!;
  const patch = document.querySelector<HTMLElement>("#theme-colors")!;
  const preview = document.querySelector<HTMLElement>("#theme-preview")!;
  const error = document.querySelector<HTMLElement>("#theme-error")!;
  const remove = document.querySelector<HTMLButtonElement>("#theme-remove")!;
  let colors: Colors = {};

  function syncChoices() {
    select.querySelector('[data-custom-themes]')?.remove();
    if (!themes.size) return;
    const group = document.createElement("optgroup");
    group.label = "Personalizados";
    group.dataset.customThemes = "";
    for (const [id, theme] of themes) group.append(new Option(theme.name, id));
    select.append(group);
  }

  function load(theme?: Theme) {
    name.value = theme?.name ?? "";
    base.value = theme?.base ?? themeProfile(select.value)?.base ??
      (document.documentElement.matches('[data-theme="dark"]') ||
        (select.value === "system" && matchMedia("(prefers-color-scheme: dark)").matches) ? "dark" : "light");
    colors = { ...(theme?.colors ?? themeProfile(select.value)?.colors) };
    remove.disabled = !theme;
    error.hidden = true;
    render();
  }

  function render() {
    preview.dataset.theme = base.value;
    applyThemeColors(preview, colors);
    patch.replaceChildren();
    // A paleta da prévia fornece os valores padrão da base selecionada.
    const computed = getComputedStyle(preview);
    for (const field of colorKeys) {
      const label = document.createElement("label");
      const input = document.createElement("input");
      input.type = "color";
      input.value = computed.getPropertyValue(field).trim();
      input.oninput = () => {
        colors[field] = input.value;
        applyThemeColors(preview, colors);
      };
      label.append(fields[field], input);
      patch.append(label);
    }
  }

  function persist() {
    try {
      localStorage.setItem(storeKey, JSON.stringify([...themes.values()]));
    } catch {
      // As personalizações continuam disponíveis nesta sessão.
    }
    syncChoices();
  }

  function open() {
    list.replaceChildren(new Option("Novo tema", ""));
    for (const [id, theme] of themes) list.add(new Option(theme.name, id));
    list.value = themes.has(select.value) ? select.value : "";
    dialog.showModal();
    load(themes.get(list.value));
  }

  list.onchange = () => load(themes.get(list.value));
  base.onchange = render;
  document.querySelector<HTMLButtonElement>("#theme-reset")!.onclick = () => {
    colors = {};
    render();
  };
  document.querySelector<HTMLButtonElement>("#theme-apply-base")!.onclick = () => {
    select.value = base.value;
    apply(base.value);
    dialog.close();
  };
  document.querySelector<HTMLButtonElement>("#theme-cancel")!.onclick = () => dialog.close();
  remove.onclick = () => {
    const previous = select.value;
    const theme = themes.get(list.value);
    if (!theme) return;
    themes.delete(list.value);
    persist();
    const choice = list.value === previous ? theme.base : previous;
    select.value = choice;
    apply(choice);
    dialog.close();
  };
  form.onsubmit = (event) => {
    event.preventDefault();
    const clean = name.value.trim();
    const id = key(clean);
    if (!clean || (id !== list.value && themes.has(id))) {
      error.textContent = "Escolha um nome de tema diferente.";
      error.hidden = false;
      return;
    }
    if (list.value !== id) themes.delete(list.value);
    themes.set(id, { name: clean, base: base.value as Base, colors });
    persist();
    select.value = id;
    apply(id);
    dialog.close();
  };

  syncChoices();
  return { open };
}
