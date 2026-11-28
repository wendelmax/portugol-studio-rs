use ferro_tela::edit::{Edit, edit_source, element_at, next_control_name};
use ferro_tela::{analyze, parse};

const BASE: &str = r#"<!-- Tela de teste -->
<Janela Nome="J" Titulo="T" Largura="300" Altura="200" Formato="1">
    <Botao Nome="btnOk" X="10" Y="10" Texto="OK" />

    <!-- Os campos -->
    <Pilha Nome="pilha" X="10" Y="50" Largura="200" Altura="100">
        <Rotulo Nome="lbl" Texto="Nome" />
        <CaixaTexto Nome="txt" Alinhamento="Esticar" />
    </Pilha>
</Janela>
"#;

fn edit(edits: &[Edit]) -> String {
    let out = edit_source(BASE, edits).unwrap();
    let analysis = analyze(&out);
    assert!(
        analysis.diagnostics.is_empty(),
        "{out}\n{:?}",
        analysis.diagnostics
    );
    out
}

fn names(source: &str, path: &[usize]) -> Vec<String> {
    let document = parse(source).unwrap();
    element_at(&document, path)
        .unwrap()
        .elements()
        .map(|e| {
            e.control_name()
                .map_or(e.name.text.clone(), |n| n.value.clone())
        })
        .collect()
}

#[test]
fn muda_atributo_existente_sem_mexer_no_resto() {
    let out = edit(&[Edit::SetAttribute {
        path: vec![0],
        name: "Texto".into(),
        value: "Salvar & \"sair\"".into(),
    }]);
    assert!(
        out.contains(
            r#"<Botao Nome="btnOk" X="10" Y="10" Texto="Salvar &amp; &quot;sair&quot;" />"#
        ),
        "{out}"
    );
    assert!(out.starts_with("<!-- Tela de teste -->\n"));
    assert!(out.contains("\n\n    <!-- Os campos -->\n"));
}

