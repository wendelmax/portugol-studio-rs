use portugol_vm::Value;
use portugol_vm::stdlib::{constant, native};

#[test]
fn todos_os_recursos_da_ajuda_existem_na_vm() {
    let catalog = include_str!("../../../apps/desktop/src/help-assets/bibliotecas.json");
    let libraries: serde_json::Value = serde_json::from_str(catalog).unwrap();
    for (library, item) in libraries.as_object().unwrap() {
        for resource in item["resources"].as_array().unwrap() {
            let name = resource["name"].as_str().unwrap();
            let signature = resource["signature"].as_str().unwrap();
            if signature.starts_with("funcao ") {
                assert!(native(library, name).is_some(), "{library}.{name}");
            } else {
                assert!(constant(library, name).is_some(), "{library}.{name}");
            }
        }
    }
}

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
