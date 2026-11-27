#[cfg(test)]
mod tests {
    use super::{
        WorkspaceEntry, WorkspaceSession, cargo_project_for_source, copy_selected_files,
        copy_selected_files_with_hook, create_source_file, delete_relative_file, list_entries,
        read_relative_file, rename_relative_file, restore_saved_workspace, save_last_workspace,
        workspace_preference_key,
    };
    #[cfg(unix)]
    use super::{copy_selected_files_from_dir_with_hook, open_selected_file_with_hook};
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "portugol-workspace-test-{}-{}",
                std::process::id(),
                NEXT_DIR.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn workspace_preference_key_is_stable_and_unique_per_canonical_root() {
        let first = TempDir::new();
        let second = TempDir::new();

        let key = workspace_preference_key(&first.0);

        assert_eq!(key, workspace_preference_key(&first.0));
        assert_ne!(key, workspace_preference_key(&second.0));
        assert!(key.starts_with("sha256-"));
        assert_eq!(key.len(), "sha256-".len() + 64);
    }

    fn file_paths(entries: &[WorkspaceEntry], output: &mut Vec<String>) {
        for entry in entries {
            match entry {
                WorkspaceEntry::Folder { path, entries, .. } => {
                    assert!(!Path::new(path).is_absolute());
                    file_paths(entries, output);
                }
                WorkspaceEntry::File { path, .. } => output.push(path.clone()),
            }
        }
    }

    #[test]
    fn lists_supported_files_recursively_and_ignores_other_extensions() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aula/subpasta")).unwrap();
        fs::write(temp.0.join("inicio.FER"), "funcao inicio() {}").unwrap();
        fs::write(temp.0.join("aula/exercicio.licao"), "{}").unwrap();
        fs::write(temp.0.join("aula/subpasta/rust.fer"), "funcao").unwrap();
        fs::write(temp.0.join("aula/subpasta/main.rs"), "fn main() {}").unwrap();
        fs::write(temp.0.join("legado.por"), "programa {}").unwrap();
        fs::write(temp.0.join("legado.pex"), "{}").unwrap();
        fs::write(temp.0.join("notas.txt"), "ignorar").unwrap();

        let entries = list_entries(&temp.0).unwrap();
        let mut paths = Vec::new();
        file_paths(&entries, &mut paths);
        paths.sort();

