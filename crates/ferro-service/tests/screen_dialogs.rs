use std::sync::{Arc, Mutex};

use ferro_service::{FileFilter, Host, ScreenEvent, ScreenRun};

const DESIGN: &str = r#"<Janela Nome="J" Titulo="Notas" Largura="300" Altura="200" Dados="Estado">
    <Botao Nome="btnAbrir" Texto="Abrir" X="8" Y="8" AoClicar="btnAbrir_clicar" />
    <Botao Nome="btnApagar" Texto="Apagar" X="8" Y="48" AoClicar="btnApagar_clicar" />
    <DialogoArquivo Nome="dlgAbrir" Titulo="Escolha as notas" Filtro="*.txt;*.csv" />
</Janela>
"#;

const CODE: &str = r#"registro Estado {
    arquivo: cadeia,
    apagado: logico
}

funcao btnAbrir_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    escolha Dialogo.abrir_arquivo(t.dlgAbrir) {
        caso Opcao::Algum(caminho) => {
            t.dados.arquivo = caminho;
            Dialogo.mensagem("Abri " + caminho);
        }
        caso Opcao::Nenhum => {
            t.dados.arquivo = "(cancelado)";
        }
    }
    retorne t;
}

funcao btnApagar_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    t.dados.apagado = Dialogo.confirmar("Apagar tudo?");
    retorne t;
}
"#;

#[derive(Default)]
struct Calls {
    log: Vec<String>,
    pick: Option<String>,
    confirm: bool,
}

struct FakeHost(Arc<Mutex<Calls>>);

impl Host for FakeHost {
    fn show_message(&mut self, text: &str) -> Result<(), String> {
        self.0.lock().unwrap().log.push(format!("mensagem: {text}"));
        Ok(())
    }
    fn confirm(&mut self, text: &str) -> Result<bool, String> {
        let mut calls = self.0.lock().unwrap();
        calls.log.push(format!("confirmar: {text}"));
        Ok(calls.confirm)
    }
    fn file_dialog(
        &mut self,
        save: bool,
        title: &str,
        filters: &[FileFilter],
    ) -> Result<Option<String>, String> {
        let mut calls = self.0.lock().unwrap();
        let extensions: Vec<_> = filters.iter().flat_map(|f| f.extensions.clone()).collect();
        calls
            .log
            .push(format!("arquivo(salvar={save}): {title} {extensions:?}"));
        Ok(calls.pick.clone())
    }
}

fn click(control: &str) -> ScreenEvent {
    ScreenEvent {
        control: control.into(),
        event: "AoClicar".into(),
        value: None,
    }
}

fn run_with(calls: &Arc<Mutex<Calls>>) -> ScreenRun {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    let shared = Arc::clone(calls);
    run.set_host(Box::new(move || Box::new(FakeHost(Arc::clone(&shared)))));
    run
}

#[test]
fn abrir_arquivo_usa_titulo_e_filtro_do_componente_e_mensagem_vai_ao_host() {
    let calls = Arc::new(Mutex::new(Calls {
        pick: Some("/aulas/notas.txt".into()),
        ..Calls::default()
    }));
    let mut run = run_with(&calls);
    let step = run.dispatch(&click("btnAbrir")).unwrap();
    assert_eq!(step.snapshot["dados"]["arquivo"], "/aulas/notas.txt");
    assert_eq!(
        calls.lock().unwrap().log,
        [
            "arquivo(salvar=false): Escolha as notas [\"txt\", \"csv\"]",
            "mensagem: Abri /aulas/notas.txt",
        ]
    );
}

#[test]
fn cancelar_devolve_nenhum_e_confirmar_devolve_a_resposta() {
    let calls = Arc::new(Mutex::new(Calls {
        confirm: true,
        ..Calls::default()
    }));
    let mut run = run_with(&calls);
    assert_eq!(
        run.dispatch(&click("btnAbrir")).unwrap().snapshot["dados"]["arquivo"],
        "(cancelado)"
    );
    assert_eq!(
        run.dispatch(&click("btnApagar")).unwrap().snapshot["dados"]["apagado"],
        true
    );
    assert_eq!(
        calls.lock().unwrap().log.last().unwrap(),
        "confirmar: Apagar tudo?"
    );
}

#[test]
fn sem_host_o_dialogo_explica_onde_funciona() {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    let error = run.dispatch(&click("btnApagar")).unwrap_err();
    assert!(
        error.contains("`Dialogo.confirmar` só funciona numa tela desktop"),
        "{error}"
    );
}

#[test]
fn programa_de_console_com_dialogo_da_erro_claro() {
    let source = "funcao inicio() {\n    Dialogo.mensagem(\"oi\");\n}\n";
    let program = ferro_frontend::lower(source).unwrap();
    let module = ferro_bytecode::compile_ferro(&program).unwrap();
    let mut io = ferro_vm::BufferIo::default();
    let error = ferro_vm::run(&module, &mut io).unwrap_err();
    assert!(
        error
            .message
            .contains("num programa de console, use `escreva` e `leia`"),
        "{}",
        error.message
    );
}

#[test]
fn erros_de_tipo_no_dialogo() {
    for (code, expected) in [
        (
            "funcao inicio() {\n    Dialogo.mensagem(1);\n}\n",
            "deve ter tipo `cadeia`",
        ),
        (
            "funcao inicio() {\n    seja x: logico = Dialogo.confirmar();\n}\n",
            "espera 1 argumento",
        ),
        (
            "funcao inicio() {\n    Dialogo.abrir_arquivo(\"x\");\n}\n",
            "espera um `<DialogoArquivo>`",
        ),
        (
            "funcao inicio() {\n    Dialogo.piscar();\n}\n",
            "Use `mensagem`, `confirmar`",
        ),
    ] {
        let diagnostics = ferro_frontend::lower(code).unwrap_err();
        assert!(
            diagnostics[0].message.contains(expected),
            "{code}: {}",
            diagnostics[0].message
        );
    }
}
