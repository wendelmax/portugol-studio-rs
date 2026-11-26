//! Catálogo oficial de exemplos Portugol-RS (`.pors`), sob licença do
//! projeto (`LGPL-3.0-only`), validado e executado automaticamente em testes.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PorsExample {
    pub id: &'static str,
    pub category: &'static str,
    pub title: &'static str,
    pub file_name: &'static str,
    pub description: &'static str,
    pub is_visual: bool,
    pub license: &'static str,
    pub source: &'static str,
}

const EXAMPLES: &[PorsExample] = &[
    PorsExample {
        id: "pors-01-ola-variaveis",
        category: "Portugol-RS (destaques) / Fundamentos",
        title: "Olá e variáveis tipadas",
        file_name: "01-ola-variaveis.pors",
        description: "Demonstra seja, seja mutavel, tipos escalares e escrita com escreva.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/01-ola-variaveis.pors"),
    },
    PorsExample {
        id: "pors-02-condicionais",
        category: "Portugol-RS (destaques) / Controle de fluxo",
        title: "Condicionais com se/senao",
        file_name: "02-condicionais.pors",
        description: "Demonstra desvios condicionais com comparações e operador lógico &&.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/02-condicionais.pors"),
    },
    PorsExample {
        id: "pors-03-lacos",
        category: "Portugol-RS (destaques) / Controle de fluxo",
        title: "Laço para com pare e continue",
        file_name: "03-lacos.pors",
        description: "Demonstra contagem com para, salto com continue e interrupção com pare.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/03-lacos.pors"),
    },
    PorsExample {
        id: "pors-04-funcoes-recursao",
        category: "Portugol-RS (destaques) / Funções",
        title: "Função recursiva (fatorial)",
        file_name: "04-funcoes-recursao.pors",
        description: "Demonstra funções auxiliares tipadas com retorno explícito e recursão.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/04-funcoes-recursao.pors"),
    },
    PorsExample {
        id: "pors-05-vetores",
        category: "Portugol-RS (destaques) / Coleções",
        title: "Vetores tipados e tamanho",
        file_name: "05-vetores.pors",
        description: "Demonstra vetor<inteiro>, leitura/escrita indexada, tamanho() e passagem por valor.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/05-vetores.pors"),
    },
    PorsExample {
        id: "pors-06-registros-enums",
        category: "Portugol-RS (destaques) / Tipos compostos",
        title: "Registros, enums e escolha exaustiva",
        file_name: "06-registros-enums.pors",
        description: "Demonstra registro nominal, enum com payload e escolha exaustiva.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/06-registros-enums.pors"),
    },
    PorsExample {
        id: "pors-07-entrada-tipada",
        category: "Portugol-RS (destaques) / Entrada e saída",
        title: "Entrada tipada com leia",
        file_name: "07-entrada-tipada.pors",
        description: "Demonstra leitura tipada de cadeia e inteiro pela entrada padrão.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/07-entrada-tipada.pors"),
    },
    PorsExample {
        id: "pors-08-ui-contador",
        category: "Portugol-RS (destaques) / UI Visual",
        title: "Contador visual declarativo (MVU)",
        file_name: "08-ui-contador.pors",
        description: "Demonstra iniciar, atualizar, visualizar e componentes UI.coluna, UI.texto e UI.botao.",
        is_visual: true,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/08-ui-contador.pors"),
    },
    PorsExample {
        id: "pors-09-opcao-resultado",
        category: "Portugol-RS (destaques) / Tratamento de erros",
        title: "Opcao, Resultado e operador ?",
        file_name: "09-opcao-resultado.pors",
        description: "Demonstra Resultado<inteiro, cadeia>, escolha exaustiva e propagação segura com ?.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/09-opcao-resultado.pors"),
    },
    PorsExample {
        id: "pors-10-vetores-dinamicos",
        category: "Portugol-RS (destaques) / Coleções",
        title: "Vetores dinâmicos e Texto.dividir",
        file_name: "10-vetores-dinamicos.pors",
        description: "Demonstra vetor vazio [], adicionar, remover, obter (Opcao<T>) e Texto.dividir.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/10-vetores-dinamicos.pors"),
    },
    PorsExample {
        id: "pors-11-matematica-texto-tipos",
        category: "Portugol-RS (destaques) / Bibliotecas padrão",
        title: "Matematica, Texto e Tipos com ?",
        file_name: "11-matematica-texto-tipos.pors",
        description: "Combina Matematica.PI, Texto.aparar/preencher_a_esquerda e Tipos.para_inteiro com ?.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/11-matematica-texto-tipos.pors"),
    },
    PorsExample {
        id: "pors-12-util-calendario",
        category: "Portugol-RS (destaques) / Bibliotecas padrão",
        title: "Util e Calendario determinísticos",
        file_name: "12-util-calendario.pors",
        description: "Demonstra Util.definir_semente/sorteia e Calendario.eh_bissexto/dias_no_mes/nome_mes.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/12-util-calendario.pors"),
    },
    PorsExample {
        id: "pors-13-json-dados",
        category: "Portugol-RS (destaques) / Bibliotecas padrão",
        title: "Manipulação imutável de JSON",
        file_name: "13-json-dados.pors",
        description: "Demonstra Json.definir_* / Json.obter_* / Json.obter_lista_inteiro e propagação com ?.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/13-json-dados.pors"),
    },
    PorsExample {
        id: "pors-14-arquivos-notas",
        category: "Portugol-RS (destaques) / Bibliotecas padrão",
        title: "Arquivos confinados ao workspace",
        file_name: "14-arquivos-notas.pors",
        description: "Demonstra Arquivos.escrever_texto, acrescentar_texto, ler_linhas e remover com Resultado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/14-arquivos-notas.pors"),
    },
    PorsExample {
        id: "classic-entrada-saida-ola-mundo",
        category: "Entrada e Saída",
        title: "Olá Mundo",
        file_name: "ola_mundo.pors",
        description: "Este exemplo utiliza a saída de dados do Portugol para exibir a mensagem \"Olá mundo\".",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/entrada_saida/ola_mundo.pors"),
    },
    PorsExample {
        id: "classic-entrada-saida-numero-digitado",
        category: "Entrada e Saída",
        title: "Número Digitado",
        file_name: "numero_digitado.pors",
        description: "Este exemplo utiliza a entrada de dados do Portugol para ler e armazenar um número inteiro em uma variável. Logo após, utiliza a saída de dados para exibir o número digitado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/entrada_saida/numero_digitado.pors"
        ),
    },
    PorsExample {
        id: "classic-entrada-saida-seu-nome",
        category: "Entrada e Saída",
        title: "Seu Nome",
        file_name: "seu_nome.pors",
        description: "Este exemplo utiliza a entrada de dados do Portugol para obter e armazenar o nome do usuário em um variável. Logo após, utiliza a saída de dados para exibir o nome digitado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/entrada_saida/seu_nome.pors"),
    },
    PorsExample {
        id: "classic-operacoes-aritmeticas-operacoes-simples",
        category: "Operações Aritméticas",
        title: "Operações Simples",
        file_name: "operacoes_simples.pors",
        description: "Este exemplo pede ao usuário que informe dois números. Logo após, calcula e exibe: a) A soma entre os números b) A subtração entre os números c) A multiplicação entre os números d) A divisão entre os números",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/operacoes_aritmeticas/operacoes_simples.pors"
        ),
    },
    PorsExample {
        id: "classic-operacoes-aritmeticas-prioridades",
        category: "Operações Aritméticas",
        title: "Prioridades",
        file_name: "prioridades.pors",
        description: "Este exemplo demonstra a prioridade das operações aritméticas no Portugol. As operações de multiplicação (*), divisão (/) e módulo (%) têm prioridade sobre as operações de soma (+) e subtração (-). Além disso, o exemplo demonstra como os parênteses podem ser utilizados para alterar esta prioridade, fazendo com que uma operação de soma ocorra antes de uma operação de multiplicação.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/operacoes_aritmeticas/prioridades.pors"
        ),
    },
    PorsExample {
        id: "classic-operacoes-aritmeticas-divisoes-inteiras",
        category: "Operações Aritméticas",
        title: "Divisões Inteiras",
        file_name: "divisoes_inteiras.pors",
        description: "Este exemplo pede ao usuário que informe um número inteiro. Logo após, calcula e exibe: a) O resultado da divisão inteira por 2 b) O resto da divisão inteira por 3 (mod)",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/operacoes_aritmeticas/divisoes_inteiras.pors"
        ),
    },
    PorsExample {
        id: "classic-operacoes-aritmeticas-potencia-raiz",
        category: "Operações Aritméticas",
        title: "Potencia e Raiz",
        file_name: "potencia_raiz.pors",
        description: "Este exemplo pede ao usuário que informe um valor. Logo após, utiliza a biblioteca \"Matematica\" para calcular e exibir: a) O número elevado ao cubo b) A raiz quadrada do número",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/operacoes_aritmeticas/potencia_raiz.pors"
        ),
    },
    PorsExample {
        id: "classic-algoritmos-sequenciais-troca-variaveis",
        category: "Algoritmos Sequencias",
        title: "Troca Variáveis",
        file_name: "troca_variaveis.pors",
        description: "Este exemplo pede ao usuário que informe dois valores inteiros e os\tarmazena em duas variáveis. Logo após, o programa troca os valores contidos nas variáveis entre si e os exibe ao usuário.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/algoritmos_sequenciais/troca_variaveis.pors"
        ),
    },
    PorsExample {
        id: "classic-algoritmos-sequenciais-maioridade-penal",
        category: "Algoritmos Sequencias",
        title: "Maioridade Penal",
        file_name: "maioridade_penal.pors",
        description: "Este exemplo define qual o valor da maior Idade penal usando uma constante. Logo após, pede ao usuário que informe sua idade e calcula quantos anos faltam para ele atingir a maioridade.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/algoritmos_sequenciais/maioridade_penal.pors"
        ),
    },
    PorsExample {
        id: "classic-algoritmos-sequenciais-altura-media",
        category: "Algoritmos Sequencias",
        title: "Altura média",
        file_name: "altura_media.pors",
        description: "Este exemplo pede ao usuário a altura de 3 pessoas. Logo após, calcula e exibe a média das alturas informadas.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/algoritmos_sequenciais/altura_media.pors"
        ),
    },
    PorsExample {
        id: "classic-algoritmos-sequenciais-loja-ferramentas",
        category: "Algoritmos Sequencias",
        title: "Loja de Ferramentas",
        file_name: "loja_ferramentas.pors",
        description: "Este exemplo pede o nome do usuário e três valores inteiros, os quais representam a quantidade de porcas, parafusos e arruelas compradas. Após, exibe o nome do usuário seguido da quantidade de cada item comprado e o valor total a ser pago.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/algoritmos_sequenciais/loja_ferramentas.pors"
        ),
    },
    PorsExample {
        id: "classic-algoritmos-sequenciais-quantos-dias",
        category: "Algoritmos Sequencias",
        title: "Quantos Dias",
        file_name: "quantos_dias.pors",
        description: "Este exemplo pede ao usuário que informe o ano atual. Logo após, calcula e exibe a quantidade de dias que se passaram desde o dia 01/01/0001 (ano 1 dc)\taté o dia 01/01 do ano atual.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/algoritmos_sequenciais/quantos_dias.pors"
        ),
    },
    PorsExample {
        id: "classic-desvios-condicionais-vogal",
        category: "Desvios Condicionais",
        title: "Vogal",
        file_name: "vogal.pors",
        description: "Este exemplo pede ao usuário que informe uma letra (tipo caracter). Logo após verifica se a letra digitada é uma vogal ou uma consoante e exibe o resultado ao usuário.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/desvios_condicionais/vogal.pors"),
    },
    PorsExample {
        id: "classic-desvios-condicionais-medias",
        category: "Desvios Condicionais",
        title: "Médias",
        file_name: "medias.pors",
        description: "Este exemplo pede ao usuário que informe três médias. Logo após, calcula e exibe a média final destas notas. Por último, verifica se alguma das médias parciais é menor que a média final e a exibe (caso exista).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/desvios_condicionais/medias.pors"),
    },
    PorsExample {
        id: "classic-desvios-condicionais-idade",
        category: "Desvios Condicionais",
        title: "Idade do Usuário",
        file_name: "idade.pors",
        description: "Este exemplo pede ao usuário que informa a sua idade. Logo após exibe uma mensagem informando se ele for maior ou menor de idade.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/desvios_condicionais/idade.pors"),
    },
    PorsExample {
        id: "classic-desvios-condicionais-multiplo",
        category: "Desvios Condicionais",
        title: "Múltiplo de Cinco",
        file_name: "multiplo.pors",
        description: "Este exemplo pede ao usuário que informe um valor inteiro, logo após, exibe uma mensagem indicando se o número informado é múltiplo de 5 ou não.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/desvios_condicionais/multiplo.pors"
        ),
    },
    PorsExample {
        id: "classic-desvios-condicionais-numeros-iguais",
        category: "Desvios Condicionais",
        title: "Números Iguais",
        file_name: "numeros_iguais.pors",
        description: "Este exemplo pede ao usuário que informe um valor de 0 a 6. Logo após, o programa sorteia um valor também de 0 a 6 e exibe uma mensagem informando se o número sorteado e o número digitado são iguais.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/desvios_condicionais/numeros_iguais.pors"
        ),
    },
    PorsExample {
        id: "classic-desvios-condicionais-media-final",
        category: "Desvios Condicionais",
        title: "Média Final",
        file_name: "media_final.pors",
        description: "Este exemplo pede ao usuário que informe seu nome e três notas. Logo após, calcula a média final do usuário e exibe uma mensagem informando se ele foi aprovado ou reprovado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/desvios_condicionais/media_final.pors"
        ),
    },
    PorsExample {
        id: "classic-desvios-condicionais-numero-inteiro",
        category: "Desvios Condicionais",
        title: "Número Inteiro",
        file_name: "numero_inteiro.pors",
        description: "Este exemplo pede ao usuário que informe um número inteiro. Logo após, exibe uma mensagem indicando se o número informado é positivo, negativo ou igual a zero.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/desvios_condicionais/numero_inteiro.pors"
        ),
    },
    PorsExample {
        id: "classic-desvios-condicionais-tipo-triangulo",
        category: "Desvios Condicionais",
        title: "Tipo de Triângulo",
        file_name: "tipo_triangulo.pors",
        description: "Este exemplo pede ao usuário que informe o tamanho dos 3 lados de um triângulo. Logo após, compara os lados do triângulo e exibe ao usuário o tipo de triângulo informado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/desvios_condicionais/tipo_triangulo.pors"
        ),
    },
    PorsExample {
        id: "classic-desvios-condicionais-mini-calculadora",
        category: "Desvios Condicionais",
        title: "Mini Calculadora",
        file_name: "mini_calculadora.pors",
        description: "Este exemplo demonstra o uso do desvio condicional para criar uma mini calculadora. O programa pede ao usuário que informe dois números reais e a operação a ser executada entre estes números (soma, divisão, etc.). Por fim, é exibido o valor resultante da operação entre os dois números.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/desvios_condicionais/mini_calculadora.pors"
        ),
    },
    PorsExample {
        id: "classic-desvios-condicionais-escolha-caso",
        category: "Desvios Condicionais",
        title: "Escolha-Caso",
        file_name: "escolha_caso.pors",
        description: "Este exemplo ilustra o uso da instrução \"escolha\". Para isso, o programa pede ao usuário que escolha uma opção e exibe uma frase correspondente à opção escolhida.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/desvios_condicionais/escolha_caso.pors"
        ),
    },
    PorsExample {
        id: "classic-lacos-repeticao-contagem-regressiva",
        category: "Laços de Repetição",
        title: "Contagem Regressiva",
        file_name: "contagem_regressiva.pors",
        description: "Este exemplo utiliza um laço de repetição e uma variável para exibir uma contagem regressiva na tela.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/lacos_repeticao/contagem_regressiva.pors"
        ),
    },
    PorsExample {
        id: "classic-lacos-repeticao-tabuada",
        category: "Laços de Repetição",
        title: "Tabuada",
        file_name: "tabuada.pors",
        description: "Este exemplo pede ao usuário que informe um número inteiro. Logo após, utiliza um laço de repetição do tipo \"para\", para calcular e exibir a tabuada do número informado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/lacos_repeticao/tabuada.pors"),
    },
    PorsExample {
        id: "classic-lacos-repeticao-consistencia",
        category: "Laços de Repetição",
        title: "Consistência",
        file_name: "consistencia.pors",
        description: "Este exemplo ilustra o uso de um laço de repetição do tipo \"faca-enquanto\" para\tvalidar a entrada dos dados informados pelo usuário.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/lacos_repeticao/consistencia.pors"),
    },
    PorsExample {
        id: "classic-lacos-repeticao-media-dez-numeros",
        category: "Laços de Repetição",
        title: "Média de Dez Números",
        file_name: "media_dez_numeros.pors",
        description: "Este exemplo pede ao usuário que informe 10 números. Logo após, calcula e\texibe a média dos números digitados. O exemplo utiliza um laço de repetição do tipo \"enquanto\" para determinar se todos os 10 valores já foram lidos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/lacos_repeticao/media_dez_numeros.pors"
        ),
    },
    PorsExample {
        id: "classic-lacos-repeticao-soma-um-x",
        category: "Laços de Repetição",
        title: "Soma de Um a X",
        file_name: "soma_um_x.pors",
        description: "Este exemplo pede ao usuário que informe um valor inteiro. Logo após, calcula e\texibe a soma dos números de 1 e até o número digitado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/lacos_repeticao/soma_um_x.pors"),
    },
    PorsExample {
        id: "classic-lacos-repeticao-fatorial",
        category: "Laços de Repetição",
        title: "Fatorial",
        file_name: "fatorial.pors",
        description: "Este exemplo ilustra o uso do laço de repetição \"enquanto\". O exemplo pede ao usuario que informe um número, logo após, calcula e exibe o fatorial do número digitado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/lacos_repeticao/fatorial.pors"),
    },
    PorsExample {
        id: "classic-lacos-repeticao-eleicao",
        category: "Laços de Repetição",
        title: "Eleição",
        file_name: "eleicao.pors",
        description: "Este exemplo ilustra o uso do laço \"faca-enquanto\", simulando uma eleição entre\tdois candidatos. O exemplo ilustra também o uso do comando \"escolha\" para contabilizar os votos de cada candidato.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/lacos_repeticao/eleicao.pors"),
    },
    PorsExample {
        id: "classic-vetores-matrizes-preenche-vetor",
        category: "Vetores e Matrizes",
        title: "Preenche e Exibe Vetor",
        file_name: "preenche_vetor.pors",
        description: "Este exemplo cria e preenche um vetor com números aleatórios. Logo após exibe o vetor no console de duas formas diferentes: a) Na ordem em que os números foram preenchidos b) Na ordem inversa em que os números foram preenchidos",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/vetores_matrizes/preenche_vetor.pors"
        ),
    },
    PorsExample {
        id: "classic-vetores-matrizes-pesquisa-vetor",
        category: "Vetores e Matrizes",
        title: "Pesquisa Vetor",
        file_name: "pesquisa_vetor.pors",
        description: "Este exemplo ilustra o uso dos vetores na linguagem Portugol. Neste exemplo, é criado um vetor com valores pré-definidos. Logo após, o programa pede ao usuário que informe um valor e percorre o vetor procurando pelo valor informado. Por último, o programa exibe uma mensagem informando se o valor informado foi encontrado dentro do vetor ou não.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/vetores_matrizes/pesquisa_vetor.pors"
        ),
    },
    PorsExample {
        id: "classic-vetores-matrizes-tabela-dados",
        category: "Vetores e Matrizes",
        title: "Tabela de Dados",
        file_name: "tabela_dados.pors",
        description: "Este exemplo ilustra o uso dos vetores da linguagem Portugol. Neste exemplo, são criados dois vetores. O primeiro, é do tipo cadeia e armazena os nomes de várias pessoas. O segundo, é do tipo real e armazena as alturas destas pessoas. O programa então percorre cada um dos vetores, montando no console uma tabela dos dados existentes. Cada nome é associado à sua respectiva altura.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/vetores_matrizes/tabela_dados.pors"
        ),
    },
    PorsExample {
        id: "classic-vetores-matrizes-diagonal-principal",
        category: "Vetores e Matrizes",
        title: "Diagonal Principal",
        file_name: "diagonal_principal.pors",
        description: "Este exemplo ilustra o uso das matrizes na linguagem Portugol criando uma matriz e preenchendo sua diagonal principal. As matrizes nada mais são do que vetores de duas dimensões (bidimensionais). Em outras palavras, a matriz é um vetor aonde cada uma de suas posições (colunas) armazena um outro vetor (linhas).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/vetores_matrizes/diagonal_principal.pors"
        ),
    },
    PorsExample {
        id: "classic-vetores-matrizes-exibe-matriz",
        category: "Vetores e Matrizes",
        title: "Exibe Matriz",
        file_name: "exibe_matriz.pors",
        description: "Este exemplo cria e exibe uma matriz para ilustrar o uso das matrizes na linguagem Portugol.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/vetores_matrizes/exibe_matriz.pors"
        ),
    },
    PorsExample {
        id: "classic-subrotinas-procedimento-funcao",
        category: "Subrotinas",
        title: "Procedimento e Função",
        file_name: "procedimento_funcao.pors",
        description: "Este exemplo ilustra o uso das funções da linguagem Portugol. Neste exemplo, foi criado um procedimento que formata uma mensagem qualquer e uma função que realiza um cálculo matemático entre dois números informados. Para saber mais sobre o fatorial, acesse: http://www.infoescola.com/matematica/fatorial/",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/subrotinas/procedimento_funcao.pors"
        ),
    },
    PorsExample {
        id: "classic-subrotinas-lagarta",
        category: "Subrotinas",
        title: "Animação",
        file_name: "lagarta.pors",
        description: "Este exemplo ilustra o uso das funções da linguagem Portugol para criar uma animação no console.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/subrotinas/lagarta.pors"),
    },
    PorsExample {
        id: "classic-subrotinas-parametros-referencia",
        category: "Subrotinas",
        title: "Parâmetros por Referencia",
        file_name: "parametros_referencia.pors",
        description: "Este exemplo ilustra a passagem de parâmetros para uma função. O exemplo demonstra tanto a passagem de parâmetro por valor, quanto a passagem de parâmetro por referência. Quando um parâmetro é passado por valor, o seu valor é copiado para dentro da função. Desta forma, se a função altera o valor do parâmetro, o valor só é alterado dentro da função, mas é mantido o valor original fora dela. Por outro lado, quando um parâmetro é passado por referência, qualquer alteração dentro da função é imediatamente refletida na variável fora da função. Isto porque, na verdade, o que a função recebe não é uma cópia do valor contido na variável, mas sim, uma referência (um atalho) para a variável original. No Portugol, a passagem de parâmetro por referência é representada pelo operador '&'. Caso não compreenda estes conceitos, experimente depurar o programa e visualizar como o valor das variáveis são alterados na árvore estrutural à esquerda. Se ainda assim tiver dificuldades, peça a ajuda de um professor ou alguém experiente em programação.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/subrotinas/parametros_referencia.pors"
        ),
    },
    PorsExample {
        id: "classic-subrotinas-vetores-parametro",
        category: "Subrotinas",
        title: "Vetores por Parâmetros",
        file_name: "vetores_parametro.pors",
        description: "Este exemplo ilustra a passagem de parâmetros para uma função. O exemplo demonstra que vetores não precisam ser passados por referência, pois eles já fazem isso naturalmente.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/subrotinas/vetores_parametro.pors"),
    },
    PorsExample {
        id: "classic-subrotinas-fatorial-recursivo",
        category: "Subrotinas",
        title: "Fatorial Recursivo",
        file_name: "fatorial_recursivo.pors",
        description: "Este exemplo pede ao usuário que informe um número inteiro. Logo após, calcula e exibe o fatorial do número informado. Neste exemplo, o fatorial é calculado chamando recursivamente a função \"fatorial\" definida no programa. O fatorial de um número é calculado multiplicando todos os valores inteiros entre 1 e o próprio número.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/subrotinas/fatorial_recursivo.pors"
        ),
    },
    PorsExample {
        id: "classic-subrotinas-fibonacci-recursivo",
        category: "Subrotinas",
        title: "Fibonacci Recursivo",
        file_name: "fibonacci_recursivo.pors",
        description: "Este exemplo pede ao usuário que informe um número inteiro. Logo após, calcula e exibe todos os números da sequência de Fibonacci até a posição informada pelo usuário. Neste exemplo, os números da sequência de Fibonacci são calculados chamando recursivamente a função \"fibonacci\" definida no programa. A sequência de Fibonacci é uma sequência numérica especial, na qual cada elemento da sequência é calculado somando os dois elementos anteriores.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/subrotinas/fibonacci_recursivo.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-ler-arquivo",
        category: "Bibliotecas / Arquivos",
        title: "Ler arquivo",
        file_name: "ler_arquivo.pors",
        description: "Este exemplo mostra como ler um arquivo",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/ler_arquivo.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-ler-linha",
        category: "Bibliotecas / Arquivos",
        title: "Ler linha",
        file_name: "ler_linha.pors",
        description: "Este exemplo mostra como ler uma linha",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/ler_linha.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-loop-linhas",
        category: "Bibliotecas / Arquivos",
        title: "Ler linhas em loop",
        file_name: "loop_linhas.pors",
        description: "Este exemplo mostra como ler uma linha em loop",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/loop_linhas.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-gravar-arquivo",
        category: "Bibliotecas / Arquivos",
        title: "Gravar arquivo",
        file_name: "gravar_arquivo.pors",
        description: "Este exemplo mostra como gravar uma linha em um arquivo",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/gravar_arquivo.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-ler-selecionado",
        category: "Bibliotecas / Arquivos",
        title: "Selecionar e Ler Arquivo",
        file_name: "ler_selecionado.pors",
        description: "Este exemplo demonstra como abrir uma janela para permitir ao usuário selecionar um arquivo. Após abrir a janela de seleção, o programa abre o arquivo selecionado caso o usuário tenha selecionado um arquivo ou exibe uma mensagem caso o usuário não tenha selecionado nenhum arquivo",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/ler_selecionado.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-selecionar-arquivo",
        category: "Bibliotecas / Arquivos",
        title: "selecionar arquivo",
        file_name: "selecionar_arquivo.pors",
        description: "Exemplo selecionar arquivo portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/selecionar_arquivo.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-substituir-arquivo",
        category: "Bibliotecas / Arquivos",
        title: "substituir arquivo",
        file_name: "substituir_arquivo.pors",
        description: "Exemplo substituir arquivo portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/substituir_arquivo.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-arquivos-verificar",
        category: "Bibliotecas / Arquivos",
        title: "verificar",
        file_name: "verificar.pors",
        description: "Exemplo verificar portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/arquivos/verificar.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-raiz",
        category: "Bibliotecas / Matemática",
        title: "Raiz de Um Número",
        file_name: "raiz.pors",
        description: "Este exemplo demonstra como obter a raiz de um número usando a função \"raiz\" da biblioteca \"Matematica\".",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/matematica/raiz.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-potencia",
        category: "Bibliotecas / Matemática",
        title: "Potência Entre Dois Números",
        file_name: "potencia.pors",
        description: "Este exemplo demonstra como obter a potência de um número utilizando a função \"potencia\" da biblioteca \"Matematica\".",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/potencia.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-logaritmo",
        category: "Bibliotecas / Matemática",
        title: "Logaritmo de Um Número",
        file_name: "logaritmo.pors",
        description: "Este exemplo utiliza a função \"logaritmo\" da biblioteca \"Matematica\" para calcular o logaritmo de um número para uma determinada base. Calcular o logaritmo de um número significa encontrar o expoente ao qual a base foi elevada para chegar ao número.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/logaritmo.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-funcoes-trigonometricas",
        category: "Bibliotecas / Matemática",
        title: "Funções Trigonométricas",
        file_name: "funcoes_trigonometricas.pors",
        description: "Este exemplo demonstra o uso das funções trigonométricas \"seno\", \"cosseno\", e \"tangente\" da biblioteca \"Matematica\".",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/funcoes_trigonometricas.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-maior-numero",
        category: "Bibliotecas / Matemática",
        title: "Maior Número",
        file_name: "maior_numero.pors",
        description: "Este exemplo demonstra como utilizar a função \"maior_numero\" da biblioteca \"Matematica\" para escolher o maior entre dois números.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/maior_numero.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-menor-numero",
        category: "Bibliotecas / Matemática",
        title: "Menor Número",
        file_name: "menor_numero.pors",
        description: "Este exemplo demonstra como utilizar a função \"menor_numero\" da biblioteca \"Matematica\" para escolher o menor entre dois números.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/menor_numero.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-valor-absoluto",
        category: "Bibliotecas / Matemática",
        title: "Valor Absoluto",
        file_name: "valor_absoluto.pors",
        description: "Este exemplo demonstra como utilizar a função \"valor_absoluto\" da biblioteca \"Matematica\" para obter o valor absoluto de um número.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/valor_absoluto.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-arredondamento",
        category: "Bibliotecas / Matemática",
        title: "Arredondamento",
        file_name: "arredondamento.pors",
        description: "Este exemplo demonstra como utilizar a função \"arredondar\" da biblioteca \"Matematica\" para arredondar um número qualquer.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/arredondamento.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-matematica-area-circunferencia",
        category: "Bibliotecas / Matemática",
        title: "Área da Circunferência",
        file_name: "area_circunferencia.pors",
        description: "Este exemplo demonstra o uso da constante \"PI\" da biblioteca \"Matematica\". Neste exemplo, a constante é utilizada para calcular a área de uma circunferência.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/matematica/area_circunferencia.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-tamanho",
        category: "Bibliotecas / Texto",
        title: "Tamanho do Texto",
        file_name: "tamanho.pors",
        description: "Este exemplo demonstra como utilizar a função \"numero_caracteres\" da biblioteca \"Texto\" para descobrir o tamanho de um dado do tipo cadeia.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/texto/tamanho.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-caracteres",
        category: "Bibliotecas / Texto",
        title: "Ler Caracteres",
        file_name: "caracteres.pors",
        description: "Este exemplo demonstra o uso da função \"obter_caracter\" da biblioteca \"Texto\" para obter um caractere específico dentro de um dado do tipo cadeia. Neste exemplo, o usuário deve informar um valor e o programa irá verificar se o valor digitado é um número binário válido.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/texto/caracteres.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-pesquisar",
        category: "Bibliotecas / Texto",
        title: "Pesquisar Texto",
        file_name: "pesquisar.pors",
        description: "Este exemplo demonstra o uso da função \"posicao_texto\" da biblioteca \"Texto\" para descobrir se um dado do tipo cadeia contém uma sequência de caracteres.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/texto/pesquisar.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-subtexto",
        category: "Bibliotecas / Texto",
        title: "Extrair Texto",
        file_name: "subtexto.pors",
        description: "Este exemplo demonstra como utilizar a função \"extrair_subtexto\" da biblioteca \"Texto\" para obter uma parte de um texto armazenado em uma variável do tipo cadeia.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/texto/subtexto.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-substituir",
        category: "Bibliotecas / Texto",
        title: "Substituir Texto",
        file_name: "substituir.pors",
        description: "Este exemplo demonstra o uso da função \"substituir\" da biblioteca \"Texto\" para trocar uma sequência de caracteres por outra em um dado do tipo cadeia.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/texto/substituir.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-caixa-alta",
        category: "Bibliotecas / Texto",
        title: "Caixa Alta",
        file_name: "caixa_alta.pors",
        description: "Este exemplo demonstra como utilizar a função \"caixa_alta\" da biblioteca \"Texto\" para transformar os caracteres de um dado do tipo cadeia em caracteres maiúsculos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/texto/caixa_alta.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-caixa-baixa",
        category: "Bibliotecas / Texto",
        title: "Caixa Baixa",
        file_name: "caixa_baixa.pors",
        description: "Este exemplo demonstra como utilizar a função \"caixa_baixa\" da biblioteca \"Texto\" para transformar os caracteres de um dado do tipo cadeia em caracteres minúsculos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/texto/caixa_baixa.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-texto-preencher",
        category: "Bibliotecas / Texto",
        title: "Preencher Texto",
        file_name: "preencher.pors",
        description: "Este exemplo demonstra o uso da função \"preencher_a_esquerda\" da biblioteca \"Texto\" para inserir uma sequência de caracteres em um dado do tipo cadeia.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/texto/preencher.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-util-aguarde",
        category: "Bibliotecas / Util",
        title: "Aguardar Intervalo",
        file_name: "aguarde.pors",
        description: "Este exemplo ilustra o uso da função \"aguarde\" da biblioteca \"Util\" simulando uma contagem regressiva. O programa utiliza a função aguarde para fazer com que o programa espere 1 segundo antes de escrever o próximo número da contagem regressiva na tela.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/util/aguarde.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-util-sorteio",
        category: "Bibliotecas / Util",
        title: "Sorteio de Um Número",
        file_name: "sorteio.pors",
        description: "Este exemplo pede ao usuário que informe um valor inicial, um valor final e uma quantidade de sorteios a ser executada. Logo após, utiliza a função \"sorteia\" da biblioteca \"Util\" para sortear números entre os dois valores informados pelo usuário.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/util/sorteio.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-util-vetor",
        category: "Bibliotecas / Util",
        title: "Tamanho do Vetor",
        file_name: "vetor.pors",
        description: "Este exemplo demonstra como descobrir dinamicamente o número de elementos contidos em um vetor. Para isto, o programa utiliza a função \"numero_elementos\" da biblioteca \"Matematica\".",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/util/vetor.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-util-matriz",
        category: "Bibliotecas / Util",
        title: "Tamanho da Matriz",
        file_name: "matriz.pors",
        description: "Este exemplo demonstra como descobrir dinamicamente as dimensões de uma matriz, isto é, o número de linhas e colunas que ela possui. Para isto, o programa utiliza as funções \"numero_linhas\" e \"numero_colunas\" da biblioteca \"Util\".",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/util/matriz.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-util-tempo",
        category: "Bibliotecas / Util",
        title: "Controle de Tempo",
        file_name: "tempo.pors",
        description: "Este exemplo ilustra como a função \"tempo_decorrido\" da biblioteca \"Util\" pode ser utilizada para realizar medição e controle de tempo em um algoritmo sem utilizar a função \"aguarde\".",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/util/tempo.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-onda",
        category: "Bibliotecas / Graficos",
        title: "Efeito de Onda",
        file_name: "onda.pors",
        description: "Este exemplo utiliza primitivas gráficas e cálculos de seno e cosseno para simular o movimento de uma onda na tela.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/graficos/onda.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-senoides",
        category: "Bibliotecas / Graficos",
        title: "Curvas de Lissajous",
        file_name: "senoides.pors",
        description: "Este exemplo utiliza as funções de desenho da biblioteca gráfica para criar formas baseadas nas funções de seno e cosseno",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/graficos/senoides.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-solar",
        category: "Bibliotecas / Graficos",
        title: "Solar",
        file_name: "solar.pors",
        description: "Este exemplo utiliza as funções de desenho da biblioteca gráfica para apresentar as órbitas do sistema solar a partir de alguns cálculos com diâmetro e ângulo",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/graficos/solar.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-salvar-imagem",
        category: "Bibliotecas / Graficos",
        title: "Salvar Imagem",
        file_name: "salvar_imagem.pors",
        description: "Este exemplo utiliza as funções de desenho da biblioteca gráfica para salvar uma imagem no computador",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/graficos/salvar_imagem.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-sphere",
        category: "Bibliotecas / Graficos",
        title: "Esfera 3D",
        file_name: "sphere.pors",
        description: "Este exemplo renderiza 3d.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/graficos/sphere.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-fractal-fern",
        category: "Bibliotecas / Graficos",
        title: "Fractal de Fern",
        file_name: "fractal_fern.pors",
        description: "Este exemplo calcula e exibe um fractal de Fern na tela.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/graficos/fractal_fern.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-paint",
        category: "Bibliotecas / Graficos",
        title: "Portugol Paint",
        file_name: "paint.pors",
        description: "Este exemplo utiliza as funções de desenho da biblioteca gráfica para criar um programa de paint simples.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/graficos/paint.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-basico-iniciando",
        category: "Bibliotecas / Graficos / basico",
        title: "iniciando",
        file_name: "iniciando.pors",
        description: "Exemplo iniciando portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/graficos/basico/iniciando.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-qr-code",
        category: "Bibliotecas / Graficos",
        title: "qr code",
        file_name: "qr_code.pors",
        description: "Exemplo qr code portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/graficos/qr_code.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-tela-cheia-teste-tela-cheia",
        category: "Bibliotecas / Graficos / tela cheia",
        title: "teste tela cheia",
        file_name: "teste_tela_cheia.pors",
        description: "Exemplo teste tela cheia portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/graficos/tela_cheia/teste_tela_cheia.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-graficos-varios",
        category: "Bibliotecas / Graficos",
        title: "varios",
        file_name: "varios.pors",
        description: "Exemplo varios portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/graficos/varios.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-calendario-relogio-analogico",
        category: "Bibliotecas / Calendario",
        title: "Relógio Analógico",
        file_name: "relogio_analogico.pors",
        description: "Este é um exemplo de utilização da biblioteca calendário que exibe um relógio analógico.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/calendario/relogio_analogico.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-calendario-relogio-digital",
        category: "Bibliotecas / Calendario",
        title: "Relógio Digital",
        file_name: "relogio_digital.pors",
        description: "Este é um exemplo de utilização da biblioteca calendário que exibe um relógio digital.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/calendario/relogio_digital.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-internet-obtertexto",
        category: "Bibliotecas / Internet",
        title: "Obter html site",
        file_name: "obtertexto.pors",
        description: "Este é um exemplo de utilização da biblioteca internet que baixa o HTML de um site.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/internet/obtertexto.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-internet-robos",
        category: "Bibliotecas / Internet",
        title: "Robô Aleatorio",
        file_name: "robos.pors",
        description: "Este é um exemplo de utilização da biblioteca internet que exibe um robo aleatório sempre que executado.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/internet/robos.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-internet-gerador-palavras",
        category: "Bibliotecas / Internet",
        title: "Frase gerada com imagens aleatórias de letras",
        file_name: "gerador_palavras.pors",
        description: "Este é um exemplo de utilização da biblioteca internet que gera uma imagem composta por imagens aleatórias de cada letra.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/internet/gerador_palavras.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-teclado-ler-tecla",
        category: "Bibliotecas / Teclado",
        title: "ler tecla",
        file_name: "ler_tecla.pors",
        description: "Exemplo ler tecla portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/teclado/ler_tecla.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-teclado-tecla-pressionada",
        category: "Bibliotecas / Teclado",
        title: "tecla pressionada",
        file_name: "tecla_pressionada.pors",
        description: "Exemplo tecla pressionada portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/teclado/tecla_pressionada.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-tipos-caracter",
        category: "Bibliotecas / Tipos",
        title: "caracter",
        file_name: "caracter.pors",
        description: "Exemplo caracter portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/tipos/caracter.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-tipos-inteiro",
        category: "Bibliotecas / Tipos",
        title: "inteiro",
        file_name: "inteiro.pors",
        description: "Exemplo inteiro portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/tipos/inteiro.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-tipos-logico",
        category: "Bibliotecas / Tipos",
        title: "logico",
        file_name: "logico.pors",
        description: "Exemplo logico portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/tipos/logico.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-tipos-real",
        category: "Bibliotecas / Tipos",
        title: "real",
        file_name: "real.pors",
        description: "Exemplo real portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/tipos/real.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-mouse-botao-pressionado",
        category: "Bibliotecas / Mouse",
        title: "botao pressionado",
        file_name: "botao_pressionado.pors",
        description: "Exemplo botao pressionado portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/mouse/botao_pressionado.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-mouse-cursor",
        category: "Bibliotecas / Mouse",
        title: "cursor",
        file_name: "cursor.pors",
        description: "Exemplo cursor portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/mouse/cursor.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-mouse-ler-botao",
        category: "Bibliotecas / Mouse",
        title: "ler botao",
        file_name: "ler_botao.pors",
        description: "Exemplo ler botao portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/mouse/ler_botao.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-mouse-posicao-mouse",
        category: "Bibliotecas / Mouse",
        title: "posicao mouse",
        file_name: "posicao_mouse.pors",
        description: "Exemplo posicao mouse portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/mouse/posicao_mouse.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-sons-interromper",
        category: "Bibliotecas / Sons",
        title: "interromper",
        file_name: "interromper.pors",
        description: "Exemplo interromper portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/bibliotecas/sons/interromper.pors"),
    },
    PorsExample {
        id: "classic-bibliotecas-sons-reproduzir-um",
        category: "Bibliotecas / Sons",
        title: "reproduzir um",
        file_name: "reproduzir_um.pors",
        description: "Exemplo reproduzir um portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/sons/reproduzir_um.pors"
        ),
    },
    PorsExample {
        id: "classic-bibliotecas-sons-reproduzir-varios",
        category: "Bibliotecas / Sons",
        title: "reproduzir varios",
        file_name: "reproduzir_varios.pors",
        description: "Exemplo reproduzir varios portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/bibliotecas/sons/reproduzir_varios.pors"
        ),
    },
    PorsExample {
        id: "classic-musica-bateria",
        category: "Música",
        title: "Bateria",
        file_name: "bateria.pors",
        description: "Este exemplo apresenta o novo motor de áudio do portugol em um jogo de criar batidas com instrumentos diferentes",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/musica/bateria.pors"),
    },
    PorsExample {
        id: "classic-musica-player",
        category: "Música",
        title: "Player de Música",
        file_name: "player.pors",
        description: "Este exemplo apresenta um player de música criado utilizando os recursos de reprodução de sons e música do Portugol Studio. O exemplo também utiliza os recursos da biblioteca gráfica para criar um fundo animado e os botões de controle de reprodução do playr",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/musica/player.pors"),
    },
    PorsExample {
        id: "classic-jogos-colisoes-ex1-colisao-ponto-retangulo",
        category: "Jogos / Detectando Colisões",
        title: "Colisão entre um ponto e um retângulo",
        file_name: "ex1_colisao_ponto_retangulo.pors",
        description: "Este exemplo demonstra como é detectada a colisão entre um ponto e um retângulo.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/colisoes/ex1_colisao_ponto_retangulo.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-colisoes-ex2-colisao-ponto-circulo",
        category: "Jogos / Detectando Colisões",
        title: "Colisão entre um ponto e um círculo",
        file_name: "ex2_colisao_ponto_circulo.pors",
        description: "Este exemplo demonstra como é detectada a colisão entre um ponto e um círculo.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/colisoes/ex2_colisao_ponto_circulo.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-colisoes-ex3-colisao-retangulos",
        category: "Jogos / Detectando Colisões",
        title: "Colisão entre dois retângulos",
        file_name: "ex3_colisao_retangulos.pors",
        description: "Este exemplo demonstra como é detectada a colisão entre dois retângulos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/colisoes/ex3_colisao_retangulos.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-colisoes-ex4-colisao-circulos",
        category: "Jogos / Detectando Colisões",
        title: "Colisão entre dois círculos",
        file_name: "ex4_colisao_circulos.pors",
        description: "Este exemplo demonstra como é detectada a colisão entre dois círculos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/colisoes/ex4_colisao_circulos.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-colisoes-ex5-colisao-circulo-retangulo",
        category: "Jogos / Detectando Colisões",
        title: "Colisão entre um círculo e um retângulo",
        file_name: "ex5_colisao_circulo_retangulo.pors",
        description: "Este exemplo demonstra como é detectada a colisão entre um círculo e um retângulo.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/colisoes/ex5_colisao_circulo_retangulo.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-colisoes-ex6-colisao-todos",
        category: "Jogos / Detectando Colisões",
        title: "Várias colisões",
        file_name: "ex6_colisao_todos.pors",
        description: "Este exemplo demonstra a detecção de vários tipos de colisão ao mesmo tempo.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/colisoes/ex6_colisao_todos.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-moon-lander",
        category: "Jogos",
        title: "Moon Lander",
        file_name: "moon_lander.pors",
        description: "Este exemplo é um Jogo escrito em Portugol no qual o jogador controla uma nave que deve pousar na lua o mais rápido possível. O exemplo demonstra como utilizar algumas\tdas bibliotecas existentes no Portugol. Neste exemplo, também é possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/moon_lander.pors"),
    },
    PorsExample {
        id: "classic-jogos-serpente",
        category: "Jogos",
        title: "Serpente",
        file_name: "serpente.pors",
        description: "Este exemplo é um Jogo da Serpente (Snake) escrito em Portugol. O exemplo demonstra\tcomo utilizar algumas das bibliotecas existentes no Portugol. Neste exemplo, também\té possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/serpente.pors"),
    },
    PorsExample {
        id: "classic-jogos-corrida",
        category: "Jogos",
        title: "Corrida",
        file_name: "corrida.pors",
        description: "Este exemplo é um Jogo de corrida escrito em Portugol (ainda em desenvolvimento). O exemplo demonstra\tcomo utilizar algumas das bibliotecas existentes no Portugol. Neste exemplo, também\té possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/corrida.pors"),
    },
    PorsExample {
        id: "classic-jogos-pong",
        category: "Jogos",
        title: "Pong",
        file_name: "pong.pors",
        description: "Este exemplo é um Jogo de pong escrito em Portugol (ainda em desenvolvimento). O exemplo demonstra\tcomo utilizar algumas das bibliotecas existentes no Portugol. Neste exemplo, também\té possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/pong.pors"),
    },
    PorsExample {
        id: "classic-jogos-slide",
        category: "Jogos",
        title: "Sliding",
        file_name: "slide.pors",
        description: "Este exemplo é um Jogo de quebra-cabeça escrito em Portugol. O exemplo demonstra\tcomo utilizar algumas das bibliotecas existentes no Portugol. Neste exemplo, também\té possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/slide.pors"),
    },
    PorsExample {
        id: "classic-jogos-medieval",
        category: "Jogos",
        title: "Crie personagem Medieval",
        file_name: "medieval.pors",
        description: "Este exemplo é um Jogo",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/medieval.pors"),
    },
    PorsExample {
        id: "classic-jogos-theballsaga-theballsaga",
        category: "Jogos / The Ball Saga",
        title: "The Ball Saga",
        file_name: "theballsaga.pors",
        description: "Este exemplo é um Jogo de Habilidade (Bounce) escrito em Portugol.. O exemplo demonstra como utilizar algumas\tdas bibliotecas existentes no Portugol. Neste exemplo, também é possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/theballsaga/theballsaga.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-theballsaga-theballsagacriador",
        category: "Jogos / The Ball Saga",
        title: "Criador de Fases",
        file_name: "theballsagacriador.pors",
        description: "Este exemplo é um gerador de Fases para o game The Ball Saga, um Jogo de Habilidade (Bounce) escrito em Portugol. O exemplo demonstra\tcomo utilizar algumas das bibliotecas existentes no Portugol. Neste exemplo, também\té possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/theballsaga/theballsagacriador.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-programe-programe",
        category: "Jogos / Programe",
        title: "Programe",
        file_name: "programe.pors",
        description: "Este exemplo é um jogo de programar o caminho de um personagem escrito em Portugol. O exemplo demonstra como utilizar algumas das bibliotecas existentes no Portugol. Neste exemplo, também é possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/programe/programe.pors"),
    },
    PorsExample {
        id: "classic-jogos-programe-criador-de-fases",
        category: "Jogos / Programe",
        title: "Criador de Fases",
        file_name: "criador_de_fases.pors",
        description: "Este exemplo é um gerador de Fases para o game Programe, um Jogo de programar escrito em Portugol. O exemplo demonstra\tcomo utilizar algumas das bibliotecas existentes no Portugol. Neste exemplo, também\té possível ver algumas técnicas utilizadas na criação de jogos.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!(
            "../assets/exemplos-pors/classicos/jogos/programe/criador_de_fases.pors"
        ),
    },
    PorsExample {
        id: "classic-jogos-arkanoid",
        category: "Jogos",
        title: "arkanoid",
        file_name: "arkanoid.pors",
        description: "Exemplo arkanoid portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/arkanoid.pors"),
    },
    PorsExample {
        id: "classic-jogos-slide-slide",
        category: "Jogos / slide",
        title: "slide",
        file_name: "slide.pors",
        description: "Exemplo slide portado para Portugol-RS (.pors).",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/classicos/jogos/slide/slide.pors"),
    },
];

