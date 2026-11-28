// Fonte única de verdade da biblioteca padrão Ferro (.fer) para
// autocompletar, hover do editor e documentação da Ajuda.
// Validada por apps/desktop/tests/pors-stdlib-parity.test.mjs contra
// crates/ferro-frontend/src/parser/stdlib.rs e constants.rs.

export type PorsStdlibGroup = "standard" | "visual";

export interface PorsStdlibMember {
  readonly name: string;
  readonly kind: "function" | "constant";
  readonly signature: string;
  readonly description: string;
}

export interface PorsStdlibExample {
  readonly title: string;
  readonly fileName: string;
  readonly source: string;
}

export interface PorsStdlibModule {
  readonly id: string;
  readonly module: PorsModuleName;
  readonly name: string;
  readonly group: PorsStdlibGroup;
  readonly description: string;
  readonly notes?: string;
  readonly example?: PorsStdlibExample;
  readonly items: readonly PorsStdlibMember[];
}

export const PORS_STDLIB_MODULES = [
  "UI",
  "Matematica",
  "Texto",
  "Tipos",
  "Util",
  "Calendario",
  "Arquivos",
  "Dialogo",
  "Json",
  "Internet",
  "Graficos",
  "Teclado",
  "Mouse",
  "Sons",
] as const;

export type PorsModuleName = (typeof PORS_STDLIB_MODULES)[number];

const fn = (name: string, signature: string, description: string): PorsStdlibMember => ({
  name,
  kind: "function",
  signature,
  description,
});

const constant = (name: string, signature: string, description: string): PorsStdlibMember => ({
  name,
  kind: "constant",
  signature,
  description,
});

const TECLADO_NAMED_CONSTANTS: readonly PorsStdlibMember[] = [
  constant("TECLA_ENTER", "Teclado.TECLA_ENTER: inteiro", "Código da tecla Enter (10)."),
  constant("TECLA_BACKSPACE", "Teclado.TECLA_BACKSPACE: inteiro", "Código da tecla Backspace (8)."),
  constant("TECLA_TAB", "Teclado.TECLA_TAB: inteiro", "Código da tecla Tab (9)."),
  constant("TECLA_CANCELAR", "Teclado.TECLA_CANCELAR: inteiro", "Código da tecla Cancelar (3)."),
  constant("TECLA_LIMPAR", "Teclado.TECLA_LIMPAR: inteiro", "Código da tecla Limpar (12)."),
  constant("TECLA_SHIFT", "Teclado.TECLA_SHIFT: inteiro", "Código da tecla Shift (16)."),
  constant("TECLA_CONTROL", "Teclado.TECLA_CONTROL: inteiro", "Código da tecla Control (17)."),
  constant("TECLA_ALT", "Teclado.TECLA_ALT: inteiro", "Código da tecla Alt (18)."),
  constant("TECLA_PAUSE", "Teclado.TECLA_PAUSE: inteiro", "Código da tecla Pause (19)."),
  constant("TECLA_CAPS_LOCK", "Teclado.TECLA_CAPS_LOCK: inteiro", "Código da tecla Caps Lock (20)."),
  constant("TECLA_ESC", "Teclado.TECLA_ESC: inteiro", "Código da tecla Escape (27)."),
  constant("TECLA_ESPACO", "Teclado.TECLA_ESPACO: inteiro", "Código da tecla Espaço (32)."),
  constant("TECLA_PAGE_UP", "Teclado.TECLA_PAGE_UP: inteiro", "Código da tecla Page Up (33)."),
  constant("TECLA_PAGE_DOWN", "Teclado.TECLA_PAGE_DOWN: inteiro", "Código da tecla Page Down (34)."),
  constant("TECLA_END", "Teclado.TECLA_END: inteiro", "Código da tecla End (35)."),
  constant("TECLA_HOME", "Teclado.TECLA_HOME: inteiro", "Código da tecla Home (36)."),
  constant("TECLA_SETA_ESQUERDA", "Teclado.TECLA_SETA_ESQUERDA: inteiro", "Código da seta para a esquerda (37)."),
  constant("TECLA_SETA_ACIMA", "Teclado.TECLA_SETA_ACIMA: inteiro", "Código da seta para cima (38)."),
  constant("TECLA_SETA_DIREITA", "Teclado.TECLA_SETA_DIREITA: inteiro", "Código da seta para a direita (39)."),
  constant("TECLA_SETA_ABAIXO", "Teclado.TECLA_SETA_ABAIXO: inteiro", "Código da seta para baixo (40)."),
  constant("TECLA_VIRGULA", "Teclado.TECLA_VIRGULA: inteiro", "Código da tecla vírgula (44)."),
  constant("TECLA_MENOS", "Teclado.TECLA_MENOS: inteiro", "Código da tecla menos (45)."),
  constant("TECLA_PONTO_FINAL", "Teclado.TECLA_PONTO_FINAL: inteiro", "Código da tecla ponto final (46)."),
  constant("TECLA_BARRA", "Teclado.TECLA_BARRA: inteiro", "Código da tecla barra `/` (47)."),
  constant("TECLA_PONTO_VIRGULA", "Teclado.TECLA_PONTO_VIRGULA: inteiro", "Código da tecla ponto e vírgula `;` (59)."),
  constant("TECLA_IGUAL", "Teclado.TECLA_IGUAL: inteiro", "Código da tecla igual `=` (61)."),
  constant("TECLA_ABRE_COLCHETES", "Teclado.TECLA_ABRE_COLCHETES: inteiro", "Código da tecla abre colchetes `[` (91)."),
  constant("TECLA_BARRA_INVERTIDA", "Teclado.TECLA_BARRA_INVERTIDA: inteiro", "Código da tecla barra invertida `\\` (92)."),
  constant("TECLA_FECHA_COLCHETES", "Teclado.TECLA_FECHA_COLCHETES: inteiro", "Código da tecla fecha colchetes `]` (93)."),
  constant("TECLA_MULTIPLICACAO", "Teclado.TECLA_MULTIPLICACAO: inteiro", "Código da tecla multiplicação do teclado numérico (106)."),
  constant("TECLA_ADICAO", "Teclado.TECLA_ADICAO: inteiro", "Código da tecla adição do teclado numérico (107)."),
  constant("TECLA_NUM_SEPARADOR_DECIMAL", "Teclado.TECLA_NUM_SEPARADOR_DECIMAL: inteiro", "Código do separador decimal numérico (108)."),
  constant("TECLA_SUBTRACAO", "Teclado.TECLA_SUBTRACAO: inteiro", "Código da tecla subtração do teclado numérico (109)."),
  constant("TECLA_DECIMAL", "Teclado.TECLA_DECIMAL: inteiro", "Código da tecla decimal do teclado numérico (110)."),
  constant("TECLA_DIVISAO", "Teclado.TECLA_DIVISAO: inteiro", "Código da tecla divisão do teclado numérico (111)."),
  constant("TECLA_DELETAR", "Teclado.TECLA_DELETAR: inteiro", "Código da tecla Delete (127)."),
  constant("TECLA_NUM_LOCK", "Teclado.TECLA_NUM_LOCK: inteiro", "Código da tecla Num Lock (144)."),
  constant("TECLA_SCROLL_LOCK", "Teclado.TECLA_SCROLL_LOCK: inteiro", "Código da tecla Scroll Lock (145)."),
  constant("TECLA_AMPERSAND", "Teclado.TECLA_AMPERSAND: inteiro", "Código do símbolo `&` (150)."),
  constant("TECLA_ASTERISK", "Teclado.TECLA_ASTERISK: inteiro", "Código do símbolo `*` (151)."),
  constant("TECLA_QUOTEDBL", "Teclado.TECLA_QUOTEDBL: inteiro", "Código de aspas duplas `\"` (152)."),
  constant("TECLA_LESS", "Teclado.TECLA_LESS: inteiro", "Código do símbolo `<` (153)."),
  constant("TECLA_PRINTSCREEN", "Teclado.TECLA_PRINTSCREEN: inteiro", "Código da tecla Print Screen (154)."),
  constant("TECLA_INSERT", "Teclado.TECLA_INSERT: inteiro", "Código da tecla Insert (155)."),
  constant("TECLA_AJUDA", "Teclado.TECLA_AJUDA: inteiro", "Código da tecla Ajuda (156)."),
  constant("TECLA_META", "Teclado.TECLA_META: inteiro", "Código da tecla Meta (157)."),
  constant("TECLA_GREATER", "Teclado.TECLA_GREATER: inteiro", "Código do símbolo `>` (160)."),
  constant("TECLA_BRACELEFT", "Teclado.TECLA_BRACELEFT: inteiro", "Código da chave esquerda `{` (161)."),
  constant("TECLA_BRACERIGHT", "Teclado.TECLA_BRACERIGHT: inteiro", "Código da chave direita `}` (162)."),
  constant("TECLA_BACK_QUOTE", "Teclado.TECLA_BACK_QUOTE: inteiro", "Código da tecla acento grave `` ` `` (192)."),
  constant("TECLA_QUOTE", "Teclado.TECLA_QUOTE: inteiro", "Código da tecla aspa simples `'` (222)."),
  constant("TECLA_AT", "Teclado.TECLA_AT: inteiro", "Código do símbolo `@` (512)."),
  constant("TECLA_COLON", "Teclado.TECLA_COLON: inteiro", "Código de dois-pontos `:` (513)."),
  constant("TECLA_CIRCUMFLEX", "Teclado.TECLA_CIRCUMFLEX: inteiro", "Código do acento circunflexo `^` (514)."),
  constant("TECLA_DOLLAR", "Teclado.TECLA_DOLLAR: inteiro", "Código do símbolo `$` (515)."),
  constant("TECLA_EURO_SIGN", "Teclado.TECLA_EURO_SIGN: inteiro", "Código do símbolo `€` (516)."),
  constant("TECLA_EXCLAMATION_MARK", "Teclado.TECLA_EXCLAMATION_MARK: inteiro", "Código do ponto de exclamação `!` (517)."),
  constant("TECLA_INVERTED_EXCLAMATION_MARK", "Teclado.TECLA_INVERTED_EXCLAMATION_MARK: inteiro", "Código de exclamação invertida `¡` (518)."),
  constant("TECLA_LEFT_PARENTHESIS", "Teclado.TECLA_LEFT_PARENTHESIS: inteiro", "Código do parêntese esquerdo `(` (519)."),
  constant("TECLA_NUMBER_SIGN", "Teclado.TECLA_NUMBER_SIGN: inteiro", "Código do símbolo `#` (520)."),
  constant("TECLA_PLUS", "Teclado.TECLA_PLUS: inteiro", "Código do símbolo `+` (521)."),
  constant("TECLA_RIGHT_PARENTHESIS", "Teclado.TECLA_RIGHT_PARENTHESIS: inteiro", "Código do parêntese direito `)` (522)."),
  constant("TECLA_UNDERSCORE", "Teclado.TECLA_UNDERSCORE: inteiro", "Código do sublinhado `_` (523)."),
  constant("TECLA_WINDOWS", "Teclado.TECLA_WINDOWS: inteiro", "Código da tecla Windows / Super (524)."),
  constant("TECLA_MENU_CONTEXTO", "Teclado.TECLA_MENU_CONTEXTO: inteiro", "Código da tecla Menu de Contexto (525)."),
];

