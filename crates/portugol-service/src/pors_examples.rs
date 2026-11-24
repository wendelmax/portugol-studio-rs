//! Corpus inicial próprio de exemplos Portugol-RS (`.pors`), sob licença do
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
        category: "Fundamentos",
        title: "Olá e variáveis tipadas",
        file_name: "01-ola-variaveis.pors",
        description: "Demonstra seja, seja mutavel, tipos escalares e escrita com escreva.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/01-ola-variaveis.pors"),
    },
    PorsExample {
        id: "pors-02-condicionais",
        category: "Controle de fluxo",
        title: "Condicionais com se/senao",
        file_name: "02-condicionais.pors",
        description: "Demonstra desvios condicionais com comparações e operador lógico &&.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/02-condicionais.pors"),
    },
    PorsExample {
        id: "pors-03-lacos",
        category: "Controle de fluxo",
        title: "Laço para com pare e continue",
        file_name: "03-lacos.pors",
        description: "Demonstra contagem com para, salto com continue e interrupção com pare.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/03-lacos.pors"),
    },
    PorsExample {
        id: "pors-04-funcoes-recursao",
        category: "Funções",
        title: "Função recursiva (fatorial)",
        file_name: "04-funcoes-recursao.pors",
        description: "Demonstra funções auxiliares tipadas com retorno explícito e recursão.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/04-funcoes-recursao.pors"),
    },
    PorsExample {
        id: "pors-05-vetores",
        category: "Coleções",
        title: "Vetores tipados e tamanho",
        file_name: "05-vetores.pors",
        description: "Demonstra vetor<inteiro>, leitura/escrita indexada, tamanho() e passagem por valor.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/05-vetores.pors"),
    },
    PorsExample {
        id: "pors-06-registros-enums",
        category: "Tipos compostos",
        title: "Registros, enums e escolha exaustiva",
        file_name: "06-registros-enums.pors",
        description: "Demonstra registro nominal, enum com payload e escolha exaustiva.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/06-registros-enums.pors"),
    },
    PorsExample {
        id: "pors-07-entrada-tipada",
        category: "Entrada e saída",
        title: "Entrada tipada com leia",
        file_name: "07-entrada-tipada.pors",
        description: "Demonstra leitura tipada de cadeia e inteiro pela entrada padrão.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/07-entrada-tipada.pors"),
    },
    PorsExample {
        id: "pors-08-ui-contador",
        category: "UI Visual",
        title: "Contador visual declarativo (MVU)",
        file_name: "08-ui-contador.pors",
        description: "Demonstra iniciar, atualizar, visualizar e componentes UI.coluna, UI.texto e UI.botao.",
        is_visual: true,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/08-ui-contador.pors"),
    },
    PorsExample {
        id: "pors-09-opcao-resultado",
        category: "Tratamento de erros",
        title: "Opcao, Resultado e operador ?",
        file_name: "09-opcao-resultado.pors",
        description: "Demonstra Resultado<inteiro, cadeia>, escolha exaustiva e propagação segura com ?.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/09-opcao-resultado.pors"),
    },
    PorsExample {
        id: "pors-10-vetores-dinamicos",
        category: "Coleções",
        title: "Vetores dinâmicos e Texto.dividir",
        file_name: "10-vetores-dinamicos.pors",
        description: "Demonstra vetor vazio [], adicionar, remover, obter (Opcao<T>) e Texto.dividir.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/10-vetores-dinamicos.pors"),
    },
    PorsExample {
        id: "pors-11-matematica-texto-tipos",
        category: "Bibliotecas padrão",
        title: "Matematica, Texto e Tipos com ?",
        file_name: "11-matematica-texto-tipos.pors",
        description: "Combina Matematica.PI, Texto.aparar/preencher_a_esquerda e Tipos.para_inteiro com ?.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/11-matematica-texto-tipos.pors"),
    },
    PorsExample {
        id: "pors-12-util-calendario",
        category: "Bibliotecas padrão",
        title: "Util e Calendario determinísticos",
        file_name: "12-util-calendario.pors",
        description: "Demonstra Util.definir_semente/sorteia e Calendario.eh_bissexto/dias_no_mes/nome_mes.",
        is_visual: false,
        license: "LGPL-3.0-only",
        source: include_str!("../assets/exemplos-pors/12-util-calendario.pors"),
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
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use portugol_vm::ui::{UiBudget, UiEvent, UiSession};
    use portugol_vm::{BufferIo, run};

    use super::*;

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
        ];

        assert_eq!(pors_examples().len(), expected_console.len() + 1);

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
            } else {
                let (_, input, expected) = expected_console
                    .iter()
                    .find(|(id, _, _)| *id == example.id)
                    .unwrap_or_else(|| panic!("expectativa ausente para {}", example.id));
                let mut io = BufferIo::new(input);
                run(&module, &mut io)
                    .unwrap_or_else(|err| panic!("exemplo {} falhou na VM: {err:?}", example.id));
                assert_eq!(io.output, *expected, "saída inesperada em {}", example.id);
            }
        }
    }
}