        assert_eq!(
            paths,
            [
                "aula/exercicio.licao",
                "aula/subpasta/main.rs",
                "aula/subpasta/rust.fer",
                "inicio.FER"
            ]
        );
    }

    #[test]
    fn creates_a_desktop_project_folder_without_overwriting() {
        let temp = TempDir::new();
        let folder = super::create_desktop_project_in(&temp.0, "  Meu App  ").unwrap();
        assert!(folder.ends_with("Meu App"));
        assert!(folder.join("meu-app.ferproj").is_file());
        assert!(folder.join("JanelaPrincipal.design").is_file());
        assert!(folder.join("recursos").is_dir());
        let entries = list_entries(&folder).unwrap();
        let mut paths = Vec::new();
        file_paths(&entries, &mut paths);
        paths.sort();
        assert_eq!(paths, ["JanelaPrincipal.design", "meu-app.ferproj"]);

        let again = super::create_desktop_project_in(&temp.0, "Meu App").unwrap_err();
        assert!(again.contains("Já existe uma pasta"), "{again}");

        for name in ["", "   ", "a/b", "a\\b", "nome?", ".oculto", "fim.", "!!!"] {
            assert!(
                super::create_desktop_project_in(&temp.0, name).is_err(),
                "{name:?}"
            );
        }
        assert!(!temp.0.join("a").exists());
    }

    #[test]
    fn handles_desktop_project_markup_like_other_sources() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("App/recursos")).unwrap();
        fs::write(temp.0.join("App/meu-app.ferproj"), "<Projeto />").unwrap();
        fs::write(temp.0.join("App/JanelaPrincipal.design"), "<Janela />").unwrap();
        fs::write(temp.0.join("App/recursos/logo.png"), "png").unwrap();

        let entries = list_entries(&temp.0).unwrap();
        let mut paths = Vec::new();
        file_paths(&entries, &mut paths);
        paths.sort();
        assert_eq!(paths, ["App/JanelaPrincipal.design", "App/meu-app.ferproj"]);

        let design = Path::new("App/JanelaPrincipal.design");
        assert_eq!(read_relative_file(&temp.0, design).unwrap(), "<Janela />");
        super::write_relative_file(&temp.0, design, "<Janela Nome=\"J\" />").unwrap();
        assert_eq!(
            fs::read_to_string(temp.0.join(design)).unwrap(),
            "<Janela Nome=\"J\" />"
        );

        // Renomear mantém a extensão; trocar .design por .fer é recusado.
        assert!(rename_relative_file(&temp.0, design, "Outra.fer").is_err());
        let renamed = rename_relative_file(&temp.0, design, "Outra.design").unwrap();
        assert!(renamed.ends_with("App/Outra.design"));

        let manifest = Path::new("App/meu-app.ferproj");
        delete_relative_file(&temp.0, manifest).unwrap();
        assert!(!temp.0.join(manifest).exists());

        // Criar pelo explorador continua restrito a .fer e .rs.
        assert!(create_source_file(&temp.0, "Nova.design").is_err());
    }

    #[test]
    fn reads_a_supported_file_relative_to_the_workspace_root() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aula")).unwrap();
        fs::write(temp.0.join("aula/programa.fer"), "escreva(42);").unwrap();

        let content = read_relative_file(&temp.0, Path::new("aula/programa.fer")).unwrap();

        assert_eq!(content, "escreva(42);");
    }

    #[test]
    fn finds_the_nearest_cargo_project_for_a_workspace_rust_file() {
        let temp = TempDir::new();
        let project = temp.0.join("aula");
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(project.join("Cargo.toml"), "[package]").unwrap();
        fs::write(project.join("src/main.rs"), "fn main() {}").unwrap();
        let session = WorkspaceSession::default();
        let id = session.register(temp.0.clone()).unwrap();

        assert_eq!(
            cargo_project_for_source(&session, &id, "aula/src/main.rs").unwrap(),
            Some(project.canonicalize().unwrap())
        );
        assert!(
            cargo_project_for_source(&session, &id, "aula/src/main.rs/../../escape.rs").is_err()
        );
    }

    #[test]
    fn standalone_workspace_rust_file_has_no_cargo_project() {
        let temp = TempDir::new();
        fs::write(temp.0.join("main.rs"), "fn main() {}").unwrap();
        let session = WorkspaceSession::default();
        let id = session.register(temp.0.clone()).unwrap();

        assert_eq!(
            cargo_project_for_source(&session, &id, "main.rs").unwrap(),
            None
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_a_cargo_manifest_symlinked_outside_the_workspace() {
        use std::os::unix::fs::symlink;

        let workspace = TempDir::new();
        let external = TempDir::new();
        fs::write(external.0.join("Cargo.toml"), "[package]").unwrap();
        fs::write(workspace.0.join("main.rs"), "fn main() {}").unwrap();
        symlink(
            external.0.join("Cargo.toml"),
            workspace.0.join("Cargo.toml"),
        )
        .unwrap();
        let session = WorkspaceSession::default();
        let id = session.register(workspace.0.clone()).unwrap();

        assert!(cargo_project_for_source(&session, &id, "main.rs").is_err());
    }

    #[test]
    fn exports_selected_supported_files_and_preserves_relative_paths() {
        let source = TempDir::new();
        let destination = TempDir::new();
        let root = source.0.join("Minha Aula");
        fs::create_dir_all(root.join("Aulas/Nível 1")).unwrap();
        fs::write(root.join("Aulas/Nível 1/inicio.fer"), "escreva(42);").unwrap();
        fs::write(root.join("Aulas/atividade.licao"), "{\"id\":\"1\"}").unwrap();
        fs::write(root.join("ponte.fer"), "funcao inicio() {}").unwrap();
        fs::write(root.join("ignorar.rs"), "fn main() {} ").unwrap();
        fs::write(root.join("notas.txt"), "não exportar").unwrap();

        let result = copy_selected_files(
            &root,
            &destination.0,
            &[
                "Aulas/Nível 1/inicio.fer".into(),
                "Aulas/atividade.licao".into(),
                "ponte.fer".into(),
                "ponte.fer".into(),
            ],
        )
        .unwrap();

        let exported = destination.0.join("Minha Aula");
        assert_eq!(result.project_name, "Minha Aula");
        assert_eq!(result.file_count, 3);
        assert_eq!(
            fs::read_to_string(exported.join("Aulas/Nível 1/inicio.fer")).unwrap(),
            "escreva(42);"
        );
        assert_eq!(
            fs::read_to_string(exported.join("Aulas/atividade.licao")).unwrap(),
            "{\"id\":\"1\"}"
        );
        assert_eq!(
            fs::read_to_string(exported.join("ponte.fer")).unwrap(),
            "funcao inicio() {}"
        );
        assert!(!exported.join("ignorar.rs").exists());
        assert!(root.join("notas.txt").exists());
    }

    #[test]
    fn removes_partial_export_when_copy_fails_midway() {
        let source = TempDir::new();
        let destination = TempDir::new();
        let root = source.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.fer"), "copiado antes da falha").unwrap();
        fs::write(root.join("b.fer"), "falha aqui").unwrap();

        let result = copy_selected_files_with_hook(
            &root,
            &destination.0,
            &["a.fer".into(), "b.fer".into()],
            |relative| {
                if relative == "b.fer" {
                    Err("falha de leitura injetada".into())
                } else {
                    Ok(())
                }
            },
        );

        assert!(result.is_err());
        assert!(
            !destination.0.join("projeto").exists(),
            "partial export was not removed: {result:?}"
        );
    }

    #[test]
    fn export_does_not_truncate_a_conflicting_destination_file() {
        let source = TempDir::new();
        let destination = TempDir::new();
        let root = source.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("programa.fer"), "origem").unwrap();
        let conflict = destination.0.join("projeto/programa.fer");

        let result =
            copy_selected_files_with_hook(&root, &destination.0, &["programa.fer".into()], |_| {
                fs::create_dir_all(conflict.parent().unwrap()).unwrap();
                fs::write(&conflict, "arquivo preexistente").unwrap();
                Ok(())
            });

        let error = result.unwrap_err();
        assert!(error.contains("sem sobrescrever"));
        assert!(error.contains("limpeza da cópia parcial também falhou"));
        assert_eq!(
            fs::read_to_string(&conflict).unwrap(),
            "arquivo preexistente"
        );
    }

    #[cfg(unix)]
    #[test]
    fn export_uses_the_workspace_directory_handle_registered_at_open_time() {
        use std::os::unix::fs::symlink;

        let source = TempDir::new();
        let destination = TempDir::new();
        let root = source.0.join("projeto");
        let moved = source.0.join("projeto-original");
        let replacement = source.0.join("substituto");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&replacement).unwrap();
        fs::write(root.join("programa.fer"), "conteúdo original").unwrap();
        fs::write(replacement.join("programa.fer"), "conteúdo externo").unwrap();

        let session = WorkspaceSession::default();
        let workspace_id = session.register(root.clone()).unwrap();
        fs::rename(&root, &moved).unwrap();
        symlink(&replacement, &root).unwrap();
        let registered_dir = session.directory(&workspace_id).unwrap();

        assert!(
            copy_selected_files_from_dir_with_hook(
                &registered_dir,
                &root,
                "projeto",
                &destination.0,
                &["programa.fer".into()],
                |_| Ok(()),
            )
            .unwrap_err()
            .contains("mudou")
        );
        copy_selected_files_from_dir_with_hook(
            &registered_dir,
            &moved,
            "projeto",
            &destination.0,
            &["programa.fer".into()],
            |_| Ok(()),
        )
        .unwrap();

        assert_eq!(
            fs::read_to_string(destination.0.join("projeto/programa.fer")).unwrap(),
            "conteúdo original"
        );
        assert_eq!(
            fs::read_to_string(replacement.join("programa.fer")).unwrap(),
            "conteúdo externo"
        );
    }

    #[cfg(unix)]
    #[test]
    fn rollback_preserves_a_replacement_folder_and_cleans_the_created_folder_handle() {
        let source = TempDir::new();
        let destination = TempDir::new();
        let root = source.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.fer"), "primeiro").unwrap();
        fs::write(root.join("b.fer"), "segundo").unwrap();
        let moved = destination.0.join("pasta-criada-movida");

        let result = copy_selected_files_with_hook(
            &root,
            &destination.0,
            &["a.fer".into(), "b.fer".into()],
            |relative| {
                if relative == "b.fer" {
                    fs::rename(destination.0.join("projeto"), &moved).unwrap();
                    fs::create_dir(destination.0.join("projeto")).unwrap();
                    fs::write(destination.0.join("projeto/nao-apagar.txt"), "substituta").unwrap();
                    Err("falha de cópia injetada".into())
                } else {
                    Ok(())
                }
            },
        );

        assert!(result.unwrap_err().contains("não corresponde mais"));
        assert_eq!(
            fs::read_to_string(destination.0.join("projeto/nao-apagar.txt")).unwrap(),
            "substituta"
        );
        assert_eq!(fs::read_dir(&moved).unwrap().count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn export_rejects_a_file_replaced_by_a_symlink_after_validation() {
        use std::os::unix::fs::symlink;

        let source = TempDir::new();
        let outside = TempDir::new();
        let root = source.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("programa.fer"), "original").unwrap();
        fs::write(outside.0.join("segredo.fer"), "segredo").unwrap();
        let directory =
            cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority()).unwrap();

        fs::write(root.join("outro.fer"), "outro conteúdo").unwrap();
        let result = open_selected_file_with_hook(&directory, &root, "programa.fer", |_| {
            fs::remove_file(root.join("programa.fer")).unwrap();
            symlink(root.join("outro.fer"), root.join("programa.fer")).unwrap();
            Ok(())
        });

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(root.join("outro.fer")).unwrap(),
            "outro conteúdo"
        );
        assert_eq!(
            fs::read_to_string(outside.0.join("segredo.fer")).unwrap(),
            "segredo"
        );
    }

    #[test]
    fn export_rejects_empty_or_unsafe_selections_without_creating_output() {
        let source = TempDir::new();
        let destination = TempDir::new();
        let root = source.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("programa.fer"), "origem").unwrap();
        fs::write(root.join("legado.por"), "programa {}").unwrap();

        assert!(copy_selected_files(&root, &destination.0, &[]).is_err());
        for invalid in [
            "../fora.fer",
            "/tmp/fora.fer",
            "notas.txt",
            "missing.fer",
            "legado.por",
        ] {
            assert!(
                copy_selected_files(&root, &destination.0, &[invalid.into()]).is_err(),
                "{invalid}"
            );
            assert!(!destination.0.join("projeto").exists());
        }
    }

    #[test]
    fn export_refuses_symlinks_destinations_inside_workspace_and_collisions() {
        let source = TempDir::new();
        let destination = TempDir::new();
        let root = source.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("programa.fer"), "origem").unwrap();
        fs::create_dir_all(destination.0.join("projeto")).unwrap();
        assert!(copy_selected_files(&root, &destination.0, &["programa.fer".into()]).is_err());
        assert!(copy_selected_files(&root, &root, &["programa.fer".into()]).is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let outside = TempDir::new();
            let symlink_destination = TempDir::new();
            fs::write(outside.0.join("segredo.fer"), "segredo").unwrap();
            symlink(outside.0.join("segredo.fer"), root.join("link.fer")).unwrap();
            assert!(
                copy_selected_files(&root, &symlink_destination.0, &["link.fer".into()]).is_err()
            );
            assert!(!symlink_destination.0.join("projeto").exists());
            assert_eq!(
                fs::read_to_string(outside.0.join("segredo.fer")).unwrap(),
                "segredo"
            );
        }
    }

    #[test]
    fn rejects_path_traversal_and_unsupported_files() {
        let temp = TempDir::new();
        let root = temp.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(temp.0.join("fora.fer"), "segredo").unwrap();
        fs::write(temp.0.join("notas.txt"), "não suportado").unwrap();
        fs::write(temp.0.join("legado.por"), "programa {}").unwrap();
        fs::write(temp.0.join("legado.pex"), "{}").unwrap();

        assert!(read_relative_file(&root, Path::new("../fora.fer")).is_err());
        assert!(read_relative_file(&temp.0, Path::new("notas.txt")).is_err());
        assert!(
            read_relative_file(&temp.0, Path::new("legado.por"))
                .unwrap_err()
                .contains("Portugol Studio original")
        );
        assert!(
            read_relative_file(&temp.0, Path::new("legado.pex"))
                .unwrap_err()
                .contains("Portugol Studio original")
        );
    }

    #[test]
    fn saves_existing_supported_program_sources() {
        let temp = TempDir::new();
        fs::write(temp.0.join("programa.fer"), "antes").unwrap();
        fs::write(temp.0.join("main.rs"), "fn main() {} ").unwrap();
        fs::write(temp.0.join("atividade.licao"), "exercício").unwrap();

        super::write_relative_file(&temp.0, Path::new("programa.fer"), "depois").unwrap();

        assert_eq!(
            fs::read_to_string(temp.0.join("programa.fer")).unwrap(),
            "depois"
        );
        super::write_relative_file(
            &temp.0,
            Path::new("main.rs"),
            "fn main() { println!(\"ok\"); }",
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(temp.0.join("main.rs")).unwrap(),
            "fn main() { println!(\"ok\"); }"
        );
        assert!(
            super::write_relative_file(&temp.0, Path::new("atividade.licao"), "alterado").is_err()
        );
        assert_eq!(
            fs::read_to_string(temp.0.join("atividade.licao")).unwrap(),
            "exercício"
        );
    }

    #[test]
    fn creates_empty_portugol_rs_and_rust_files_exclusively() {
        let temp = TempDir::new();

        create_source_file(&temp.0, "ponte.fer").unwrap();
        create_source_file(&temp.0, "main.rs").unwrap();
        assert!(create_source_file(&temp.0, "ola.por").is_err());

        assert_eq!(fs::read_to_string(temp.0.join("ponte.fer")).unwrap(), "");
        assert_eq!(fs::read_to_string(temp.0.join("main.rs")).unwrap(), "");
    }

    #[test]
    fn rejects_invalid_names_and_never_overwrites_an_existing_file() {
        let temp = TempDir::new();
        fs::write(temp.0.join("aula.fer"), "conteúdo existente").unwrap();

        for invalid in [
            "",
            ".",
            "..",
            "../fora.fer",
            "subpasta\\programa.fer",
            "dados.pex",
            "legado.por",
        ] {
            assert!(create_source_file(&temp.0, invalid).is_err(), "{invalid}");
        }
        assert!(create_source_file(&temp.0, "aula.fer").is_err());
        assert_eq!(
            fs::read_to_string(temp.0.join("aula.fer")).unwrap(),
            "conteúdo existente"
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_replace_a_symlink_when_creating_a_source_file() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("segredo"), "não alterar").unwrap();
        symlink(outside.0.join("segredo"), temp.0.join("novo.fer")).unwrap();

        assert!(create_source_file(&temp.0, "novo.fer").is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("segredo")).unwrap(),
            "não alterar"
        );
    }

    #[test]
    fn renames_a_source_file_without_changing_its_contents() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aulas")).unwrap();
        fs::write(temp.0.join("aulas/antigo.fer"), "escreva(42);").unwrap();

        let renamed =
            rename_relative_file(&temp.0, Path::new("aulas/antigo.fer"), "novo.fer").unwrap();

        let expected = fs::canonicalize(temp.0.join("aulas/novo.fer")).unwrap();
        assert_eq!(renamed, expected);
        assert_eq!(fs::read_to_string(&renamed).unwrap(), "escreva(42);");
        assert!(!temp.0.join("aulas/antigo.fer").exists());

        fs::write(temp.0.join("aulas/main.rs"), "fn main() {}").unwrap();
        let renamed_rust =
            rename_relative_file(&temp.0, Path::new("aulas/main.rs"), "renomeado.rs").unwrap();
        assert_eq!(fs::read_to_string(renamed_rust).unwrap(), "fn main() {}");

        fs::write(temp.0.join("aulas/ex1.licao"), "{}").unwrap();
        let renamed_ex =
            rename_relative_file(&temp.0, Path::new("aulas/ex1.licao"), "ex2.licao").unwrap();
        assert_eq!(fs::read_to_string(renamed_ex).unwrap(), "{}");

        assert!(create_source_file(&temp.0, "novo.fer").is_ok());
        assert!(temp.0.join("novo.fer").is_file());
        fs::write(temp.0.join("aulas/legado.fer"), "escreva(7);").unwrap();
        let migrated_ok =
            rename_relative_file(&temp.0, Path::new("aulas/legado.fer"), "legado.fer").unwrap();
        assert_eq!(fs::read_to_string(migrated_ok).unwrap(), "escreva(7);");
    }

    #[test]
    fn preserves_original_extension_and_rejects_path_changes() {
        let temp = TempDir::new();
        fs::write(temp.0.join("programa.fer"), "funcao inicio() {}").unwrap();

        for invalid in [
            "novo.rs",
            "novo.por",
            "../fora.fer",
            "pasta\\novo.fer",
            "anotacao.txt",
        ] {
            assert!(
                rename_relative_file(&temp.0, Path::new("programa.fer"), invalid).is_err(),
                "{invalid}"
            );
        }
        assert_eq!(
            fs::read_to_string(temp.0.join("programa.fer")).unwrap(),
            "funcao inicio() {}"
        );
    }

    #[test]
    fn rename_collision_never_overwrites_either_file() {
        let temp = TempDir::new();
        fs::write(temp.0.join("antes.fer"), "origem").unwrap();
        fs::write(temp.0.join("depois.fer"), "destino").unwrap();

        assert!(rename_relative_file(&temp.0, Path::new("antes.fer"), "depois.fer").is_err());
        assert_eq!(
            fs::read_to_string(temp.0.join("antes.fer")).unwrap(),
            "origem"
        );
        assert_eq!(
            fs::read_to_string(temp.0.join("depois.fer")).unwrap(),
            "destino"
        );
    }

    #[test]
    fn deletes_supported_files_and_preserves_neighbors() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aulas")).unwrap();
        fs::write(temp.0.join("aulas/ponte.fer"), "funcao inicio() {}").unwrap();
        fs::write(temp.0.join("aulas/main.rs"), "fn main() {}").unwrap();
        fs::write(temp.0.join("aulas/exercicio.licao"), "{}").unwrap();
        fs::write(temp.0.join("aulas/manter.fer"), "manter").unwrap();

        for name in ["ponte.fer", "main.rs", "exercicio.licao"] {
            let relative = Path::new("aulas").join(name);
            let expected = fs::canonicalize(temp.0.join("aulas").join(name)).unwrap();
            let deleted = delete_relative_file(&temp.0, &relative).unwrap();
            assert_eq!(deleted, expected);
            assert!(!temp.0.join("aulas").join(name).exists());
        }
        assert_eq!(
            fs::read_to_string(temp.0.join("aulas/manter.fer")).unwrap(),
            "manter"
        );
    }

    #[test]
    fn delete_rejects_traversal_directories_missing_and_unsupported_files() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("pasta.fer")).unwrap();
        fs::write(temp.0.join("notas.txt"), "manter").unwrap();
        fs::write(temp.0.join("legado.por"), "manter").unwrap();
        fs::write(temp.0.join("programa.fer"), "manter").unwrap();

        for relative in [
            Path::new("../fora.fer"),
            Path::new("pasta.fer"),
            Path::new("ausente.fer"),
            Path::new("notas.txt"),
            Path::new("legado.por"),
        ] {
            assert!(
                delete_relative_file(&temp.0, relative).is_err(),
                "{relative:?}"
            );
        }
        assert_eq!(
            fs::read_to_string(temp.0.join("programa.fer")).unwrap(),
            "manter"
        );
        assert_eq!(
            fs::read_to_string(temp.0.join("notas.txt")).unwrap(),
            "manter"
        );
        assert!(temp.0.join("pasta.fer").is_dir());
    }

    #[test]
    fn removes_deleted_path_from_a_previous_workspace_listing() {
        let mut entries = vec![
            WorkspaceEntry::Folder {
                name: "aulas".into(),
                path: "aulas".into(),
                entries: vec![
                    WorkspaceEntry::File {
                        name: "manter.fer".into(),
                        path: "aulas/manter.fer".into(),
                    },
                    WorkspaceEntry::File {
                        name: "apagar.fer".into(),
                        path: "aulas/apagar.fer".into(),
                    },
                ],
            },
            WorkspaceEntry::File {
                name: "fora.fer".into(),
                path: "fora.fer".into(),
            },
        ];

        super::remove_file_from_listing(&mut entries, "aulas/apagar.fer");

        assert!(matches!(
            &entries[0],
            WorkspaceEntry::Folder { entries, .. } if entries.len() == 1
                && matches!(&entries[0], WorkspaceEntry::File { path, .. } if path == "aulas/manter.fer")
        ));
        assert!(matches!(
            &entries[1],
            WorkspaceEntry::File { path, .. } if path == "fora.fer"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_delete_a_symlink_outside_the_workspace() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("programa.fer"), "fora").unwrap();
        symlink(outside.0.join("programa.fer"), temp.0.join("ligacao.fer")).unwrap();

        assert!(delete_relative_file(&temp.0, Path::new("ligacao.fer")).is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("programa.fer")).unwrap(),
            "fora"
        );
        assert!(temp.0.join("ligacao.fer").exists());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_rename_a_symlink_outside_the_workspace() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("programa.fer"), "fora").unwrap();
        symlink(outside.0.join("programa.fer"), temp.0.join("ligacao.fer")).unwrap();

        assert!(rename_relative_file(&temp.0, Path::new("ligacao.fer"), "novo.fer").is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("programa.fer")).unwrap(),
            "fora"
        );
        assert!(temp.0.join("ligacao.fer").exists());
    }

    #[test]
    fn saves_the_canonical_workspace_path_for_the_next_session() {
        let config = TempDir::new();
        let root = TempDir::new();
        fs::create_dir_all(root.0.join("Aulas com acento")).unwrap();
        let workspace = root.0.join("Aulas com acento");

        save_last_workspace(&config.0, &workspace).unwrap();

        assert_eq!(
            super::load_last_workspace(&config.0).unwrap(),
            Some(fs::canonicalize(workspace).unwrap())
        );
    }

    #[test]
    fn restores_a_saved_workspace_with_a_fresh_session_id() {
        let config = TempDir::new();
        let root = TempDir::new();
        fs::write(root.0.join("programa.fer"), "funcao inicio() {}").unwrap();
        save_last_workspace(&config.0, &root.0).unwrap();

        let session = super::WorkspaceSession::default();
        let restored = restore_saved_workspace(&session, &config.0)
            .unwrap()
            .unwrap();

        assert_eq!(
            restored.name,
            root.0.file_name().unwrap().to_string_lossy().into_owned()
        );
        assert_eq!(restored.id, "workspace-1");
        assert!(
            matches!(&restored.entries[0], WorkspaceEntry::File { name, .. } if name == "programa.fer")
        );
    }

    #[test]
    fn no_saved_workspace_returns_none_without_an_error() {
        let config = TempDir::new();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_removed_workspace_is_forgotten_after_restore() {
        let config = TempDir::new();
        let root = TempDir::new();
        save_last_workspace(&config.0, &root.0).unwrap();
        fs::remove_dir_all(&root.0).unwrap();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
        assert!(super::load_last_workspace(&config.0).unwrap().is_none());
    }

    #[test]
    fn malformed_saved_state_is_discarded_without_blocking_startup() {
        let config = TempDir::new();
        let saved_path = config.0.join(super::LAST_WORKSPACE_FILE);
        fs::write(&saved_path, "not valid JSON").unwrap();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
        assert!(!saved_path.exists());
    }

    #[test]
    fn relative_saved_paths_are_discarded() {
        let config = TempDir::new();
        let saved_path = config.0.join(super::LAST_WORKSPACE_FILE);
        fs::write(&saved_path, serde_json::to_vec("relative/project").unwrap()).unwrap();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
        assert!(!saved_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlinks_and_rejects_symlink_reads() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("segredo.fer"), "segredo").unwrap();
        symlink(outside.0.join("segredo.fer"), temp.0.join("link.fer")).unwrap();
        symlink(&outside.0, temp.0.join("pasta-link")).unwrap();

        assert!(list_entries(&temp.0).unwrap().is_empty());
        assert!(read_relative_file(&temp.0, Path::new("link.fer")).is_err());
        assert!(super::write_relative_file(&temp.0, Path::new("link.fer"), "alterado").is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("segredo.fer")).unwrap(),
            "segredo"
        );
    }

    #[test]
    fn creates_source_file_inside_nested_subfolder_and_rejects_traversal() {
        let temp = TempDir::new();

        let created =
            super::create_source_file_in_folder(&temp.0, "ola.fer", Some("aulas/modulo1")).unwrap();
        assert_eq!(created, "aulas/modulo1/ola.fer");
        assert!(temp.0.join("aulas/modulo1/ola.fer").is_file());

        assert!(
            super::create_source_file_in_folder(&temp.0, "ola.fer", Some("aulas/modulo1")).is_err()
        );
        assert!(super::create_source_file_in_folder(&temp.0, "fora.fer", Some("../fora")).is_err());
        assert!(
            super::create_source_file_in_folder(&temp.0, "fora.fer", Some("aulas\\sub")).is_err()
        );
        assert!(super::create_source_file_in_folder(&temp.0, "fora.fer", Some("/abs")).is_err());
    }

    #[test]
    fn sanitizes_cargo_package_names_for_export() {
        assert_eq!(
            super::sanitize_cargo_package_name("Olá Mundo!.fer"),
            "ol_mundo"
        );
        assert_eq!(
            super::sanitize_cargo_package_name("123_teste.rs"),
            "programa_123_teste"
        );
        assert_eq!(super::sanitize_cargo_package_name("match"), "match_app");
        assert_eq!(
            super::sanitize_cargo_package_name("   "),
            "programa_portugol"
        );
    }

    #[test]
    fn exports_complete_cargo_project_scaffold_without_overwriting_existing_folder() {
        let destination = TempDir::new();
        let code = "fn main() {\n    println!(\"Olá, Rust!\");\n}";

        let exported =
            super::write_cargo_project_scaffold(&destination.0, "meu-programa.fer", code).unwrap();

        assert_eq!(exported.package_name, "meu_programa");
        let project_dir = destination.0.join("meu_programa");
        assert_eq!(
            fs::read_to_string(project_dir.join("Cargo.toml")).unwrap(),
            "[package]\nname = \"meu_programa\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n"
        );
        assert_eq!(
            fs::read_to_string(project_dir.join("src/main.rs")).unwrap(),
            "fn main() {\n    println!(\"Olá, Rust!\");\n}\n"
        );
        assert!(
            super::write_cargo_project_scaffold(&destination.0, "meu-programa.fer", code)
                .unwrap_err()
                .contains("Já existe")
        );
    }
}
use portugol_formatos::Kind;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct WorkspaceState {
    next_id: u64,
    roots: HashMap<String, PathBuf>,
    directories: HashMap<String, cap_std::fs::Dir>,
    ids: HashMap<PathBuf, String>,
}

