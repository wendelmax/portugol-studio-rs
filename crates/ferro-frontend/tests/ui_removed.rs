//! ADR 0021, Etapa C: `UI.*` saiu da linguagem.

use ferro_frontend::lower;

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
fn ui_e_erro_com_a_equivalencia_das_telas() {
    let errors = lower(MVU).unwrap_err();
    let error = errors
        .iter()
        .find(|d| d.code == "ferro.ui-removido")
        .expect("erro de UI removido");
    assert_eq!(error.severity, ferro_hir::Severity::Error);
    assert_eq!(
        &MVU[error.span.start as usize..error.span.end as usize],
        "UI.coluna"
    );
    let help = error.help.as_deref().unwrap();
    assert!(
        help.contains("`UI.botao` → `<Botao>` com `AoClicar`"),
        "{help}"
    );
}

#[test]
fn iniciar_atualizar_visualizar_nao_substituem_inicio() {
    let source = "funcao iniciar() -> inteiro { retorne 0; }\nfuncao atualizar(estado: inteiro, alvo: cadeia) -> inteiro { retorne estado; }\nfuncao visualizar(estado: inteiro) -> inteiro { retorne estado; }\n";
    let errors = lower(source).unwrap_err();
    assert!(
        errors.iter().any(|d| d.code == "ferro.entry-function"),
        "{errors:?}"
    );
}

#[test]
fn variavel_chamada_ui_continua_valendo() {
    let source = "registro R { coluna: inteiro }\nfuncao inicio() {\n    seja UI: R = R { coluna: 1 };\n    escreva(UI.coluna);\n}\n";
    assert!(lower(source).is_ok());
}
