// Acesso a arquivos .fer locais: File System Access API quando disponível,
// com fallback de <input type="file"> e download por Blob.

const FERRO_TYPES = [
  { description: "Arquivos Ferro", accept: { "text/plain": [".fer"] } },
];

function context(host = {}) {
  const isWindow =
    typeof host.showOpenFilePicker === "function" || typeof host.showSaveFilePicker === "function";
  const win = host.window ?? (isWindow ? host : globalThis);
  const doc = host.document ?? win?.document ?? globalThis.document;
  const createObjectURL = host.createObjectURL ?? ((blob) => URL.createObjectURL(blob));
  const revokeObjectURL = host.revokeObjectURL ?? ((url) => URL.revokeObjectURL(url));
  const schedule = host.setTimeout ?? ((fn, delay) => globalThis.setTimeout(fn, delay));
  return { win, doc, createObjectURL, revokeObjectURL, schedule };
}

function isAbort(error) {
  return error?.name === "AbortError";
}

/** Abre um .fer e devolve { name, content, handle } ou null quando cancelado. */
export async function openFerFile(host) {
  const { win, doc } = context(host);

  if (typeof win?.showOpenFilePicker === "function") {
    let handles;
    try {
      handles = await win.showOpenFilePicker({ multiple: false, types: FERRO_TYPES });
    } catch (error) {
      if (isAbort(error)) return null;
      throw error;
    }
    const handle = handles?.[0];
    if (!handle) return null;
    const file = await handle.getFile();
    return { name: file.name, content: await file.text(), handle };
  }

  if (!doc?.createElement) throw new Error("Não há seletor de arquivos disponível neste navegador.");
  return new Promise((resolve, reject) => {
    const input = doc.createElement("input");
    const parent = doc.body ?? doc.documentElement;
    input.type = "file";
    input.accept = ".fer,text/plain";
    parent.append(input);
    let settled = false;
    const finish = (value, error) => {
      if (settled) return;
      settled = true;
      if (typeof input.remove === "function") input.remove();
      else parent.removeChild(input);
      if (error) reject(error);
      else resolve(value);
    };
    input.addEventListener("change", async () => {
      try {
        const file = input.files?.[0];
        if (!file) {
          finish(null);
          return;
        }
        finish({ name: file.name, content: await file.text(), handle: null });
      } catch (error) {
        finish(null, error);
      }
    });
    input.addEventListener("cancel", () => finish(null));
    input.click();
  });
}

/** Salva o conteúdo no handle existente, em novo arquivo nativo ou por download. */
export async function saveFerFile(host, { name, content, handle = null }) {
  const { win, doc, createObjectURL, revokeObjectURL, schedule } = context(host);
  let target = handle;

  if (!target && typeof win?.showSaveFilePicker === "function") {
    try {
      target = await win.showSaveFilePicker({ suggestedName: name, types: FERRO_TYPES });
    } catch (error) {
      if (isAbort(error)) return null;
      throw error;
    }
  }

  if (target) {
    const writable = await target.createWritable();
    await writable.write(content);
    await writable.close();
    return { name: target.name ?? name, handle: target, downloaded: false };
  }

  if (!doc?.createElement) throw new Error("Não há forma de salvar arquivos neste navegador.");
  const blob = new Blob([content], { type: "text/plain;charset=utf-8" });
  const url = createObjectURL(blob);
  const anchor = doc.createElement("a");
  anchor.href = url;
  anchor.download = name;
  (doc.body ?? doc.documentElement).append(anchor);
  anchor.click();
  if (typeof anchor.remove === "function") anchor.remove();
  else (doc.body ?? doc.documentElement).removeChild(anchor);
  schedule(() => revokeObjectURL(url), 0);
  return { name, handle: null, downloaded: true };
}
