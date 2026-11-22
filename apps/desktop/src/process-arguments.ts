/** Parse a small, predictable argv notation without invoking a shell. */
export function parseProcessArguments(text: string): string[] {
  const args: string[] = [];
  let token = "";
  let inQuotes = false;
  let started = false;

  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (character === "\\") {
      const next = text[index + 1];
      if (next === '"' || next === "\\" || (!inQuotes && next !== undefined && /\s/u.test(next))) {
        token += next;
        index += 1;
      } else {
        token += character;
      }
      started = true;
      continue;
    }
    if (character === '"') {
      inQuotes = !inQuotes;
      started = true;
      continue;
    }
    if (!inQuotes && /\s/u.test(character)) {
      if (started) {
        args.push(token);
        token = "";
        started = false;
      }
      continue;
    }
    token += character;
    started = true;
  }

  if (inQuotes) throw new Error("As aspas duplas não foram fechadas nos argumentos.");
  if (started) args.push(token);
  return args;
}
