# FT-15 — Gutter de mapeamento Portugol ↔ Rust

- **Entrega:** no modo bridge, o espaço entre o editor Portugol/Portugol-RS e a prévia Rust mostra pares de linhas derivados dos source maps reais para a linha do cursor.
- **Navegação:** cada par foca e rola ao segmento Rust correspondente; os segmentos mapeados do Rust continuam navegando à origem.
- **Qualidade e ausências:** pares duplicados são reduzidos a um link por linha; mapas `exact` prevalecem quando competem pelo mesmo par. Correspondências aproximadas recebem rótulo próprio e linhas sem fonte válida anunciam que não há relação direta.
- **Defesa do contrato:** mapas `generated_only` não criam navegação em nenhum dos sentidos, mesmo se uma resposta inconsistente trouxer spans de origem.
- **Concorrência:** editar, trocar a aba, ou desligar o modo bridge segue invalidando respostas e ocultando vínculos fora do documento/geração corrente.
- **Acessibilidade:** gutter tem nome de navegação, estado com anúncio polido e links com nome descritivo; foco do link no gutter se move ao trecho somente leitura Rust sem alterar o cursor da origem.
- **Compatibilidade:** nenhum comando Tauri, DTO IPC, gerador Rust ou semântica Portugol foi alterado; sem novas dependências.
- **Validação local:** `npm test` (15 arquivos de teste), TypeScript usando configuração temporária e declaração para `@codemirror/lang-rust` ausente localmente, `cargo test --locked --workspace --exclude portugol-desktop`, `cargo clippy --locked --workspace --exclude portugol-desktop --all-targets -- -D warnings`, `cargo check --locked -p portugol-desktop --tests` e `cargo fmt --all -- --check` passaram. `npm run build` não está disponível nesta worktree por falta dos binários `tsc`/Vite; CI fará o build com dependências instaladas. `git diff --check` verificado localmente.
