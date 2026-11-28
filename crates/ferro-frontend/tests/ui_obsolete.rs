use ferro_frontend::{LowerOptions, lower, lower_with_warnings};

const MVU: &str = r#"funcao iniciar() -> inteiro {
    retorne 0;
}

funcao atualizar(estado: inteiro, tipo: cadeia, alvo: cadeia) -> inteiro {
    retorne estado;
}

funcao visualizar(estado: inteiro) -> inteiro {
    retorne UI.coluna([
        UI.texto("Contagem: ", estado),
        UI.botao("inc", "+1")
    ]);
}
"#;

#[test]
fn ui_ainda_compila_mas_avisa_uma_vez_no_primeiro_uso() {
    assert!(lower(MVU).is_ok(), "programas antigos continuam rodando");
    let (_, warnings) = lower_with_warnings(MVU, &LowerOptions::default()).unwrap();
    assert_eq!(warnings.len(), 1);
    let warning = &warnings[0];
    assert_eq!(warning.code, "ferro.ui-obsoleto");
    assert_eq!(warning.severity, ferro_hir::Severity::Warning);
    assert_eq!(
        &MVU[warning.span.start as usize..warning.span.end as usize],
        "UI.coluna"
    );
    let help = warning.help.as_deref().unwrap();
    assert!(
        help.contains("`UI.botao` → `<Botao>` com `AoClicar`"),
        "{help}"
    );
}

#[test]
fn programa_sem_ui_nao_tem_aviso() {
    let (_, warnings) = lower_with_warnings(
        "funcao inicio() {\n    escreva(1);\n}\n",
        &LowerOptions::default(),
    )
    .unwrap();
    assert!(warnings.is_empty());
}