#[derive(Default, Clone)]
pub struct WorkspaceSession(Arc<Mutex<WorkspaceState>>);

impl WorkspaceSession {
    fn register(&self, root: PathBuf) -> Result<String, String> {
        let directory = cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority())
            .map_err(|error| format!("Não foi possível fixar a pasta do projeto: {error}"))?;
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(id) = state.ids.get(&root) {
            return Ok(id.clone());
        }
        state.next_id += 1;
        let next_id = state.next_id;
        let id = format!("workspace-{next_id}");
        state.ids.insert(root.clone(), id.clone());
        state.roots.insert(id.clone(), root);
        state.directories.insert(id.clone(), directory);
        Ok(id)
    }

    fn root(&self, id: &str) -> Result<PathBuf, String> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .roots
            .get(id)
            .cloned()
            .ok_or_else(|| {
                "Esta pasta não está mais aberta. Abra-a novamente para continuar.".into()
            })
    }

    fn directory(&self, id: &str) -> Result<cap_std::fs::Dir, String> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .directories
            .get(id)
            .ok_or_else(|| {
                "Esta pasta não está mais aberta. Abra-a novamente para continuar.".to_string()
            })?
            .try_clone()
            .map_err(|error| format!("Não foi possível acessar a pasta aberta: {error}"))
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkspaceEntry {
    Folder {
        name: String,
        path: String,
        entries: Vec<WorkspaceEntry>,
    },
    File {
        name: String,
        path: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListing {
    id: String,
    name: String,
    preference_key: String,
    entries: Vec<WorkspaceEntry>,
}

/// Projeto criado pelo modelo *Aplicativo Desktop PORS* e já aberto.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopProjectCreated {
    listing: WorkspaceListing,
    /// Tela inicial, relativa à pasta do projeto, para abrir no editor.
    design_path: String,
}

/// Nome do projeto que também serve de nome da pasta.
fn desktop_project_folder_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    portugol_tela::project::validate_project_name(trimmed)
        .map_err(|message| format!("{message}."))?;
    if trimmed.starts_with('.') || trimmed.ends_with('.') {
        return Err("O nome do projeto não pode começar nem terminar com ponto.".into());
    }
    if let Some(invalid) = trimmed
        .chars()
        .find(|c| matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'))
    {
        return Err(format!(
            "O nome do projeto vira o nome da pasta e não pode ter `{invalid}`."
        ));
    }
    Ok(trimmed.to_owned())
}

/// Cria a pasta `parent/name` com o modelo de projeto desktop. A pasta não
/// pode existir; se algo falhar depois de criá-la, ela é removida.
fn create_desktop_project_in(parent: &Path, name: &str) -> Result<PathBuf, String> {
    let folder_name = desktop_project_folder_name(name)?;
    let template = portugol_tela::project::desktop_template(&folder_name)?;
    let parent = fs::canonicalize(parent)
        .map_err(|_| "A pasta escolhida não está disponível.".to_string())?;
    if !parent.is_dir() {
        return Err("A seleção não é uma pasta.".into());
    }
    let folder = parent.join(&folder_name);
    fs::create_dir(&folder).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            format!("Já existe uma pasta chamada \"{folder_name}\" nesse local.")
        } else {
            format!("Não foi possível criar a pasta do projeto: {error}")
        }
    })?;
    let written = (|| -> std::io::Result<()> {
        for directory in &template.directories {
            fs::create_dir(folder.join(directory))?;
        }
        for file in &template.files {
            use std::io::Write as _;
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(folder.join(&file.path))?
                .write_all(file.content.as_bytes())?;
        }
        Ok(())
    })();
    if let Err(error) = written {
        let _ = fs::remove_dir_all(&folder);
        return Err(format!(
            "Não foi possível criar os arquivos do projeto: {error}"
        ));
    }
    Ok(folder)
}

