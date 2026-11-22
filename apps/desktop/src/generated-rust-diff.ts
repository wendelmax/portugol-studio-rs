export type GeneratedRustDiffLineKind = "context" | "removed" | "added";

export interface GeneratedRustDiffLine {
  kind: GeneratedRustDiffLineKind;
  oldLine: number | null;
  newLine: number | null;
  text: string;
}

export interface GeneratedRustDiff {
  rows: GeneratedRustDiffLine[];
  added: number;
  removed: number;
  coarse: boolean;
  truncated: boolean;
}

export function generatedRustDiffSummary(diff: GeneratedRustDiff): string {
  const addedLabel = diff.added === 1 ? "linha adicionada" : "linhas adicionadas";
  const removedLabel = diff.removed === 1 ? "linha removida" : "linhas removidas";
  let summary = `${diff.added} ${addedLabel}, ${diff.removed} ${removedLabel}.`;
  if (diff.coarse) summary += " Arquivo extenso: diferenças agrupadas por trecho.";
  if (diff.truncated) summary += " Exibição limitada às primeiras linhas para manter a interface responsiva.";
  return summary;
}

const MAX_LCS_CELLS = 1_000_000;
const MAX_RENDERED_ROWS = 10_000;

export function diffGeneratedRust(previous: string, current: string): GeneratedRustDiff {
  const oldLines = previous.split(/\r?\n/);
  const newLines = current.split(/\r?\n/);
  const result: GeneratedRustDiff = { rows: [], added: 0, removed: 0, coarse: false, truncated: false };
  const append = (kind: GeneratedRustDiffLineKind, oldLine: number | null, newLine: number | null, text: string) => {
    if (kind === "added") result.added++;
    if (kind === "removed") result.removed++;
    if (result.rows.length < MAX_RENDERED_ROWS) result.rows.push({ kind, oldLine, newLine, text });
    else result.truncated = true;
  };

  const cells = (oldLines.length + 1) * (newLines.length + 1);
  if (cells > MAX_LCS_CELLS) {
    result.coarse = true;
    let prefix = 0;
    while (prefix < oldLines.length && prefix < newLines.length && oldLines[prefix] === newLines[prefix]) {
      append("context", prefix + 1, prefix + 1, oldLines[prefix]);
      prefix++;
    }
    let suffix = 0;
    while (suffix < oldLines.length - prefix && suffix < newLines.length - prefix
      && oldLines[oldLines.length - suffix - 1] === newLines[newLines.length - suffix - 1]) suffix++;
    for (let i = prefix; i < oldLines.length - suffix; i++) append("removed", i + 1, null, oldLines[i]);
    for (let i = prefix; i < newLines.length - suffix; i++) append("added", null, i + 1, newLines[i]);
    for (let i = suffix; i > 0; i--) {
      const oldIndex = oldLines.length - i;
      const newIndex = newLines.length - i;
      append("context", oldIndex + 1, newIndex + 1, oldLines[oldIndex]);
    }
    return result;
  }

  const width = newLines.length + 1;
  const lcs = new Uint32Array(cells);
  for (let oldIndex = oldLines.length - 1; oldIndex >= 0; oldIndex--) {
    for (let newIndex = newLines.length - 1; newIndex >= 0; newIndex--) {
      const index = oldIndex * width + newIndex;
      lcs[index] = oldLines[oldIndex] === newLines[newIndex]
        ? lcs[(oldIndex + 1) * width + newIndex + 1] + 1
        : Math.max(lcs[(oldIndex + 1) * width + newIndex], lcs[index + 1]);
    }
  }

  let oldIndex = 0;
  let newIndex = 0;
  while (oldIndex < oldLines.length && newIndex < newLines.length) {
    if (oldLines[oldIndex] === newLines[newIndex]) {
      append("context", oldIndex + 1, newIndex + 1, oldLines[oldIndex]);
      oldIndex++;
      newIndex++;
    } else if (lcs[(oldIndex + 1) * width + newIndex] >= lcs[oldIndex * width + newIndex + 1]) {
      append("removed", oldIndex + 1, null, oldLines[oldIndex]);
      oldIndex++;
    } else {
      append("added", null, newIndex + 1, newLines[newIndex]);
      newIndex++;
    }
  }
  while (oldIndex < oldLines.length) {
    append("removed", oldIndex + 1, null, oldLines[oldIndex]);
    oldIndex++;
  }
  while (newIndex < newLines.length) {
    append("added", null, newIndex + 1, newLines[newIndex]);
    newIndex++;
  }
  return result;
}
