// Código-fonte do frontend da IDE (main.ts e módulos em src/), concatenado.
// Testes que verificam ligações entre a interface e os comandos continuam
// válidos quando um trecho sai do main.ts para um módulo próprio.

import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const srcDir = fileURLToPath(new URL("../../src/", import.meta.url));

function collect(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return collect(path);
    return entry.name.endsWith(".ts") ? [path] : [];
  });
}

let cached;

/** `main.ts` primeiro, depois os demais módulos em ordem alfabética. */
export function ideSource() {
  if (cached === undefined) {
    const main = join(srcDir, "main.ts");
    const others = collect(srcDir).filter((path) => path !== main).sort();
    cached = [main, ...others].map((path) => readFileSync(path, "utf8")).join("\n");
  }
  return cached;
}
