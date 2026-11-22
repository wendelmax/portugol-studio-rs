import test from "node:test";
import assert from "node:assert/strict";

const stateModule = await import("../src/rust-preview-state.ts");
const { beginRustPreview, markRustPreviewStale, acceptRustPreview, failRustPreview, generatedRustComparison } = stateModule;

const map = [{
  generatedStart: { offset: 0, line: 1, column: 1 },
  generatedEnd: { offset: 8, line: 1, column: 9 },
  sourceStart: { offset: 0, line: 1, column: 1 },
  sourceEnd: { offset: 8, line: 1, column: 9 },
  quality: "exact",
}];

function artifact(source, code = "fn main() {}", sourceMap = map) {
  return { source, code, sourceMap };
}

test("primeira geração não inventa artefato e entra em estado pendente", () => {
  assert.deepEqual(beginRustPreview(undefined, "escreva(1)"), {
    artifact: undefined,
    pending: true,
    stale: false,
    error: undefined,
  });
});

test("edição conserva o último Rust e os mapas, marcando o artefato stale", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: false, stale: false, error: undefined };
  const next = beginRustPreview(previous, "escreva(2)");
  assert.equal(next.artifact, previous.artifact);
  assert.equal(next.artifact.code, "fn main() {}");
  assert.deepEqual(next.artifact.sourceMap, map);
  assert.equal(next.pending, true);
  assert.equal(next.stale, true);
});

test("revisitar a mesma fonte mantém o artefato atual durante a atualização", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: false, stale: false, error: undefined };
  const next = beginRustPreview(previous, "escreva(1)");
  assert.equal(next.artifact, previous.artifact);
  assert.equal(next.pending, true);
  assert.equal(next.stale, false);
});

test("nova tentativa não revalida um artefato que já falhou para a fonte atual", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: false, stale: true, error: "IPC offline" };
  const next = beginRustPreview(previous, "escreva(1)");
  assert.equal(next.artifact, previous.artifact);
  assert.equal(next.pending, true);
  assert.equal(next.stale, true);
});

test("edição sem regeneração pendente marca o artefato stale sem simular carregamento", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: false, stale: false, error: undefined };
  const next = markRustPreviewStale(previous, "escreva(2)");
  assert.equal(next.artifact, previous.artifact);
  assert.equal(next.pending, false);
  assert.equal(next.stale, true);
});

test("resultado atual substitui o artefato e libera o estado stale", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: true, stale: true, error: undefined };
  const next = acceptRustPreview(previous, "escreva(2)", {
    code: "fn main() { println!(\"2\"); }",
    diagnostic: null,
    sourceMap: [],
  });
  assert.deepEqual(next, {
    artifact: { source: "escreva(2)", code: "fn main() { println!(\"2\"); }", sourceMap: [] },
    previousArtifact: artifact("escreva(1)"),
    pending: false,
    stale: false,
    error: undefined,
  });
});

test("nova geração válida diferente conserva o artefato anterior para comparação", () => {
  const previousArtifact = artifact("escreva(1)", "fn main() { println!(\"1\"); }");
  const state = { artifact: previousArtifact, pending: true, stale: true, error: undefined };
  const next = acceptRustPreview(state, "escreva(2)", {
    code: "fn main() { println!(\"2\"); }",
    diagnostic: null,
    sourceMap: [],
  });

  assert.equal(next.previousArtifact, previousArtifact);
  assert.equal(next.artifact.code, "fn main() { println!(\"2\"); }");
  assert.equal(next.stale, false);
});

test("regeneração idêntica não mantém um diff de uma revisão antiga", () => {
  const previousArtifact = artifact("escreva(1)", "fn main() {}");
  const olderArtifact = artifact("escreva(0)", "fn main() { /* antigo */ }");
  const next = acceptRustPreview({
    artifact: previousArtifact,
    previousArtifact: olderArtifact,
    pending: true,
    stale: false,
  }, "escreva(1)", {
    code: "fn main() {}",
    diagnostic: null,
    sourceMap: [],
  });

  assert.equal(next.previousArtifact, undefined);
});

test("pendente e stale preservam a comparação sem apresentá-la como atual", () => {
  const currentArtifact = artifact("escreva(2)", "fn main() { println!(\"2\"); }");
  const previousArtifact = artifact("escreva(1)", "fn main() { println!(\"1\"); }");
  const state = { artifact: currentArtifact, previousArtifact, pending: false, stale: false, error: undefined };

  const pending = beginRustPreview(state, "escreva(3)");
  assert.equal(pending.previousArtifact, previousArtifact);
  assert.equal(pending.pending, true);
  assert.equal(pending.stale, true);

  const stale = markRustPreviewStale(state, "escreva(3)");
  assert.equal(stale.previousArtifact, previousArtifact);
  assert.equal(stale.stale, true);
});

test("comparação só fica disponível entre duas traduções diferentes e atuais", () => {
  const previousArtifact = artifact("escreva(1)", "fn main() { println!(\"1\"); }");
  const currentArtifact = artifact("escreva(2)", "fn main() { println!(\"2\"); }");
  const ready = { artifact: currentArtifact, previousArtifact, pending: false, stale: false };
  assert.deepEqual(generatedRustComparison(ready), { previous: previousArtifact, current: currentArtifact });
  assert.equal(generatedRustComparison({ ...ready, stale: true }), null);
  assert.equal(generatedRustComparison({ ...ready, pending: true }), null);
  assert.equal(generatedRustComparison({ ...ready, previousArtifact: undefined }), null);
  assert.equal(generatedRustComparison({ ...ready, previousArtifact: artifact("antiga", currentArtifact.code) }), null);
});

test("diagnóstico atual mantém o artefato anterior claramente stale", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: true, stale: true, error: undefined };
  const next = acceptRustPreview(previous, "escreva(2", {
    code: null,
    diagnostic: { start: { line: 2, column: 4 }, message: "expressão incompleta" },
    sourceMap: [],
  });
  assert.equal(next.artifact, previous.artifact);
  assert.equal(next.pending, false);
  assert.equal(next.stale, true);
  assert.match(next.error, /expressão incompleta/);
  assert.equal(next.error, "Linha 2, coluna 4: expressão incompleta");
});

test("resposta parcial com diagnóstico não substitui o último Rust válido", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: true, stale: false, error: undefined };
  const next = acceptRustPreview(previous, "escreva(2", {
    code: "fn main() { /* parcial */ }",
    diagnostic: { start: { line: 2, column: 4 }, message: "expressão incompleta" },
    sourceMap: [],
  });
  assert.equal(next.artifact, previous.artifact);
  assert.equal(next.stale, true);
  assert.equal(next.error, "Linha 2, coluna 4: expressão incompleta");
});

test("falha da solicitação preserva o artefato anterior e não afirma sucesso", () => {
  const previous = { artifact: artifact("escreva(1)"), pending: true, stale: true, error: undefined };
  const next = failRustPreview(previous, "IPC offline");
  assert.equal(next.artifact, previous.artifact);
  assert.equal(next.pending, false);
  assert.equal(next.stale, true);
  assert.equal(next.error, "IPC offline");
});

test("falha sem artefato anterior não cria código Rust", () => {
  assert.deepEqual(failRustPreview(beginRustPreview(undefined, "fonte"), "IPC offline"), {
    artifact: undefined,
    pending: false,
    stale: false,
    error: "IPC offline",
  });
});
