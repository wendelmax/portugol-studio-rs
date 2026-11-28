use ferro_frontend::{
    ExternalField, ExternalRecord, ExternalType, LowerOptions, lower, lower_with,
};

fn field(name: &str, ty: ExternalType) -> ExternalField {
    ExternalField {
        name: name.into(),
        ty,
    }
}

/// Os tipos que a tela da calculadora do ADR 0018 sintetiza.
fn screen() -> LowerOptions {
    LowerOptions {
        records: vec![
            ExternalRecord {
                name: "CaixaTexto".into(),
                fields: vec![
                    field("texto", ExternalType::String),
                    field("habilitado", ExternalType::Boolean),
                ],
            },
            ExternalRecord {
                name: "Combo".into(),
                fields: vec![
                    field("itens", ExternalType::VectorString),
                    field("selecionado", ExternalType::Integer),
                ],
            },
            ExternalRecord {
                name: "JanelaPrincipal".into(),
                fields: vec![
                    field("titulo", ExternalType::String),
                    field("txtA", ExternalType::Named("CaixaTexto".into())),
                    field("cmb", ExternalType::Named("Combo".into())),
                    field("dados", ExternalType::Named("Conta".into())),
                ],
            },
        ],
        require_entry: false,
    }
}

const CODE_BEHIND: &str = r#"registro Conta {
    mensagem: cadeia
}

funcao btnSomar_clicar(tela: JanelaPrincipal) -> JanelaPrincipal {
    seja mutavel t: JanelaPrincipal = tela;
    escolha Tipos.para_inteiro(t.txtA.texto) {
        caso Resultado::Ok(a) => {
            t.dados.mensagem = "Resultado: " + Tipos.inteiro_para_cadeia(a);
        }
        caso Resultado::Erro(_) => {
            t.dados.mensagem = "Não é um número";
        }
    }
    t.titulo = "Calculadora";
    t.txtA.habilitado = falso;
    t.cmb.selecionado = 0;
    retorne t;
}
"#;

#[test]
fn code_behind_enxerga_a_janela_sintetizada_sem_inicio() {
    let program = lower_with(CODE_BEHIND, &screen()).unwrap_or_else(|d| panic!("{d:?}"));
    assert!(program.body.is_empty());
    assert_eq!(program.functions.len(), 1);
    let names: Vec<_> = program
        .record_types
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert_eq!(names, ["CaixaTexto", "Combo", "JanelaPrincipal", "Conta"]);
    let janela = &program.record_types[2];
    let conta = program
        .record_types
        .iter()
        .find(|r| r.name == "Conta")
        .unwrap();
    let dados = janela.fields.iter().find(|f| f.name == "dados").unwrap();
    assert_eq!(
        dados.ty,
        ferro_hir::Type::Record(conta.id),
        "dados aponta para o registro do aluno"
    );
}

#[test]
fn campo_inexistente_e_tipo_errado_sao_diagnosticos() {
    let wrong_field = CODE_BEHIND.replace("t.txtA.habilitado = falso;", "t.txtA.cor = falso;");
    assert!(lower_with(&wrong_field, &screen()).is_err());
    let wrong_type = CODE_BEHIND.replace("t.cmb.selecionado = 0;", "t.cmb.selecionado = \"zero\";");
    assert!(lower_with(&wrong_type, &screen()).is_err());
}

#[test]
fn registro_de_dados_ausente_explica_de_onde_vem() {
    let without = CODE_BEHIND.replace("registro Conta {\n    mensagem: cadeia\n}\n", "");
    let diagnostics = lower_with(&without, &screen()).unwrap_err();
    let missing = diagnostics
        .iter()
        .find(|d| d.code == "pors.external-type-missing")
        .expect("diagnóstico");
    assert!(missing.message.contains("`Conta`"), "{}", missing.message);
    assert!(
        missing.message.contains("JanelaPrincipal.dados"),
        "{}",
        missing.message
    );
}

#[test]
fn aluno_nao_pode_redeclarar_o_tipo_da_janela() {
    let redeclared = format!("{CODE_BEHIND}\nregistro JanelaPrincipal {{\n    x: inteiro\n}}\n");
    let diagnostics = lower_with(&redeclared, &screen()).unwrap_err();
    assert!(
        diagnostics.iter().any(|d| d.code == "pors.type-redeclared"),
        "{diagnostics:?}"
    );
}

#[test]
fn programa_comum_continua_exigindo_inicio() {
    let diagnostics = lower("funcao f() {\n}\n").unwrap_err();
    assert!(
        diagnostics.iter().any(|d| d.code == "pors.entry-function"),
        "{diagnostics:?}"
    );
    let with_entry = LowerOptions {
        require_entry: true,
        ..screen()
    };
    assert!(lower_with(CODE_BEHIND, &with_entry).is_err());
}