fn workspace_preference_key(root: &Path) -> String {
    let canonical = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    let hex = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256-{hex}")
}

const LAST_WORKSPACE_FILE: &str = "last-workspace.json";

fn save_last_workspace(config_dir: &Path, root: &Path) -> Result<(), String> {
    let root = fs::canonicalize(root)
        .map_err(|error| format!("Não foi possível salvar a pasta do projeto: {error}"))?;
    let root = root.to_str().ok_or_else(|| {
        "O caminho do projeto não pode ser representado nesta interface.".to_string()
    })?;
    let encoded = serde_json::to_vec(root)
        .map_err(|error| format!("Não foi possível salvar a pasta do projeto: {error}"))?;
    fs::create_dir_all(config_dir)
        .map_err(|error| format!("Não foi possível acessar as preferências da IDE: {error}"))?;

    let saved_path = config_dir.join(LAST_WORKSPACE_FILE);
    let temporary_path = config_dir.join(format!("{LAST_WORKSPACE_FILE}.tmp"));
    fs::write(&temporary_path, encoded)
        .map_err(|error| format!("Não foi possível salvar a pasta do projeto: {error}"))?;
    match fs::rename(&temporary_path, &saved_path) {
        Ok(()) => Ok(()),
        Err(error) if saved_path.exists() => {
            fs::remove_file(&saved_path).map_err(|remove_error| {
                format!("Não foi possível atualizar a pasta salva: {remove_error}")
            })?;
            fs::rename(&temporary_path, &saved_path).map_err(|rename_error| {
                format!("Não foi possível salvar a pasta do projeto: {rename_error}; {error}")
            })
        }
        Err(error) => Err(format!(
            "Não foi possível salvar a pasta do projeto: {error}"
        )),
    }
}

fn load_last_workspace(config_dir: &Path) -> Result<Option<PathBuf>, String> {
    let saved_path = config_dir.join(LAST_WORKSPACE_FILE);
    let encoded = match fs::read(&saved_path) {
        Ok(encoded) => encoded,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Não foi possível ler as preferências da IDE: {error}"
            ));
        }
    };
    let Ok(path) = serde_json::from_slice::<String>(&encoded) else {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    };
    if path.is_empty() {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    }
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    }
    Ok(Some(path))
}

fn clear_last_workspace(config_dir: &Path) -> Result<(), String> {
    match fs::remove_file(config_dir.join(LAST_WORKSPACE_FILE)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Não foi possível limpar a preferência de projeto: {error}"
        )),
    }
}