export const PORS_STDLIB_CATALOG: readonly PorsStdlibModule[] = [
  {
    id: "pors-lib:Matematica",
    module: "Matematica",
    name: "PORS: Matematica",
    group: "standard",
    description: "Constantes (`PI`, `E`) e 16 funções matemáticas tipadas sem coerção implícita (VM mínima, WASM e Rust).",
    example: {
      title: "Cálculo de hipotenusa e arredondamento",
      fileName: "exemplo-matematica.fer",
      source: `funcao inicio() {\n    seja a = 3.0;\n    seja b = 4.0;\n    seja hipotenusa: real = Matematica.raiz_quadrada(Matematica.potencia(a, 2.0) + Matematica.potencia(b, 2.0));\n    escreval("Hipotenusa: ", Matematica.arredondar(hipotenusa, 2));\n    escreva("PI ≈ ", Matematica.arredondar(Matematica.PI, 4));\n}\n`,
    },
    items: [
      constant("PI", "Matematica.PI: real", "Constante matemática π (≈ 3.141592653589793)."),
      constant("E", "Matematica.E: real", "Constante de Euler e (≈ 2.718281828459045)."),
      fn("potencia", "Matematica.potencia(base: real, expoente: real) -> real", "Calcula `base` elevado a `expoente`."),
      fn("raiz", "Matematica.raiz(radicando: real, indice: real) -> real", "Calcula a raiz de índice `indice` de `radicando`."),
      fn("raiz_quadrada", "Matematica.raiz_quadrada(numero: real) -> real", "Calcula a raiz quadrada de `numero`."),
      fn("arredondar", "Matematica.arredondar(numero: real, casas: inteiro) -> real", "Arredonda `numero` para a quantidade de casas decimais informada (0 a 15)."),
      fn("piso", "Matematica.piso(numero: real) -> real", "Retorna o maior inteiro (como `real`) menor ou igual a `numero`."),
      fn("teto", "Matematica.teto(numero: real) -> real", "Retorna o menor inteiro (como `real`) maior ou igual a `numero`."),
      fn("logaritmo", "Matematica.logaritmo(numero: real, base: real) -> real", "Calcula o logaritmo de `numero` na base especificada."),
      fn("seno", "Matematica.seno(angulo: real) -> real", "Calcula o seno de `angulo` em radianos."),
      fn("cosseno", "Matematica.cosseno(angulo: real) -> real", "Calcula o cosseno de `angulo` em radianos."),
      fn("tangente", "Matematica.tangente(angulo: real) -> real", "Calcula a tangente de `angulo` em radianos."),
      fn("valor_absoluto", "Matematica.valor_absoluto(numero: real) -> real", "Retorna o módulo (valor absoluto) de um número `real`."),
      fn("valor_absoluto_inteiro", "Matematica.valor_absoluto_inteiro(numero: inteiro) -> inteiro", "Retorna o módulo (valor absoluto) de um número `inteiro`."),
      fn("maior_numero", "Matematica.maior_numero(a: real, b: real) -> real", "Retorna o maior entre dois valores `real`."),
      fn("menor_numero", "Matematica.menor_numero(a: real, b: real) -> real", "Retorna o menor entre dois valores `real`."),
      fn("maior_inteiro", "Matematica.maior_inteiro(a: inteiro, b: inteiro) -> inteiro", "Retorna o maior entre dois valores `inteiro`."),
      fn("menor_inteiro", "Matematica.menor_inteiro(a: inteiro, b: inteiro) -> inteiro", "Retorna o menor entre dois valores `inteiro`."),
    ],
  },
  {
    id: "pors-lib:Texto",
    module: "Texto",
    name: "PORS: Texto",
    group: "standard",
    description: "13 operações Unicode-safe indexadas por `char`, com busca via `Opcao<T>`, fatiamento via `Resultado` e orçamento de memória.",
    example: {
      title: "Normalização e divisão de texto",
      fileName: "exemplo-texto.fer",
      source: `funcao inicio() {\n    seja frase = Texto.aparar("  Ferro e Rust  ");\n    seja partes: vetor<cadeia> = Texto.dividir(frase, " ");\n    escreval("Maiúsculas: ", Texto.caixa_alta(frase));\n    escreval("Palavras: ", tamanho(partes));\n}\n`,
    },
    items: [
      fn("numero_caracteres", "Texto.numero_caracteres(texto: cadeia) -> inteiro", "Retorna a quantidade de caracteres Unicode (`char`) da cadeia."),
      fn("caixa_alta", "Texto.caixa_alta(texto: cadeia) -> cadeia", "Retorna uma nova cadeia convertida para letras maiúsculas."),
      fn("caixa_baixa", "Texto.caixa_baixa(texto: cadeia) -> cadeia", "Retorna uma nova cadeia convertida para letras minúsculas."),
      fn("aparar", "Texto.aparar(texto: cadeia) -> cadeia", "Remove espaços e quebras de linha no início e no fim da cadeia."),
      fn("contem", "Texto.contem(texto: cadeia, trecho: cadeia) -> logico", "Verifica se `texto` contém a subcadeia `trecho`."),
      fn("comeca_com", "Texto.comeca_com(texto: cadeia, prefixo: cadeia) -> logico", "Verifica se `texto` começa com `prefixo`."),
      fn("termina_com", "Texto.termina_com(texto: cadeia, sufixo: cadeia) -> logico", "Verifica se `texto` termina com `sufixo`."),
      fn("substituir", "Texto.substituir(texto: cadeia, antigo: cadeia, novo: cadeia) -> cadeia", "Substitui todas as ocorrências de `antigo` por `novo`, respeitando o limite de memória."),
      fn("preencher_a_esquerda", "Texto.preencher_a_esquerda(car: caracter, tamanho: inteiro, texto: cadeia) -> cadeia", "Preenche `texto` à esquerda com `car` até atingir `tamanho` caracteres."),
      fn("obter_caracter", "Texto.obter_caracter(texto: cadeia, indice: inteiro) -> Opcao<caracter>", "Retorna `Opcao::Algum(c)` no índice Unicode informado, ou `Opcao::Nenhum` fora dos limites."),
      fn("posicao_texto", "Texto.posicao_texto(trecho: cadeia, texto: cadeia, inicio: inteiro) -> Opcao<inteiro>", "Procura `trecho` em `texto` a partir do índice Unicode `inicio`, retornando `Opcao<inteiro>`."),
      fn("extrair_subtexto", "Texto.extrair_subtexto(texto: cadeia, inicio: inteiro, fim: inteiro) -> Resultado<cadeia, cadeia>", "Extrai a fatia Unicode `[inicio..fim)` retornando `Resultado<cadeia, cadeia>`."),
      fn("dividir", "Texto.dividir(texto: cadeia, separador: cadeia) -> vetor<cadeia>", "Divide `texto` nas ocorrências de `separador` (ou em caracteres Unicode quando `separador` é vazio)."),
    ],
  },
  {
    id: "pors-lib:Tipos",
    module: "Tipos",
    name: "PORS: Tipos",
    group: "standard",
    description: "19 conversões explícitas e predicados de validação com `Resultado<T, cadeia>` para parses falíveis.",
    example: {
      title: "Conversão segura de texto para inteiro",
      fileName: "exemplo-tipos.fer",
      source: `funcao inicio() {\n    escolha Tipos.para_inteiro("42") {\n        caso Resultado::Ok(n) => { escreva("Número: ", n, ", real: ", Tipos.inteiro_para_real(n)); }\n        caso Resultado::Erro(e) => { escreva("Erro: ", e); }\n    }\n}\n`,
    },
    items: [
      fn("para_inteiro", "Tipos.para_inteiro(texto: cadeia) -> Resultado<inteiro, cadeia>", "Converte uma cadeia decimal em `inteiro`, retornando `Resultado::Erro` se for inválida."),
      fn("para_inteiro_base", "Tipos.para_inteiro_base(texto: cadeia, base: inteiro) -> Resultado<inteiro, cadeia>", "Converte uma cadeia em `inteiro` na base 2, 10 ou 16."),
      fn("para_real", "Tipos.para_real(texto: cadeia) -> Resultado<real, cadeia>", "Converte uma cadeia em `real` finito, retornando `Resultado<real, cadeia>`."),
      fn("para_logico", "Tipos.para_logico(texto: cadeia) -> Resultado<logico, cadeia>", "Converte `\"verdadeiro\"` ou `\"falso\"` em `logico`, retornando `Resultado<logico, cadeia>`."),
      fn("para_caracter", "Tipos.para_caracter(texto: cadeia) -> Resultado<caracter, cadeia>", "Converte uma cadeia de exatamente 1 caractere Unicode em `caracter`."),
      fn("inteiro_para_real", "Tipos.inteiro_para_real(valor: inteiro) -> real", "Converte explicitamente um `inteiro` em `real`."),
      fn("real_para_inteiro", "Tipos.real_para_inteiro(valor: real) -> inteiro", "Converte explicitamente um `real` em `inteiro` truncando a parte fracionária."),
      fn("inteiro_para_cadeia", "Tipos.inteiro_para_cadeia(valor: inteiro) -> cadeia", "Converte um `inteiro` para representação textual decimal."),
      fn("inteiro_para_cadeia_base", "Tipos.inteiro_para_cadeia_base(valor: inteiro, base: inteiro) -> Resultado<cadeia, cadeia>", "Formata um `inteiro` em cadeia na base 2, 10 ou 16."),
      fn("real_para_cadeia", "Tipos.real_para_cadeia(valor: real) -> cadeia", "Converte um `real` em cadeia."),
      fn("logico_para_cadeia", "Tipos.logico_para_cadeia(valor: logico) -> cadeia", "Converte um `logico` em `\"verdadeiro\"` ou `\"falso\"`."),
      fn("caracter_para_cadeia", "Tipos.caracter_para_cadeia(valor: caracter) -> cadeia", "Converte um `caracter` em cadeia de 1 caractere."),
      fn("logico_para_inteiro", "Tipos.logico_para_inteiro(valor: logico) -> inteiro", "Converte `verdadeiro` em `1` e `falso` em `0`."),
      fn("caracter_para_inteiro", "Tipos.caracter_para_inteiro(valor: caracter) -> Resultado<inteiro, cadeia>", "Converte um caractere de dígito (`'0'` a `'9'`) no inteiro correspondente."),
      fn("inteiro_para_caracter", "Tipos.inteiro_para_caracter(valor: inteiro) -> Resultado<caracter, cadeia>", "Converte um inteiro entre `0` e `9` no caractere de dígito correspondente."),
      fn("cadeia_e_inteiro", "Tipos.cadeia_e_inteiro(texto: cadeia) -> logico", "Verifica se `texto` representa um `inteiro` válido."),
      fn("cadeia_e_real", "Tipos.cadeia_e_real(texto: cadeia) -> logico", "Verifica se `texto` representa um `real` finito válido."),
      fn("cadeia_e_logico", "Tipos.cadeia_e_logico(texto: cadeia) -> logico", "Verifica se `texto` é `\"verdadeiro\"` ou `\"falso\"`."),
      fn("cadeia_e_caracter", "Tipos.cadeia_e_caracter(texto: cadeia) -> logico", "Verifica se `texto` contém exatamente 1 caractere Unicode."),
    ],
  },
  {
    id: "pors-lib:Util",
    module: "Util",
    name: "PORS: Util",
    group: "standard",
    description: "4 utilitários determinísticos e canceláveis (VM mínima, WASM e Rust).",
    example: {
      title: "Sorteio determinístico com semente fixa",
      fileName: "exemplo-util.fer",
      source: `funcao inicio() {\n    Util.definir_semente(42);\n    escolha Util.sorteia(1, 6) {\n        caso Resultado::Ok(dado) => { escreva("Dado sorteado: ", dado); }\n        caso Resultado::Erro(e) => { escreva("Erro: ", e); }\n    }\n}\n`,
    },
    items: [
      fn("sorteia", "Util.sorteia(min: inteiro, max: inteiro) -> Resultado<inteiro, cadeia>", "Sorteia um inteiro no intervalo fechado `[min, max]`, retornando `Resultado::Erro` se `min > max`."),
      fn("definir_semente", "Util.definir_semente(semente: inteiro)", "Fixa a semente determinística do gerador de `Util.sorteia`."),
      fn("tempo_decorrido", "Util.tempo_decorrido() -> inteiro", "Retorna o tempo decorrido em milissegundos desde o início do programa."),
      fn("aguarde", "Util.aguarde(ms: inteiro)", "Pausa a execução por `ms` milissegundos (0 a 60000 ms, cancelável na IDE)."),
    ],
  },
  {
    id: "pors-lib:Calendario",
    module: "Calendario",
    name: "PORS: Calendario",
    group: "standard",
    description: "Contrato UTC gregoriano determinístico com 19 constantes (`DIA_*`, `MES_*`) e 15 funções.",
    example: {
      title: "Data UTC atual e verificação de ano bissexto",
      fileName: "exemplo-calendario.fer",
      source: `funcao inicio() {\n    seja ano = Calendario.ano_atual();\n    escreval("Data UTC: ", Calendario.data_iso_atual());\n    escreva("Ano ", ano, " é bissexto? ", Calendario.eh_bissexto(ano));\n}\n`,
    },
    items: [
      constant("DIA_DOMINGO", "Calendario.DIA_DOMINGO: inteiro", "Constante para domingo (1)."),
      constant("DIA_SEGUNDA_FEIRA", "Calendario.DIA_SEGUNDA_FEIRA: inteiro", "Constante para segunda-feira (2)."),
      constant("DIA_TERCA_FEIRA", "Calendario.DIA_TERCA_FEIRA: inteiro", "Constante para terça-feira (3)."),
      constant("DIA_QUARTA_FEIRA", "Calendario.DIA_QUARTA_FEIRA: inteiro", "Constante para quarta-feira (4)."),
      constant("DIA_QUINTA_FEIRA", "Calendario.DIA_QUINTA_FEIRA: inteiro", "Constante para quinta-feira (5)."),
      constant("DIA_SEXTA_FEIRA", "Calendario.DIA_SEXTA_FEIRA: inteiro", "Constante para sexta-feira (6)."),
      constant("DIA_SABADO", "Calendario.DIA_SABADO: inteiro", "Constante para sábado (7)."),
      constant("MES_JANEIRO", "Calendario.MES_JANEIRO: inteiro", "Constante para janeiro (1)."),
      constant("MES_FEVEREIRO", "Calendario.MES_FEVEREIRO: inteiro", "Constante para fevereiro (2)."),
      constant("MES_MARCO", "Calendario.MES_MARCO: inteiro", "Constante para março (3)."),
      constant("MES_ABRIL", "Calendario.MES_ABRIL: inteiro", "Constante para abril (4)."),
      constant("MES_MAIO", "Calendario.MES_MAIO: inteiro", "Constante para maio (5)."),
      constant("MES_JUNHO", "Calendario.MES_JUNHO: inteiro", "Constante para junho (6)."),
      constant("MES_JULHO", "Calendario.MES_JULHO: inteiro", "Constante para julho (7)."),
      constant("MES_AGOSTO", "Calendario.MES_AGOSTO: inteiro", "Constante para agosto (8)."),
      constant("MES_SETEMBRO", "Calendario.MES_SETEMBRO: inteiro", "Constante para setembro (9)."),
      constant("MES_OUTUBRO", "Calendario.MES_OUTUBRO: inteiro", "Constante para outubro (10)."),
      constant("MES_NOVEMBRO", "Calendario.MES_NOVEMBRO: inteiro", "Constante para novembro (11)."),
      constant("MES_DEZEMBRO", "Calendario.MES_DEZEMBRO: inteiro", "Constante para dezembro (12)."),
      fn("ano_atual", "Calendario.ano_atual() -> inteiro", "Retorna o ano UTC atual."),
      fn("mes_atual", "Calendario.mes_atual() -> inteiro", "Retorna o mês UTC atual (1 a 12)."),
      fn("dia_mes_atual", "Calendario.dia_mes_atual() -> inteiro", "Retorna o dia do mês UTC atual (1 a 31)."),
      fn("dia_semana_atual", "Calendario.dia_semana_atual() -> inteiro", "Retorna o dia da semana UTC atual (1 = Domingo a 7 = Sábado)."),
      fn("hora_atual", "Calendario.hora_atual(formato_12h: logico) -> inteiro", "Retorna a hora UTC atual (0..23 ou 0..11 se `formato_12h` for verdadeiro)."),
      fn("minuto_atual", "Calendario.minuto_atual() -> inteiro", "Retorna o minuto UTC atual (0 a 59)."),
      fn("segundo_atual", "Calendario.segundo_atual() -> inteiro", "Retorna o segundo UTC atual (0 a 59)."),
      fn("milisegundo_atual", "Calendario.milisegundo_atual() -> inteiro", "Retorna o milissegundo UTC atual (0 a 999)."),
      fn("dia_semana_completo", "Calendario.dia_semana_completo(dia: inteiro, caixa_alta: logico, caixa_baixa: logico) -> Resultado<cadeia, cadeia>", "Retorna o nome completo do dia da semana (1 a 7) em `Resultado`."),
      fn("dia_semana_curto", "Calendario.dia_semana_curto(dia: inteiro, caixa_alta: logico, caixa_baixa: logico) -> Resultado<cadeia, cadeia>", "Retorna o nome curto do dia da semana (1 a 7) em `Resultado`."),
      fn("dia_semana_abreviado", "Calendario.dia_semana_abreviado(dia: inteiro, caixa_alta: logico, caixa_baixa: logico) -> Resultado<cadeia, cadeia>", "Retorna a abreviação de 3 letras do dia da semana (1 a 7) em `Resultado`."),
      fn("nome_mes", "Calendario.nome_mes(mes: inteiro) -> Resultado<cadeia, cadeia>", "Retorna o nome em português do mês informado (1 a 12)."),
      fn("eh_bissexto", "Calendario.eh_bissexto(ano: inteiro) -> logico", "Verifica se o ano gregoriano informado é bissexto."),
      fn("dias_no_mes", "Calendario.dias_no_mes(ano: inteiro, mes: inteiro) -> Resultado<inteiro, cadeia>", "Retorna a quantidade de dias do mês/ano informados (28 a 31)."),
      fn("data_iso_atual", "Calendario.data_iso_atual() -> cadeia", "Retorna a data UTC atual formatada como `AAAA-MM-DD`."),
    ],
  },
  {
    id: "pors-lib:Arquivos",
    module: "Arquivos",
    name: "PORS: Arquivos",
    group: "standard",
    description: "7 operações de leitura e escrita UTF-8 confinadas ao workspace (bloqueia caminhos absolutos, `..` e symlinks; limite de 2 MiB).",
    example: {
      title: "Verificando existência de arquivo no workspace",
      fileName: "exemplo-arquivos.fer",
      source: `funcao inicio() {\n    escolha Arquivos.existe("notas.txt") {\n        caso Resultado::Ok(achou) => { escreva("Arquivo existe? ", achou); }\n        caso Resultado::Erro(e) => { escreva("Erro: ", e); }\n    }\n}\n`,
    },
    items: [
      fn("ler_texto", "Arquivos.ler_texto(caminho: cadeia) -> Resultado<cadeia, cadeia>", "Lê um arquivo de texto UTF-8 confinado ao workspace (limite de 2 MiB)."),
      fn("ler_linhas", "Arquivos.ler_linhas(caminho: cadeia) -> Resultado<vetor<cadeia>, cadeia>", "Lê as linhas de um arquivo UTF-8 do workspace em `vetor<cadeia>`."),
      fn("escrever_texto", "Arquivos.escrever_texto(caminho: cadeia, conteudo: cadeia) -> Resultado<logico, cadeia>", "Grava texto UTF-8 em um caminho relativo ao workspace (criando subpastas se necessário)."),
      fn("acrescentar_texto", "Arquivos.acrescentar_texto(caminho: cadeia, conteudo: cadeia) -> Resultado<logico, cadeia>", "Acrescenta texto ao final de um arquivo relativo ao workspace."),
      fn("existe", "Arquivos.existe(caminho: cadeia) -> Resultado<logico, cadeia>", "Verifica se o caminho relativo existe dentro do workspace."),
      fn("remover", "Arquivos.remover(caminho: cadeia) -> Resultado<logico, cadeia>", "Remove um arquivo dentro do workspace."),
      fn("listar", "Arquivos.listar(diretorio: cadeia) -> Resultado<vetor<cadeia>, cadeia>", "Lista de forma ordenada até 1024 entradas de um diretório relativo ao workspace."),
    ],
  },
  {
    id: "pors-lib:Dialogo",
    module: "Dialogo",
    name: "PORS: Dialogo",
    group: "visual",
    description: "Caixas do sistema para as telas desktop: aviso, confirmação e escolha de arquivo. Fora de uma tela, cada função explica onde funciona.",
    example: {
      title: "Confirmando antes de agir (numa função de tela)",
      fileName: "exemplo-dialogo.fer",
      source: `funcao inicio() {\n    se (Dialogo.confirmar("Salvar as notas?")) {\n        Dialogo.mensagem("Notas salvas.");\n    }\n}\n`,
    },
    items: [
      fn("mensagem", "Dialogo.mensagem(texto: cadeia) -> vazio", "Mostra um aviso com o botão OK, preso à janela da tela."),
      fn("confirmar", "Dialogo.confirmar(texto: cadeia) -> logico", "Pergunta com Sim e Não; devolve `verdadeiro` para Sim."),
      fn("abrir_arquivo", "Dialogo.abrir_arquivo(dialogo: DialogoArquivo) -> Opcao<cadeia>", "Escolhe um arquivo para abrir com o título e o filtro do `<DialogoArquivo>` da tela (`t.dlgAbrir`); `Opcao::Nenhum` se o usuário cancelar."),
      fn("salvar_arquivo", "Dialogo.salvar_arquivo(dialogo: DialogoArquivo) -> Opcao<cadeia>", "Escolhe onde salvar, com o título e o filtro do `<DialogoArquivo>` da tela; `Opcao::Nenhum` se o usuário cancelar."),
    ],
  },
  {
    id: "pors-lib:Json",
    module: "Json",
    name: "PORS: Json",
    group: "standard",
    description: "19 operações JSON (RFC 8259) puras e zero-dependência para validar, consultar, construir e formatar objetos JSON (VM mínima, WASM e Rust).",
    example: {
      title: "Construindo e consultando um objeto JSON",
      fileName: "exemplo-json.fer",
      source: `funcao demo() -> Resultado<cadeia, cadeia> {\n    seja base: cadeia = Json.definir_cadeia("{}", "linguagem", "Ferro")?;\n    seja completo: cadeia = Json.definir_inteiro(base, "versao", 1)?;\n    retorne Json.formatar(completo);\n}\n\nfuncao inicio() {\n    escolha demo() {\n        caso Resultado::Ok(saida) => { escreva(saida); }\n        caso Resultado::Erro(e) => { escreva("Erro: ", e); }\n    }\n}\n`,
    },
    items: [
      fn("valido", "Json.valido(texto: cadeia) -> logico", "Verifica se `texto` é um documento JSON válido (RFC 8259, profundidade máxima 64)."),
      fn("formatar", "Json.formatar(texto: cadeia) -> Resultado<cadeia, cadeia>", "Indenta um documento JSON válido com 2 espaços por nível."),
      fn("minificar", "Json.minificar(texto: cadeia) -> Resultado<cadeia, cadeia>", "Compacta um documento JSON válido removendo espaços desnecessários."),
      fn("escapar_cadeia", "Json.escapar_cadeia(texto: cadeia) -> cadeia", "Codifica uma cadeia como literal de string JSON escapado (`\"...\"`)."),
      fn("contem_chave", "Json.contem_chave(json: cadeia, chave: cadeia) -> Resultado<logico, cadeia>", "Verifica se o objeto JSON raiz contém a chave informada."),
      fn("obter_chaves", "Json.obter_chaves(json: cadeia) -> Resultado<vetor<cadeia>, cadeia>", "Retorna a lista de chaves do objeto JSON raiz em ordem."),
      fn("obter_cadeia", "Json.obter_cadeia(json: cadeia, chave: cadeia) -> Resultado<cadeia, cadeia>", "Lê uma propriedade de texto (`cadeia`) do objeto JSON."),
      fn("obter_inteiro", "Json.obter_inteiro(json: cadeia, chave: cadeia) -> Resultado<inteiro, cadeia>", "Lê uma propriedade numérica inteira (`inteiro`) do objeto JSON."),
      fn("obter_real", "Json.obter_real(json: cadeia, chave: cadeia) -> Resultado<real, cadeia>", "Lê uma propriedade numérica finita (`real`) do objeto JSON."),
      fn("obter_logico", "Json.obter_logico(json: cadeia, chave: cadeia) -> Resultado<logico, cadeia>", "Lê uma propriedade booleana (`logico`) do objeto JSON."),
      fn("obter_json", "Json.obter_json(json: cadeia, chave: cadeia) -> Resultado<cadeia, cadeia>", "Extrai o subvalor JSON (objeto, lista ou primitivo) de uma chave como cadeia JSON minificada."),
      fn("obter_lista_cadeia", "Json.obter_lista_cadeia(json: cadeia, chave: cadeia) -> Resultado<vetor<cadeia>, cadeia>", "Lê uma propriedade de lista JSON de strings em `vetor<cadeia>`."),
      fn("obter_lista_inteiro", "Json.obter_lista_inteiro(json: cadeia, chave: cadeia) -> Resultado<vetor<inteiro>, cadeia>", "Lê uma propriedade de lista JSON de inteiros em `vetor<inteiro>`."),
      fn("definir_cadeia", "Json.definir_cadeia(json: cadeia, chave: cadeia, valor: cadeia) -> Resultado<cadeia, cadeia>", "Retorna um novo objeto JSON com a chave definida para a cadeia informada."),
      fn("definir_inteiro", "Json.definir_inteiro(json: cadeia, chave: cadeia, valor: inteiro) -> Resultado<cadeia, cadeia>", "Retorna um novo objeto JSON com a chave definida para o inteiro informado."),
      fn("definir_real", "Json.definir_real(json: cadeia, chave: cadeia, valor: real) -> Resultado<cadeia, cadeia>", "Retorna um novo objeto JSON com a chave definida para o número real finito informado."),
      fn("definir_logico", "Json.definir_logico(json: cadeia, chave: cadeia, valor: logico) -> Resultado<cadeia, cadeia>", "Retorna um novo objeto JSON com a chave definida para o valor lógico informado."),
      fn("definir_json", "Json.definir_json(json: cadeia, chave: cadeia, valor_json: cadeia) -> Resultado<cadeia, cadeia>", "Retorna um novo objeto JSON com a chave definida para um subvalor JSON já válido (objeto ou lista)."),
      fn("remover_chave", "Json.remover_chave(json: cadeia, chave: cadeia) -> Resultado<cadeia, cadeia>", "Retorna um novo objeto JSON sem a chave especificada."),
    ],
  },
  {
    id: "pors-lib:Internet",
    module: "Internet",
    name: "PORS: Internet",
    group: "standard",
    description: "4 operações HTTP/HTTPS com consentimento explícito (negado por padrão), timeout (100..30000 ms), cancelamento no debugger e limite de 1 MiB.",
    example: {
      title: "Ajustando timeout e testando um endereço",
      fileName: "exemplo-internet.fer",
      source: `funcao inicio() {\n    Internet.definir_tempo_limite(3000);\n    escolha Internet.endereco_disponivel("https://example.com") {\n        caso Resultado::Ok(ok) => { escreva("Disponível: ", ok); }\n        caso Resultado::Erro(e) => { escreva("Bloqueado ou indisponível: ", e); }\n    }\n}\n`,
    },
    items: [
      fn("definir_tempo_limite", "Internet.definir_tempo_limite(ms: inteiro)", "Define o tempo limite das requisições HTTP entre 100 e 30000 ms (padrão: 5000 ms)."),
      fn("obter_texto", "Internet.obter_texto(url: cadeia) -> Resultado<cadeia, cadeia>", "Executa HTTP GET com consentimento explícito, cancelamento cooperativo e limite de 1 MiB."),
      fn("enviar_json", "Internet.enviar_json(url: cadeia, corpo_json: cadeia) -> Resultado<cadeia, cadeia>", "Valida o JSON e envia via HTTP POST (`application/json`), retornando a resposta em `Resultado`."),
      fn("endereco_disponivel", "Internet.endereco_disponivel(url: cadeia) -> Resultado<logico, cadeia>", "Verifica se um endereço HTTP/HTTPS responde via HEAD com status 2xx/3xx."),
    ],
  },
  {
    id: "pors-lib:UI",
    module: "UI",
    name: "PORS: UI",
    group: "visual",
    description: "6 componentes declarativos MVU (`iniciar`, `atualizar`, `visualizar`) para interfaces gráficas interativas na IDE, WASM e Rust.",
    example: {
      title: "Contador interativo com UI.*",
      fileName: "exemplo-ui.fer",
      source: `funcao iniciar() -> inteiro {\n    retorne 0;\n}\n\nfuncao atualizar(estado: inteiro, tipo: cadeia, alvo: cadeia) -> inteiro {\n    se (tipo == "clique" && alvo == "inc") {\n        retorne estado + 1;\n    }\n    retorne estado;\n}\n\nfuncao visualizar(estado: inteiro) -> inteiro {\n    retorne UI.coluna([\n        UI.texto("Contagem: ", estado),\n        UI.botao("inc", "+1")\n    ]);\n}\n`,
    },
    items: [
      fn("texto", "UI.texto(valor, ...) -> inteiro", "Cria um nó de texto declarativo concatenando valores escalares."),
      fn("botao", "UI.botao(id: cadeia, rotulo: cadeia) -> inteiro", "Cria um botão interativo que dispara eventos `clique` com o identificador informado."),
      fn("entrada", "UI.entrada(id: cadeia, valor: cadeia, dica: cadeia) -> inteiro", "Cria um campo de texto controlado que dispara eventos `entrada_alterada`."),
      fn("coluna", "UI.coluna(filhos: vetor<inteiro>) -> inteiro", "Organiza nós filhos verticalmente em uma coluna."),
      fn("linha", "UI.linha(filhos: vetor<inteiro>) -> inteiro", "Organiza nós filhos horizontalmente em uma linha."),
      fn("lista", "UI.lista(itens: vetor<inteiro>) -> inteiro", "Renderiza uma lista de nós visuais filhos."),
    ],
  },
  {
    id: "pors-lib:Graficos",
    module: "Graficos",
    name: "PORS: Graficos",
    group: "visual",
    description: "17 constantes (`COR_*`, `GRADIENTE_*`, `CANAL_*`) e 54 funções para janela gráfica 2D, cores, formas, texto, imagens e GIFs na VM.",
    example: {
      title: "Desenhando formas básicas na janela gráfica",
      fileName: "exemplo-graficos.fer",
      source: `funcao inicio() {\n    Graficos.iniciar_modo_grafico(verdadeiro);\n    Graficos.definir_dimensoes_janela(400, 260);\n    Graficos.definir_titulo_janela("Ferro Gráficos");\n    Graficos.definir_cor(Graficos.COR_PRETO);\n    Graficos.limpar();\n    Graficos.definir_cor(Graficos.COR_AMARELO);\n    Graficos.desenhar_elipse(150, 80, 100, 100, verdadeiro);\n    Graficos.renderizar();\n    Util.aguarde(2000);\n    Graficos.encerrar_modo_grafico();\n}\n`,
    },
    items: [
      constant("COR_PRETO", "Graficos.COR_PRETO: inteiro", "Constante de cor preta (ARGB `0xFF000000`)."),
      constant("COR_BRANCO", "Graficos.COR_BRANCO: inteiro", "Constante de cor branca (ARGB `0xFFFFFFFF`)."),
      constant("COR_AZUL", "Graficos.COR_AZUL: inteiro", "Constante de cor azul (ARGB `0xFF0000FF`)."),
      constant("COR_VERMELHO", "Graficos.COR_VERMELHO: inteiro", "Constante de cor vermelha (ARGB `0xFFFF0000`)."),
      constant("COR_VERDE", "Graficos.COR_VERDE: inteiro", "Constante de cor verde (ARGB `0xFF00FF00`)."),
      constant("COR_AMARELO", "Graficos.COR_AMARELO: inteiro", "Constante de cor amarela (ARGB `0xFFFFFF00`)."),
      constant("GRADIENTE_DIREITA", "Graficos.GRADIENTE_DIREITA: inteiro", "Direção de gradiente horizontal para a direita (0)."),
      constant("GRADIENTE_ESQUERDA", "Graficos.GRADIENTE_ESQUERDA: inteiro", "Direção de gradiente horizontal para a esquerda (1)."),
      constant("GRADIENTE_ACIMA", "Graficos.GRADIENTE_ACIMA: inteiro", "Direção de gradiente vertical para cima (2)."),
      constant("GRADIENTE_ABAIXO", "Graficos.GRADIENTE_ABAIXO: inteiro", "Direção de gradiente vertical para baixo (3)."),
      constant("GRADIENTE_INFERIOR_DIREITO", "Graficos.GRADIENTE_INFERIOR_DIREITO: inteiro", "Direção de gradiente diagonal para o canto inferior direito (4)."),
      constant("GRADIENTE_INFERIOR_ESQUERDO", "Graficos.GRADIENTE_INFERIOR_ESQUERDO: inteiro", "Direção de gradiente diagonal para o canto inferior esquerdo (5)."),
      constant("GRADIENTE_SUPERIOR_DIREITO", "Graficos.GRADIENTE_SUPERIOR_DIREITO: inteiro", "Direção de gradiente diagonal para o canto superior direito (6)."),
      constant("GRADIENTE_SUPERIOR_ESQUERDO", "Graficos.GRADIENTE_SUPERIOR_ESQUERDO: inteiro", "Direção de gradiente diagonal para o canto superior esquerdo (7)."),
      constant("CANAL_R", "Graficos.CANAL_R: inteiro", "Canal vermelho (0) para `Graficos.obter_rgb`."),
      constant("CANAL_G", "Graficos.CANAL_G: inteiro", "Canal verde (1) para `Graficos.obter_rgb`."),
      constant("CANAL_B", "Graficos.CANAL_B: inteiro", "Canal azul (2) para `Graficos.obter_rgb`."),
      fn("iniciar_modo_grafico", "Graficos.iniciar_modo_grafico(manter_visivel: logico)", "Abre a janela gráfica nativa do host da VM."),
      fn("encerrar_modo_grafico", "Graficos.encerrar_modo_grafico()", "Encerra e fecha a janela gráfica ativa."),
      fn("fechar_janela", "Graficos.fechar_janela()", "Fecha a janela gráfica sem encerrar o restante do programa."),
      fn("minimizar_janela", "Graficos.minimizar_janela()", "Minimiza a janela gráfica."),
      fn("restaurar_janela", "Graficos.restaurar_janela()", "Restaura a janela gráfica minimizada ou maximizada."),
      fn("ocultar_borda_janela", "Graficos.ocultar_borda_janela()", "Remove a moldura/decoração da janela gráfica."),
      fn("exibir_borda_janela", "Graficos.exibir_borda_janela()", "Exibe a moldura/decoração da janela gráfica."),
      fn("definir_dimensoes_janela", "Graficos.definir_dimensoes_janela(largura: inteiro, altura: inteiro)", "Define a largura e altura da janela gráfica em pixels."),
      fn("definir_titulo_janela", "Graficos.definir_titulo_janela(titulo: cadeia)", "Define o título exibido na janela gráfica."),
      fn("entrar_modo_tela_cheia", "Graficos.entrar_modo_tela_cheia()", "Coloca a janela gráfica em modo tela cheia."),
      fn("sair_modo_tela_cheia", "Graficos.sair_modo_tela_cheia()", "Sai do modo tela cheia da janela gráfica."),
      fn("definir_icone_janela", "Graficos.definir_icone_janela(imagem: inteiro)", "Define o ícone da janela gráfica usando uma imagem carregada."),
      fn("largura_janela", "Graficos.largura_janela() -> inteiro", "Retorna a largura atual da janela gráfica em pixels."),
      fn("altura_janela", "Graficos.altura_janela() -> inteiro", "Retorna a altura atual da janela gráfica em pixels."),
      fn("largura_tela", "Graficos.largura_tela() -> inteiro", "Retorna a largura do monitor principal em pixels."),
      fn("altura_tela", "Graficos.altura_tela() -> inteiro", "Retorna a altura do monitor principal em pixels."),
      fn("limpar", "Graficos.limpar()", "Limpa o quadro atual com a cor selecionada."),
      fn("renderizar", "Graficos.renderizar()", "Apresenta as operações de desenho acumuladas na janela."),
      fn("renderizar_imagem", "Graficos.renderizar_imagem(largura: inteiro, altura: inteiro) -> Resultado<inteiro, cadeia>", "Captura o quadro desenhado em uma nova imagem de `largura` × `altura`."),
      fn("definir_cor", "Graficos.definir_cor(cor: inteiro)", "Define a cor ativa de desenho e preenchimento."),
      fn("definir_gradiente", "Graficos.definir_gradiente(tipo: inteiro, cor1: inteiro, cor2: inteiro)", "Configura preenchimento com gradiente entre `cor1` e `cor2` usando uma constante `GRADIENTE_*`."),
      fn("definir_opacidade", "Graficos.definir_opacidade(opacidade: inteiro)", "Define a opacidade (0 a 255) das próximas operações."),
      fn("definir_rotacao", "Graficos.definir_rotacao(angulo: inteiro)", "Define o ângulo de rotação em graus para as próximas formas."),
      fn("criar_cor", "Graficos.criar_cor(r: inteiro, g: inteiro, b: inteiro) -> Resultado<inteiro, cadeia>", "Cria uma cor RGB (0..255) retornando `Resultado<inteiro, cadeia>`."),
      fn("obter_rgb", "Graficos.obter_rgb(cor: inteiro, canal: inteiro) -> inteiro", "Extrai o valor (0..255) do canal `CANAL_R`, `CANAL_G` ou `CANAL_B`."),
      fn("desenhar_retangulo", "Graficos.desenhar_retangulo(x: inteiro, y: inteiro, largura: inteiro, altura: inteiro, arredondar: logico, preencher: logico)", "Desenha um retângulo no quadro atual."),
      fn("desenhar_elipse", "Graficos.desenhar_elipse(x: inteiro, y: inteiro, largura: inteiro, altura: inteiro, preencher: logico)", "Desenha uma elipse ou círculo no quadro atual."),
      fn("desenhar_linha", "Graficos.desenhar_linha(x1: inteiro, y1: inteiro, x2: inteiro, y2: inteiro)", "Desenha um segmento de reta entre `(x1, y1)` e `(x2, y2)`."),
      fn("desenhar_ponto", "Graficos.desenhar_ponto(x: inteiro, y: inteiro)", "Desenha um ponto em `(x, y)`."),
      fn("desenhar_poligono", "Graficos.desenhar_poligono(xs: vetor<inteiro>, ys: vetor<inteiro>, preencher: logico) -> Resultado<logico, cadeia>", "Desenha um polígono a partir de vetores tipados de coordenadas X e Y."),
      fn("desenhar_texto", "Graficos.desenhar_texto(x: inteiro, y: inteiro, texto: cadeia)", "Desenha texto no quadro atual na posição `(x, y)`."),
      fn("definir_fonte_texto", "Graficos.definir_fonte_texto(nome: cadeia)", "Seleciona o nome da família de fonte para desenhos de texto."),
      fn("definir_tamanho_texto", "Graficos.definir_tamanho_texto(tamanho: real)", "Define o tamanho da fonte de texto."),
      fn("definir_estilo_texto", "Graficos.definir_estilo_texto(italico: logico, negrito: logico, sublinhado: logico)", "Configura itálico, negrito e sublinhado do texto."),
      fn("carregar_fonte", "Graficos.carregar_fonte(caminho: cadeia) -> Resultado<logico, cadeia>", "Carrega uma fonte TTF/OTF confinada ao workspace."),
      fn("largura_texto", "Graficos.largura_texto(texto: cadeia) -> inteiro", "Calcula a largura do texto na fonte atual em pixels."),
      fn("altura_texto", "Graficos.altura_texto(texto: cadeia) -> inteiro", "Calcula a altura do texto na fonte atual em pixels."),
      fn("carregar_imagem", "Graficos.carregar_imagem(caminho: cadeia) -> Resultado<inteiro, cadeia>", "Carrega imagem (PNG/JPG/GIF) confinada ao workspace retornando seu identificador."),
      fn("liberar_imagem", "Graficos.liberar_imagem(imagem: inteiro)", "Libera uma imagem carregada da memória da VM."),
      fn("largura_imagem", "Graficos.largura_imagem(imagem: inteiro) -> Resultado<inteiro, cadeia>", "Retorna a largura da imagem em pixels via `Resultado`."),
      fn("altura_imagem", "Graficos.altura_imagem(imagem: inteiro) -> Resultado<inteiro, cadeia>", "Retorna a altura da imagem em pixels via `Resultado`."),
      fn("desenhar_imagem", "Graficos.desenhar_imagem(x: inteiro, y: inteiro, imagem: inteiro)", "Desenha a imagem carregada na posição `(x, y)`."),
      fn("desenhar_porcao_imagem", "Graficos.desenhar_porcao_imagem(x: inteiro, y: inteiro, xi: inteiro, yi: inteiro, largura: inteiro, altura: inteiro, imagem: inteiro)", "Desenha apenas o recorte `(xi, yi, largura, altura)` da imagem na posição `(x, y)`."),
      fn("transformar_imagem", "Graficos.transformar_imagem(imagem: inteiro, espelhar_h: logico, espelhar_v: logico, rotacao: inteiro, cor_transparente: inteiro) -> Resultado<inteiro, cadeia>", "Gera uma nova imagem com espelhamento, rotação e cor transparente opcional."),
      fn("transformar_porcao_imagem", "Graficos.transformar_porcao_imagem(imagem: inteiro, xi: inteiro, yi: inteiro, largura: inteiro, altura: inteiro, espelhar_h: logico, espelhar_v: logico, rotacao: inteiro, cor_transparente: inteiro) -> Resultado<inteiro, cadeia>", "Gera uma nova imagem transformada a partir de um recorte da imagem original."),
      fn("redimensionar_imagem", "Graficos.redimensionar_imagem(imagem: inteiro, largura: inteiro, altura: inteiro, manter_qualidade: logico) -> Resultado<inteiro, cadeia>", "Cria uma nova imagem redimensionada para `largura` × `altura`."),
      fn("obter_cor_pixel", "Graficos.obter_cor_pixel(imagem: inteiro, x: inteiro, y: inteiro) -> Resultado<inteiro, cadeia>", "Lê a cor ARGB do pixel `(x, y)` de uma imagem carregada."),
      fn("salvar_imagem", "Graficos.salvar_imagem(imagem: inteiro, caminho: cadeia) -> Resultado<logico, cadeia>", "Salva a imagem em PNG dentro do workspace."),
      fn("proximo_frame_gif", "Graficos.proximo_frame_gif(imagem: inteiro)", "Avança um GIF animado para o próximo quadro."),
      fn("obter_intervalo_gif", "Graficos.obter_intervalo_gif(imagem: inteiro) -> Resultado<inteiro, cadeia>", "Retorna o intervalo em milissegundos do quadro atual de um GIF."),
      fn("obter_numero_quadros_gif", "Graficos.obter_numero_quadros_gif(imagem: inteiro) -> Resultado<inteiro, cadeia>", "Retorna o total de quadros de um GIF carregado."),
      fn("obter_numero_quadro_atual_gif", "Graficos.obter_numero_quadro_atual_gif(imagem: inteiro) -> Resultado<inteiro, cadeia>", "Retorna o índice do quadro atual de um GIF."),
      fn("obter_quadro_gif", "Graficos.obter_quadro_gif(imagem: inteiro, quadro: inteiro) -> Resultado<inteiro, cadeia>", "Extrai um quadro específico do GIF como uma nova imagem."),
      fn("definir_quadro_gif", "Graficos.definir_quadro_gif(imagem: inteiro, quadro: inteiro) -> Resultado<logico, cadeia>", "Seleciona o quadro ativo de um GIF animado."),
    ],
  },
  {
    id: "pors-lib:Teclado",
    module: "Teclado",
    name: "PORS: Teclado",
    group: "visual",
    description: "4 funções de leitura e estado de teclas na janela gráfica + constantes `TECLA_*` (também aceita `TECLA_A`..`TECLA_Z`, `TECLA_0`..`TECLA_9`, `TECLA_F1`..`TECLA_F24` e `TECLA_0_NUM`..`TECLA_9_NUM`).",
    example: {
      title: "Aguardando a tecla ESC na janela gráfica",
      fileName: "exemplo-teclado.fer",
      source: `funcao inicio() {\n    Graficos.iniciar_modo_grafico(verdadeiro);\n    Graficos.definir_titulo_janela("Pressione ESC para sair");\n    enquanto (!Teclado.tecla_pressionada(Teclado.TECLA_ESC)) {\n        Graficos.definir_cor(Graficos.COR_PRETO);\n        Graficos.limpar();\n        Graficos.definir_cor(Graficos.COR_BRANCO);\n        Graficos.desenhar_texto(20, 40, "Pressione ESC para encerrar");\n        Graficos.renderizar();\n        Util.aguarde(16);\n    }\n    Graficos.encerrar_modo_grafico();\n}\n`,
    },
    items: [
      fn("tecla_pressionada", "Teclado.tecla_pressionada(tecla: inteiro) -> logico", "Verifica se a tecla informada está pressionada na janela gráfica."),
      fn("alguma_tecla_pressionada", "Teclado.alguma_tecla_pressionada() -> logico", "Verifica se qualquer tecla está pressionada."),
      fn("ler_tecla", "Teclado.ler_tecla() -> Resultado<inteiro, cadeia>", "Aguarda uma tecla na janela gráfica retornando `Resultado<inteiro, cadeia>`."),
      fn("caracter_tecla", "Teclado.caracter_tecla(tecla: inteiro) -> caracter", "Converte um código de tecla em `caracter`."),
      ...TECLADO_NAMED_CONSTANTS,
    ],
  },
  {
    id: "pors-lib:Mouse",
    module: "Mouse",
    name: "PORS: Mouse",
    group: "visual",
    description: "3 constantes (`BOTAO_*`) e 7 funções para posição, clique e cursor do mouse na janela gráfica.",
    example: {
      title: "Acompanhando a posição do mouse",
      fileName: "exemplo-mouse.fer",
      source: `funcao inicio() {\n    Graficos.iniciar_modo_grafico(verdadeiro);\n    Graficos.definir_dimensoes_janela(400, 260);\n    enquanto (!Teclado.tecla_pressionada(Teclado.TECLA_ESC)) {\n        Graficos.definir_cor(Graficos.COR_PRETO);\n        Graficos.limpar();\n        Graficos.definir_cor(Graficos.COR_VERDE);\n        Graficos.desenhar_elipse(Mouse.posicao_x() - 10, Mouse.posicao_y() - 10, 20, 20, verdadeiro);\n        Graficos.renderizar();\n        Util.aguarde(16);\n    }\n    Graficos.encerrar_modo_grafico();\n}\n`,
    },
    items: [
      constant("BOTAO_ESQUERDO", "Mouse.BOTAO_ESQUERDO: inteiro", "Código do botão esquerdo do mouse (0)."),
      constant("BOTAO_DIREITO", "Mouse.BOTAO_DIREITO: inteiro", "Código do botão direito do mouse (1)."),
      constant("BOTAO_MEIO", "Mouse.BOTAO_MEIO: inteiro", "Código do botão central do mouse (2)."),
      fn("botao_pressionado", "Mouse.botao_pressionado(botao: inteiro) -> logico", "Verifica se o botão especificado está pressionado."),
      fn("algum_botao_pressionado", "Mouse.algum_botao_pressionado() -> logico", "Verifica se algum botão do mouse está pressionado."),
      fn("ler_botao", "Mouse.ler_botao() -> Resultado<inteiro, cadeia>", "Aguarda um clique na janela gráfica retornando `Resultado<inteiro, cadeia>`."),
      fn("posicao_x", "Mouse.posicao_x() -> inteiro", "Retorna a coordenada X atual do mouse na janela."),
      fn("posicao_y", "Mouse.posicao_y() -> inteiro", "Retorna a coordenada Y atual do mouse na janela."),
      fn("ocultar_cursor", "Mouse.ocultar_cursor()", "Oculta o cursor do mouse sobre a janela gráfica."),
      fn("exibir_cursor", "Mouse.exibir_cursor()", "Volta a exibir o cursor do mouse sobre a janela gráfica."),
    ],
  },
  {
    id: "pors-lib:Sons",
    module: "Sons",
    name: "PORS: Sons",
    group: "visual",
    description: "12 funções de carregamento de áudio confinado ao workspace, reprodução em loop, busca temporal e volume na VM.",
    example: {
      title: "Ajustando o volume mestre de áudio",
      fileName: "exemplo-sons.fer",
      source: `funcao inicio() {\n    Sons.definir_volume(80);\n    escreva("Volume atual: ", Sons.obter_volume(), "%");\n}\n`,
    },
    items: [
      fn("carregar_som", "Sons.carregar_som(caminho: cadeia) -> Resultado<inteiro, cadeia>", "Carrega um arquivo de áudio confinado ao workspace e retorna seu identificador."),
      fn("liberar_som", "Sons.liberar_som(som: inteiro)", "Libera o som carregado da memória."),
      fn("reproduzir_som", "Sons.reproduzir_som(som: inteiro, repetir: logico) -> Resultado<inteiro, cadeia>", "Inicia a reprodução do som, opcionalmente em loop."),
      fn("pausar_som", "Sons.pausar_som(som: inteiro)", "Pausa a reprodução do som informado."),
      fn("interromper_som", "Sons.interromper_som(som: inteiro)", "Interrompe a reprodução do som e volta ao início."),
      fn("obter_tamanho_musica", "Sons.obter_tamanho_musica(som: inteiro) -> inteiro", "Retorna a duração total da faixa em milissegundos."),
      fn("obter_posicao_atual_musica", "Sons.obter_posicao_atual_musica(som: inteiro) -> inteiro", "Retorna a posição atual da reprodução em milissegundos."),
      fn("definir_posicao_atual_musica", "Sons.definir_posicao_atual_musica(som: inteiro, posicao_ms: inteiro) -> Resultado<logico, cadeia>", "Move a reprodução para a posição informada em milissegundos."),
      fn("definir_volume", "Sons.definir_volume(volume: inteiro)", "Define o volume mestre de áudio (0 a 100)."),
      fn("definir_volume_reproducao", "Sons.definir_volume_reproducao(som: inteiro, volume: inteiro)", "Define o volume individual de um som carregado (0 a 100)."),
      fn("obter_volume", "Sons.obter_volume() -> inteiro", "Retorna o volume mestre atual (0 a 100)."),
      fn("obter_volume_reproducao", "Sons.obter_volume_reproducao(som: inteiro) -> inteiro", "Retorna o volume individual do som informado."),
    ],
  },
];

