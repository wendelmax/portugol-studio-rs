import test from "node:test";
import assert from "node:assert/strict";
import { EditorState } from "@codemirror/state";
import { syntaxTree } from "@codemirror/language";
import { CompletionContext } from "@codemirror/autocomplete";
import { portugolRs, porsHoverInfo } from "../src/portugol.ts";

function completeAt(doc, pos = doc.length) {
  const state = EditorState.create({ doc, extensions: [portugolRs()] });
  const [source] = state.languageDataAt("autocomplete", pos);
  return source(new CompletionContext(state, pos, true));
}

test("PORS highlighting recognizes records, enums, Opcao, Resultado, modules and tamanho", () => {
  const source = "registro Ponto { x: inteiro } enum Cor { Azul } funcao inicio() { seja v: Opcao<inteiro> = Opcao::Algum(Matematica.maior_inteiro(1, tamanho(x))); escolha v { caso Opcao::Algum(n) => { escreva(n); } caso Opcao::Nenhum => {} } }";
  const state = EditorState.create({ doc: source, extensions: [portugolRs()] });
  const keywords = [];
  const types = [];
  const namespaces = [];
  const builtins = [];

  syntaxTree(state).iterate({
    enter(node) {
      const slice = state.doc.sliceString(node.from, node.to);
      if (node.name === "keyword") keywords.push(slice);
      if (node.name === "typeName") types.push(slice);
      if (node.name === "namespace") namespaces.push(slice);
      if (node.name === "variableName.standard") builtins.push(slice);
    },
  });

  assert.ok(keywords.includes("registro"));
  assert.ok(keywords.includes("enum"));
  assert.ok(keywords.includes("escolha"));
  assert.ok(keywords.includes("caso"));
  assert.ok(types.includes("Opcao"));
  assert.ok(namespaces.includes("Matematica"));
  assert.ok(builtins.includes("tamanho"));
  assert.ok(builtins.includes("escreva"));
});

test("PORS completes members of UI, Matematica, Texto and Tipos after dot", async () => {
  const ui = await completeAt("UI.");
  assert.ok(ui.options.some((o) => o.label === "coluna" && o.apply === "coluna("));
  assert.ok(ui.options.some((o) => o.label === "botao" && o.detail.includes("UI.botao")));

  const math = await completeAt("Matematica.");
  assert.ok(math.options.some((o) => o.label === "PI" && o.type === "constant" && !o.apply));
  assert.ok(math.options.some((o) => o.label === "E" && o.type === "constant"));
  assert.ok(math.options.some((o) => o.label === "potencia" && o.apply === "potencia("));

  const texto = await completeAt("Texto.");
  assert.ok(texto.options.some((o) => o.label === "obter_caracter" && o.detail.includes("Opcao<caracter>")));
  assert.ok(texto.options.some((o) => o.label === "extrair_subtexto" && o.detail.includes("Resultado<cadeia, cadeia>")));

  const tipos = await completeAt("Tipos.");
  assert.ok(tipos.options.some((o) => o.label === "para_inteiro" && o.detail.includes("Resultado<inteiro, cadeia>")));
  assert.ok(tipos.options.some((o) => o.label === "inteiro_para_real" && o.apply === "inteiro_para_real("));

  const util = await completeAt("Util.");
  assert.ok(util.options.some((o) => o.label === "sorteia" && o.detail.includes("Resultado<inteiro, cadeia>")));
  assert.ok(util.options.some((o) => o.label === "aguarde" && o.apply === "aguarde("));

  const cal = await completeAt("Calendario.");
  assert.ok(cal.options.some((o) => o.label === "MES_JANEIRO" && o.type === "constant" && !o.apply));
  assert.ok(cal.options.some((o) => o.label === "data_iso_atual" && o.apply === "data_iso_atual("));

  const files = await completeAt("Arquivos.");
  assert.ok(files.options.some((o) => o.label === "ler_texto" && o.detail.includes("Resultado<cadeia, cadeia>")));
  assert.ok(files.options.some((o) => o.label === "listar" && o.detail.includes("Resultado<vetor<cadeia>, cadeia>")));

  const json = await completeAt("Json.");
  assert.ok(json.options.some((o) => o.label === "valido" && o.detail.includes("-> logico")));
  assert.ok(json.options.some((o) => o.label === "obter_cadeia" && o.detail.includes("Resultado<cadeia, cadeia>")));
  assert.ok(json.options.some((o) => o.label === "definir_json" && o.apply === "definir_json("));

  const net = await completeAt("Internet.");
  assert.ok(net.options.some((o) => o.label === "obter_texto" && o.detail.includes("Resultado<cadeia, cadeia>")));
  assert.ok(net.options.some((o) => o.label === "enviar_json" && o.apply === "enviar_json("));
  assert.ok(net.options.some((o) => o.label === "definir_tempo_limite" && o.apply === "definir_tempo_limite("));

  const graf = await completeAt("Graficos.");
  assert.ok(graf.options.some((o) => o.label === "COR_VERDE" && !o.apply));
  assert.ok(graf.options.some((o) => o.label === "desenhar_poligono" && o.apply === "desenhar_poligono("));

  const kb = await completeAt("Teclado.");
  assert.ok(kb.options.some((o) => o.label === "TECLA_ESC" && !o.apply));
  assert.ok(kb.options.some((o) => o.label === "ler_tecla" && o.apply === "ler_tecla("));

  const mouse = await completeAt("Mouse.");
  assert.ok(mouse.options.some((o) => o.label === "BOTAO_ESQUERDO" && !o.apply));
  assert.ok(mouse.options.some((o) => o.label === "posicao_x" && o.apply === "posicao_x("));

  const sons = await completeAt("Sons.");
  assert.ok(sons.options.some((o) => o.label === "carregar_som" && o.apply === "carregar_som("));
});