fn restore_saved_workspace(
    session: &WorkspaceSession,
    config_dir: &Path,
) -> Result<Option<WorkspaceListing>, String> {
    let Some(saved_root) = load_last_workspace(config_dir)? else {
        return Ok(None);
    };
    let Ok(root) = fs::canonicalize(saved_root) else {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    };
    if !root.is_dir() {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    }
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    let preference_key = workspace_preference_key(&root);
    let id = session.register(root)?;
    Ok(Some(WorkspaceListing {
        id,
        name,
        preference_key,
        entries,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFile {
    path: String,
    relative_path: String,
    source: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFilePath {
    path: String,
    relative_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRenameResult {
    listing: WorkspaceListing,
    file: WorkspaceFilePath,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDeleteResult {
    listing: WorkspaceListing,
    file: WorkspaceFilePath,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceExportResult {
    project_name: String,
    file_count: usize,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CargoScaffoldExportResult {
    project_dir: String,
    manifest_path: String,
    main_path: String,
    package_name: String,
}

#[cfg(test)]
fn copy_selected_files(
    source_root: &Path,
    destination_parent: &Path,
    selected_paths: &[String],
) -> Result<WorkspaceExportResult, String> {
    let source_root = fs::canonicalize(source_root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let source = cap_std::fs::Dir::open_ambient_dir(&source_root, cap_std::ambient_authority())
        .map_err(|error| format!("Não foi possível abrir o projeto: {error}"))?;
    let project_name = source_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto");
    copy_selected_files_from_dir_with_hook(
        &source,
        &source_root,
        project_name,
        destination_parent,
        selected_paths,
        |_| Ok(()),
    )
}

#[cfg(test)]
fn copy_selected_files_with_hook(
    source_root: &Path,
    destination_parent: &Path,
    selected_paths: &[String],
    before_copy: impl FnMut(&str) -> Result<(), String>,
) -> Result<WorkspaceExportResult, String> {
    let source_root = fs::canonicalize(source_root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let source = cap_std::fs::Dir::open_ambient_dir(&source_root, cap_std::ambient_authority())
        .map_err(|error| format!("Não foi possível abrir o projeto: {error}"))?;
    let project_name = source_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto");
    copy_selected_files_from_dir_with_hook(
        &source,
        &source_root,
        project_name,
        destination_parent,
        selected_paths,
        before_copy,
    )
}

fn copy_selected_files_from_dir_with_hook(
    source: &cap_std::fs::Dir,
    source_root_path: &Path,
    project_name: &str,
    destination_parent: &Path,
    selected_paths: &[String],
    mut before_copy: impl FnMut(&str) -> Result<(), String>,
) -> Result<WorkspaceExportResult, String> {
    if selected_paths.is_empty() {
        return Err("Selecione ao menos um arquivo para exportar.".into());
    }

    let source_root_path = fs::canonicalize(source_root_path)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let destination_parent = fs::canonicalize(destination_parent)
        .map_err(|_| "A pasta de destino não está mais disponível.".to_string())?;
    if !path_matches_directory_handle(&source_root_path, source) {
        return Err(
            "A pasta aberta mudou desde que o projeto foi carregado. Abra-a novamente.".into(),
        );
    }
    if !destination_parent.is_dir() {
        return Err("A origem ou o destino não é uma pasta disponível.".into());
    }
    if destination_parent.starts_with(&source_root_path) {
        return Err("Escolha uma pasta de destino fora do projeto aberto.".into());
    }
    let selected_paths: BTreeSet<_> = selected_paths.iter().cloned().collect();
    let mut inputs = Vec::with_capacity(selected_paths.len());
    for relative in &selected_paths {
        inputs.push((
            relative.clone(),
            open_selected_file(source, &source_root_path, relative)?,
        ));
    }

    let destination =
        cap_std::fs::Dir::open_ambient_dir(&destination_parent, cap_std::ambient_authority())
            .map_err(|error| format!("Não foi possível abrir a pasta de destino: {error}"))?;
    destination.create_dir(project_name).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            format!("Já existe uma pasta chamada ‘{project_name}’ no destino escolhido.")
        } else {
            format!("Não foi possível criar a pasta exportada: {error}")
        }
    })?;
    let exported = destination
        .open_dir(project_name)
        .map_err(|error| format!("Não foi possível acessar a pasta exportada: {error}"))?;

    let mut created_files: Vec<(cap_std::fs::Dir, OsString)> = Vec::new();
    let mut created_directories: Vec<(cap_std::fs::Dir, OsString)> = Vec::new();
    let copy_result = (|| {
        for (relative, mut input) in inputs {
            before_copy(&relative)?;
            let relative_path = Path::new(&relative);
            let parent = if let Some(parent_path) = relative_path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                open_or_create_export_parent(&exported, parent_path, &mut created_directories)
                    .map_err(|error| {
                        format!("Não foi possível criar a subpasta de ‘{relative}’: {error}")
                    })?
            } else {
                exported.try_clone().map_err(|error| {
                    format!("Não foi possível acessar a pasta exportada: {error}")
                })?
            };
            let file_name = relative_path
                .file_name()
                .ok_or_else(|| "O caminho do arquivo é inválido.".to_string())?
                .to_os_string();
            let mut options = cap_std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            let mut output = parent.open_with(&file_name, &options).map_err(|error| {
                format!("Não foi possível criar o arquivo exportado ‘{relative}’ sem sobrescrever: {error}")
            })?;
            created_files.push((parent, file_name));
            std::io::copy(&mut input, &mut output)
                .map_err(|error| format!("Não foi possível copiar ‘{relative}’: {error}"))?;
        }
        Ok(())
    })();

    if let Err(error) = copy_result {
        if let Err(cleanup_error) = rollback_export(
            &destination,
            project_name,
            exported,
            created_files,
            created_directories,
        ) {
            return Err(format!(
                "{error} A limpeza da cópia parcial também falhou: {cleanup_error}"
            ));
        }
        return Err(error);
    }

    Ok(WorkspaceExportResult {
        project_name: project_name.to_string(),
        file_count: selected_paths.len(),
    })
}

fn open_selected_file(
    source: &cap_std::fs::Dir,
    source_root_path: &Path,
    relative: &str,
) -> Result<cap_std::fs::File, String> {
    open_selected_file_with_hook(source, source_root_path, relative, |_| Ok(()))
}

fn open_selected_file_with_hook(
    source: &cap_std::fs::Dir,
    source_root_path: &Path,
    relative: &str,
    mut after_metadata: impl FnMut(&Path) -> Result<(), String>,
) -> Result<cap_std::fs::File, String> {
    let relative_path = Path::new(relative);
    if relative_path.as_os_str().is_empty()
        || relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("O caminho do arquivo é inválido.".into());
    }

    let components: Vec<_> = relative_path.components().collect();
    let mut current = source
        .try_clone()
        .map_err(|error| format!("Não foi possível acessar o projeto: {error}"))?;
    let mut checked_path = source_root_path.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("O caminho do arquivo é inválido.".into());
        };
        let metadata = current
            .symlink_metadata(name)
            .map_err(|_| "O arquivo não está mais disponível.".to_string())?;
        checked_path.push(name);
        if metadata.file_type().is_symlink() {
            return Err("Links simbólicos não podem ser exportados.".into());
        }
        let is_last = index + 1 == components.len();
        let path_metadata = fs::symlink_metadata(&checked_path)
            .map_err(|_| "O arquivo selecionado mudou durante a validação.".to_string())?;
        if path_metadata.file_type().is_symlink() {
            return Err("Links simbólicos não podem ser exportados.".into());
        }
        let validated_handle = open_path_handle(&checked_path, !is_last)
            .map_err(|_| "O arquivo selecionado mudou durante a validação.".to_string())?;
        if is_last {
            if !metadata.is_file()
                || !(supported_program(relative_path)
                    || is_exercise(relative_path)
                    || supported_markup(relative_path))
            {
                return Err(format!(
                    "O explorador só exporta arquivos {}.",
                    all_extensions()
                ));
            }
            after_metadata(relative_path)?;
            let file = current
                .open(name)
                .map_err(|_| "O arquivo selecionado não pode ser lido.".to_string())?;
            let opened_handle = file
                .try_clone()
                .map(cap_std::fs::File::into_std)
                .map_err(|_| "O arquivo selecionado não pode ser lido.".to_string())?;
            if !same_file_identity(&validated_handle, &opened_handle) {
                return Err(
                    "O arquivo mudou durante a validação. Atualize o Explorador e tente novamente."
                        .into(),
                );
            }
            return Ok(file);
        }
        if !metadata.is_dir() {
            return Err("O caminho do arquivo é inválido.".into());
        }
        after_metadata(relative_path)?;
        let opened = current
            .open_dir(name)
            .map_err(|_| "A subpasta selecionada não está mais disponível.".to_string())?;
        let opened_handle = opened
            .try_clone()
            .map_err(|_| "A subpasta selecionada não está mais disponível.".to_string())?
            .into_std_file();
        if !same_file_identity(&validated_handle, &opened_handle) {
            return Err(
                "A pasta mudou durante a validação. Atualize o Explorador e tente novamente."
                    .into(),
            );
        }
        current = opened;
    }
    Err("O caminho do arquivo é inválido.".into())
}

fn open_path_handle(path: &Path, directory: bool) -> std::io::Result<fs::File> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        let mut options = fs::OpenOptions::new();
        options.read(true).custom_flags(
            FILE_FLAG_OPEN_REPARSE_POINT
                | if directory {
                    FILE_FLAG_BACKUP_SEMANTICS
                } else {
                    0
                },
        );
        options.open(path)
    }
    #[cfg(not(windows))]
    {
        let _ = directory;
        fs::File::open(path)
    }
}

fn open_or_create_export_parent(
    root: &cap_std::fs::Dir,
    relative: &Path,
    created: &mut Vec<(cap_std::fs::Dir, OsString)>,
) -> std::io::Result<cap_std::fs::Dir> {
    let mut current = root.try_clone()?;
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "caminho de subpasta inválido",
            ));
        };
        let name = name.to_os_string();
        match current.symlink_metadata(&name) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "o caminho de destino existe e não é uma pasta segura",
                ));
            }
            Ok(_) => current = current.open_dir(&name)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let parent = current.try_clone()?;
                current.create_dir(&name)?;
                created.push((parent, name.clone()));
                current = current.open_dir(&name)?;
            }
            Err(error) => return Err(error),
        }
    }
    Ok(current)
}

