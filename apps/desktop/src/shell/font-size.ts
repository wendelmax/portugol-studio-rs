// Tamanho das fontes do editor e do console, salvo entre sessões.

const MIN_SIZE = 10;
const MAX_SIZE = 50;
const FALLBACK_SIZE = 12;

/** Limita o tamanho digitado ao intervalo aceito. */
export function clampFontSize(value: number): number {
  return Math.round(Math.max(MIN_SIZE, Math.min(MAX_SIZE, value || FALLBACK_SIZE)));
}

/** Liga cada campo `id` à variável CSS `--id` e à preferência `portugol.id`. */
export function mountFontSizeControls(ids: readonly string[]) {
  for (const id of ids) {
    const input = document.getElementById(id) as HTMLInputElement;
    const key = `portugol.${id}`;
    const applyFontSize = () => {
      input.value = String(clampFontSize(input.valueAsNumber));
      document.documentElement.style.setProperty(`--${id}`, `${input.value}px`);
      return input.value;
    };
    try {
      input.value = localStorage.getItem(key) ?? input.value;
    } catch {
      // O ajuste manual permanece disponível.
    }
    applyFontSize();
    input.onchange = () => {
      const value = applyFontSize();
      try {
        localStorage.setItem(key, value);
      } catch {
        // A preferência vale nesta sessão.
      }
    };
  }
}
