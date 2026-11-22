use portugol_service::check_rust_source;

#[test]
fn checks_valid_rust_and_keeps_compiler_warnings() {
    let diagnostics = check_rust_source("fn main() { let resultado = 1 + 2; }").unwrap();

    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != "error")
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == "warning")
    );
}

#[test]
fn returns_type_errors_at_utf16_offsets_after_non_bmp_text() {
    let source = "// 🦀\nfn main() { let valor: i32 = \"texto\"; }";

    let diagnostics = check_rust_source(source).unwrap();
    let error = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == "error")
        .expect("rustc deve apontar incompatibilidade de tipo");

    let expected_start = source.find("\"texto\"").unwrap();
    let expected_start_utf16 = source[..expected_start].encode_utf16().count() as u32;
    let expected_end_utf16 = expected_start_utf16 + "\"texto\"".encode_utf16().count() as u32;
    assert_eq!(error.start.offset, expected_start_utf16);
    assert_eq!(error.end.offset, expected_end_utf16);
}

#[test]
fn rejects_source_larger_than_one_mib_before_rustc() {
    let source = " ".repeat(1_048_577);

    assert!(check_rust_source(&source).unwrap_err().contains("1 MiB"));
}
