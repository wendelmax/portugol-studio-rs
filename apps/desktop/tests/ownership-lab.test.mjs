import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const moduleUrl = new URL("../src/ownership-lab.ts", import.meta.url);
const fixtureRoot = new URL("./fixtures/ownership/", import.meta.url);
const rustcProbe = spawnSync("rustc", ["--version"], { encoding: "utf8" });
const compilerAvailable = rustcProbe.error === undefined && rustcProbe.status === 0;
const compilerSkip = compilerAvailable ? false : `rustc não disponível: ${rustcProbe.error?.code ?? "não instalado"}`;

test("laboratório oferece move, borrowing compartilhado, mutável e escopo", async () => {
  const { ownershipScenarioIds, getOwnershipScenario } = await import(moduleUrl.href);
  assert.deepEqual(ownershipScenarioIds(), ["move", "shared-borrow", "mutable-borrow", "scope"]);
  for (const id of ownershipScenarioIds()) {
    const scenario = getOwnershipScenario(id);
    assert.ok(scenario.title);
    assert.ok(scenario.source);
    assert.ok(scenario.snapshots.length >= 2);
    assert.ok(scenario.snapshots.every(({ provenance }) => provenance === "verified-simulation"));
  }
  assert.equal(getOwnershipScenario("unknown"), undefined);
});

test("snapshots mantêm owner, referência, heap e linha atual coerentes", async () => {
  const { ownershipScenarioIds, getOwnershipScenario } = await import(moduleUrl.href);
  for (const id of ownershipScenarioIds()) {
    const scenario = getOwnershipScenario(id);
    assert.ok(scenario);
    const lineCount = scenario.source.split("\n").length;
    for (const snapshot of scenario.snapshots) {
      assert.ok(snapshot.currentLine > 0 && snapshot.currentLine <= lineCount, `${id}: linha ${snapshot.currentLine}`);
      const owners = snapshot.bindings.filter(({ state }) => state === "owner");
      assert.ok(owners.length <= 1, `${id}: no máximo um owner desta String`);
      for (const binding of snapshot.bindings.filter(({ state }) => state === "shared-reference" || state === "mutable-reference")) {
        assert.ok(owners.some(({ name }) => name === binding.target), `${id}: ${binding.name} aponta para o owner ativo`);
      }
      for (const block of snapshot.heap) {
        if (block.state === "allocated") {
          assert.ok(block.owner);
          assert.ok(owners.some(({ name }) => name === block.owner), `${id}: o heap aponta para seu owner`);
        } else {
          assert.equal(block.owner, null);
          assert.ok(snapshot.bindings.some(({ state }) => state === "dropped"), `${id}: heap liberado após drop`);
        }
      }
    }
  }
});

test("movimento transfere o owner e o uso posterior de a é um erro de compilação", async () => {
  const { getOwnershipScenario } = await import(moduleUrl.href);
  const scenario = getOwnershipScenario("move");
  assert.ok(scenario);
  const moved = scenario.snapshots.find((snapshot) => snapshot.bindings.some(({ name, state }) => name === "a" && state === "moved"));
  assert.ok(moved);
  assert.ok(moved.bindings.some(({ name, state }) => name === "b" && state === "owner"));
  assert.equal(moved.heap[0].owner, "b");
  assert.match(scenario.snapshots.at(-1).diagnostic ?? "", /compila|movido/i);
  assert.equal(scenario.expected, "compile-error");
});

test("empréstimos compartilhados preservam o owner e coexistem", async () => {
  const { getOwnershipScenario } = await import(moduleUrl.href);
  const scenario = getOwnershipScenario("shared-borrow");
  assert.ok(scenario);
  assert.deepEqual(scenario.snapshots.map(({ currentLine }) => currentLine), [2, 3, 4, 5, 6]);
  const snapshot = scenario.snapshots.find(({ bindings }) => bindings.some(({ name }) => name === "ref2"));
  assert.ok(snapshot);
  assert.ok(snapshot.bindings.some(({ name, state }) => name === "texto" && state === "owner"));
  assert.ok(snapshot.bindings.some(({ name, state, target }) => name === "ref1" && state === "shared-reference" && target === "texto"));
  assert.ok(snapshot.bindings.some(({ name, state, target }) => name === "ref2" && state === "shared-reference" && target === "texto"));
  assert.equal(scenario.expected, "compiles");
});

test("empréstimo mutável termina antes de o owner voltar a ser acessado", async () => {
  const { getOwnershipScenario } = await import(moduleUrl.href);
  const scenario = getOwnershipScenario("mutable-borrow");
  assert.ok(scenario);
  const borrowed = scenario.snapshots.find(({ bindings }) => bindings.some(({ state }) => state === "mutable-reference"));
  assert.ok(borrowed);
  assert.equal(borrowed.bindings.filter(({ state }) => state === "mutable-reference").length, 1);
  assert.ok(borrowed.bindings.some(({ name, state }) => name === "texto" && state === "owner"));
  const returned = scenario.snapshots.at(-1);
  assert.ok(returned.bindings.some(({ name, state }) => name === "texto" && state === "owner"));
  assert.ok(returned.bindings.every(({ state }) => state !== "mutable-reference"));
  assert.equal(scenario.expected, "compiles");
});

