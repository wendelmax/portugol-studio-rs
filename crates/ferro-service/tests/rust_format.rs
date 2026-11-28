use ferro_service::{format_rust_source, rustfmt_available};

#[test]
fn formats_standalone_rust_source_with_installed_rustfmt() {
    assert!(
        rustfmt_available(),
        "rustfmt deve estar instalado nos testes"
    );

    let formatted =
        format_rust_source("fn main(){let mensagem=\"Olá, 😀\";println!(\"{}\",mensagem);}")
            .expect("rustfmt deve formatar Rust válido");

    assert_eq!(
        formatted,
        "fn main() {\n    let mensagem = \"Olá, 😀\";\n    println!(\"{}\", mensagem);\n}\n"
    );
}

#[test]
fn preserves_unicode_in_a_long_rust_literal_while_formatting_around_it() {
    let literal = "Olá, Rust 🦀 — ".repeat(80);
    let source = format!("fn main(){{let mensagem=\"{literal}\";println!(\"{{}}\",mensagem);}}");

    let formatted = format_rust_source(&source).expect("a linha longa deve ser formatada");

    assert!(formatted.starts_with("fn main() {\n"));
    assert!(formatted.contains("mensagem"));
    assert!(formatted.contains(&literal));
    assert!(formatted.ends_with("\n    println!(\"{}\", mensagem);\n}\n"));
}

#[test]
fn rejects_invalid_rust_instead_of_returning_partial_source() {
    let error = format_rust_source("fn main( {").unwrap_err();

    assert!(error.contains("rustfmt"));
}

#[test]
fn rejects_rust_source_larger_than_one_mib_before_formatting() {
    let source = "x".repeat(1_048_577);

    let error = format_rust_source(&source).unwrap_err();

    assert!(error.contains("1 MiB"));
}
