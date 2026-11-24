export interface PorsHelpExample {
  readonly title: string;
  readonly fileName: string;
  readonly source: string;
}

export interface PorsHelpTopic {
  readonly id: string;
  readonly title: string;
  readonly summary: string;
  readonly paragraphs: readonly string[];
  readonly example?: PorsHelpExample;
}

export interface PorsHelpLibraryItem {
  readonly name: string;
  readonly signature: string;
  readonly description: string;
}

export interface PorsHelpLibrary {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly items: readonly PorsHelpLibraryItem[];
}

export const PORS_HELP_DEFAULT_TOPIC = "pors:visao-geral";

export const PORS_HELP_TOPICS: readonly PorsHelpTopic[] = [
  {
    id: "pors:visao-geral",
    title: "Portugol-RS: Visão geral e tipos",
    summary: "Sintaxe tipada com imutabilidade padrão e compilação direta para VM, WASM e Rust.",
    paragraphs: [
      "Portugol-RS (.pors) é o dialeto autoral em português projetado como ponte direta para Rust. Todo binding começa imutável com `seja` e exige `seja mutavel` para reatribuição ou mutação.",
      "Os tipos escalares disponíveis são `inteiro` (i32), `real` (f64), `logico` (bool), `cadeia` (UTF-8 com orçamento de memória) e `caracter` (char Unicode), sem coerções implícitas entre tipos.",
    ],
    example: {
      title: "Variáveis, funções e controle de fluxo",
      fileName: "guia-pors-tipos.pors",
      source: `funcao dobro(valor: inteiro) -> inteiro {\n    retorne valor * 2;\n}\n\nfuncao inicio() {\n    seja mutavel total: inteiro = 0;\n    para (seja mutavel i: inteiro = 1; i <= 3; i += 1) {\n        total += dobro(i);\n    }\n    escreva("Total: ", total);\n}\n`,
    },
  },
  {
    id: "pors:opcao-resultado",
    title: "Opcao<T>, Resultado<T, E> e operador ?",
    summary: "Tratamento explícito de ausência e falhas sem null ou exceções, mapeado para Option e Result em Rust.",
    paragraphs: [
      "`Opcao<T>` possui as variantes `Opcao::Algum(valor)` e `Opcao::Nenhum`. `Resultado<T, E>` possui `Resultado::Ok(valor)` e `Resultado::Erro(erro)`.",
      "Use `escolha` para tratar todas as variantes de forma exaustiva, ou o operador pós-fixo `?` dentro de funções que retornam `Opcao<...>` ou `Resultado<..., E>` compatível.",
    ],
    example: {
      title: "Divisão segura com Resultado e ?",
      fileName: "guia-pors-resultado.pors",
      source: `funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {\n    se (b == 0) {\n        retorne Resultado::Erro("divisão por zero");\n    }\n    retorne Resultado::Ok(a / b);\n}\n\nfuncao dobro_divisao(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {\n    seja q: inteiro = dividir(a, b)?;\n    retorne Resultado::Ok(q * 2);\n}\n\nfuncao inicio() {\n    escolha dobro_divisao(20, 4) {\n        caso Resultado::Ok(v) => { escreva(v); }\n        caso Resultado::Erro(e) => { escreva(e); }\n    }\n}\n`,
    },
  },
  {
    id: "pors:colecoes-tipos-compostos",
    title: "Vetores dinâmicos, registros e enums",
    summary: "Construção vazia tipada [], adicionar/remover/obter e tipos nominais (registro e enum).",
    paragraphs: [
      "Vetores tipados `vetor<T>` aceitam construção vazia tipada `[]`, inserção com `adicionar(v, valor);`, remoção segura com `remover(v) -> Opcao<T>`, leitura segura com `obter(v, indice) -> Opcao<T>` e tamanho com `tamanho(v)`.",
      "Tipos compostos nominais usam `registro Nome { campo: Tipo }` e `enum Nome { A, B(Tipo) }`, sempre com cópia por valor e `escolha` exaustiva.",
    ],
    example: {
      title: "Vetor dinâmico com obter e remover",
      fileName: "guia-pors-vetores.pors",
      source: `funcao inicio() {\n    seja mutavel notas: vetor<inteiro> = [];\n    adicionar(notas, 8);\n    adicionar(notas, 10);\n    escolha obter(notas, 1) {\n        caso Opcao::Algum(n) => { escreva("Nota=", n, " Total=", tamanho(notas)); }\n        caso Opcao::Nenhum => { escreva("Sem nota"); }\n    }\n}\n`,
    },
  },
  {
    id: "pors:ui-mvu",
    title: "UI visual declarativa (MVU)",
    summary: "Aplicativos interativos com iniciar(), atualizar() e visualizar() usando UI.*.",
    paragraphs: [
      "Programas visuais em Portugol-RS declaram `iniciar() -> Estado`, `atualizar(estado, tipo, alvo, valor) -> Estado` e `visualizar(estado) -> inteiro`.",
      "A visualização compõe nós imutáveis com `UI.coluna`, `UI.linha`, `UI.lista`, `UI.texto`, `UI.botao` e `UI.entrada`, executáveis na prévia visual da IDE, em WebAssembly e em projetos Cargo standalone.",
    ],
    example: {
      title: "Contador MVU declarativo",
      fileName: "guia-pors-ui.pors",
      source: `funcao iniciar() -> inteiro {\n    retorne 0;\n}\n\nfuncao atualizar(estado: inteiro, tipo: cadeia, alvo: cadeia) -> inteiro {\n    se (tipo == "clique" && alvo == "inc") {\n        retorne estado + 1;\n    }\n    retorne estado;\n}\n\nfuncao visualizar(estado: inteiro) -> inteiro {\n    retorne UI.coluna([\n        UI.texto("Contagem: ", estado),\n        UI.botao("inc", "+1")\n    ]);\n}\n`,
    },
  },
];

