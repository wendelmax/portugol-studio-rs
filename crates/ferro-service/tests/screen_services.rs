use std::sync::{Arc, Mutex};

use ferro_service::{Host, ScreenEvent, ScreenRun};

const DESIGN: &str = r#"<Janela Nome="J" Titulo="Notas" Largura="300" Altura="200" Dados="Estado">
    <Botao Nome="btnCopiar" Texto="Copiar" X="8" Y="8" AoClicar="btnCopiar_clicar" />
    <Botao Nome="btnColar" Texto="Colar" X="8" Y="48" AoClicar="btnColar_clicar" />
</Janela>
"#;

const CODE: &str = r#"registro Estado {
    texto: cadeia
}

funcao btnCopiar_clicar(tela: J) -> J {
    AreaTransferencia.copiar("nota 10");
    Notificacao.mostrar("", "Copiado!");
    retorne tela;
}

funcao btnColar_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    escolha AreaTransferencia.colar() {
        caso Opcao::Algum(texto) => { t.dados.texto = texto; }
        caso Opcao::Nenhum => { t.dados.texto = "(vazio)"; }
    }
    retorne t;
}
"#;

#[derive(Default)]
struct System {
    clipboard: Option<String>,
    notifications: Vec<(String, String)>,
}

struct FakeHost(Arc<Mutex<System>>);

impl Host for FakeHost {
    fn clipboard_write(&mut self, text: &str) -> Result<(), String> {
        self.0.lock().unwrap().clipboard = Some(text.to_owned());
        Ok(())
    }
    fn clipboard_read(&mut self) -> Result<Option<String>, String> {
        Ok(self.0.lock().unwrap().clipboard.clone())
    }
    fn notify(&mut self, title: &str, text: &str) -> Result<(), String> {
        self.0
            .lock()
            .unwrap()
            .notifications
            .push((title.to_owned(), text.to_owned()));
        Ok(())
    }
}

fn click(control: &str) -> ScreenEvent {
    ScreenEvent {
        control: control.into(),
        event: "AoClicar".into(),
        value: None,
    }
}

#[test]
fn copiar_colar_e_notificar_passam_pelo_host() {
    let system = Arc::new(Mutex::new(System::default()));
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    let shared = Arc::clone(&system);
    run.set_host(Box::new(move || Box::new(FakeHost(Arc::clone(&shared)))));

    assert_eq!(
        run.dispatch(&click("btnColar")).unwrap().snapshot["dados"]["texto"],
        "(vazio)"
    );
    run.dispatch(&click("btnCopiar")).unwrap();
    assert_eq!(system.lock().unwrap().clipboard.as_deref(), Some("nota 10"));
    assert_eq!(
        system.lock().unwrap().notifications,
        [(String::new(), "Copiado!".to_owned())]
    );
    assert_eq!(
        run.dispatch(&click("btnColar")).unwrap().snapshot["dados"]["texto"],
        "nota 10"
    );
}

#[test]
fn fora_de_uma_tela_os_servicos_explicam_onde_funcionam() {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    let error = run.dispatch(&click("btnCopiar")).unwrap_err();
    assert!(
        error.contains("`AreaTransferencia.copiar` só funciona numa tela desktop"),
        "{error}"
    );
    for (source, expected) in [
        (
            "funcao inicio() {\n    AreaTransferencia.copiar(1);\n}\n",
            "deve ter tipo `cadeia`",
        ),
        (
            "funcao inicio() {\n    Notificacao.mostrar(\"so titulo\");\n}\n",
            "espera 2 argumento(s)",
        ),
    ] {
        let diagnostics = ferro_frontend::lower(source).unwrap_err();
        assert!(
            diagnostics[0].message.contains(expected),
            "{source}: {}",
            diagnostics[0].message
        );
    }
}
