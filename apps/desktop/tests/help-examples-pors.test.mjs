import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(__dirname, "..");
const helpAssetsDir = path.join(desktopRoot, "src", "help-assets");
const topicosDir = path.join(helpAssetsDir, "topicos");
const exemplosPorsDir = path.join(helpAssetsDir, "recursos", "exemplos-pors");
const helpTsPath = path.join(desktopRoot, "src", "help.ts");

function walk(dir, ext) {
  const results = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      results.push(...walk(full, ext));
    } else if (entry.name.endsWith(ext)) {
      results.push(full);
    }
  }
  return results.sort();
}

test("help-assets ships all 77 ported .fer examples without classic .por boilerplate", () => {
  const porsFiles = walk(exemplosPorsDir, ".fer");
  assert.equal(porsFiles.length, 77, `esperava 77 arquivos .fer, encontrou ${porsFiles.length}`);

  for (const file of porsFiles) {
    const source = fs.readFileSync(file, "utf8");
    const rel = path.relative(exemplosPorsDir, file);
    assert.ok(!source.includes("programa {"), `${rel} nao deve conter "programa {"`);
    assert.ok(!source.includes("inclua biblioteca"), `${rel} nao deve conter "inclua biblioteca"`);
    assert.ok(!source.includes("$$$ Portugol Studio $$$"), `${rel} nao deve conter o rodape legado do material de origem`);
  }
});

test("all 48 HTML help topics and help.ts reference .fer examples with zero broken links", () => {
  const htmlFiles = walk(topicosDir, ".html");
  assert.equal(htmlFiles.length, 48, `esperava 48 arquivos HTML de topicos, encontrou ${htmlFiles.length}`);

  const resolvedFiles = new Set();
  let exemploCount = 0;
  let sintaxeCount = 0;

  for (const htmlFile of htmlFiles) {
    const content = fs.readFileSync(htmlFile, "utf8");
    const relHtml = path.relative(helpAssetsDir, htmlFile).replaceAll("\\", "/");

    assert.equal(
      /data-file="[^"]*\.por"/.test(content),
      false,
      `${relHtml} ainda contem referencia .por em data-file`,
    );

    const blocks = content.matchAll(/<div\b[^>]*class="[^"]*codigo-portugol[^"]*"[^>]*>/gs);
    for (const [tag] of blocks) {
      const fileMatch = tag.match(/data-file=["']([^"']+)["']/);
      assert.ok(fileMatch, `bloco codigo-portugol sem data-file em ${relHtml}`);
      const typeMatch = tag.match(/data-type=["']([^"']+)["']/);
      const dataType = typeMatch ? typeMatch[1] : null;

      const resolved = decodeURI(new URL(fileMatch[1], "https://ajuda.invalid/" + relHtml).pathname.slice(1));
      assert.ok(resolved.startsWith("recursos/exemplos-pors/"), `${relHtml}: caminho inesperado ${resolved}`);
      assert.ok(resolved.endsWith(".fer"), `${relHtml}: extensao inesperada ${resolved}`);

      const absTarget = path.join(helpAssetsDir, resolved);
      assert.ok(fs.existsSync(absTarget), `${relHtml}: arquivo ausente ${resolved}`);
      resolvedFiles.add(resolved);

      if (dataType === "exemplo") {
        exemploCount += 1;
        const src = fs.readFileSync(absTarget, "utf8");
        assert.ok(src.includes("funcao inicio()"), `${resolved} (exemplo) deve declarar funcao inicio()`);
      } else if (dataType === "sintaxe") {
        sintaxeCount += 1;
      }
    }
  }

  assert.equal(resolvedFiles.size, 77, `esperava 77 referencias unicas, encontrou ${resolvedFiles.size}`);
  assert.equal(exemploCount, 38);
  assert.equal(sintaxeCount, 39);

  const helpTs = fs.readFileSync(helpTsPath, "utf8");
  assert.ok(helpTs.includes("./help-assets/recursos/exemplos-pors/**/*.fer"));
  assert.ok(!helpTs.includes("tests/corpus/ajuda/recursos/exemplos"));
  assert.ok(!helpTs.includes('"exemplo.por"'));
});

test("HTML help topics do not teach removed classic .por syntax", () => {
  const htmlFiles = walk(topicosDir, ".html");
  const forbidden = [
    { label: "inclua biblioteca", pattern: /inclua\s+biblioteca/i },
    { label: "palavra reservada const", pattern: /palavra\s+reservada[^.<]*\bconst\b/i },
    { label: "caso contrario", pattern: /caso\s+contr[aá]rio/i },
    { label: "funcao do tipo vazio", pattern: /fun[çc][ãa]o\s+do\s+tipo\s+vazio|retorno\s+[ée]\s+do\s+tipo\s+vazio/i },
    { label: "passagem por referencia com &", pattern: /s[ií]mbolo\s+&/i },
  ];

  for (const htmlFile of htmlFiles) {
    const content = fs.readFileSync(htmlFile, "utf8");
    const relHtml = path.relative(helpAssetsDir, htmlFile).replaceAll("\\", "/");
    for (const rule of forbidden) {
      assert.equal(rule.pattern.test(content), false, `${relHtml} ainda ensina sintaxe legada: ${rule.label}`);
    }
  }
});