export const PORS_HELP_LIBRARIES: readonly PorsHelpLibrary[] = [
  {
    id: "pors-lib:Matematica",
    name: "PORS: Matematica",
    description: "Constantes (`PI`, `E`) e 16 funções matemáticas tipadas sem coerção implícita (VM mínima, WASM e Rust).",
    items: [
      { name: "PI", signature: "Matematica.PI: real", description: "Constante π." },
      { name: "E", signature: "Matematica.E: real", description: "Constante de Euler e." },
      { name: "potencia", signature: "Matematica.potencia(base: real, expoente: real) -> real", description: "Exponenciação real." },
      { name: "raiz_quadrada", signature: "Matematica.raiz_quadrada(numero: real) -> real", description: "Raiz quadrada." },
      { name: "arredondar", signature: "Matematica.arredondar(numero: real, casas: inteiro) -> real", description: "Arredonda para 0..15 casas decimais." },
      { name: "valor_absoluto_inteiro", signature: "Matematica.valor_absoluto_inteiro(numero: inteiro) -> inteiro", description: "Valor absoluto de inteiro." },
      { name: "maior_inteiro", signature: "Matematica.maior_inteiro(a: inteiro, b: inteiro) -> inteiro", description: "Maior entre dois inteiros." },
      { name: "menor_inteiro", signature: "Matematica.menor_inteiro(a: inteiro, b: inteiro) -> inteiro", description: "Menor entre dois inteiros." },
    ],
  },
  {
    id: "pors-lib:Texto",
    name: "PORS: Texto",
    description: "13 operações Unicode-safe indexadas por `char`, com orçamento de memória verificado antes de alocar.",
    items: [
      { name: "numero_caracteres", signature: "Texto.numero_caracteres(texto: cadeia) -> inteiro", description: "Conta caracteres Unicode (`char`)." },
      { name: "aparar", signature: "Texto.aparar(texto: cadeia) -> cadeia", description: "Remove espaços nas extremidades." },
      { name: "caixa_alta", signature: "Texto.caixa_alta(texto: cadeia) -> cadeia", description: "Converte para maiúsculas." },
      { name: "caixa_baixa", signature: "Texto.caixa_baixa(texto: cadeia) -> cadeia", description: "Converte para minúsculas." },
      { name: "substituir", signature: "Texto.substituir(texto: cadeia, antigo: cadeia, novo: cadeia) -> cadeia", description: "Substitui ocorrências respeitando o orçamento de memória." },
      { name: "preencher_a_esquerda", signature: "Texto.preencher_a_esquerda(car: caracter, tamanho: inteiro, texto: cadeia) -> cadeia", description: "Preenche à esquerda até o tamanho desejado." },
      { name: "obter_caracter", signature: "Texto.obter_caracter(texto: cadeia, indice: inteiro) -> Opcao<caracter>", description: "Leitura segura de caractere por índice Unicode." },
      { name: "posicao_texto", signature: "Texto.posicao_texto(trecho: cadeia, texto: cadeia, inicio: inteiro) -> Opcao<inteiro>", description: "Busca Unicode retornando `Opcao<inteiro>`." },
      { name: "extrair_subtexto", signature: "Texto.extrair_subtexto(texto: cadeia, inicio: inteiro, fim: inteiro) -> Resultado<cadeia, cadeia>", description: "Fatia Unicode segura retornando `Resultado`." },
      { name: "dividir", signature: "Texto.dividir(texto: cadeia, separador: cadeia) -> vetor<cadeia>", description: "Divide a cadeia em um vetor de partes." },
    ],
  },
  {
    id: "pors-lib:Tipos",
    name: "PORS: Tipos",
    description: "19 conversões explícitas e predicados com `Resultado<T, cadeia>` para parses falíveis.",
    items: [
      { name: "para_inteiro", signature: "Tipos.para_inteiro(texto: cadeia) -> Resultado<inteiro, cadeia>", description: "Converte cadeia decimal em `inteiro`." },
      { name: "para_inteiro_base", signature: "Tipos.para_inteiro_base(texto: cadeia, base: inteiro) -> Resultado<inteiro, cadeia>", description: "Converte cadeia em `inteiro` nas bases 2, 10 ou 16." },
      { name: "para_real", signature: "Tipos.para_real(texto: cadeia) -> Resultado<real, cadeia>", description: "Converte cadeia em `real` finito." },
      { name: "para_logico", signature: "Tipos.para_logico(texto: cadeia) -> Resultado<logico, cadeia>", description: "Converte `\"verdadeiro\"`/`\"falso\"` em `logico`." },
      { name: "inteiro_para_real", signature: "Tipos.inteiro_para_real(valor: inteiro) -> real", description: "Conversão explícita `inteiro` -> `real`." },
      { name: "inteiro_para_cadeia", signature: "Tipos.inteiro_para_cadeia(valor: inteiro) -> cadeia", description: "Formata `inteiro` em cadeia." },
      { name: "cadeia_e_inteiro", signature: "Tipos.cadeia_e_inteiro(texto: cadeia) -> logico", description: "Valida se uma cadeia representa um inteiro." },
    ],
  },
  {
    id: "pors-lib:Util",
    name: "PORS: Util",
    description: "Utilitários determinísticos e canceláveis (VM mínima, WASM e Rust).",
    items: [
      { name: "sorteia", signature: "Util.sorteia(min: inteiro, max: inteiro) -> Resultado<inteiro, cadeia>", description: "Sorteia inteiro em `[min, max]` com erro tipado se `min > max`." },
      { name: "definir_semente", signature: "Util.definir_semente(semente: inteiro)", description: "Fixa a semente determinística de `Util.sorteia`." },
      { name: "tempo_decorrido", signature: "Util.tempo_decorrido() -> inteiro", description: "Milissegundos decorridos desde o início." },
      { name: "aguarde", signature: "Util.aguarde(ms: inteiro)", description: "Espera cancelável de 0 a 60000 ms." },
    ],
  },
  {
    id: "pors-lib:Calendario",
    name: "PORS: Calendario",
    description: "Contrato UTC gregoriano determinístico com 19 constantes (`DIA_*`, `MES_*`) e 15 funções.",
    items: [
      { name: "data_iso_atual", signature: "Calendario.data_iso_atual() -> cadeia", description: "Retorna a data UTC atual no formato `AAAA-MM-DD`." },
      { name: "eh_bissexto", signature: "Calendario.eh_bissexto(ano: inteiro) -> logico", description: "Verifica se o ano gregoriano é bissexto." },
      { name: "dias_no_mes", signature: "Calendario.dias_no_mes(ano: inteiro, mes: inteiro) -> Resultado<inteiro, cadeia>", description: "Retorna 28..31 dias para o mês/ano ou erro tipado." },
      { name: "nome_mes", signature: "Calendario.nome_mes(mes: inteiro) -> Resultado<cadeia, cadeia>", description: "Nome em português do mês (1 a 12)." },
      { name: "dia_semana_completo", signature: "Calendario.dia_semana_completo(dia: inteiro, alta: logico, baixa: logico) -> Resultado<cadeia, cadeia>", description: "Nome completo do dia da semana (1 a 7)." },
    ],
  },
  {
    id: "pors-lib:Arquivos",
    name: "PORS: Arquivos",
    description: "Leitura e escrita UTF-8 estritamente confinadas ao workspace (bloqueia caminhos absolutos, `..` e symlinks; limite de 2 MiB).",
    items: [
      { name: "ler_texto", signature: "Arquivos.ler_texto(caminho: cadeia) -> Resultado<cadeia, cadeia>", description: "Lê arquivo UTF-8 relativo ao workspace." },
      { name: "ler_linhas", signature: "Arquivos.ler_linhas(caminho: cadeia) -> Resultado<vetor<cadeia>, cadeia>", description: "Lê as linhas de um arquivo do workspace." },
      { name: "escrever_texto", signature: "Arquivos.escrever_texto(caminho: cadeia, conteudo: cadeia) -> Resultado<logico, cadeia>", description: "Grava texto UTF-8 em caminho relativo ao workspace." },
      { name: "acrescentar_texto", signature: "Arquivos.acrescentar_texto(caminho: cadeia, conteudo: cadeia) -> Resultado<logico, cadeia>", description: "Acrescenta texto ao final de um arquivo do workspace." },
      { name: "existe", signature: "Arquivos.existe(caminho: cadeia) -> Resultado<logico, cadeia>", description: "Verifica se o caminho relativo existe no workspace." },
      { name: "remover", signature: "Arquivos.remover(caminho: cadeia) -> Resultado<logico, cadeia>", description: "Remove um arquivo dentro do workspace." },
      { name: "listar", signature: "Arquivos.listar(diretorio: cadeia) -> Resultado<vetor<cadeia>, cadeia>", description: "Lista entradas ordenadas de um diretório relativo ao workspace." },
    ],
  },
  {
    id: "pors-lib:Json",
    name: "PORS: Json",
    description: "19 operações JSON (RFC 8259) puras e zero-dependência para validar, consultar, construir e formatar objetos JSON (VM mínima, WASM e Rust).",
    items: [
      { name: "valido", signature: "Json.valido(texto: cadeia) -> logico", description: "Valida sintaxe JSON (profundidade máxima 64)." },
      { name: "formatar", signature: "Json.formatar(texto: cadeia) -> Resultado<cadeia, cadeia>", description: "Indenta JSON válido com 2 espaços." },
      { name: "minificar", signature: "Json.minificar(texto: cadeia) -> Resultado<cadeia, cadeia>", description: "Compacta JSON válido removendo espaços." },
      { name: "obter_cadeia", signature: "Json.obter_cadeia(json: cadeia, chave: cadeia) -> Resultado<cadeia, cadeia>", description: "Lê uma propriedade de texto do objeto JSON." },
      { name: "obter_inteiro", signature: "Json.obter_inteiro(json: cadeia, chave: cadeia) -> Resultado<inteiro, cadeia>", description: "Lê uma propriedade inteira do objeto JSON." },
      { name: "obter_lista_cadeia", signature: "Json.obter_lista_cadeia(json: cadeia, chave: cadeia) -> Resultado<vetor<cadeia>, cadeia>", description: "Lê uma lista JSON de cadeias." },
      { name: "obter_json", signature: "Json.obter_json(json: cadeia, chave: cadeia) -> Resultado<cadeia, cadeia>", description: "Extrai um subobjeto ou sublista como cadeia JSON." },
      { name: "definir_cadeia", signature: "Json.definir_cadeia(json: cadeia, chave: cadeia, valor: cadeia) -> Resultado<cadeia, cadeia>", description: "Retorna novo objeto JSON com a chave atualizada." },
      { name: "definir_json", signature: "Json.definir_json(json: cadeia, chave: cadeia, valor_json: cadeia) -> Resultado<cadeia, cadeia>", description: "Define um subvalor JSON (objeto ou lista) na chave." },
      { name: "remover_chave", signature: "Json.remover_chave(json: cadeia, chave: cadeia) -> Resultado<cadeia, cadeia>", description: "Retorna novo objeto JSON sem a chave." },
    ],
  },
  {
    id: "pors-lib:UI",
    name: "PORS: UI",
    description: "Componentes declarativos MVU para interfaces gráficas interativas na IDE, WASM e Rust.",
    items: [
      { name: "coluna", signature: "UI.coluna(filhos: vetor<inteiro>) -> inteiro", description: "Empilha componentes verticalmente." },
      { name: "linha", signature: "UI.linha(filhos: vetor<inteiro>) -> inteiro", description: "Alinha componentes horizontalmente." },
      { name: "lista", signature: "UI.lista(itens: vetor<inteiro>) -> inteiro", description: "Lista de nós filhos." },
      { name: "texto", signature: "UI.texto(valor, ...) -> inteiro", description: "Texto formatado a partir de escalares." },
      { name: "botao", signature: "UI.botao(id: cadeia, rotulo: cadeia) -> inteiro", description: "Botão que emite eventos de clique." },
      { name: "entrada", signature: "UI.entrada(id: cadeia, valor: cadeia, dica: cadeia) -> inteiro", description: "Campo controlado que emite `entrada_alterada`." },
    ],
  },
];

function normalizeHelpQuery(value: string): string {
  return value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLocaleLowerCase("pt-BR");
}

export function filterPorsHelp(query: string): {
  topics: readonly PorsHelpTopic[];
  libraries: readonly PorsHelpLibrary[];
} {
  const normalized = normalizeHelpQuery(query.trim());
  if (!normalized) {
    return { topics: PORS_HELP_TOPICS, libraries: PORS_HELP_LIBRARIES };
  }
  const topics = PORS_HELP_TOPICS.filter((topic) =>
    normalizeHelpQuery(`${topic.title} ${topic.summary} ${topic.paragraphs.join(" ")}`).includes(normalized),
  );
  const libraries = PORS_HELP_LIBRARIES.filter((lib) =>
    normalizeHelpQuery(
      `${lib.name} ${lib.description} ${lib.items.map((item) => `${item.name} ${item.signature} ${item.description}`).join(" ")}`,
    ).includes(normalized),
  );
  return { topics, libraries };
}
