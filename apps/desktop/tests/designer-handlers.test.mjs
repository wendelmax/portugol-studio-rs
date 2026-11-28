import test from "node:test";
import assert from "node:assert/strict";
import { codeBehindTemplate, handlerName, handlerStub, placeHandler } from "../src/designer/handlers.ts";
import { inspectMarkupFile, inspectSource, isScreenFile, screenFileSource } from "../src/designer/screen-files.ts";

test("nome do handler: o ligado ou o sugerido", () => {
  assert.equal(handlerName(undefined, "btnOk", "AoClicar"), "btnOk_clicar");
  assert.equal(handlerName("  ", "btnOk", "AoClicar"), "btnOk_clicar");
  assert.equal(handlerName("somar", "btnOk", "AoClicar"), "somar");
});

test("função do evento com o cursor no meio do corpo", () => {
  const stub = handlerStub("btnOk_clicar", "J");
  assert.equal(stub.text, "funcao btnOk_clicar(tela: J) -> J {\n    seja mutavel t: J = tela;\n    \n    retorne t;\n}\n");
  assert.equal(stub.text.slice(stub.cursor, stub.cursor + 1), "\n");
  assert.equal(stub.text.slice(0, stub.cursor).endsWith("tela;\n    "), true);
});

test("modelo do code-behind declara o registro de Dados", () => {
  assert.doesNotMatch(codeBehindTemplate("J", null), /registro/);
  assert.match(codeBehindTemplate("J", "Conta"), /\nregistro Conta \{\n/);
});

test("insere no fim com uma linha em branco e não duplica a função", () => {
  const empty = placeHandler("", "f", "J");
  assert.equal(empty.from, 0);
  assert.equal(empty.insert.startsWith("funcao f("), true);

  const code = "registro Conta {\n}\n";
  const appended = placeHandler(code, "f", "J");
  assert.equal(appended.from, code.length);
  assert.equal(appended.insert.startsWith("\nfuncao f("), true);
  const result = code + appended.insert;
  assert.equal(result.slice(appended.cursor - 4, appended.cursor), "    ");

  const existing = "funcao f(tela: J) -> J {\n    retorne tela;\n}\n";
  const found = placeHandler(existing, "f", "J");
  assert.equal(found.insert, null);
  assert.equal(existing.slice(found.cursor).startsWith("    retorne"), true);
  assert.equal(placeHandler(existing, "ff", "J").insert !== null, true, "f não é ff");
});

function host({ tabs = [], files = [], disk = {} } = {}) {
  return {
    tabs: () => tabs,
    tabSource: (tab) => tab.text,
    hasFile: (_ws, path) => files.includes(path),
    readFile: async (_ws, path) => disk[path],
  };
}

test("texto do irmão vem da aba aberta antes do disco", async () => {
  const tabs = [{ id: 1, workspaceId: "w", workspacePath: "App/J.design", text: "aba" }];
  const files = ["App/J.design", "App/J.fer"];
  const h = host({ tabs, files, disk: { "App/J.design": "disco", "App/J.fer": "codigo" } });
  assert.equal(await screenFileSource(h, "w", "App/J.design"), "aba");
  assert.equal(await screenFileSource(h, "w", "App/J.fer"), "codigo");
  assert.equal(await screenFileSource(h, "w", "App/Outra.fer"), null);
});

test("code-behind usa os tipos do .design; programa solto continua comum", async () => {
  const calls = [];
  const inspectors = {
    inspectPortugolRs: async (source) => (calls.push(["pors", source]), { diagnostics: [], symbols: [] }),
    inspectCodeBehind: async (source, design) => (calls.push(["tela", source, design]), { diagnostics: [], symbols: [] }),
    inspectMarkup: async (...args) => (calls.push(["marcacao", ...args]), { diagnostics: [], symbols: [] }),
  };
  const h = host({ files: ["App/J.design", "App/J.fer"], disk: { "App/J.design": "<Janela />", "App/J.fer": "codigo" } });
  const code = { id: 2, workspaceId: "w", workspacePath: "App/J.fer" };
  await inspectSource(h, inspectors, "codigo", code);
  await inspectSource(h, inspectors, "solto", { id: 3, workspaceId: "w", workspacePath: "App/outro.fer" });
  await inspectMarkupFile(h, inspectors, "<Janela />", { id: 4, workspaceId: "w", workspacePath: "App/J.design" });
  await inspectMarkupFile(h, inspectors, "<Projeto />", { id: 5, workspaceId: "w", workspacePath: "App/app.ferproj" });
  assert.deepEqual(calls, [
    ["tela", "codigo", "<Janela />"],
    ["pors", "solto"],
    ["marcacao", "<Janela />", "App/J.design", true, "codigo"],
    ["marcacao", "<Projeto />", "App/app.ferproj"],
  ]);
  assert.equal(isScreenFile(h, code), true);
  assert.equal(isScreenFile(h, { id: 6, workspaceId: "w", workspacePath: "App/outro.fer" }), false);
});

test("tela sem code-behind confere os eventos como ausentes", async () => {
  const calls = [];
  const inspectors = {
    inspectPortugolRs: async () => ({ diagnostics: [], symbols: [] }),
    inspectCodeBehind: async () => ({ diagnostics: [], symbols: [] }),
    inspectMarkup: async (...args) => (calls.push(args), { diagnostics: [], symbols: [] }),
  };
  const h = host({ files: ["App/J.design"] });
  await inspectMarkupFile(h, inspectors, "<Janela />", { id: 4, workspaceId: "w", workspacePath: "App/J.design" });
  assert.deepEqual(calls, [["<Janela />", "App/J.design", true, null]]);
});
