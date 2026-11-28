import test from "node:test";
import assert from "node:assert/strict";

const { explainMappedConcept, learningContextAtSelection } = await import("../src/learning-context.ts");

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

test("maps exact conditionals and loops to their learning concepts", () => {
  const countedSource = "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 3; i += 1) { escreva(i); } }";
  const snippets = [
    {
      conceptId: "conditionals",
      source: "se (idade >= 18) { escreva(idade); }",
      rust: "if idade >= 18 {\n    println!(\"{idade}\");\n}",
    },
    {
      conceptId: "loops",
      source: "enquanto (contador < 3) { contador++; }",
      rust: "while contador < 3 {\n    contador += 1;\n}",
    },
    {
      conceptId: "loops",
      source: countedSource,
      rust: `{\n    let mut pt_i: i32 = 0_i32;\n    'ferro_for_${countedSource.indexOf("para")}: while pt_i < 3_i32 {\n        println!("{pt_i}");\n        pt_i = (pt_i).wrapping_add(1_i32);\n    }\n}`,
    },
  ];

  for (const { conceptId, source, rust } of snippets) {
    const result = learningContextAtSelection({
      source,
      selectionFrom: 0,
      selectionTo: source.length,
      artifact: { source, code: rust, sourceMap: [mapping(0, source.length, 0, rust.length)] },
      stale: false,
    });
    assert.equal(result?.conceptId, conceptId);
    assert.equal(result?.sourceSnippet, source);
    assert.equal(result?.rustSnippet, rust);
    assert.match(explainMappedConcept(result, "portugol-rs"), conceptId === "conditionals" ? /decisão/ : /repetição/);
  }
});

test("does not infer a learning concept from unrelated generated Rust", () => {
  const source = "escreva(42);";
  const rust = "println!(\"{}\", 42_i32);";
  assert.equal(learningContextAtSelection({
    source,
    selectionFrom: 0,
    selectionTo: source.length,
    artifact: { source, code: rust, sourceMap: [mapping(0, source.length, 0, rust.length)] },
    stale: false,
  }), null);
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

test("explains mapped mutability for Ferro", () => {
  const mapped = context();
  assert.match(explainMappedConcept(mapped, "portugol-rs"), /Ferro/);
  assert.match(explainMappedConcept(mapped, "portugol-rs"), /let mut/);
});

test("maps exact PORS functions, match/Option/Result, Vec collections, and struct/enum declarations", () => {
  const cases = [
    {
      conceptId: "functions",
      source: "funcao dobro(x: inteiro) -> inteiro { retorne x * 2; }",
      rust: "fn dobro(x: i32) -> i32 { x * 2 }",
      explanation: /fn tipada/,
    },
    {
      conceptId: "option-result",
      source: "escolha res { caso Resultado::Ok(v) => { escreva(v); } caso Resultado::Erro(e) => { escreva(e); } }",
      rust: "match res { Result::Ok(v) => println!(\"{v}\"), Result::Err(e) => println!(\"{e}\") }",
      explanation: /match\/Option\/Result/,
    },
    {
      conceptId: "collections",
      source: "seja mutavel notas: vetor<inteiro> = [];",
      rust: "let mut notas: Vec<i32> = Vec::new();",
      explanation: /Vec<T>/,
    },
    {
      conceptId: "records-enums",
      source: "registro Ponto { x: inteiro, y: inteiro }",
      rust: "#[derive(Clone, Debug)]\nstruct Ponto { x: i32, y: i32 }",
      explanation: /struct ou enum/,
    },
  ];

  for (const item of cases) {
    const result = learningContextAtSelection({
      source: item.source,
      selectionFrom: 0,
      selectionTo: item.source.length,
      artifact: { source: item.source, code: item.rust, sourceMap: [mapping(0, item.source.length, 0, item.rust.length)] },
      stale: false,
    });
    assert.equal(result?.conceptId, item.conceptId);
    assert.equal(result?.provenance, "source-map");
    assert.match(explainMappedConcept(result, "portugol-rs"), item.explanation);
  }
});