export interface ParsedSignatureDetails {
  readonly parameters: readonly { name: string; type: string }[];
  readonly returns: string | null;
}

/** Extrai parâmetros e tipo de retorno de uma assinatura PORS tipada. */
export function parsePorsSignature(signature: string): ParsedSignatureDetails {
  const open = signature.indexOf("(");
  const close = signature.lastIndexOf(")");
  if (open === -1 || close === -1 || close < open) {
    const colon = signature.indexOf(":");
    return {
      parameters: [],
      returns: colon >= 0 ? signature.slice(colon + 1).trim() : null,
    };
  }
  const rawParams = signature.slice(open + 1, close).trim();
  const parameters: { name: string; type: string }[] = [];
  if (rawParams) {
    let depth = 0;
    let start = 0;
    const parts: string[] = [];
    for (let i = 0; i < rawParams.length; i += 1) {
      const ch = rawParams[i];
      if (ch === "<") depth += 1;
      else if (ch === ">") depth = Math.max(0, depth - 1);
      else if (ch === "," && depth === 0) {
        parts.push(rawParams.slice(start, i).trim());
        start = i + 1;
      }
    }
    const tail = rawParams.slice(start).trim();
    if (tail) parts.push(tail);
    for (const part of parts) {
      const idx = part.indexOf(":");
      if (idx >= 0) {
        parameters.push({ name: part.slice(0, idx).trim(), type: part.slice(idx + 1).trim() });
      } else if (part) {
        parameters.push({ name: part, type: "escalar" });
      }
    }
  }
  const after = signature.slice(close + 1).trim();
  const returns = after.startsWith("->") ? after.slice(2).trim() : "sem retorno";
  return { parameters, returns };
}
