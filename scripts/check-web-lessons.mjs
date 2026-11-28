// Verifica as soluções da trilha contra o WASM real gerado pelo build.
import { readFileSync } from "node:fs";
import { PLAYGROUND_EXAMPLES, createPlaygroundRuntime } from "../apps/web-playground/playground-core.mjs";
import { LESSONS, LESSON_SOLUTIONS, checkLesson } from "../apps/web-playground/lessons.mjs";

const wasm = readFileSync(new URL("../apps/web-playground/dist/ferro_wasm.wasm", import.meta.url));
const { instance } = await WebAssembly.instantiate(wasm);
const runtime = createPlaygroundRuntime(instance.exports);

for (const lesson of LESSONS) {
  const result = checkLesson(runtime, lesson, LESSON_SOLUTIONS[lesson.id]);
  if (!result.ok) {
    throw new Error(
      `${lesson.id}: esperado ${JSON.stringify(result.expected)}, obtido ${JSON.stringify(result.actual)}` +
        (result.diagnostics.length ? ` (${result.diagnostics.map((d) => d.message).join("; ")})` : ""),
    );
  }
  console.log(`ok ${lesson.id}`);
}

for (const example of PLAYGROUND_EXAMPLES) {
  const result = runtime.run(example.source, example.input);
  if (!result.ok) {
    throw new Error(`exemplo ${example.id}: ${result.error ?? "falha ao executar"}`);
  }
  console.log(`ok exemplo ${example.id}`);
}