pub fn pors_examples() -> &'static [PorsExample] {
    EXAMPLES
}

pub fn find_pors_example(id: &str) -> Option<&'static PorsExample> {
    EXAMPLES.iter().find(|example| example.id == id)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use portugol_vm::ui::{UiBudget, UiEvent, UiSession};
    use portugol_vm::{BufferIo, Vm, run};

    use super::*;

    #[test]
    fn initial_catalog_preserves_all_migrated_and_native_pors_examples() {
        const MIGRATED_CLASSIC_COUNT: usize = 119;
        let examples = pors_examples();

        let migrated_count = examples
            .iter()
            .filter(|example| example.id.starts_with("classic-"))
            .count();
        assert_eq!(migrated_count, MIGRATED_CLASSIC_COUNT);
        assert_eq!(examples.len(), 14 + MIGRATED_CLASSIC_COUNT);
        assert!(
            examples
                .iter()
                .all(|example| example.file_name.ends_with(".pors"))
        );

        let ids = examples
            .iter()
            .map(|example| example.id)
            .collect::<HashSet<_>>();
        assert_eq!(
            ids.len(),
            examples.len(),
            "IDs do catálogo PORS devem ser únicos"
        );
    }

    #[test]
    fn every_pors_example_parses_formats_compiles_and_executes_with_expected_behavior() {
        let expected_console: &[(&str, &str, &str)] = &[
            (
                "pors-01-ola-variaveis",
                "",
                "Olá, Portugol-RS! Próximo ano: 2027",
            ),
            ("pors-02-condicionais", "", "Aprovado"),
            ("pors-03-lacos", "", "Soma=23"),
            ("pors-04-funcoes-recursao", "", "5! = 120"),
            ("pors-05-vetores", "", "Total=25 Qtd=3"),
            (
                "pors-06-registros-enums",
                "",
                "Estudar PORS: concluída (100%)",
            ),
            (
                "pors-07-entrada-tipada",
                "Ada\n36\n",
                "Olá, Ada! Próxima idade: 37",
            ),
            (
                "pors-09-opcao-resultado",
                "",
                "ok=10 | erro=divisão por zero",
            ),
            (
                "pors-10-vetores-dinamicos",
                "",
                "removida=TESTAR primeira=LER restantes=2",
            ),
            ("pors-11-matematica-texto-tipos", "", "PI=3.14 total=0042"),
            (
                "pors-12-util-calendario",
                "",
                "Fevereiro/2024=29 bissexto=verdadeiro dado_valido=verdadeiro",
            ),
            ("pors-13-json-dados", "", "Portugol-RS soma=60"),
            ("pors-14-arquivos-notas", "", "linhas=2 segunda=Testar Rust"),
        ];

        let classic_smoke: &[(&str, &str, &str)] = &[
            ("classic-entrada-saida-ola-mundo", "", "Olá Mundo"),
            ("classic-operacoes-aritmeticas-prioridades", "", "13.0"),
            ("classic-subrotinas-fatorial-recursivo", "5\n", "120"),
            (
                "classic-subrotinas-fibonacci-recursivo",
                "7\n",
                "0 1 1 2 3 5 8",
            ),
            ("classic-bibliotecas-arquivos-ler-arquivo", "", ""),
            ("classic-bibliotecas-arquivos-gravar-arquivo", "", ""),
            ("classic-bibliotecas-internet-obtertexto", "", ""),
        ];

        let workspace =
            std::env::temp_dir().join(format!("portugol-pors-examples-ws-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&workspace);
        std::fs::create_dir_all(&workspace).unwrap();

        for example in pors_examples() {
            let program = portugol_rs_frontend::lower(example.source)
                .unwrap_or_else(|err| panic!("exemplo {} falhou no lower: {err:?}", example.id));
            assert!(
                portugol_rs_frontend::format(example.source).is_some(),
                "formatador recusou exemplo {}",
                example.id
            );
            let module = portugol_bytecode::compile_pors(&program)
                .unwrap_or_else(|err| panic!("exemplo {} falhou no bytecode: {err:?}", example.id));

            if example.is_visual {
                let mut io = BufferIo::default();
                let mut session = UiSession::start(
                    &module,
                    &mut io,
                    UiBudget::default(),
                    Arc::new(AtomicBool::new(false)),
                )
                .expect("exemplo visual deve iniciar sessão MVU");
                assert!(session.tree().to_json().contains("Contagem: 0"));
                let updated = session
                    .dispatch(UiEvent::Click { id: "inc".into() }, &mut io)
                    .expect("clique no exemplo visual");
                assert!(updated.tree.to_json().contains("Contagem: 1"));
            } else if let Some((_, input, expected)) =
                expected_console.iter().find(|(id, _, _)| *id == example.id)
            {
                let mut io = BufferIo::new(input);
                if example.id == "pors-14-arquivos-notas" {
                    Vm::new(&module, &mut io)
                        .with_pors_workspace_dir(&workspace)
                        .run()
                        .unwrap_or_else(|err| {
                            panic!("exemplo {} falhou na VM: {err:?}", example.id)
                        });
                } else {
                    run(&module, &mut io).unwrap_or_else(|err| {
                        panic!("exemplo {} falhou na VM: {err:?}", example.id)
                    });
                }
                assert_eq!(io.output, *expected, "saída inesperada em {}", example.id);
            } else if let Some((_, input, must_contain)) =
                classic_smoke.iter().find(|(id, _, _)| *id == example.id)
            {
                let mut io = BufferIo::new(input);
                Vm::new(&module, &mut io)
                    .with_pors_workspace_dir(&workspace)
                    .run()
                    .unwrap_or_else(|err| panic!("exemplo {} falhou na VM: {err:?}", example.id));
                if !must_contain.is_empty() {
                    assert!(
                        io.output.contains(must_contain),
                        "saída inesperada em {}: {:?}",
                        example.id,
                        io.output
                    );
                }
            }
        }
        let _ = std::fs::remove_dir_all(&workspace);
    }
}
