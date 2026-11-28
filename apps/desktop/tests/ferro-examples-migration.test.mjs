import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { FERRO_STDLIB_CATALOG } from "../src/ferro-stdlib-catalog.ts";
import { FERRO_HELP_LIBRARIES, FERRO_HELP_TOPICS } from "../src/ferro-help.ts";
import { FERRO_UI_EXAMPLES } from "../src/ferro-ui-examples.ts";
import { FERRO_TEMPLATE } from "../src/state/tabs.ts";
import { PLAYGROUND_EXAMPLES } from "../../web-playground/playground-core.mjs";

const repoRoot = new URL("../../../", import.meta.url);
const allowlistUrl = new URL("tests/fixtures/ferro-migration-allowlist.json", repoRoot);

// `leia()` é a forma preferida; o marcador versa sobre `leia(variavel)`.
const READ_WITH_ARGUMENT = /leia\(\s*[^\s)]/;
// Alvo: `+ "\n"` em .fer/Markdown e `+ "\\n"` dentro de strings TypeScript/JavaScript.
const NEWLINE_CONCAT = /\+\s*"\\{1,2}n"/;
// `escreva(...)` sem "\n" é sintaxe válida: escreval só acrescenta a quebra
// (RFC 003). Só a chamada que ainda passa "\n" explicitamente é legada.
const EXPLICIT_NEWLINE_WRITE = "escreva-com-quebra-explicita";

const MARKERS = [
  EXPLICIT_NEWLINE_WRITE,
  "leia(",
  ": inteiro = 0;",
  '+ "\\n"',
  "Tipos.para_inteiro(leia())",
];

const DOC_FILES = [
  "README.md",
  "apps/desktop/index.html",
  "apps/desktop/src/help-assets/topicos/linguagem_ferro/entrada_saida/escreva.html",
  "apps/desktop/src/help-assets/topicos/linguagem_ferro/entrada_saida/index.html",
  "apps/desktop/src/help-assets/topicos/linguagem_ferro/entrada_saida/leia.html",
  "docs/adr/0018-projetos-desktop-ferro-com-designer-visual.md",
  "docs/adr/0019-contrato-minimo-de-tipos-ferro.md",
  "docs/next-ui/product/LANGUAGE_SEMANTICS_CONTRACT.md",
  "docs/research/ferro_MIGRATION_MATRIX.md",
  "docs/superpowers/plans/2026-11-27-ferro-contratos-e-testes-de-aceite.md",
  "docs/superpowers/plans/2026-11-27-ferro-nucleo-sintaxe.md",
  "docs/superpowers/specs/2026-11-22-ferro-records-enums-design.md",
  "docs/superpowers/specs/2026-11-22-ferro-vectors-design.md",
  "docs/superpowers/specs/2026-11-26-ferro-syntax-evolution-design.md",
  "docs/superpowers/specs/2026-11-27-rfc-001-inferencia-em-seja.md",
  "docs/superpowers/specs/2026-11-27-rfc-002-leia-como-expressao.md",
  "docs/superpowers/specs/2026-11-27-rfc-003-escreval.md",
  "docs/superpowers/specs/2026-11-27-rfc-004-interpolacao-de-cadeias.md",
  "docs/superpowers/specs/2026-11-27-rfc-005-retorno-implicito.md",
  "docs/superpowers/specs/2026-11-27-rfc-007-tratamento-de-erros.md",
  "docs/superpowers/specs/2026-11-27-rfc-008-coalescencia-e-navegacao-segura.md",
];

function readText(relativePath) {
  return readFileSync(new URL(relativePath, repoRoot), "utf8");
}

function readAllowlist() {
  const entries = JSON.parse(readFileSync(allowlistUrl, "utf8"));
  assert.ok(Array.isArray(entries), "a allowlist deve ser um array");
  for (const [index, entry] of entries.entries()) {
    assert.ok(entry && typeof entry === "object", `entrada ${index} deve ser um objeto`);
    assert.ok(typeof entry.path === "string" && entry.path.length > 0, `entrada ${index} sem path`);
    assert.ok(Array.isArray(entry.markers), `entrada ${index} sem markers`);
    for (const marker of entry.markers) {
      assert.ok(MARKERS.includes(marker), `marcador desconhecido em ${entry.path}: ${marker}`);
    }
    assert.ok(
      typeof entry.reason === "string" && entry.reason.trim().length > 0,
      `entrada ${index} sem reason`,
    );
  }
  return entries;
}

function quotedLiteralHasNewline(inner) {
  for (let at = 0; at < inner.length; at += 1) {
    const delimiter = inner[at];
    if (delimiter !== '"' && delimiter !== "'") continue;
    at += 1;
    while (at < inner.length) {
      if (inner[at] === "\\") {
        let slashes = 0;
        while (at < inner.length && inner[at] === "\\") {
          slashes += 1;
          at += 1;
        }
        if (inner[at] === "n" && slashes >= 1 && slashes <= 2) return true;
        if (at < inner.length) at += 1;
        continue;
      }
      if (inner[at] === delimiter) break;
      at += 1;
    }
  }
  return false;
}

