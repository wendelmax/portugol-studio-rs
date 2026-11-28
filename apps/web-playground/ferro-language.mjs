// Referência única das instruções e sintaxes do Ferro (.fer) no playground.
// A mesma lista alimenta o autocomplete e o painel de referência; sem dependências externas.

const entry = (label, kind, group, signature, description, example) => ({
  id: label.toLowerCase().replace(/[^a-z0-9]+/g, "-"),
  label,
  kind,
  group,
  signature,
  description,
  example,
  insertText: label,
});

export const FERRO_SYNTAX_ENTRIES = [
  entry(
    "funcao",
    "keyword",
    "Estrutura",
    "funcao nome(parametro: Tipo) -> Tipo { ... }",
    "Declara uma função nomeada. Os parâmetros e o retorno são tipados; o corpo fica entre chaves.",
    "funcao soma(a: inteiro, b: inteiro) -> inteiro {\n    retorne a + b;\n}",
  ),
  entry(
    "inicio",
    "keyword",
    "Estrutura",
    "funcao inicio() { ... }",
    "Ponto de entrada de um programa Ferro de console.",
    "funcao inicio() {\n    escreval(\"Olá, Ferro!\");\n}",
  ),
  entry(
    "retorne",
    "keyword",
    "Estrutura",
    "retorne expressao;",
    "Encerra a função atual e devolve o valor da expressão quando há tipo de retorno.",
    "funcao dobro(n: inteiro) -> inteiro {\n    retorne n * 2;\n}",
  ),
  entry(
    "seja",
    "keyword",
    "Declarações",
    "seja [mutavel] nome[: Tipo] = valor;",
    "Declara um binding imutável por padrão; o tipo é inferido quando omitido.",
    "seja nome = \"Ada\";\nseja idade: inteiro = 36;",
  ),
  entry(
    "mutavel",
    "keyword",
    "Declarações",
    "seja mutavel nome[: Tipo] = valor;",
    "Permite reatribuir o binding ou mutar campos e índices.",
    "seja mutavel contador = 0;\ncontador += 1;",
  ),
  entry(
    "inteiro",
    "type",
    "Tipos",
    "inteiro",
    "Número inteiro com verificação de overflow em tempo de execução.",
    "seja total: inteiro = 10;",
  ),
  entry(
    "real",
    "type",
    "Tipos",
    "real",
    "Número de ponto flutuante para cálculos com casas decimais.",
    "seja media: real = 7.5;",
  ),
  entry(
    "logico",
    "type",
    "Tipos",
    "logico",
    "Valor booleano: verdadeiro ou falso.",
    "seja pronto: logico = verdadeiro;",
  ),
  entry(
    "cadeia",
    "type",
    "Tipos",
    "cadeia",
    "Texto Unicode imutável; aceita interpolação com {expressao}.",
    "seja saudacao: cadeia = \"Olá, {nome}!\";",
  ),
  entry(
    "caracter",
    "type",
    "Tipos",
    "caracter",
    "Um único caractere Unicode.",
    "seja inicial: caracter = 'A';",
  ),
  entry(
    "vetor",
    "type",
    "Tipos",
    "vetor<T>",
    "Coleção homogênea tipada com limites verificados.",
    "seja numeros: vetor<inteiro> = [1, 2, 3];",
  ),
  entry(
    "Opcao",
    "type",
    "Tipos",
    "Opcao<T> { Algum(T), Nenhum }",
    "Tipo opcional nativo, mapeado para Option<T> em Rust.",
    "seja encontrado: Opcao<inteiro> = Opcao::Algum(42);",
  ),
  entry(
    "Resultado",
    "type",
    "Tipos",
    "Resultado<T, E> { Ok(T), Erro(E) }",
    "Tipo falível nativo, mapeado para Result<T, E> em Rust.",
    "seja valor: Resultado<inteiro, cadeia> = Resultado::Ok(7);",
  ),
  entry(
    "se",
    "keyword",
    "Controle",
    "se (condicao) { ... }",
    "Executa o bloco quando a condição lógica é verdadeira.",
    "se (idade >= 18) {\n    escreval(\"Maior de idade\");\n}",
  ),
  entry(
    "senao",
    "keyword",
    "Controle",
    "senao { ... }",
    "Ramificação executada quando a condição do se anterior é falsa.",
    "se (pronto) {\n    escreval(\"Sim\");\n} senao {\n    escreval(\"Não\");\n}",
  ),
  entry(
    "senao_se",
    "keyword",
    "Controle",
    "senao se (condicao) { ... }",
    "Encadeia uma nova condição após um se ou senao se.",
    "se (n < 0) {\n    escreval(\"Negativo\");\n} senao se (n == 0) {\n    escreval(\"Zero\");\n}",
  ),
  entry(
    "enquanto",
    "keyword",
    "Controle",
    "enquanto (condicao) { ... }",
    "Repete o bloco enquanto a condição for verdadeira.",
    "enquanto (n > 0) {\n    n -= 1;\n}",
  ),
  entry(
    "faca_enquanto",
    "keyword",
    "Controle",
    "faca { ...} enquanto (condicao);",
    "Executa o bloco ao menos uma vez e repete enquanto a condição for verdadeira.",
    "faca {\n    n -= 1;\n} enquanto (n > 0);",
  ),
  entry(
    "para",
    "keyword",
    "Controle",
    "para (inicializacao; condicao; atualizacao) { ... }",
    "Laço contado com inicialização, condição e atualização.",
    "para (seja mutavel i = 0; i < 10; i += 1) {\n    escreval(i);\n}",
  ),
  entry(
    "pare",
    "keyword",
    "Controle",
    "pare;",
    "Interrompe o laço mais interno.",
    "se (n == 0) {\n    pare;\n}",
  ),
  entry(
    "continue",
    "keyword",
    "Controle",
    "continue;",
    "Vai diretamente para a próxima iteração do laço mais interno.",
    "se (n % 2 == 0) {\n    continue;\n}",
  ),
  entry(
    "escolha",
    "keyword",
    "Controle",
    "escolha valor { caso padrao => { ... } }",
    "Despacho exaustivo sobre variantes de enum, Opcao ou Resultado.",
    "escolha resultado {\n    caso Resultado::Ok(v) => { escreval(v); }\n    caso Resultado::Erro(e) => { escreval(e); }\n}",
  ),
  entry(
    "caso",
    "keyword",
    "Controle",
    "caso padrao => { ... }",
    "Rótulo de um braço dentro de escolha.",
    "caso Opcao::Nenhum => {\n    escreval(\"Vazio\");\n}",
  ),
  entry(
    "registro",
    "keyword",
    "Dados",
    "registro Nome { campo: Tipo }",
    "Declara um tipo composto nominal com campos tipados, equivalente a struct.",
    "registro Pessoa {\n    nome: cadeia,\n    idade: inteiro\n}",
  ),
  entry(
    "enum",
    "keyword",
    "Dados",
    "enum Nome { Variante, ComPayload(Tipo) }",
    "Declara uma enumeração nominal com variantes simples ou com payload.",
    "enum Cor {\n    Vermelho,\n    Verde,\n    Azul\n}",
  ),
  entry(
    "adicionar",
    "function",
    "Dados",
    "adicionar(v: mutavel vetor<T>, valor: T)",
    "Insere um valor ao final de um vetor mutável.",
    "adicionar(notas, 10);",
  ),
  entry(
    "remover",
    "function",
    "Dados",
    "remover(v: mutavel vetor<T>) -> Opcao<T>",
    "Remove e devolve o último elemento como Opcao<T>.",
    "seja ultimo = remover(notas);",
  ),
  entry(
    "obter",
    "function",
    "Dados",
    "obter(v: vetor<T>, indice: inteiro) -> Opcao<T>",
    "Lê um elemento com segurança, sem sair dos limites do vetor.",
    "seja item = obter(notas, 2);",
  ),
  entry(
    "tamanho",
    "function",
    "Dados",
    "tamanho(v: vetor<T>) -> inteiro",
    "Devolve a quantidade de elementos de um vetor.",
    "escreval(tamanho(notas));",
  ),
  entry(
    "escreva",
    "function",
    "Entrada e saída",
    "escreva(valor, ...)",
    "Escreve um ou mais valores na saída padrão sem quebra automática.",
    "escreva(\"Nome: \", nome);",
  ),
  entry(
    "escreval",
    "function",
    "Entrada e saída",
    "escreval(valor, ...)",
    "Escreve os valores e acrescenta uma quebra de linha ao final.",
    "escreval(\"Olá, {nome}!\");",
  ),
  entry(
    "leia",
    "function",
    "Entrada e saída",
    "leia() | leia(variavel_mutavel)",
    "Lê uma linha da entrada padrão e converte ao tipo esperado pelo contexto.",
    "seja nome = leia();",
  ),
  entry(
    "verdadeiro",
    "constant",
    "Valores",
    "verdadeiro",
    "Valor lógico verdadeiro.",
    "seja ativo: logico = verdadeiro;",
  ),
  entry(
    "falso",
    "constant",
    "Valores",
    "falso",
    "Valor lógico falso.",
    "seja ativo: logico = falso;",
  ),
  entry(
    "nulo",
    "constant",
    "Valores",
    "nulo",
    "Ausência de valor em contextos que a aceitam.",
    "seja vazio = nulo;",
  ),
  entry(
    "Resultado::Ok",
    "variant",
    "Valores",
    "Resultado::Ok(valor: T) -> Resultado<T, E>",
    "Variante de sucesso de Resultado<T, E>.",
    "retorne Resultado::Ok(42);",
  ),
  entry(
    "Resultado::Erro",
    "variant",
    "Valores",
    "Resultado::Erro(erro: E) -> Resultado<T, E>",
    "Variante de falha de Resultado<T, E>, propagável com ?.",
    "retorne Resultado::Erro(\"divisão por zero\");",
  ),
  entry(
    "Opcao::Algum",
    "variant",
    "Valores",
    "Opcao::Algum(valor: T) -> Opcao<T>",
    "Variante que representa a presença de um valor em Opcao<T>.",
    "retorne Opcao::Algum(10);",
  ),
  entry(
    "Opcao::Nenhum",
    "variant",
    "Valores",
    "Opcao::Nenhum -> Opcao<T>",
    "Variante que representa a ausência de valor em Opcao<T>.",
    "retorne Opcao::Nenhum;",
  ),
  entry(
    "?",
    "operator",
    "Operadores",
    "expressao?",
    "Propaga Resultado::Erro encerrando a função atual; em Opcao encerra com Opcao::Nenhum.",
    "seja n: inteiro = dividir(a, b)?;",
  ),
  entry(
    "??",
    "operator",
    "Operadores",
    "esquerda ?? direita",
    "Escolhe a direita quando a esquerda é Opcao::Nenhum.",
    "seja nome = encontrado ?? \"anônimo\";",
  ),
  entry(
    "??=",
    "operator",
    "Operadores",
    "variavel ??= valor",
    "Atribui valor quando a variável ainda está vazia.",
    "nome ??= \"anônimo\";",
  ),
  entry(
    "?.",
    "operator",
    "Operadores",
    "valor?.campo",
    "Acesso encadeado seguro que preserva ausência em vez de falhar.",
    "seja cidade = pessoa?.endereco?.cidade;",
  ),
  entry(
    "+=",
    "operator",
    "Operadores",
    "variavel += valor",
    "Adiciona ao valor atual da variável.",
    "contador += 1;",
  ),
  entry(
    "&&",
    "operator",
    "Operadores",
    "esquerda && direita",
    "E lógico com curto-circuito.",
    "se (ativo && pronto) {\n    escreval(\"OK\");\n}",
  ),
  entry(
    "||",
    "operator",
    "Operadores",
    "esquerda || direita",
    "OU lógico com curto-circuito.",
    "se (a || b) {\n    escreval(\"Algum\");\n}",
  ),
];

