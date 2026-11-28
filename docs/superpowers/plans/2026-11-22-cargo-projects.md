# Plano — execução de projetos Cargo

1. Criar teste de integração que compile/execute um projeto mínimo e observar a falha antes da API.
2. Reutilizar o supervisor de processos do `ferro-service`, com build/run separados, saída limitada, cancelamento e diagnósticos JSON renderizados.
3. Descobrir `Cargo.toml` ancestral pelo workspace validado; cobrir ausência, caminho inválido e symlink.
4. Registrar comandos Tauri tipados, sem shell arbitrário nem novas permissões.
5. Ligar o botão Executar do arquivo Rust de workspace ao fluxo Cargo; informar consentimento, salvar alterações e preservar standalone.
6. Documentar decisões, limites e gaps restantes; rodar testes do service, Tauri, Node, build frontend, format e Clippy.
7. Fazer revisão read-only, abrir PR, aguardar CI verde, corrigir falhas e mesclar removendo a branch remota.
