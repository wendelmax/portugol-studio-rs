import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

test("Vite dev filesystem allowlist includes only the repository root needed for LICENSE", () => {
  const config = readFileSync(new URL("../vite.config.ts", import.meta.url), "utf8");

  assert.match(config, /const repositoryRoot = fileURLToPath\(new URL\("\.\.\/\.\.", import\.meta\.url\)\);/);
  assert.match(config, /fs:\s*\{\s*allow:\s*\[repositoryRoot\]\s*\}/);
});