#[test]
fn acrescenta_e_remove_atributo() {
    let out = edit(&[
        Edit::SetAttribute {
            path: vec![1, 0],
            name: "Dica".into(),
            value: "Seu nome".into(),
        },
        Edit::RemoveAttribute {
            path: vec![0],
            name: "Texto".into(),
        },
    ]);
    assert!(
        out.contains(r#"<Rotulo Nome="lbl" Texto="Nome" Dica="Seu nome" />"#),
        "{out}"
    );
    assert!(
        out.contains(r#"<Botao Nome="btnOk" X="10" Y="10" />"#),
        "{out}"
    );
}

#[test]
fn texto_sem_edicoes_volta_formatado() {
    assert_eq!(edit_source(BASE, &[]).unwrap(), BASE);
}

#[test]
fn insere_controle_na_janela_e_na_pilha() {
    let out = edit(&[
        Edit::Insert {
            parent: vec![],
            index: 2,
            tag: "Rotulo".into(),
            attributes: vec![
                ("Nome".into(), "rotulo1".into()),
                ("X".into(), "20".into()),
                ("Y".into(), "160".into()),
            ],
        },
        Edit::Insert {
            parent: vec![1],
            index: 0,
            tag: "Botao".into(),
            attributes: vec![("Nome".into(), "botao1".into())],
        },
    ]);
    assert_eq!(names(&out, &[]), ["btnOk", "pilha", "rotulo1"]);
    assert_eq!(names(&out, &[1]), ["botao1", "lbl", "txt"]);
}

#[test]
fn insere_item_em_combo_e_recusa_fora_dele() {
    let source = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\">\n    <Combo Nome=\"c\" X=\"0\" Y=\"0\" />\n</Janela>\n";
    let out = edit_source(
        source,
        &[Edit::Insert {
            parent: vec![0],
            index: 0,
            tag: "Item".into(),
            attributes: vec![],
        }],
    )
    .unwrap();
    assert!(
        out.contains("<Combo Nome=\"c\" X=\"0\" Y=\"0\">\n        <Item />\n    </Combo>"),
        "{out}"
    );
    let err = edit_source(
        source,
        &[Edit::Insert {
            parent: vec![],
            index: 0,
            tag: "Item".into(),
            attributes: vec![],
        }],
    )
    .unwrap_err();
    assert!(err.0.contains("<Lista>"), "{err}");
}

#[test]
fn recusa_insercoes_invalidas() {
    for (parent, index, tag) in [
        (vec![0], 0, "Rotulo"),       // Botao não é contêiner
        (vec![], 9, "Rotulo"),        // posição fora
        (vec![5], 0, "Rotulo"),       // pai não existe
        (vec![], 0, "Janela"),        // outra janela
        (vec![], 0, "Botoa"),         // tag desconhecida
        (vec![1], 0, "Temporizador"), // componente fora da janela
    ] {
        let result = edit_source(
            BASE,
            &[Edit::Insert {
                parent: parent.clone(),
                index,
                tag: tag.into(),
                attributes: vec![],
            }],
        );
        assert!(result.is_err(), "{parent:?} {index} {tag}");
    }
}

#[test]
fn remove_elemento_e_nao_remove_a_janela() {
    let out = edit(&[Edit::Remove { path: vec![1, 0] }]);
    assert_eq!(names(&out, &[1]), ["txt"]);
    assert!(edit_source(BASE, &[Edit::Remove { path: vec![] }]).is_err());
    assert!(edit_source(BASE, &[Edit::Remove { path: vec![2] }]).is_err());
}

#[test]
fn reordena_no_mesmo_pai() {
    // Para o fim: índice lido antes da mudança.
    let out = edit(&[Edit::Move {
        path: vec![1, 0],
        parent: vec![1],
        index: 2,
    }]);
    assert_eq!(names(&out, &[1]), ["txt", "lbl"]);
    let out = edit(&[Edit::Move {
        path: vec![1, 1],
        parent: vec![1],
        index: 0,
    }]);
    assert_eq!(names(&out, &[1]), ["txt", "lbl"]);
    // Para a própria posição: nada muda.
    for index in [0, 1] {
        let same = edit_source(
            BASE,
            &[Edit::Move {
                path: vec![1, 0],
                parent: vec![1],
                index,
            }],
        )
        .unwrap();
        assert_eq!(same, BASE);
    }
}

#[test]
fn mover_para_a_pilha_tira_x_e_y() {
    let out = edit(&[Edit::Move {
        path: vec![0],
        parent: vec![1],
        index: 1,
    }]);
    // O botão vinha antes da pilha: o caminho da pilha muda de [1] para [0].
    assert_eq!(names(&out, &[]), ["pilha"]);
    assert_eq!(names(&out, &[0]), ["lbl", "btnOk", "txt"]);
    assert!(
        out.contains(r#"<Botao Nome="btnOk" Texto="OK" />"#),
        "{out}"
    );
}

#[test]
fn mover_para_a_janela_tira_alinhamento_e_aceita_posicao() {
    let out = edit(&[
        Edit::Move {
            path: vec![1, 1],
            parent: vec![],
            index: 2,
        },
        Edit::SetAttribute {
            path: vec![2],
            name: "X".into(),
            value: "30".into(),
        },
        Edit::SetAttribute {
            path: vec![2],
            name: "Y".into(),
            value: "170".into(),
        },
    ]);
    assert_eq!(names(&out, &[]), ["btnOk", "pilha", "txt"]);
    assert!(
        out.contains(r#"<CaixaTexto Nome="txt" X="30" Y="170" />"#),
        "{out}"
    );
}

#[test]
fn nao_move_para_dentro_de_si() {
    let err = edit_source(
        BASE,
        &[Edit::Move {
            path: vec![1],
            parent: vec![1],
            index: 0,
        }],
    )
    .unwrap_err();
    assert!(err.0.contains("dele mesmo"), "{err}");
}

#[test]
fn falha_numa_edicao_desfaz_todas() {
    let result = edit_source(
        BASE,
        &[
            Edit::Remove { path: vec![0] },
            Edit::Remove { path: vec![9] },
        ],
    );
    assert!(result.is_err());
}

#[test]
fn texto_com_erro_de_sintaxe_e_recusado() {
    let err = edit_source("<Janela", &[]).unwrap_err();
    assert!(err.0.starts_with("Corrija o erro no texto"), "{err}");
}

#[test]
fn nomes_novos_no_estilo_windows_forms() {
    let document = parse(BASE).unwrap();
    assert_eq!(next_control_name(&document, "Botao"), "botao1");
    assert_eq!(next_control_name(&document, "CaixaTexto"), "caixaTexto1");
    let source = BASE.replace("btnOk", "botao1");
    let document = parse(&source).unwrap();
    assert_eq!(next_control_name(&document, "Botao"), "botao2");
}