function hasExplicitNewlineWrite(source) {
  let from = 0;
  while (true) {
    const at = source.indexOf("escreva(", from);
    if (at < 0) return false;
    let depth = 0;
    let delimiter = null;
    let close = at + "escreva".length;
    for (; close < source.length; close += 1) {
      const char = source[close];
      if (delimiter) {
        if (char === "\\") {
          close += 1;
          continue;
        }
        if (char === delimiter) delimiter = null;
      } else if (char === '"' || char === "'") {
        delimiter = char;
      } else if (char === "(") {
        depth += 1;
      } else if (char === ")") {
        depth -= 1;
        if (depth === 0) break;
      }
    }
    if (quotedLiteralHasNewline(source.slice(at + "escreva(".length, close))) return true;
    from = close + 1;
  }
}

function markerPresent(source, marker) {
  if (marker === EXPLICIT_NEWLINE_WRITE) return hasExplicitNewlineWrite(source);
  if (marker === "leia(") return READ_WITH_ARGUMENT.test(source);
  if (marker === '+ "\\n"') return NEWLINE_CONCAT.test(source);
  return source.includes(marker);
}

function checkSources(sources, allowlist, label) {
  assert.ok(sources.length > 0, `${label}: nenhuma fonte coletada`);
  const exceptions = new Set(
    allowlist.flatMap((entry) => entry.markers.map((marker) => `${entry.path}\u0000${marker}`)),
  );
  const failures = [];
  for (const { path, source } of sources) {
    assert.ok(typeof source === "string" && source.length > 0, `${path}: fonte vazia`);
    for (const marker of MARKERS) {
      if (markerPresent(source, marker) && !exceptions.has(`${path}\u0000${marker}`)) {
        failures.push(`${path}: marcador legado ${JSON.stringify(marker)}`);
      }
    }
  }
  assert.equal(failures.length, 0, `${label}:\n${failures.join("\n")}`);
}

function embeddedSources() {
  return [
    { path: "apps/desktop/src/state/tabs.ts#FERRO_TEMPLATE", source: FERRO_TEMPLATE },
    ...FERRO_STDLIB_CATALOG.map((module) => ({
      path: `apps/desktop/src/ferro-stdlib-catalog.ts#${module.module}.example`,
      source: module.example.source,
    })),
    // FERRO_HELP_LIBRARIES é alias de FERRO_STDLIB_CATALOG; não varrer os mesmos
    // 13 exemplos duas vezes sob caminhos sintéticos diferentes.
    ...FERRO_HELP_LIBRARIES.filter((library) => !FERRO_STDLIB_CATALOG.includes(library)).map(
      (library) => ({
      path: `apps/desktop/src/ferro-help.ts#biblioteca:${library.module}.example`,
        source: library.example.source,
      }),
    ),
    ...FERRO_HELP_TOPICS.filter((topic) => topic.example).map((topic) => ({
      path: `apps/desktop/src/ferro-help.ts#${topic.id}.example`,
      source: topic.example.source,
    })),
    ...FERRO_UI_EXAMPLES.map((example) => ({
      path: `apps/desktop/src/ferro-ui-examples.ts#${example.id}.example`,
      source: example.source,
    })),
    ...PLAYGROUND_EXAMPLES.map((example) => ({
      path: `apps/web-playground/playground-core.mjs#${example.id}.example`,
      source: example.source,
    })),
  ];
}

test("embedded FERRO examples avoid legacy syntax outside the allowlist", () => {
  checkSources(embeddedSources(), readAllowlist(), "exemplos embutidos");
});

test("documented FERRO snippets avoid legacy syntax outside the allowlist", () => {
  const sources = DOC_FILES.map((path) => ({ path, source: readText(path) }));
  checkSources(sources, readAllowlist(), "documentação");
});

test("RFC 007 and RFC 008 warn about main and avoid invalid interpolation", () => {
  for (const name of [
    "2026-11-27-rfc-007-tratamento-de-erros",
    "2026-11-27-rfc-008-coalescencia-e-navegacao-segura",
  ]) {
    const source = readText(`docs/superpowers/specs/${name}.md`);
    assert.ok(source.includes("não executam no `main`"), `${name}: falta aviso de não execução no main`);
    for (const placeholder of ["{valor}", "{numero}", "{erro}"]) {
      assert.ok(!source.includes(placeholder), `${name}: interpolação inválida ${placeholder}`);
    }
  }
});

test("core syntax plan does not describe escreval as reserved", () => {
  const relative = "docs/superpowers/plans/2026-11-27-ferro-nucleo-sintaxe.md";
  const source = readText(relative);
  const forbidden = [
    {
      label: "escreval descrito como reservado",
      // Markdown permite crases e "é" entre o nome e o adjetivo.
      pattern: /escreval\s*`?\s*(?:é\s+)?reservad/i,
    },
    {
      label: "pedido para adicionar escreval às reservadas",
      pattern: (line) => line.includes("Adicionar `escreval` à lista de reservadas"),
    },
  ];

  const failures = [];
  source.split(/\r?\n/).forEach((line, index) => {
    for (const rule of forbidden) {
      const matched =
        typeof rule.pattern === "function" ? rule.pattern(line) : rule.pattern.test(line);
      if (matched) failures.push(`${relative}:${index + 1}: ${rule.label}: ${line.trim()}`);
    }
  });
  assert.equal(failures.length, 0, failures.join("\n"));
});
