import test from "node:test";
import assert from "node:assert/strict";

const { explainMappedMutability, learningContextAtSelection } = await import("../src/learning-context.ts");

function position(offset) {
  return { offset, line: 1, column: offset + 1 };
}

function mapping(sourceFrom, sourceTo, generatedFrom, generatedTo, quality = "exact") {
  return {
    sourceStart: position(sourceFrom),
    sourceEnd: position(sourceTo),
    generatedStart: position(generatedFrom),
    generatedEnd: position(generatedTo),
    quality,
  };
}

function context(input) {
  return learningContextAtSelection({
    source: "inteiro idade = 16;",
    selectionFrom: 8,
    selectionTo: 8,
    artifact: {
      source: "inteiro idade = 16;",
      code: "fn main() {\n    let mut idade: i32 = 16_i32;\n}",
    sourceMap: [mapping(0, 19, 14, 44)],
    },
    stale: false,
    ...input,
  });
}

test("maps a source caret to a current mutable Rust binding", () => {
  assert.deepEqual(context(), {
    conceptId: "mutability",
    provenance: "source-map",
    bindingMutability: "mutable",
    sourceStart: 0,
    sourceEnd: 19,
    generatedStart: 14,
    generatedEnd: 44,
    sourceSnippet: "inteiro idade = 16;",
    rustSnippet: "let mut idade: i32 = 16_i32;",
  });
});

test("classifies immutable Rust let bindings without claiming mutability", () => {
  const source = "constante inteiro LIMITE = 3;";
  const code = "fn main() {\n    let LIMITE: i32 = 3_i32;\n}";
  const generatedFrom = code.indexOf("let LIMITE");
  const generatedTo = code.indexOf("\n", generatedFrom);
  const result = learningContextAtSelection({
    source,
    selectionFrom: source.indexOf("LIMITE"),
    selectionTo: source.indexOf("LIMITE") + "LIMITE".length,
    artifact: { source, code, sourceMap: [mapping(0, source.length, generatedFrom, generatedTo)] },
    stale: false,
  });
  assert.equal(result?.bindingMutability, "immutable");
  assert.match(result?.rustSnippet ?? "", /^let LIMITE/);
});

test("requires the caret or complete selection to remain inside a mapped source range", () => {
  const source = "prefix inteiro idade = 16; suffix";
  const code = "let mut idade: i32 = 16_i32;";
  const start = source.indexOf("inteiro");
  const end = start + "inteiro idade = 16;".length;
  const artifact = { source, code, sourceMap: [mapping(start, end, 0, code.length)] };

  assert.equal(learningContextAtSelection({ source, selectionFrom: start, selectionTo: start, artifact, stale: false })?.conceptId, "mutability");
  assert.equal(learningContextAtSelection({ source, selectionFrom: start - 1, selectionTo: end, artifact, stale: false }), null);
});

test("refuses stale artifacts and artifacts from another source revision", () => {
  const stale = context({ stale: true });
  const wrongRevision = context({ source: "inteiro idade = 17;" });
  assert.equal(stale, null);
  assert.equal(wrongRevision, null);
});

test("ignores approximate and generated-only maps", () => {
  const source = "inteiro idade = 16;";
  const code = "let mut idade: i32 = 16_i32;";
  for (const quality of ["approximate", "generated_only"]) {
    const result = learningContextAtSelection({
      source,
      selectionFrom: 8,
      selectionTo: 8,
      artifact: { source, code, sourceMap: [mapping(0, source.length, 0, code.length, quality)] },
      stale: false,
    });
    assert.equal(result, null, `quality ${quality} must not make a contextual claim`);
  }
});

test("chooses the narrowest overlapping exact mapping that identifies a binding", () => {
  const source = "programa { inteiro idade = 16; }";
  const code = "fn main() {\n    let mut idade: i32 = 16_i32;\n}";
  const declarationStart = source.indexOf("inteiro");
  const declarationEnd = source.indexOf(";", declarationStart) + 1;
  const rustStart = code.indexOf("let mut");
  const rustEnd = code.indexOf("\n", rustStart);
  const result = learningContextAtSelection({
    source,
    selectionFrom: source.indexOf("idade"),
    selectionTo: source.indexOf("idade") + 5,
    artifact: {
      source,
      code,
      sourceMap: [
        mapping(0, source.length, code.indexOf("fn main"), code.length),
        mapping(declarationStart, declarationEnd, rustStart, rustEnd),
      ],
    },
    stale: false,
  });
  assert.equal(result?.sourceStart, declarationStart);
  assert.equal(result?.sourceEnd, declarationEnd);
  assert.equal(result?.rustSnippet, "let mut idade: i32 = 16_i32;");
});

test("explains mutability according to the active source dialect", () => {
  const mapped = context();
  assert.match(explainMappedMutability(mapped, "portugol"), /Portugol clássico/);
  assert.match(explainMappedMutability(mapped, "portugol-rs"), /Portugol-RS/);
  assert.match(explainMappedMutability(mapped, "portugol-rs"), /let mut/);
});