fn rollback_export(
    parent: &cap_std::fs::Dir,
    project_name: &str,
    exported: cap_std::fs::Dir,
    created_files: Vec<(cap_std::fs::Dir, OsString)>,
    created_directories: Vec<(cap_std::fs::Dir, OsString)>,
) -> Result<(), String> {
    let mut errors = Vec::new();
    for (directory, name) in created_files.iter().rev() {
        if let Err(error) = directory.remove_file(name)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            errors.push(error.to_string());
        }
    }
    for (directory, name) in created_directories.iter().rev() {
        if let Err(error) = directory.remove_dir(name)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            errors.push(error.to_string());
        }
    }
    if errors.is_empty() {
        if same_directory_identity(parent, project_name, &exported) {
            drop(exported);
            drop(created_files);
            drop(created_directories);
            if let Err(error) = parent.remove_dir(project_name) {
                errors.push(error.to_string());
            }
        } else {
            errors.push(
                "a pasta de destino não corresponde mais à criada pela exportação; o conteúdo externo foi preservado"
                    .into(),
            );
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn path_matches_directory_handle(path: &Path, directory: &cap_std::fs::Dir) -> bool {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return false;
    }
    let Ok(path_handle) = open_path_handle(path, true) else {
        return false;
    };
    let Ok(handle) = directory
        .try_clone()
        .map(|directory| directory.into_std_file())
    else {
        return false;
    };
    same_file_identity(&path_handle, &handle)
}

fn same_directory_identity(
    parent: &cap_std::fs::Dir,
    name: &str,
    expected: &cap_std::fs::Dir,
) -> bool {
    let Ok(entry_metadata) = parent.symlink_metadata(name) else {
        return false;
    };
    if entry_metadata.file_type().is_symlink() || !entry_metadata.is_dir() {
        return false;
    }
    let Ok(actual) = parent.open_dir(name).map(cap_std::fs::Dir::into_std_file) else {
        return false;
    };
    let Ok(expected) = expected
        .try_clone()
        .map(|directory| directory.into_std_file())
    else {
        return false;
    };
    same_file_identity(&actual, &expected)
}

#[cfg(unix)]
fn same_file_identity(left: &fs::File, right: &fs::File) -> bool {
    use std::os::unix::fs::MetadataExt;
    let (Ok(left), Ok(right)) = (left.metadata(), right.metadata()) else {
        return false;
    };
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(windows)]
fn same_file_identity(left: &fs::File, right: &fs::File) -> bool {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct FileInformation {
        attributes: u32,
        creation_time: FileTime,
        last_access_time: FileTime,
        last_write_time: FileTime,
        volume_serial_number: u32,
        file_size_high: u32,
        file_size_low: u32,
        number_of_links: u32,
        file_index_high: u32,
        file_index_low: u32,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetFileInformationByHandle(file: *mut c_void, information: *mut FileInformation) -> i32;
    }

    fn identity(file: &fs::File) -> Option<(u32, u32, u32)> {
        let mut information = std::mem::MaybeUninit::<FileInformation>::uninit();
        // SAFETY: the handle is live for this call and the pointer refers to a correctly sized output struct.
        let succeeded = unsafe {
            GetFileInformationByHandle(file.as_raw_handle().cast(), information.as_mut_ptr())
        };
        if succeeded == 0 {
            return None;
        }
        // SAFETY: GetFileInformationByHandle initialized the output struct on success.
        let information = unsafe { information.assume_init() };
        Some((
            information.volume_serial_number,
            information.file_index_high,
            information.file_index_low,
        ))
    }

    identity(left)
        .zip(identity(right))
        .is_some_and(|(left, right)| left == right)
}

#[cfg(not(any(unix, windows)))]
fn same_file_identity(_left: &fs::File, _right: &fs::File) -> bool {
    false
}

// O que é cada arquivo vem da tabela única de formatos (ADR 0023, Emenda 2).

fn is_legacy_extension(path: &Path) -> bool {
    portugol_formatos::is_retired_path(path)
}

fn file_kind(path: &Path) -> Option<Kind> {
    portugol_formatos::classify_path(path).map(|m| m.kind())
}

/// Tudo o que o explorador lista, abre, renomeia, exporta e exclui.
fn supported_source(path: &Path) -> bool {
    file_kind(path).is_some()
}

/// Marcação de projetos desktop (ADR 0018): o desenho de uma tela e o
/// manifesto.
fn supported_markup(path: &Path) -> bool {
    file_kind(path).is_some_and(Kind::is_markup)
}

fn supported_program(path: &Path) -> bool {
    file_kind(path).is_some_and(Kind::is_program)
}

/// Extensões de tudo o que o explorador aceita, para mensagens.
fn all_extensions() -> String {
    portugol_formatos::describe(&portugol_formatos::all_kinds())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "O caminho está fora da pasta aberta.".to_string())?;
    relative
        .to_str()
        .map(|path| path.replace(std::path::MAIN_SEPARATOR, "/"))
        .ok_or_else(|| "O nome do arquivo não pode ser representado nesta interface.".to_string())
}

fn list_folder(root: &Path, folder: &Path) -> Result<Vec<WorkspaceEntry>, String> {
    let entries = fs::read_dir(folder)
        .map_err(|error| format!("Não foi possível listar a pasta: {error}"))?;
    let mut ordered = BTreeMap::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("Não foi possível ler uma entrada da pasta: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("Não foi possível inspecionar uma entrada: {error}"))?;
        if file_type.is_symlink() || entry.file_name().to_str().is_none() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            let children = list_folder(root, &path)?;
            if !children.is_empty() {
                ordered.insert(entry.file_name(), (path, Some(children)));
            }
        } else if file_type.is_file() && supported_source(&path) {
            ordered.insert(entry.file_name(), (path, None));
        }
    }

    ordered
        .into_values()
        .map(|(path, children)| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    "O nome do arquivo não pode ser representado nesta interface.".to_string()
                })?
                .to_string();
            let path = relative_path(root, &path)?;
            Ok(match children {
                Some(entries) => WorkspaceEntry::Folder {
                    name,
                    path,
                    entries,
                },
                None => WorkspaceEntry::File { name, path },
            })
        })
        .collect()
}

fn list_entries(root: &Path) -> Result<Vec<WorkspaceEntry>, String> {
    if !root.is_dir() {
        return Err("A pasta selecionada não está disponível.".into());
    }
    list_folder(root, root)
}

fn remove_file_from_listing(entries: &mut Vec<WorkspaceEntry>, relative_path: &str) {
    entries.retain_mut(|entry| match entry {
        WorkspaceEntry::File { path, .. } => path != relative_path,
        WorkspaceEntry::Folder { entries, .. } => {
            remove_file_from_listing(entries, relative_path);
            !entries.is_empty()
        }
    });
}

fn checked_file(root: &Path, relative: &Path, allow_exercise: bool) -> Result<PathBuf, String> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("O caminho do arquivo é inválido.".into());
    }
    let root = fs::canonicalize(root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let mut candidate = root.clone();
    let components: Vec<_> = relative.components().collect();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("O caminho do arquivo é inválido.".into());
        };
        candidate.push(name);
        let metadata = fs::symlink_metadata(&candidate)
            .map_err(|_| "O arquivo não está mais disponível.".to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("Links simbólicos não podem ser abertos pelo explorador.".into());
        }
        if index + 1 < components.len() && !metadata.is_dir() {
            return Err("O caminho do arquivo é inválido.".into());
        }
    }
    let metadata =
        fs::metadata(&candidate).map_err(|_| "O arquivo não está mais disponível.".to_string())?;
    if !metadata.is_file()
        || !(supported_program(&candidate)
            || supported_markup(&candidate)
            || allow_exercise && is_exercise(&candidate))
    {
        if is_legacy_extension(&candidate) {
            return Err(
                "Arquivos legados .por e .pex não são suportados no Ferro Studio. Use o Portugol Studio original para abri-los."
                    .into(),
            );
        }
        return Err(format!(
            "O explorador só abre arquivos {}.",
            all_extensions()
        ));
    }
    let canonical = fs::canonicalize(&candidate)
        .map_err(|_| "O arquivo não está mais disponível.".to_string())?;
    if !canonical.starts_with(&root) {
        return Err("O arquivo está fora da pasta aberta.".into());
    }
    Ok(canonical)
}

/// Descobre o Cargo.toml mais próximo do arquivo Rust sem sair do workspace aberto.
pub(super) fn cargo_project_for_source(
    session: &WorkspaceSession,
    workspace_id: &str,
    relative_source: &str,
) -> Result<Option<PathBuf>, String> {
    cargo_project_and_source(session, workspace_id, relative_source)
        .map(|result| result.map(|(project, _source)| project))
}

pub(super) fn cargo_project_and_source(
    session: &WorkspaceSession,
    workspace_id: &str,
    relative_source: &str,
) -> Result<Option<(PathBuf, PathBuf)>, String> {
    let root = fs::canonicalize(session.root(workspace_id)?)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let source = checked_file(&root, Path::new(relative_source), false)?;
    if file_kind(&source) != Some(Kind::Rust) {
        return Err(format!(
            "A execução Cargo exige um arquivo {} do workspace.",
            portugol_formatos::describe(&[Kind::Rust])
        ));
    }
    let mut directory = source.parent();
    while let Some(candidate) = directory {
        let manifest = candidate.join("Cargo.toml");
        match fs::symlink_metadata(&manifest) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err("Cargo.toml não pode ser um link simbólico.".into());
            }
            Ok(metadata) if metadata.is_file() => {
                let canonical = fs::canonicalize(candidate)
                    .map_err(|_| "A pasta do projeto Cargo não está disponível.".to_string())?;
                if !canonical.starts_with(&root) {
                    return Err("O projeto Cargo está fora da pasta aberta.".into());
                }
                return Ok(Some((canonical, source)));
            }
            Ok(_) => return Err("Cargo.toml não é um arquivo regular.".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("Não foi possível verificar Cargo.toml: {error}")),
        }
        if candidate == root {
            break;
        }
        directory = candidate.parent();
    }
    Ok(None)
}

fn is_exercise(path: &Path) -> bool {
    file_kind(path) == Some(Kind::Lesson)
}

#[cfg(test)]
fn read_relative_file(root: &Path, relative: &Path) -> Result<String, String> {
    let path = checked_file(root, relative, true)?;
    let bytes =
        fs::read(path).map_err(|error| format!("Não foi possível ler o arquivo: {error}"))?;
    Ok(super::decode_text(bytes))
}