test("escopo encerra o empréstimo antes da liberação do owner e do heap", async () => {
  const { getOwnershipScenario } = await import(moduleUrl.href);
  const scenario = getOwnershipScenario("scope");
  assert.ok(scenario);
  const released = scenario.snapshots.at(-1);
  assert.ok(released.bindings.some(({ name, state }) => name === "texto" && state === "dropped"));
  assert.equal(released.heap[0].state, "released");
  assert.equal(scenario.expected, "compiles");
});

test("snapshot fora da sequência retorna undefined", async () => {
  const { getOwnershipScenario, getOwnershipSnapshot } = await import(moduleUrl.href);
  const scenario = getOwnershipScenario("move");
  assert.ok(scenario);
  assert.equal(getOwnershipSnapshot("move", -1), undefined);
  assert.equal(getOwnershipSnapshot("move", scenario.snapshots.length), undefined);
});

test("cada fixture compilável executa com a saída documentada e move falha no rustc", { skip: compilerSkip }, async () => {
  const { ownershipScenarioIds, getOwnershipScenario } = await import(moduleUrl.href);
  for (const id of ownershipScenarioIds()) {
    const scenario = getOwnershipScenario(id);
    assert.ok(scenario);
    const fixturePath = new URL(scenario.fixture, fixtureRoot);
    const fixture = readFileSync(fixturePath, "utf8");
    assert.equal(scenario.source.trimEnd(), fixture.trimEnd(), `${id}: o modelo deve corresponder à fixture compilada`);

    const directory = mkdtempSync(join(tmpdir(), "ownership-lab-"));
    try {
      const executable = join(directory, process.platform === "win32" ? "scenario.exe" : "scenario");
      const result = spawnSync("rustc", [fileURLToPath(fixturePath), "--edition=2021", "-o", executable], { encoding: "utf8" });
      if (scenario.expected === "compile-error") {
        assert.notEqual(result.status, 0, `${id}: rustc deveria rejeitar a fixture`);
        assert.match(result.stderr, /borrow of moved value|value borrowed here after move/i);
      } else {
        assert.equal(result.status, 0, `${id}: ${result.stderr}`);
        const execution = spawnSync(executable, [], { encoding: "utf8" });
        assert.equal(execution.status, 0, `${id}: ${execution.stderr}`);
        assert.equal(execution.stdout, scenario.expectedOutput);
      }
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  }
});

test("painel visual só aparece para ownership e oferece controles acessíveis", () => {
  const html = readFileSync(new URL("../index.html", import.meta.url), "utf8");
  assert.ok(/id="ownership-lab"[^>]*hidden/.test(html), "laboratório começa oculto");
  assert.ok(/Simulação didática[^<]*não é execução nem trace de memória Rust/i.test(html), "a proveniência não promete trace real");
  assert.ok(/id="ownership-scenario"[^>]*aria-label=/.test(html), "seletor tem nome acessível");
  assert.ok(/id="ownership-previous"[^>]*aria-label=/.test(html), "passo anterior tem nome acessível");
  assert.ok(/id="ownership-next"[^>]*aria-label=/.test(html), "próximo passo tem nome acessível");
  assert.ok(/id="ownership-restart"[^>]*aria-label=/.test(html), "reiniciar tem nome acessível");
  assert.ok(/id="ownership-step-status"[^>]*aria-live="polite"/.test(html), "mudança de passo é anunciada sem interromper");
  assert.ok(/id="ownership-bindings"[^>]*aria-label=/.test(html), "bindings têm região nomeada");
  assert.ok(/id="ownership-heap"[^>]*aria-label=/.test(html), "heap tem região nomeada");
  assert.ok(/id="ownership-check-rustc"[^>]*aria-label=/.test(html), "botão de verificação real tem nome acessível");
  assert.ok(/id="ownership-open-rs"[^>]*aria-label=/.test(html), "botão de abrir .rs tem nome acessível");
  assert.ok(/id="ownership-rustc-panel"[^>]*hidden/.test(html), "painel de verificação rustc começa oculto");
  assert.ok(/id="ownership-rustc-provenance"/.test(html), "verificação rustc tem rótulo próprio de proveniência");
});

test("formatOwnershipRustcCheck distingue proveniência real e interpreta erros esperados", async () => {
  const { getOwnershipScenario } = await import(moduleUrl.href);
  const { formatOwnershipRustcCheck, formatOwnershipRustcUnavailable } = await import(
    new URL("../src/learning/ownership-rustc.ts", import.meta.url).href
  );

  const moveScenario = getOwnershipScenario("move");
  const moveReport = formatOwnershipRustcCheck(moveScenario, [
    {
      severity: "error",
      code: "E0382",
      message: "borrow of moved value: `a`",
      rendered: null,
      start: { offset: 0, line: 4, column: 16 },
      end: { offset: 1, line: 4, column: 17 },
    },
  ]);
  assert.equal(moveReport.provenance, "rustc-real");
  assert.equal(moveReport.status, "compile-error");
  assert.equal(moveReport.matchesExpected, true);
  assert.match(moveReport.provenanceLabel, /rustc/i);
  assert.equal(moveReport.items[0].code, "E0382");
  assert.equal(moveReport.items[0].location, "linha 4, coluna 16");

  const borrowScenario = getOwnershipScenario("shared-borrow");
  const borrowReport = formatOwnershipRustcCheck(borrowScenario, []);
  assert.equal(borrowReport.status, "compiled");
  assert.equal(borrowReport.matchesExpected, true);

  const unavailable = formatOwnershipRustcUnavailable();
  assert.equal(unavailable.status, "unavailable");
  assert.match(unavailable.summary, /rustc/i);
});
