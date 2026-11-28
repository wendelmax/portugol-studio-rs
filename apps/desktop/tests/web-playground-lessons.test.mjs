import assert from "node:assert/strict";
import { test } from "node:test";
import { LESSONS, LESSON_SOLUTIONS, checkLesson } from "../../web-playground/lessons.mjs";

test("trilha tem seis lições completas com soluções distintas do ponto de partida", () => {
  assert.deepEqual(
    LESSONS.map((lesson) => lesson.id),
    ["variaveis", "decisoes", "lacos", "vetores", "cadeias", "resultado"],
  );
  assert.deepEqual(Object.keys(LESSON_SOLUTIONS), LESSONS.map((lesson) => lesson.id));

  for (const lesson of LESSONS) {
    for (const field of ["id", "title", "prompt", "hint", "starter", "stdin", "expected"]) {
      assert.ok(field in lesson, `${lesson.id}: campo ${field} ausente`);
    }
    assert.equal(typeof lesson.expected, "object", `${lesson.id}: expected deve ser lista`);
    assert.notEqual(LESSON_SOLUTIONS[lesson.id], lesson.starter, `${lesson.id}: solução igual ao starter`);
  }
});

test("checkLesson compara stdout linha a linha com o stdin fixo da lição", () => {
  const lesson = LESSONS[0];
  let receivedStdin = null;
  const runtime = {
    run(source, stdin) {
      receivedStdin = stdin;
      assert.equal(source, "funcao inicio() {}");
      return { ok: true, stdout: "Olá, Ada\n", diagnostics: [] };
    },
  };

  const passed = checkLesson(runtime, lesson, "funcao inicio() {}");
  assert.equal(receivedStdin, lesson.stdin);
  assert.equal(passed.ok, true);
  assert.deepEqual(passed.actual, ["Olá, Ada"]);

  runtime.run = () => ({ ok: true, stdout: "errado\n", diagnostics: [] });
  const failed = checkLesson(runtime, lesson, "funcao inicio() {}");
  assert.equal(failed.ok, false);
  assert.deepEqual(failed.actual, ["errado"]);
  assert.deepEqual(failed.expected, ["Olá, Ada"]);
});

test("checkLesson preserva linha vazia no meio e reprova erro de compilação", () => {
  const lesson = { stdin: "fixo\n", expected: ["a", "", "b"] };
  const diagnostics = [{ code: "compile", message: "falha" }];
  const runtime = { run: () => ({ ok: false, stdout: "a\n\nb\n", diagnostics }) };

  const result = checkLesson(runtime, lesson, "funcao inicio() {}");

  assert.equal(result.ok, false);
  assert.deepEqual(result.actual, ["a", "", "b"]);
  assert.deepEqual(result.diagnostics, diagnostics);
});