fn write_relative_file(root: &Path, relative: &Path, source: &str) -> Result<(), String> {
    if !supported_program(relative) && !supported_markup(relative) {
        return Err(format!(
            "Apenas arquivos {} podem ser salvos pelo explorador.",
            portugol_formatos::describe(&[Kind::Source, Kind::Rust, Kind::Design, Kind::Manifest])
        ));
    }
    let path = checked_file(root, relative, false)?;
    fs::write(path, source).map_err(|error| format!("Não foi possível salvar o arquivo: {error}"))
}

#[cfg(test)]
fn create_source_file(root: &Path, file_name: &str) -> Result<(), String> {
    create_source_file_in_folder(root, file_name, None).map(|_| ())
}

fn create_source_file_in_folder(
    root: &Path,
    file_name: &str,
    folder: Option<&str>,
) -> Result<String, String> {
    let relative = Path::new(file_name);
    if file_name.trim().is_empty()
        || file_name == "."
        || file_name == ".."
        || file_name.contains('/')
        || file_name.contains('\\')
        || relative.components().count() != 1
        || !matches!(relative.components().next(), Some(Component::Normal(_)))
    {
        return Err("Informe um nome de arquivo sem pastas.".into());
    }
    if !supported_program(relative) {
        return Err("O arquivo deve usar a extensão .fer ou .rs.".into());
    }

    let root = fs::canonicalize(root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    if !root.is_dir() {
        return Err("A pasta aberta não está mais disponível.".into());
    }

    let mut target_dir = root.clone();
    let mut normalized_segments: Vec<&str> = Vec::new();
    if let Some(raw_folder) = folder {
        let trimmed = raw_folder.trim();
        if !trimmed.is_empty() {
            if trimmed.contains('\\') {
                return Err("Use '/' para separar subpastas.".into());
            }
            if trimmed.starts_with('/') || Path::new(trimmed).is_absolute() {
                return Err("Informe uma subpasta relativa dentro do projeto.".into());
            }
            for segment in trimmed.trim_end_matches('/').split('/') {
                if segment.is_empty() || segment == "." || segment == ".." {
                    return Err("Informe uma subpasta relativa válida dentro do projeto.".into());
                }
                let segment_path = Path::new(segment);
                if segment_path.components().count() != 1
                    || !matches!(segment_path.components().next(), Some(Component::Normal(_)))
                {
                    return Err("Informe uma subpasta relativa válida dentro do projeto.".into());
                }
                target_dir.push(segment);
                match fs::symlink_metadata(&target_dir) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        return Err("Links simbólicos não podem ser usados como subpasta.".into());
                    }
                    Ok(metadata) if metadata.is_dir() => {}
                    Ok(_) => {
                        return Err("Já existe um arquivo com o mesmo nome da subpasta.".into());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        fs::create_dir(&target_dir).map_err(|create_error| {
                            format!("Não foi possível criar a subpasta: {create_error}")
                        })?;
                    }
                    Err(error) => {
                        return Err(format!("Não foi possível acessar a subpasta: {error}"));
                    }
                }
                normalized_segments.push(segment);
            }
            let canonical_dir = fs::canonicalize(&target_dir)
                .map_err(|_| "Não foi possível acessar a subpasta criada.".to_string())?;
            if !canonical_dir.starts_with(&root) {
                return Err("A subpasta está fora da pasta aberta.".into());
            }
            target_dir = canonical_dir;
        }
    }

    let path = target_dir.join(relative);
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "Já existe um arquivo com esse nome.".into()
            } else {
                format!("Não foi possível criar o arquivo: {error}")
            }
        })?;

    if normalized_segments.is_empty() {
        Ok(file_name.to_string())
    } else {
        Ok(format!("{}/{file_name}", normalized_segments.join("/")))
    }
}

fn sanitize_cargo_package_name(raw: &str) -> String {
    let trimmed = raw.trim();
    let without_ext = trimmed
        .strip_suffix(".fer")
        .or_else(|| trimmed.strip_suffix(".PORS"))
        .or_else(|| trimmed.strip_suffix(".rs"))
        .or_else(|| trimmed.strip_suffix(".RS"))
        .unwrap_or(trimmed);

    let mut sanitized = String::new();
    let mut previous_underscore = false;
    for ch in without_ext.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
            previous_underscore = false;
        } else if (ch == '_' || ch == '-' || ch.is_whitespace())
            && !sanitized.is_empty()
            && !previous_underscore
        {
            sanitized.push('_');
            previous_underscore = true;
        }
    }
    while sanitized.ends_with('_') {
        sanitized.pop();
    }
    if sanitized.is_empty() {
        return "programa_portugol".to_string();
    }
    if sanitized
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
        sanitized.insert_str(0, "programa_");
    }
    const RUST_KEYWORDS: &[&str] = &[
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true",
        "type", "unsafe", "use", "where", "while",
    ];
    if RUST_KEYWORDS.contains(&sanitized.as_str()) {
        sanitized.push_str("_app");
    }
    sanitized
}

fn write_cargo_project_scaffold(
    destination_parent: &Path,
    raw_package_name: &str,
    rust_code: &str,
) -> Result<CargoScaffoldExportResult, String> {
    if rust_code.trim().is_empty() {
        return Err("Não há código Rust gerado para exportar.".into());
    }
    let parent = fs::canonicalize(destination_parent)
        .map_err(|_| "A pasta de destino não está disponível.".to_string())?;
    if !parent.is_dir() {
        return Err("A pasta de destino não é um diretório válido.".into());
    }
    let package_name = sanitize_cargo_package_name(raw_package_name);
    let project_dir = parent.join(&package_name);
    fs::create_dir(&project_dir).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            format!("Já existe uma pasta \"{package_name}\" no destino selecionado.")
        } else {
            format!("Não foi possível criar a pasta do projeto Cargo: {error}")
        }
    })?;

    let write_files = || -> Result<(PathBuf, PathBuf), String> {
        let src_dir = project_dir.join("src");
        fs::create_dir(&src_dir)
            .map_err(|error| format!("Não foi possível criar a pasta src/: {error}"))?;
        let manifest_path = project_dir.join("Cargo.toml");
        let manifest = format!(
            "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n"
        );
        fs::write(&manifest_path, manifest)
            .map_err(|error| format!("Não foi possível gravar Cargo.toml: {error}"))?;
        let main_path = src_dir.join("main.rs");
        let main_source = if rust_code.ends_with('\n') {
            rust_code.to_string()
        } else {
            format!("{rust_code}\n")
        };
        fs::write(&main_path, main_source)
            .map_err(|error| format!("Não foi possível gravar src/main.rs: {error}"))?;
        Ok((manifest_path, main_path))
    };

    let (manifest_path, main_path) = match write_files() {
        Ok(paths) => paths,
        Err(error) => {
            let _ = fs::remove_dir_all(&project_dir);
            return Err(error);
        }
    };

    Ok(CargoScaffoldExportResult {
        project_dir: project_dir.to_string_lossy().into_owned(),
        manifest_path: manifest_path.to_string_lossy().into_owned(),
        main_path: main_path.to_string_lossy().into_owned(),
        package_name,
    })
}

fn rename_relative_file(root: &Path, relative: &Path, new_name: &str) -> Result<PathBuf, String> {
    let source = checked_file(root, relative, true)?;
    let name = Path::new(new_name);
    if new_name.trim().is_empty()
        || new_name == "."
        || new_name == ".."
        || new_name.contains('/')
        || new_name.contains('\\')
        || name.components().count() != 1
        || !matches!(name.components().next(), Some(Component::Normal(_)))
    {
        return Err("Informe um nome de arquivo sem pastas.".into());
    }
    if new_name.ends_with('.') || !supported_source(name) {
        return Err(format!(
            "O arquivo deve manter uma extensão {}.",
            portugol_formatos::describe_alternatives(&portugol_formatos::all_kinds())
        ));
    }

    // Renomear pode trocar a extensão antiga pela canônica do mesmo tipo,
    // mas não muda o tipo do arquivo.
    let source_format = portugol_formatos::classify_path(&source)
        .ok_or_else(|| "A extensão original do arquivo não é válida.".to_string())?;
    let new_format = portugol_formatos::classify_path(name)
        .ok_or_else(|| "A extensão do arquivo não é válida.".to_string())?;
    if source_format.kind() != new_format.kind() {
        return Err(format!(
            "A extensão deve continuar sendo .{}.",
            source_format.extension
        ));
    }

    let parent = source
        .parent()
        .ok_or_else(|| "O arquivo selecionado não pode ser renomeado.".to_string())?;
    let destination = parent.join(name);
    if source == destination {
        return Ok(source);
    }
    fs::hard_link(&source, &destination).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "Já existe um arquivo com esse nome.".into()
        } else {
            format!("Não foi possível renomear o arquivo: {error}")
        }
    })?;
    if let Err(error) = fs::remove_file(&source) {
        let _ = fs::remove_file(&destination);
        return Err(format!("Não foi possível renomear o arquivo: {error}"));
    }
    fs::canonicalize(&destination)
        .map_err(|error| format!("Não foi possível acessar o arquivo renomeado: {error}"))
}

#[cfg(test)]
fn delete_relative_file(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let root = fs::canonicalize(root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let directory = cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority())
        .map_err(|error| format!("Não foi possível abrir a pasta do projeto: {error}"))?;
    delete_relative_file_from_dir(&directory, &root, relative)
}

fn delete_relative_file_from_dir(
    directory: &cap_std::fs::Dir,
    root_path: &Path,
    relative: &Path,
) -> Result<PathBuf, String> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("O caminho do arquivo é inválido.".into());
    }

    let components: Vec<_> = relative.components().collect();
    let mut candidate = PathBuf::new();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("O caminho do arquivo é inválido.".into());
        };
        candidate.push(name);
        let metadata = directory
            .symlink_metadata(&candidate)
            .map_err(|_| "O arquivo não está mais disponível.".to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("Links simbólicos não podem ser excluídos pelo explorador.".into());
        }
        if index + 1 < components.len() && !metadata.is_dir() {
            return Err("O caminho do arquivo é inválido.".into());
        }
        if index + 1 == components.len() && (!metadata.is_file() || !supported_source(relative)) {
            return Err(format!(
                "O explorador só exclui arquivos {}.",
                all_extensions()
            ));
        }
    }

    let path = root_path.join(relative);
    path.to_str().ok_or_else(|| {
        "O caminho do arquivo não pode ser representado nesta interface.".to_string()
    })?;
    directory
        .remove_file(relative)
        .map_err(|error| format!("Não foi possível excluir o arquivo: {error}"))?;
    Ok(path)
}

