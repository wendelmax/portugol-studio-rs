# Plano — Cargo check e diagnósticos no editor

1. Test-first: criar projeto temporário com erro em fonte UTF-8 não BMP e observar falha da API ausente.
2. Adicionar serviço `cargo check` com mensagens JSON tipadas, mapeamento de span ao arquivo ativo e limites/cancelamento compartilhados.
3. Adicionar resolução backend do projeto + fonte pelo workspace e comando Tauri cancelável.
4. Adicionar comando explícito na paleta, consentimento Cargo específico, saving/guardas de revisões e painel de diagnósticos.
5. Validar ausência de execução ao digitar, arquivo fora do workspace/symlink, mensagens multi-arquivo e cancelamento.
6. Atualizar ADR, paridade e inventário; executar testes Rust/Tauri/Node/build/Clippy.
7. Revisão read-only, PR, aguardar CI verde, corrigir e mesclar; excluir branch remota.
