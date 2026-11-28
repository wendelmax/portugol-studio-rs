import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { PORS_STDLIB_CATALOG, parsePorsSignature } from "../src/pors-stdlib-catalog.ts";

const stdlibRs = readFileSync(
  new URL("../../../crates/ferro-frontend/src/parser/stdlib.rs", import.meta.url),
  "utf8",
);
const constantsRs = readFileSync(
  new URL("../../../crates/ferro-frontend/src/parser/constants.rs", import.meta.url),
  "utf8",
);

function catalogMap() {
  const map = new Map();
  for (const mod of PORS_STDLIB_CATALOG) {
    map.set(mod.module, new Map(mod.items.map((item) => [item.name, item])));
  }
  return map;
}

test("pors-stdlib-catalog covers 100% of stdlib functions defined in parser/stdlib.rs", () => {
  const modules = catalogMap();
  const compilerFunctions = new Set();

  for (const [, mod, name] of stdlibRs.matchAll(/\("([A-Z][A-Za-z]+)",\s*"([a-z_][a-z0-9_]*)"\)\s*=>/g)) {
    compilerFunctions.add(`${mod}.${name}`);
    const entry = modules.get(mod)?.get(name);
    assert.ok(entry, `Função ${mod}.${name} existe em stdlib.rs mas falta em pors-stdlib-catalog.ts`);
    assert.equal(entry.kind, "function", `${mod}.${name} deve ter kind === "function"`);
  }

  for (const uiFn of ["texto", "botao", "entrada", "coluna", "linha", "lista"]) {
    compilerFunctions.add(`UI.${uiFn}`);
    const entry = modules.get("UI")?.get(uiFn);
    assert.ok(entry, `Componente UI.${uiFn} falta em pors-stdlib-catalog.ts`);
    assert.equal(entry.kind, "function");
  }

  assert.ok(compilerFunctions.size >= 180, `Esperava >= 180 funções no compilador, encontrou ${compilerFunctions.size}`);

  for (const mod of PORS_STDLIB_CATALOG) {
    for (const item of mod.items) {
      if (item.kind !== "function") continue;
      assert.ok(
        compilerFunctions.has(`${mod.module}.${item.name}`),
        `Função ${mod.module}.${item.name} documentada no catálogo não existe em stdlib.rs`,
      );
    }
  }
});

test("pors-stdlib-catalog covers 100% of explicit stdlib constants defined in parser/constants.rs and stdlib.rs", () => {
  const modules = catalogMap();

  for (const name of ["PI", "E"]) {
    const entry = modules.get("Matematica")?.get(name);
    assert.ok(entry && entry.kind === "constant", `Matematica.${name} deve estar no catálogo`);
  }

  const calBlock = constantsRs.match(/fn calendario_constant[\s\S]*?\n\}/)?.[0] ?? "";
  for (const [, name] of calBlock.matchAll(/"((?:DIA|MES)_[A-Z_]+)"/g)) {
    const entry = modules.get("Calendario")?.get(name);
    assert.ok(entry && entry.kind === "constant", `Calendario.${name} falta no catálogo`);
  }

  const grafBlock = constantsRs.match(/fn graficos_constant[\s\S]*?\n\}/)?.[0] ?? "";
  for (const [, name] of grafBlock.matchAll(/"((?:COR|GRADIENTE|CANAL)_[A-Z_]+)"/g)) {
    const entry = modules.get("Graficos")?.get(name);
    assert.ok(entry && entry.kind === "constant", `Graficos.${name} falta no catálogo`);
  }

  const mouseBlock = constantsRs.match(/fn mouse_constant[\s\S]*?\n\}/)?.[0] ?? "";
  for (const [, name] of mouseBlock.matchAll(/"(BOTAO_[A-Z_]+)"/g)) {
    const entry = modules.get("Mouse")?.get(name);
    assert.ok(entry && entry.kind === "constant", `Mouse.${name} falta no catálogo`);
  }

  const tecBlock = constantsRs.match(/fn teclado_constant[\s\S]*?\n\}/)?.[0] ?? "";
  for (const [, name] of tecBlock.matchAll(/"(TECLA_[A-Z0-9_]+)"\s*=>/g)) {
    const entry = modules.get("Teclado")?.get(name);
    assert.ok(entry && entry.kind === "constant", `Teclado.${name} falta no catálogo`);
  }
});

test("every stdlib module provides a valid .pors example and parseable member signatures", () => {
  for (const mod of PORS_STDLIB_CATALOG) {
    assert.ok(mod.example, `${mod.module} deve incluir um exemplo executável .pors`);
    assert.ok(mod.example.fileName.endsWith(".fer"));
    assert.ok(!mod.example.source.includes("programa {"));
    assert.ok(!mod.example.source.includes("inclua biblioteca"));

    for (const item of mod.items) {
      const parsed = parsePorsSignature(item.signature);
      assert.ok(parsed.returns, `Assinatura inválida em ${mod.module}.${item.name}: ${item.signature}`);
      if (item.kind === "constant") {
        assert.equal(parsed.parameters.length, 0);
      }
    }
  }
});