/** Trecho do token imediatamente antes do caret; `::` faz parte do token. */
export function wordRangeAt(source, caret) {
  const end = Math.max(0, Math.min(caret, source.length));
  let start = end;
  while (start > 0 && /[A-Za-z0-9_:]/.test(source[start - 1])) {
    start -= 1;
  }
  return { start, end, prefix: source.slice(start, end) };
}

/** Verdadeiro quando o caret está dentro de comentário de linha ou cadeia aberta. */
function inCommentOrString(source, caret) {
  const end = Math.max(0, Math.min(caret, source.length));
  const lineStart = source.lastIndexOf("\n", end - 1) + 1;
  let quote = null;
  let escaped = false;

  for (let i = lineStart; i < end; i += 1) {
    const ch = source[i];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (quote !== null) {
      if (ch === "\\") escaped = true;
      else if (ch === quote) quote = null;
      continue;
    }
    if (ch === '"' || ch === "'") {
      quote = ch;
      continue;
    }
    if (ch === "/" && source[i + 1] === "/") return true;
  }
  return quote !== null;
}

/** Sugestões de sintaxe Ferro para o prefixo antes do caret, no máximo 12. */
export function completeFerro(source, caret) {
  if (inCommentOrString(source, caret)) return [];
  const { prefix } = wordRangeAt(source, caret);
  const wanted = prefix.toLowerCase();
  return FERRO_SYNTAX_ENTRIES.filter((item) => item.label.toLowerCase().startsWith(wanted)).slice(0, 12);
}
