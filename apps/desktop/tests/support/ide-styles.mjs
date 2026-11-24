// Folha de estilos completa da IDE, expandindo os @import de src/style.css
// na mesma ordem de cascata empacotada pelo Vite.

import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const entryPath = fileURLToPath(new URL("../../src/style.css", import.meta.url));

function expandImports(filePath, seen = new Set()) {
  if (seen.has(filePath)) return "";
  seen.add(filePath);
  const content = readFileSync(filePath, "utf8");
  const baseDir = dirname(filePath);
  return content.replace(/@import\s+["']([^"']+)["'];/g, (_, rel) => {
    const target = resolve(baseDir, rel);
    return expandImports(target, seen);
  });
}

let cached;

export function ideStyles() {
  if (cached === undefined) {
    cached = expandImports(entryPath);
  }
  return cached;
}