test("PORS completes Opcao, Resultado and user-declared enum variants after ::", async () => {
  const opt = await completeAt("Opcao::");
  assert.ok(opt.options.some((o) => o.label === "Algum" && o.apply === "Algum("));
  assert.ok(opt.options.some((o) => o.label === "Nenhum" && !o.apply));

  const res = await completeAt("Resultado::");
  assert.ok(res.options.some((o) => o.label === "Ok" && o.apply === "Ok("));
  assert.ok(res.options.some((o) => o.label === "Erro" && o.apply === "Erro("));

  const doc = "enum Estado { Parado, Rodando(inteiro) }\nfuncao inicio() { seja e: Estado = Estado::";
  const user = await completeAt(doc);
  assert.deepEqual(user.options.map((o) => o.label), ["Parado", "Rodando"]);
});

test("PORS top-level completions include snippets and locals without leaking classic Portugol", async () => {
  const doc = "registro Aluno { nome: cadeia }\nfuncao dobrar(x: inteiro) -> inteiro { retorne x * 2; }\nfuncao inicio() { seja nota: inteiro = 10; ";
  const result = await completeAt(doc);
  const labels = result.options.map((o) => o.label);

  for (const expected of ["seja", "seja mutavel", "registro", "enum", "escolha", "Opcao", "Resultado", "UI", "Matematica", "Texto", "Tipos", "Util", "Calendario", "Arquivos", "Json", "Internet", "Graficos", "Teclado", "Mouse", "Sons", "adicionar", "remover", "obter", "Aluno", "dobrar", "nota"]) {
    assert.ok(labels.includes(expected), `esperava sugestão PORS '${expected}'`);
  }
  for (const forbidden of ["programa", "inclua", "biblioteca", "contrario", "limpa", "sorteia", "ServicosWeb", "Objetos"]) {
    assert.ok(!labels.includes(forbidden), `não deve vazar símbolo clássico '${forbidden}' em .pors`);
  }
});

test("porsHoverInfo resolves signatures and docs for stdlib, UI, enums and keywords", () => {
  const doc = "seja x: Opcao<inteiro> = Texto.obter_caracter(\"oi\", 0); seja p: real = Matematica.PI; seja r: Resultado<inteiro, cadeia> = Tipos.para_inteiro(\"42\"); seja e: Opcao<inteiro> = Opcao::Algum(1);";

  const textoCall = porsHoverInfo(doc, doc.indexOf("obter_caracter") + 3);
  assert.equal(textoCall?.title, "Texto.obter_caracter");
  assert.ok(textoCall?.signature.includes("Opcao<caracter>"));

  const mathConst = porsHoverInfo(doc, doc.indexOf("PI"));
  assert.equal(mathConst?.title, "Matematica.PI");
  assert.equal(mathConst?.signature, "Matematica.PI: real");

  const tiposCall = porsHoverInfo(doc, doc.indexOf("para_inteiro") + 2);
  assert.equal(tiposCall?.title, "Tipos.para_inteiro");
  assert.ok(tiposCall?.signature.includes("Resultado<inteiro, cadeia>"));

  const variant = porsHoverInfo(doc, doc.indexOf("Algum") + 1);
  assert.equal(variant?.title, "Opcao::Algum");
  assert.ok(variant?.description.includes("Option::Some"));

  const typeDoc = porsHoverInfo(doc, doc.indexOf("Resultado") + 2);
  assert.equal(typeDoc?.title, "Resultado");
  assert.ok(typeDoc?.description.includes("Result<T, E>"));

  assert.equal(porsHoverInfo("seja desconhecido: inteiro = 1;", 6), null);
});
