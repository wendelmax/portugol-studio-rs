use portugol_vm::Value;
use portugol_vm::stdlib::constant;

#[test]
fn constantes_do_teclado_usam_os_codigos_do_keyevent() {
    for (name, expected) in [
        ("TECLA_CAPS_LOCK", 20),
        ("TECLA_3_NUM", 99),
        ("TECLA_NUM_3", 99),
        ("TECLA_F13", 61_440),
        ("TECLA_F24", 61_451),
        ("TECLA_EURO_SIGN", 516),
        ("TECLA_MENU_CONTEXTO", 525),
    ] {
        assert!(matches!(constant("Teclado", name), Some(Value::Int(code)) if code == expected));
    }
}