#[tauri::command]
pub async fn workspace_open(
    app: AppHandle,
    session: State<'_, WorkspaceSession>,
) -> Result<Option<WorkspaceListing>, String> {
    let session = session.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(selected) = app
            .dialog()
            .file()
            .set_title("Abrir pasta do projeto")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let selected = selected
            .into_path()
            .map_err(|error| format!("Não foi possível abrir a pasta selecionada: {error}"))?;
        let root = fs::canonicalize(selected)
            .map_err(|error| format!("Não foi possível acessar a pasta selecionada: {error}"))?;
        if !root.is_dir() {
            return Err("A seleção não é uma pasta.".into());
        }
        let entries = list_entries(&root)?;
        let name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Projeto")
            .to_string();
        if let Ok(config_dir) = app.path().app_config_dir()
            && let Err(error) = save_last_workspace(&config_dir, &root)
        {
            eprintln!("{error}");
        }
        let preference_key = workspace_preference_key(&root);
        let id = session.register(root)?;
        Ok(Some(WorkspaceListing {
            id,
            name,
            preference_key,
            entries,
        }))
    })
    .await
    .map_err(|error| format!("Não foi possível abrir a pasta do projeto: {error}"))?
}

/// Cria um projeto desktop PORS numa pasta escolhida pelo usuário e abre o
/// projeto como workspace. Devolve `None` se o usuário cancelar a escolha.
#[tauri::command]
pub async fn workspace_create_desktop_project(
    app: AppHandle,
    session: State<'_, WorkspaceSession>,
    name: String,
) -> Result<Option<DesktopProjectCreated>, String> {
    desktop_project_folder_name(&name)?;
    let session = session.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(selected) = app
            .dialog()
            .file()
            .set_title("Escolha onde criar o projeto")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let parent = selected
            .into_path()
            .map_err(|error| format!("Não foi possível usar a pasta selecionada: {error}"))?;
        let root = create_desktop_project_in(&parent, &name)?;
        let root = fs::canonicalize(root)
            .map_err(|error| format!("Não foi possível acessar o projeto criado: {error}"))?;
        let entries = list_entries(&root)?;
        let folder_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Projeto")
            .to_string();
        if let Ok(config_dir) = app.path().app_config_dir()
            && let Err(error) = save_last_workspace(&config_dir, &root)
        {
            eprintln!("{error}");
        }
        let preference_key = workspace_preference_key(&root);
        let id = session.register(root)?;
        Ok(Some(DesktopProjectCreated {
            listing: WorkspaceListing {
                id,
                name: folder_name,
                preference_key,
                entries,
            },
            design_path: format!(
                "{}.{}",
                portugol_tela::project::DEFAULT_WINDOW,
                portugol_tela::FILE_EXTENSION
            ),
        }))
    })
    .await
    .map_err(|error| format!("Não foi possível criar o projeto: {error}"))?
}

#[tauri::command]
pub fn workspace_cargo_project_available(
    workspace_id: String,
    relative_source: String,
    session: State<'_, WorkspaceSession>,
) -> Result<bool, String> {
    cargo_project_for_source(&session, &workspace_id, &relative_source).map(|root| root.is_some())
}

#[tauri::command]
pub async fn workspace_restore(
    app: AppHandle,
    session: State<'_, WorkspaceSession>,
) -> Result<Option<WorkspaceListing>, String> {
    let session = session.inner().clone();
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("Não foi possível localizar as preferências da IDE: {error}"))?;
    tauri::async_runtime::spawn_blocking(move || restore_saved_workspace(&session, &config_dir))
        .await
        .map_err(|error| format!("Não foi possível restaurar o último projeto: {error}"))?
}

#[tauri::command]
pub fn workspace_read_file(
    workspace_id: String,
    relative_path: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceFile, String> {
    let root = session.root(&workspace_id)?;
    let relative = Path::new(&relative_path);
    let absolute = checked_file(&root, relative, true)?;
    let bytes =
        fs::read(&absolute).map_err(|error| format!("Não foi possível ler o arquivo: {error}"))?;
    let source = super::decode_text(bytes);
    let path = absolute.to_str().map(str::to_string).ok_or_else(|| {
        "O caminho do arquivo não pode ser representado nesta interface.".to_string()
    })?;
    let relative_path = relative
        .to_str()
        .ok_or_else(|| "O caminho do arquivo é inválido.".to_string())?
        .replace('\\', "/");
    Ok(WorkspaceFile {
        path,
        relative_path,
        source,
    })
}

#[tauri::command]
pub fn workspace_write_file(
    workspace_id: String,
    relative_path: String,
    source: String,
    session: State<'_, WorkspaceSession>,
) -> Result<(), String> {
    let root = session.root(&workspace_id)?;
    write_relative_file(&root, Path::new(&relative_path), &source)
}

#[tauri::command]
pub fn workspace_refresh(
    workspace_id: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceListing, String> {
    let root = session.root(&workspace_id)?;
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    Ok(WorkspaceListing {
        id: workspace_id,
        name,
        preference_key: workspace_preference_key(&root),
        entries,
    })
}

#[tauri::command]
pub fn workspace_create_file(
    workspace_id: String,
    file_name: String,
    folder: Option<String>,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceListing, String> {
    let root = session.root(&workspace_id)?;
    create_source_file_in_folder(&root, &file_name, folder.as_deref())?;
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    Ok(WorkspaceListing {
        id: workspace_id,
        name,
        preference_key: workspace_preference_key(&root),
        entries,
    })
}

#[tauri::command]
pub fn workspace_rename_file(
    workspace_id: String,
    relative_path: String,
    new_name: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceRenameResult, String> {
    let root = session.root(&workspace_id)?;
    let new_path = rename_relative_file(&root, Path::new(&relative_path), &new_name)?;
    let root = fs::canonicalize(&root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    let path = new_path.to_str().map(str::to_string).ok_or_else(|| {
        "O caminho do arquivo não pode ser representado nesta interface.".to_string()
    })?;
    let new_relative_path = self::relative_path(&root, &new_path)?;
    Ok(WorkspaceRenameResult {
        listing: WorkspaceListing {
            id: workspace_id,
            name,
            preference_key: workspace_preference_key(&root),
            entries,
        },
        file: WorkspaceFilePath {
            path,
            relative_path: new_relative_path,
        },
    })
}

#[tauri::command]
pub fn workspace_delete_file(
    workspace_id: String,
    relative_path: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceDeleteResult, String> {
    let root = session.root(&workspace_id)?;
    let root_directory = cap_std::fs::Dir::open_ambient_dir(&root, cap_std::ambient_authority())
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    let pre_delete_entries = list_entries(&root)?;
    let deleted_path =
        delete_relative_file_from_dir(&root_directory, &root, Path::new(&relative_path))?;
    let path = deleted_path.to_str().map(str::to_string).ok_or_else(|| {
        "O caminho do arquivo não pode ser representado nesta interface.".to_string()
    })?;
    let deleted_relative_path = self::relative_path(&root, &deleted_path)?;
    let entries = list_entries(&root).unwrap_or_else(|_| {
        let mut entries = pre_delete_entries;
        remove_file_from_listing(&mut entries, &deleted_relative_path);
        entries
    });
    Ok(WorkspaceDeleteResult {
        listing: WorkspaceListing {
            id: workspace_id,
            name,
            preference_key: workspace_preference_key(&root),
            entries,
        },
        file: WorkspaceFilePath {
            path,
            relative_path: deleted_relative_path,
        },
    })
}

#[tauri::command]
pub async fn workspace_export(
    app: AppHandle,
    workspace_id: String,
    relative_paths: Vec<String>,
    session: State<'_, WorkspaceSession>,
) -> Result<Option<WorkspaceExportResult>, String> {
    if relative_paths.is_empty() {
        return Err("Selecione ao menos um arquivo para exportar.".into());
    }
    let root = session.root(&workspace_id)?;
    let source = session.directory(&workspace_id)?;
    tauri::async_runtime::spawn_blocking(move || {
        let Some(selected) = app
            .dialog()
            .file()
            .set_title("Exportar projeto para uma pasta")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let destination = selected
            .into_path()
            .map_err(|error| format!("Não foi possível acessar a pasta selecionada: {error}"))?;
        let project_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Projeto")
            .to_string();
        copy_selected_files_from_dir_with_hook(
            &source,
            &root,
            &project_name,
            &destination,
            &relative_paths,
            |_| Ok(()),
        )
        .map(Some)
    })
    .await
    .map_err(|error| format!("Não foi possível exportar o projeto: {error}"))?
}

#[tauri::command]
pub async fn export_cargo_project(
    app: AppHandle,
    package_name: String,
    rust_code: String,
) -> Result<Option<CargoScaffoldExportResult>, String> {
    if rust_code.trim().is_empty() {
        return Err("Não há código Rust gerado para exportar.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let Some(selected) = app
            .dialog()
            .file()
            .set_title("Exportar projeto Cargo para uma pasta")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let destination = selected
            .into_path()
            .map_err(|error| format!("Não foi possível acessar a pasta selecionada: {error}"))?;
        write_cargo_project_scaffold(&destination, &package_name, &rust_code).map(Some)
    })
    .await
    .map_err(|error| format!("Não foi possível exportar o projeto Cargo: {error}"))?
}
